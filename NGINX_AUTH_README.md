# Jaeger UI and OTLP Endpoints with Nginx Basic Authentication

## Overview

Both the Jaeger UI and OTLP endpoints are now protected by basic authentication using an Nginx reverse proxy. This adds a layer of security to prevent unauthorized access to your tracing data and endpoints.

## Configuration Details

- Nginx acts as a reverse proxy in front of the Jaeger UI and OTLP endpoints
- Basic authentication is required to access the Jaeger UI and HTTP-based OTLP endpoint
- OTLP HTTP port (4318) is protected with basic authentication
- OTLP gRPC port (4317) is proxied but not authenticated (TCP streams don't support basic auth)
- The Jaeger UI is accessible on port 16686 through Nginx

## Access Credentials

To access the Jaeger UI, use the following credentials:

- Username: `admin`
- Password: `password`

## How to Access

1. Start the services using Docker Compose:
   ```
   docker-compose up -d
   ```

### Accessing Jaeger UI

1. Open your browser and navigate to:
   ```
   http://localhost:16686
   ```

2. When prompted, enter the username and password mentioned above.

### Configuring OTLP Clients

When configuring OTLP clients to send traces, you'll need to include authentication:

1. For HTTP (port 4318):
   ```
   # Example with curl
   curl -X POST http://localhost:4318/v1/traces \
     -u admin:password \
     -H "Content-Type: application/json" \
     -d '{"your-trace-data": "here"}'
   ```

2. For gRPC (port 4317):
   ```
   # Example with environment variables for OpenTelemetry SDKs
   OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
   
   # Note: Basic authentication is not supported for gRPC/TCP streams
   # The gRPC endpoint is proxied but not authenticated
   ```

## Modifying Authentication

If you want to change the username or password:

1. Generate a new .htpasswd file entry using a tool like `htpasswd` or an online generator
2. Replace the content in `nginx/.htpasswd` with your new credentials
3. Restart the Nginx container:
   ```
   docker-compose restart nginx
   ```