# Cabinet Simulation Playground

A Rust-based simulation playground for cabinet server with configurable tracing.

## Features

- TCP server for handling cabinet protocol commands
- Authentication mechanism for secure access
- Configurable server address
- Configurable tracing with endpoint and authentication options

## Command-Line Options

The application supports the following command-line options:

| Option | Description | Default |
|--------|-------------|---------|
| `-a, --address <ADDRESS>` | Address to bind the server to | 0.0.0.0:8080 |
| `--tracing-endpoint <ENDPOINT>` | Tracing endpoint URL (e.g., http://localhost:4317 for OTLP) | None |
| `--tracing-auth <AUTH>` | Tracing authentication token or header | None |

## Examples

Start the server with default settings:
```
cargo run
```

Start the server with a custom address:
```
cargo run -- --address 127.0.0.1:9090
```

Start the server with tracing configured:
```
cargo run -- --tracing-endpoint http://localhost:4317 --tracing-auth my-auth-token
```

## Authentication

### Server Authentication

The server requires authentication before processing commands. The authentication protocol works as follows:

1. Connect to the server
2. Send an authentication command: `auth "<tenant>"`
3. If authentication is successful, the server responds with `OK`
4. If authentication fails (e.g., empty tenant), the server responds with an error message
5. After successful authentication, you can execute other commands

Example session:
```
> auth "my_tenant"
OK
> get "my_key"
NIL
```

If you try to execute a command without authentication, you'll receive:
```
AUTHREQUIRED: perform auth <tenant> first
```

### Tracing Authentication

The application supports authenticated connections to OpenTelemetry tracing backends. When you provide a tracing endpoint and authentication token, the following happens:

1. The application connects to the specified tracing endpoint (e.g., http://localhost:4317)
2. It adds the authentication token as a Bearer token in the Authorization header
3. All traces are sent with this authentication header

This allows the application to work with secured tracing backends that require authentication.

Example:
```
cargo run -- --tracing-endpoint https://api.tracing-service.com/v1/traces --tracing-auth eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## Development

### Prerequisites

- Rust toolchain
- FoundationDB (environment variable `FDB_CLUSTER_PATH` should be set)

### Building

```
cargo build
```

### Running

```
cargo run
```