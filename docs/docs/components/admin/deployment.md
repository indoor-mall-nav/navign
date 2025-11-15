# Admin System Deployment Guide

This guide covers deploying the Navign Admin system (Orchestrator, Tower, and Plot components) in production environments.

## Overview

The admin system consists of three components that work together to manage robot fleets:

- **Orchestrator** (Rust) - Task assignment and business logic
- **Tower** (Go) - Robot connection management via Socket.IO
- **Plot** (Python) - Floor plan polygon extraction tool

## Architecture

```
┌─────────────────────────────────────────────────┐
│              Rust Orchestrator                  │
│         (gRPC Server - Port 50051)              │
│                                                 │
│  • Task scheduling & assignment                 │
│  • Robot registry & tracking                    │
│  • Best robot selection algorithm               │
└─────────────┬───────────────────────────────────┘
              │ gRPC
┌─────────────▼───────────────────────────────────┐
│                Go Tower                         │
│   (Socket.IO Server - Port 8080)                │
│                                                 │
│  • WebSocket connection manager                 │
│  • Robot status aggregation                     │
└─────────────┬───────────────────────────────────┘
              │ Socket.IO/WebSocket
     ┌────────┴────────┬────────────┐
     │                 │            │
┌────▼─────┐    ┌─────▼────┐  ┌───▼──────┐
│ Robot 1  │    │ Robot 2  │  │ Robot N  │
└──────────┘    └──────────┘  └──────────┘
```

## Prerequisites

### System Requirements

**Minimum:**
- CPU: 2 cores
- RAM: 2GB
- Disk: 500MB
- OS: Linux (Ubuntu 22.04+, Debian 12+, RHEL 9+)

**Recommended:**
- CPU: 4 cores
- RAM: 4GB
- Disk: 2GB
- OS: Linux with systemd

### Software Dependencies

::: tip Using Prebuilt Binaries
If you're using prebuilt binaries, you **only** need Python/uv (for Plot). Rust, Go, and protoc are only required if building from source.
:::

**Required for Prebuilt Binaries:**

1. **Python with uv** (for Plot only)
   ```bash
   curl -LsSf https://astral.sh/uv/install.sh | sh
   source "$HOME/.cargo/env"
   ```

**Required for Building from Source:**

