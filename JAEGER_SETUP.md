# Jaeger Setup with Docker Compose

This document explains how to use the provided Docker Compose configuration to run Jaeger with HTTP protocol and authentication.

## Prerequisites

- Docker and Docker Compose installed on your system
- The Cabinet Simulation Playground application

## Starting Jaeger

1. From the project root directory, run:

```bash
docker-compose up -d
```

This will start the Jaeger container in detached mode.

2. Verify that Jaeger is running:

```bash
docker-compose ps
```

## Jaeger Configuration

The Jaeger container is configured with:

- HTTP protocol enabled for OTLP (OpenTelemetry Protocol)
- Basic authentication with username `admin` and password `password`
- 100% sampling rate for all traces

## Connecting the Application to Jaeger

To connect the Cabinet Simulation Playground to Jaeger, use the following command:

```bash
cargo run -- --tracing-endpoint http://localhost:4318/v1/traces --tracing-auth admin:password
```

This configures the application to:
- Use the HTTP protocol (port 4318) for sending traces
- Include authentication credentials

## Accessing the Jaeger UI

The Jaeger UI is available at:

```
http://localhost:16686
```

Use this interface to view and analyze traces from your application.

## Stopping Jaeger

To stop the Jaeger container:

```bash
docker-compose down
```

## Troubleshooting

If you encounter issues:

1. Check that the container is running:
   ```bash
   docker-compose ps
   ```

2. View container logs:
   ```bash
   docker-compose logs jaeger
   ```

3. Ensure your application is correctly configured with the right endpoint and authentication.