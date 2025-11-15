# gRPC-Web Integration Plan

## Overview

The admin panel is designed to use **gRPC-Web** (WebRPC) to communicate with the orchestrator for all CRUD operations. This document outlines the integration plan and current status.

## Architecture

```
Admin Panel (Browser) → gRPC-Web → Orchestrator → REST API → Server → Database
```

- **Admin Panel**: Uses Connect-ES (modern gRPC-Web) to make RPC calls
- **Orchestrator**: Exposes gRPC service defined in `admin/proto/admin.proto`
- **Server**: Receives REST API calls from orchestrator to modify database

## Current Status

### ✅ Completed
- Proto service definition (`admin/proto/admin.proto`)
- Connect-ES dependencies installed (`@connectrpc/connect`, `@connectrpc/connect-web`)
- Code generation tools installed (`@bufbuild/protoc-gen-es`, `@connectrpc/protoc-gen-connect-es`)
- API client structure ready

### ⏳ Pending
- **Orchestrator Implementation**: The orchestrator needs to implement the `AdminService` from `admin.proto`
- **Proto Code Generation**: Generate TypeScript clients from proto files
- **gRPC-Web Client Integration**: Replace HTTP calls with gRPC-Web calls

### 🔄 Temporary Solution
Currently, the admin panel uses **HTTP REST** calls to the orchestrator endpoints. This is a temporary solution until the orchestrator implements the gRPC service.

## Proto Service Definition

The `AdminService` is defined in `admin/proto/admin.proto`:

```protobuf
service AdminService {
  // Beacon operations
  rpc ListBeacons(ListBeaconsRequest) returns (ListBeaconsResponse);
  rpc GetBeacon(GetBeaconRequest) returns (GetBeaconResponse);
  rpc CreateBeacon(CreateBeaconRequest) returns (CreateBeaconResponse);
  rpc UpdateBeacon(UpdateBeaconRequest) returns (UpdateBeaconResponse);
  rpc DeleteBeacon(DeleteBeaconRequest) returns (DeleteBeaconResponse);

  // Area operations
  rpc ListAreas(ListAreasRequest) returns (ListAreasResponse);
  // ... (and similar for Merchants and Connections)
}
```

## Generating TypeScript Clients

Once the orchestrator implements the gRPC service, generate the TypeScript clients:

```bash
cd mobile
chmod +x scripts/generate-proto.sh
./scripts/generate-proto.sh
```

This will generate:
- `src/lib/grpc/admin_pb.ts` - Message types
- `src/lib/grpc/AdminServiceClientPb.ts` - Service client

## Using the gRPC-Web Client

Once code is generated, update `src/lib/api/client.ts`:

```typescript
import { createPromiseClient } from '@connectrpc/connect'
import { createConnectTransport } from '@connectrpc/connect-web'
import { AdminService } from '../grpc/admin_connect'

const transport = createConnectTransport({
  baseUrl: getOrchestratorUrl(),
})

const client = createPromiseClient(AdminService, transport)

// Example usage
export async function listBeacons(entityId: string, token: string) {
  if (isTauriMode()) {
    return tauriApi.getAllBeacons(entityId)
  }

  try {
    const response = await client.listBeacons({
      entityId,
    })

    // Parse JSON-encoded beacons
    const beacons = response.beacons.map(b => JSON.parse(new TextDecoder().decode(b)))

    return {
      status: 'success',
      data: beacons,
    }
  } catch (error) {
    return {
      status: 'error',
      message: error instanceof Error ? error.message : 'gRPC error',
    }
  }
}
```

## Orchestrator Implementation

The orchestrator needs to:

1. **Implement the gRPC Service** in Rust using `tonic`:

```rust
use tonic::{Request, Response, Status};
use navign_orchestrator_admin::admin_service_server::{AdminService, AdminServiceServer};

pub struct AdminServiceImpl {
    server_url: String,
}

#[tonic::async_trait]
impl AdminService for AdminServiceImpl {
    async fn list_beacons(
        &self,
        request: Request<ListBeaconsRequest>,
    ) -> Result<Response<ListBeaconsResponse>, Status> {
        let entity_id = &request.get_ref().entity_id;

        // Forward to server via REST API
        let response = reqwest::get(format!("{}/api/entities/{}/beacons", self.server_url, entity_id))
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .json::<Vec<Beacon>>()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Encode as JSON bytes
        let beacons = response.iter()
            .map(|b| serde_json::to_vec(b).unwrap())
            .collect();

        Ok(Response::new(ListBeaconsResponse { beacons }))
    }

    // Implement other RPCs similarly...
}
```

2. **Enable gRPC-Web Support**:

```rust
use tonic_web::GrpcWebLayer;

let service = AdminServiceServer::new(AdminServiceImpl::new(server_url));

let grpc_web = tonic_web::enable(service);

Server::builder()
    .accept_http1(true)  // Required for gRPC-Web
    .layer(GrpcWebLayer::new())
    .add_service(grpc_web)
    .serve(addr)
    .await?;
```

3. **CORS Configuration**:

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_headers(Any)
    .allow_methods(Any)
    .expose_headers(Any);

Server::builder()
    .layer(cors)
    .layer(GrpcWebLayer::new())
    .add_service(service)
    .serve(addr)
    .await?;
```

## Benefits of gRPC-Web

1. **Type Safety**: Full TypeScript types generated from proto files
2. **Streaming Support**: Can add server streaming for real-time updates
3. **Efficiency**: Binary protocol is more efficient than JSON
4. **Contract-First**: Proto files serve as API documentation
5. **Bi-directional Communication**: Can implement client streaming if needed

## Migration Path

1. **Phase 1** (Current): HTTP REST API
2. **Phase 2**: Orchestrator implements gRPC service
3. **Phase 3**: Generate TypeScript clients
4. **Phase 4**: Update admin panel to use gRPC-Web
5. **Phase 5**: Remove HTTP REST fallback

## Testing

Test the gRPC-Web integration:

```bash
# Start orchestrator with gRPC service
cd admin/orchestrator
cargo run

# In another terminal, start the admin panel
cd mobile
pnpm run dev

# Open browser to http://localhost:5173/admin
# Use browser DevTools Network tab to verify gRPC-Web calls
```

## Dependencies

```json
{
  "dependencies": {
    "@connectrpc/connect": "^2.1.0",
    "@connectrpc/connect-web": "^2.1.0",
    "@bufbuild/protobuf": "^2.10.1"
  },
  "devDependencies": {
    "@bufbuild/protoc-gen-es": "^2.10.1",
    "@connectrpc/protoc-gen-connect-es": "^1.7.0"
  }
}
```

## Resources

- [Connect-ES Documentation](https://connectrpc.com/docs/node/getting-started)
- [gRPC-Web Documentation](https://grpc.io/docs/platforms/web/)
- [Tonic gRPC-Web](https://github.com/hyperium/tonic/tree/master/tonic-web)
- [Protocol Buffers](https://protobuf.dev/)

## Notes

- gRPC-Web uses HTTP/1.1 or HTTP/2
- Compatible with all modern browsers
- No special server configuration needed (unlike gRPC which requires HTTP/2)
- Can use Envoy proxy for non-Rust backends
- JWT tokens can be sent via metadata/headers