1. **Rust** (1.86+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source "$HOME/.cargo/env"
   ```

2. **Go** (1.25+)
   ```bash
   # Ubuntu/Debian
   wget https://go.dev/dl/go1.25.3.linux-amd64.tar.gz
   sudo tar -C /usr/local -xzf go1.25.3.linux-amd64.tar.gz
   echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.bashrc
   source ~/.bashrc
   ```

3. **Protocol Buffers Compiler**
   ```bash
   # Ubuntu/Debian
   sudo apt install -y protobuf-compiler

   # Verify
   protoc --version  # Should be 3.12+
   ```

4. **Protocol Buffers Go Plugins**
   ```bash
   go install google.golang.org/protobuf/cmd/protoc-gen-go@latest
   go install google.golang.org/grpc/cmd/protoc-gen-go-grpc@latest

   # Add to PATH
   echo 'export PATH=$PATH:$(go env GOPATH)/bin' >> ~/.bashrc
   source ~/.bashrc
   ```

## Installation

### Option 1: Using Prebuilt Binaries (Recommended)

Download the latest release binaries from GitHub releases:

```bash
# Set version
VERSION="v0.1.0"  # Replace with latest version

# Download Orchestrator
wget https://github.com/indoor-mall-nav/navign/releases/download/${VERSION}/navign-orchestrator-linux-amd64
chmod +x navign-orchestrator-linux-amd64

# Download Tower
wget https://github.com/indoor-mall-nav/navign/releases/download/${VERSION}/navign-tower-linux-amd64
chmod +x navign-tower-linux-amd64

# Download Plot (Python package)
wget https://github.com/indoor-mall-nav/navign/releases/download/${VERSION}/navign-plot.tar.gz
```

### Option 2: Building from Source (Development)

::: warning
Building from source is recommended only for development or when prebuilt binaries are not available for your platform.
:::

```bash
# Clone the repository
git clone https://github.com/indoor-mall-nav/navign.git
cd navign

# Generate protocol buffers
just proto

# Build Orchestrator (Rust)
cd admin/orchestrator
cargo build --release
# Binary: target/release/navign-orchestrator

# Build Tower (Go)
cd ../tower
go build -o tower ./cmd/tower
# Binary: ./tower

# Setup Plot (Python)
cd ../plot
uv sync
```

### Install Binaries

```bash
# Create installation directory
sudo mkdir -p /opt/navign/bin
sudo mkdir -p /opt/navign/data
sudo mkdir -p /var/log/navign

# Copy binaries (adjust paths based on installation method)

# If using prebuilt binaries:
sudo mv navign-orchestrator-linux-amd64 /opt/navign/bin/navign-orchestrator
sudo mv navign-tower-linux-amd64 /opt/navign/bin/navign-tower

# If built from source:
# sudo cp admin/orchestrator/target/release/navign-orchestrator /opt/navign/bin/
# sudo cp admin/tower/tower /opt/navign/bin/

# Set permissions
sudo chmod +x /opt/navign/bin/*
sudo chown -R navign:navign /opt/navign
sudo chown -R navign:navign /var/log/navign
```

## Configuration

### Environment Variables

Create `/opt/navign/config/orchestrator.env`:

```bash
# Orchestrator configuration
RUST_LOG=info
ORCHESTRATOR_ADDR=[::1]:50051
SERVER_URL=http://localhost:3000  # Navign server URL

# Optional: Database configuration
# DATABASE_URL=postgresql://user:pass@localhost/navign
```

Create `/opt/navign/config/tower.env`:

```bash
# Tower configuration
ENTITY_ID=mall-001                 # Required: Entity/building ID
ORCHESTRATOR_ADDR=localhost:50051  # Orchestrator gRPC address
TOWER_ADDR=http://[::1]:8080       # Socket.IO server address

# Optional: Logging
LOG_LEVEL=info
```

### Systemd Service Files

#### Orchestrator Service

Create `/etc/systemd/system/navign-orchestrator.service`:

```ini
[Unit]
Description=Navign Orchestrator (Robot Task Management)
Documentation=https://github.com/indoor-mall-nav/navign
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=navign
Group=navign
EnvironmentFile=/opt/navign/config/orchestrator.env
ExecStart=/opt/navign/bin/navign-orchestrator
WorkingDirectory=/opt/navign
Restart=always
RestartSec=10

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=navign-orchestrator

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/opt/navign/data /var/log/navign

[Install]
WantedBy=multi-user.target
```

#### Tower Service

Create `/etc/systemd/system/navign-tower.service`:

```ini
[Unit]
Description=Navign Tower (Robot Connection Manager)
Documentation=https://github.com/indoor-mall-nav/navign
After=network-online.target navign-orchestrator.service
Wants=network-online.target
Requires=navign-orchestrator.service

[Service]
Type=simple
User=navign
Group=navign
EnvironmentFile=/opt/navign/config/tower.env
ExecStart=/opt/navign/bin/tower \
    --entity-id ${ENTITY_ID} \
    --grpc ${ORCHESTRATOR_ADDR} \
    --tower ${TOWER_ADDR}
WorkingDirectory=/opt/navign
Restart=always
RestartSec=10

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=navign-tower

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true

[Install]
WantedBy=multi-user.target
```

### Create Service User

```bash
# Create dedicated user
sudo useradd --system --no-create-home --shell /bin/false navign

# Set ownership
sudo chown -R navign:navign /opt/navign
sudo chown -R navign:navign /var/log/navign
```

### Enable and Start Services

```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable services (start on boot)
sudo systemctl enable navign-orchestrator
sudo systemctl enable navign-tower

# Start services
sudo systemctl start navign-orchestrator
sudo systemctl start navign-tower

# Check status
sudo systemctl status navign-orchestrator
sudo systemctl status navign-tower
```

## Deployment Modes

### Single Server Deployment

Deploy all components on one server:

```
┌───────────────────────────────┐
│     Single Server             │
│                               │
│  ┌─────────────────────────┐  │
│  │   Orchestrator :50051   │  │
│  └───────────┬─────────────┘  │
│              │                │
│  ┌───────────▼─────────────┐  │
│  │   Tower :8080           │  │
│  └─────────────────────────┘  │
│                               │
└───────────────────────────────┘
        │
        ▼
    Robots (WebSocket)
```

**Configuration:**
- Orchestrator: `[::1]:50051`
- Tower: `--grpc localhost:50051`

### Multi-Entity Deployment

Deploy multiple Tower instances for different buildings:

```
┌─────────────────────────────┐
│   Orchestrator :50051       │
└──┬────────┬────────┬────────┘
   │        │        │
   ▼        ▼        ▼
Tower A  Tower B  Tower C
(Mall-A) (Mall-B) (Airport)
:8080    :8081    :8082
```

**Configuration:**

```bash
# Tower A (Mall A)
ENTITY_ID=mall-a TOWER_ADDR=http://[::1]:8080 ./tower

# Tower B (Mall B)
ENTITY_ID=mall-b TOWER_ADDR=http://[::1]:8081 ./tower

# Tower C (Airport)
ENTITY_ID=airport TOWER_ADDR=http://[::1]:8082 ./tower
```

### Distributed Deployment

Deploy on separate servers with reverse proxy:

```
┌─────────────────┐
│  Load Balancer  │
│  (Nginx/Envoy)  │
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌─────────┐ ┌─────────┐
│  Tower  │ │  Tower  │
│  Node 1 │ │  Node 2 │
└────┬────┘ └────┬────┘
     │           │
     └─────┬─────┘
           ▼
    ┌──────────────┐
    │ Orchestrator │
    └──────────────┘
```

## Networking & Firewall

### Ports

| Component     | Port  | Protocol | Purpose           |
|---------------|-------|----------|-------------------|
| Orchestrator  | 50051 | gRPC     | Task management   |
| Tower         | 8080  | HTTP/WS  | Robot connections |

### Firewall Rules

```bash
# Ubuntu/Debian (ufw)
sudo ufw allow 50051/tcp comment 'Navign Orchestrator gRPC'
sudo ufw allow 8080/tcp comment 'Navign Tower Socket.IO'
sudo ufw enable

# RHEL/CentOS (firewalld)
sudo firewall-cmd --permanent --add-port=50051/tcp
sudo firewall-cmd --permanent --add-port=8080/tcp
sudo firewall-cmd --reload
```

### Reverse Proxy (Nginx)

For WebSocket support with SSL:

```nginx
# /etc/nginx/sites-available/navign-tower
upstream tower {
    server 127.0.0.1:8080;
}

server {
    listen 443 ssl http2;
    server_name tower.example.com;

    ssl_certificate /etc/letsencrypt/live/tower.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/tower.example.com/privkey.pem;

    location / {
        proxy_pass http://tower;
        proxy_http_version 1.1;

        # WebSocket support
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";

        # Headers
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 7d;
        proxy_send_timeout 7d;
        proxy_read_timeout 7d;
    }
}
```

Enable and restart:

```bash
sudo ln -s /etc/nginx/sites-available/navign-tower /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

## Monitoring

### Systemd Journal Logs

```bash
# View Orchestrator logs
sudo journalctl -u navign-orchestrator -f

# View Tower logs
sudo journalctl -u navign-tower -f

# View recent errors
sudo journalctl -u navign-orchestrator -p err --since "1 hour ago"
```

### Log Rotation

Create `/etc/logrotate.d/navign`:

```
/var/log/navign/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 0640 navign navign
    sharedscripts
    postrotate
        systemctl reload navign-orchestrator navign-tower > /dev/null 2>&1 || true
    endscript
}
```

### Health Checks

```bash
# Check Orchestrator (gRPC)
grpcurl -plaintext localhost:50051 list

# Check Tower (HTTP endpoint)
curl http://localhost:8080/health

# Check systemd status
sudo systemctl is-active navign-orchestrator
sudo systemctl is-active navign-tower
```

### Prometheus Metrics (Future)

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'navign-orchestrator'
    static_configs:
      - targets: ['localhost:9090']

  - job_name: 'navign-tower'
    static_configs:
      - targets: ['localhost:9091']
```

## Backup & Recovery

### Data to Backup

```bash
# Configuration files
/opt/navign/config/

# Application data
/opt/navign/data/

# Logs (optional)
/var/log/navign/
```

### Backup Script

```bash
#!/bin/bash
# /opt/navign/scripts/backup.sh

BACKUP_DIR="/backup/navign/$(date +%Y%m%d)"
mkdir -p "$BACKUP_DIR"

# Backup config
tar -czf "$BACKUP_DIR/config.tar.gz" /opt/navign/config/

# Backup data
tar -czf "$BACKUP_DIR/data.tar.gz" /opt/navign/data/

# Retention: Keep 30 days
find /backup/navign/ -type d -mtime +30 -exec rm -rf {} +
```

Add to crontab:

```bash
# Daily backup at 2 AM
0 2 * * * /opt/navign/scripts/backup.sh
```

## Security

### TLS/SSL

**gRPC (Orchestrator ↔ Tower):**

Generate certificates:

```bash
# Generate CA
openssl genrsa -out ca.key 4096
openssl req -new -x509 -key ca.key -out ca.crt -days 3650

# Generate server cert
openssl genrsa -out server.key 4096
openssl req -new -key server.key -out server.csr
openssl x509 -req -in server.csr -CA ca.crt -CAkey ca.key -CAcreateserial -out server.crt -days 365
```

Update Orchestrator to use TLS (requires code changes).

### Authentication

**Robot Authentication (Future):**

```bash
# Generate JWT secret
openssl rand -base64 32 > /opt/navign/config/jwt-secret

# Set in tower.env
JWT_SECRET=$(cat /opt/navign/config/jwt-secret)
```

### Network Isolation

```bash
# Restrict Orchestrator to local only
ORCHESTRATOR_ADDR=127.0.0.1:50051

# Use firewall to block external access
sudo ufw deny from any to any port 50051
sudo ufw allow from 10.0.0.0/8 to any port 50051
```

## Troubleshooting

### Common Issues

#### Orchestrator Won't Start

```bash
# Check logs
sudo journalctl -u navign-orchestrator -n 50

# Common causes:
# - Port 50051 already in use
sudo lsof -i :50051

# - Missing dependencies
ldd /opt/navign/bin/navign-orchestrator

# - Permission issues
sudo chown navign:navign /opt/navign/bin/navign-orchestrator
```

#### Tower Can't Connect to Orchestrator

```bash
# Test gRPC connectivity
grpcurl -plaintext localhost:50051 list

# Check firewall
sudo ufw status | grep 50051

# Verify Orchestrator is running
sudo systemctl status navign-orchestrator

# Check Tower logs
sudo journalctl -u navign-tower -n 50
```

#### Robots Can't Connect to Tower

```bash
# Test WebSocket endpoint
curl -i -N \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" \
  -H "Sec-WebSocket-Key: test" \
  http://localhost:8080/socket.io/?transport=websocket

# Check Tower status
sudo systemctl status navign-tower

# Verify port is listening
sudo netstat -tlnp | grep 8080
```

#### High Memory Usage

```bash
# Check process memory
ps aux | grep navign

# Restart services
sudo systemctl restart navign-orchestrator navign-tower

# Monitor with top
top -p $(pgrep navign-orchestrator) -p $(pgrep tower)
```

### Debug Mode

Enable debug logging:

```bash
# Orchestrator
RUST_LOG=debug systemctl start navign-orchestrator

# Tower
LOG_LEVEL=debug systemctl start navign-tower
```

## Updating

### Update Process (Prebuilt Binaries)

```bash
# 1. Stop services
sudo systemctl stop navign-tower navign-orchestrator

# 2. Backup current version
sudo cp /opt/navign/bin/navign-orchestrator /opt/navign/bin/navign-orchestrator.bak
sudo cp /opt/navign/bin/navign-tower /opt/navign/bin/navign-tower.bak

# 3. Download new version
VERSION="v0.2.0"  # Replace with new version
wget https://github.com/indoor-mall-nav/navign/releases/download/${VERSION}/navign-orchestrator-linux-amd64
wget https://github.com/indoor-mall-nav/navign/releases/download/${VERSION}/navign-tower-linux-amd64

# 4. Verify checksums (recommended)
wget https://github.com/indoor-mall-nav/navign/releases/download/${VERSION}/checksums.txt
sha256sum -c checksums.txt

# 5. Install new binaries
sudo mv navign-orchestrator-linux-amd64 /opt/navign/bin/navign-orchestrator
sudo mv navign-tower-linux-amd64 /opt/navign/bin/navign-tower
sudo chmod +x /opt/navign/bin/*

# 6. Restart services
sudo systemctl start navign-orchestrator navign-tower

# 7. Verify
sudo systemctl status navign-orchestrator navign-tower

# 8. Test functionality
curl -i localhost:50051  # Orchestrator health check
curl -i localhost:8080   # Tower health check
```

### Update Process (From Source)

::: warning
Only use this method if prebuilt binaries are not available.
:::

```bash
# 1. Stop services
sudo systemctl stop navign-tower navign-orchestrator

# 2. Backup current version
sudo cp /opt/navign/bin/navign-orchestrator /opt/navign/bin/navign-orchestrator.bak
sudo cp /opt/navign/bin/navign-tower /opt/navign/bin/navign-tower.bak

# 3. Pull latest code
cd /path/to/navign
git pull origin main

# 4. Rebuild
cd admin/orchestrator && cargo build --release
cd ../tower && go build -o tower ./cmd/tower

# 5. Install new binaries
sudo cp admin/orchestrator/target/release/navign-orchestrator /opt/navign/bin/
sudo cp admin/tower/tower /opt/navign/bin/

# 6. Restart services
sudo systemctl start navign-orchestrator navign-tower

# 7. Verify
sudo systemctl status navign-orchestrator navign-tower
```

### Rollback

```bash
# Restore previous version
sudo cp /opt/navign/bin/navign-orchestrator.bak /opt/navign/bin/navign-orchestrator
sudo cp /opt/navign/bin/navign-tower.bak /opt/navign/bin/navign-tower

# Restart
sudo systemctl restart navign-orchestrator navign-tower

# Verify
sudo systemctl status navign-orchestrator navign-tower
```

## Performance Tuning

### Orchestrator

```bash
# Increase thread pool size
TOKIO_WORKER_THREADS=8

# Adjust log level for production
RUST_LOG=info  # instead of debug
```

### Tower

```bash
# Increase max connections
GOMAXPROCS=4  # Number of CPU cores
```

### System Limits

```bash
# /etc/security/limits.conf
navign soft nofile 65536
navign hard nofile 65536
```

## Plot Component Deployment

Plot is a command-line tool, not a service. Deploy on admin workstations:

```bash
# Install on admin machine
cd admin/plot
uv sync

# Create wrapper script
cat > /usr/local/bin/navign-plot << 'EOF'
#!/bin/bash
cd /opt/navign/plot
uv run python plot_client.py "$@"
EOF

chmod +x /usr/local/bin/navign-plot

# Usage
navign-plot /path/to/floorplan.png mall-001 1F
```

## Next Steps

- [Quickstart Guide](./quickstart.md) - Development setup
- [Integration Guide](./implementation-guide.md) - Integrate with Navign server
- [Protocol Documentation](./protocol.md) - gRPC and Socket.IO protocols
- [Tower Architecture](./tower.md) - Tower internals
- [Orchestrator Architecture](./orchestrator.md) - Orchestrator internals
