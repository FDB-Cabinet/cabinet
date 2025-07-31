use crate::errors::CabinetError;
use crate::state::State;
use cabinet_lib::item::Item;
use cabinet_protocol::commands::auth::Auth;
use cabinet_protocol::commands::delete::Delete;
use cabinet_protocol::commands::get::Get;
use cabinet_protocol::commands::put::Put;
use cabinet_protocol::commands::{Command, Commands};
use std::net::TcpListener as StdTcpListener;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use toolbox::foundationdb::Database;
use toolbox::with_tenant;
use tracing::{error, info, trace, warn};

/// A TCP server that can handle multiple connections simultaneously.
pub struct CabinetServer {
    address: String,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl CabinetServer {
    /// Create a new TCP server that will listen on the given address.
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            shutdown_tx: None,
        }
    }

    /// Check if the port is already in use
    fn is_port_available(&self) -> bool {
        // Use std TcpListener to check if we can bind to the address
        match StdTcpListener::bind(&self.address) {
            Ok(_) => true,
            Err(e) => {
                warn!("Port check failed: {}", e);
                false
            }
        }
    }

    /// Start the TCP server and begin accepting connections.
    /// This method will block until the server is shut down.
    #[tracing::instrument(skip(self, database))]
    pub async fn start(&mut self, database: Arc<Database>) -> Result<(), CabinetError> {
        // Check if the port is available before trying to bind
        if !self.is_port_available() {
            return Err(CabinetError::IoError(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                format!("Address {} is already in use", self.address),
            )));
        }

        let listener = TcpListener::bind(&self.address).await?;
        info!("TCP server listening on {}", self.address);

        let (shutdown_tx, _) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((socket, addr)) => {
                            info!("Accepted connection from {}", addr);

                            // Clone the shutdown sender for this connection
                            let shutdown_rx = shutdown_tx.subscribe();

                            // Spawn a new task to handle this connection
                            tokio::spawn({
                                let database = database.clone();
                                async move {
                                if let Err(e) = handle_connection(socket, shutdown_rx, database).await {
                                    error!("Error handling connection from {}: {}", addr, e);
                                }
                            }});
                        }
                        Err(e) => {
                            error!("Error accepting connection: {}", e);
                        }
                    }
                }
                // Add a way to break the loop if needed
                _ = tokio::signal::ctrl_c() => {
                    info!("Received shutdown signal");
                    self.shutdown();
                    break;
                }
            }
        }

        info!("TCP server shutting down");
        Ok(())
    }

    /// Shutdown the server gracefully.
    pub fn shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }
    }
}

/// Handle a single client connection.
#[tracing::instrument(skip(database, shutdown_rx))]
async fn handle_connection(
    mut socket: TcpStream,
    mut shutdown_rx: broadcast::Receiver<()>,
    database: Arc<Database>,
) -> Result<(), CabinetError> {
    let mut buffer = [0; 1024];
    let mut state = State::new(database);

    info!("Handling connection...");

    let (mut quit_tx, mut quit_rx) = broadcast::channel(1);

    loop {
        tokio::select! {
            // Handle incoming data
            result = socket.read(&mut buffer) => {
                match result {
                    Ok(0) => {
                        // Connection closed by client
                        break;
                    }
                    Ok(n) => {
                        // Echo the data back to the client

                        let requests_bytes = &buffer[..n];

                        handle_requests(requests_bytes, &mut socket, &mut state, &mut quit_tx).await?;
                        socket.flush().await?;
                    }
                    Err(_) => {
                        // Error reading from socket
                        break;
                    }
                }
            }
            _ = quit_rx.recv() => {
                info!("Client explicitly quit");
            }
            // Handle shutdown signal
            _ = shutdown_rx.recv() => {
                info!("Connection handler received shutdown signal");
                break;
            }
        }
    }

    Ok(())
}

#[tracing::instrument(skip(socket, state), fields(tenant=state.tenant()))]
pub async fn handle_requests(
    raw: &[u8],
    socket: &mut TcpStream,
    state: &mut State,
    quit_tx: &broadcast::Sender<()>,
) -> Result<(), CabinetError> {
    trace!(raw=?String::from_utf8_lossy(raw));
    for command in Commands::new(raw) {
        let command = command?;

        match command {
            Command::Auth(_) | Command::Unknown(_) | Command::Quit(_) => {
                handle_requests_non_authenticated(command, socket, state, quit_tx).await?;
            }
            command => handle_authenticated_requests(command, socket, state).await?,
        }
    }

    Ok(())
}

