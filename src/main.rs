use cabinet::run;
use cabinet_lib::errors::CabinetLibError;

#[tokio::main]
async fn main() -> Result<(), CabinetLibError> {
    // The run function will parse command-line arguments using clap
    // You can now use the following command-line options:
    // --address <ADDRESS>             - Set the server address (default: 0.0.0.0:8080)
    // --tracing-endpoint <ENDPOINT>   - Set the tracing endpoint URL
    // --tracing-auth <AUTH>           - Set the tracing authentication token
    if let Err(err) = run().await {
        eprintln!("Error: {}", err);
    }

    Ok(())
}