pub enum Response {
    Ok,
    Error(String),
    AuthRequired,
    Value(String),
    Stats { count: i64, size: i64 },
    Nil,
}

impl Response {
    pub fn to_bytes(&self) -> Vec<u8> {
        match self {
            Response::Ok => b"OK\n".to_vec(),
            Response::Error(message) => format!("ERROR {}\n", message).as_bytes().to_vec(),
            Response::AuthRequired => b"AUTHREQUIRED: perform auth <tenant> first\n".to_vec(),
            Response::Value(value) => format!("VALUE {}\n{}\n", value.len(), value)
                .as_bytes()
                .to_vec(),
            Response::Stats { count, size } => {
                format!("STATS cardinality: {} storage:{} bytes\n", count, size)
                    .as_bytes()
                    .to_vec()
            }
            Response::Nil => b"NIL\n".to_vec(),
        }
    }
}

#[tracing::instrument(skip(socket, state), fields(tenant=state.tenant()))]
async fn handle_requests_non_authenticated<'a>(
    command: Command<'a>,
    socket: &mut TcpStream,
    state: &mut State,
    quit_tx: &broadcast::Sender<()>,
) -> Result<(), CabinetError> {
    match command {
        Command::Auth(Auth { tenant }) => {
            // Simple authentication logic - in a real application, you would validate credentials
            // For this example, we'll authenticate if the tenant is not empty
            if !tenant.is_empty() {
                state.set_tenant(tenant);
                state.set_authenticated(true);
                socket.write_all(&Response::Ok.to_bytes()).await?;
            } else {
                socket
                    .write_all(&Response::Error("Authentication failed".to_string()).to_bytes())
                    .await?;
            }
        }
        Command::Unknown(_) => {
            socket
                .write_all(&Response::Error("Unknown command".to_string()).to_bytes())
                .await?;
        }
        Command::Quit(_) => {
            if let Err(err) = quit_tx.send(()) {
                error!("Failed to send quit signal: {}", err);
            }
            socket.write_all(&Response::Ok.to_bytes()).await?;
        }
        _ => {
            socket.write_all(&Response::AuthRequired.to_bytes()).await?;
        }
    }
    Ok(())
}

#[tracing::instrument(skip(socket, state), fields(tenant=state.tenant()))]
async fn handle_authenticated_requests<'a>(
    command: Command<'a>,
    socket: &mut TcpStream,
    state: &mut State,
) -> Result<(), CabinetError> {
    // Check if the client is authenticated
    if !state.is_authenticated() {
        socket.write_all(&Response::AuthRequired.to_bytes()).await?;
        return Ok(());
    }

    let Some(tenant) = state.tenant() else {
        socket.write_all(&Response::AuthRequired.to_bytes()).await?;
        return Ok(());
    };

    let response = with_tenant(state.database(), tenant, |db| async move {
        let response = match command {
            Command::Put(Put { key, value }) => {
                let item = Item::new(key, value);

                db.put(&item).await?;
                Response::Ok
            }
            Command::Get(Get { key }) => {
                let Some(item) = db.get::<Item>(key).await? else {
                    return Ok(Response::Nil);
                };
                let value = std::str::from_utf8(&item.value).map_err(CabinetError::Utf8Error)?;
                Response::Value(value.to_string())
            }
            Command::Delete(Delete { key }) => {
                let Some(_) = db.delete::<Item>(key).await? else {
                    return Ok(Response::Nil);
                };
                Response::Ok
            }
            Command::Clear(_) => {
                db.clear::<Item>().await?;
                return Ok(Response::Ok);
            }
            Command::Stats(_) => {
                let stats = db.get_stats();
                let size = stats.get_size().await?;
                let count = stats.get_count().await?;
                return Ok(Response::Stats { size, count });
            }
            _ => unreachable!("This should never happen"),
        };

        Ok(response)
    })
    .await?;

    socket.write_all(&response.to_bytes()).await?;

    Ok(())
}
