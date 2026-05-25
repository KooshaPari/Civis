# Docker Deployment Guide

Run DINOForge and related tools in isolated Docker containers for consistent, reproducible deployments across development, testing, and production environments.

## Overview

Docker provides several advantages for DINOForge deployment:

- **Consistency**: Same environment across all machines (dev, CI, production)
- **Isolation**: No conflicts with system dependencies
- **Scalability**: Easy horizontal scaling for multi-pack compilation
- **Reproducibility**: Version-locked .NET runtime and dependencies
- **Easy Setup**: Single `docker-compose up` command

This guide covers containerizing:
1. DINOForge SDK and PackCompiler tooling
2. MCP Bridge server
3. Optional: Game instance (Wine container for testing)

## Prerequisites

- Docker 20.10+ or Docker Desktop
- Docker Compose 2.0+ (included with Docker Desktop)
- 4 GB+ RAM allocated to Docker
- 10 GB+ free disk space

### Check Docker Installation

```bash
docker --version
docker-compose --version
```

## Dockerfile (Build DINOForge Runtime & Tools)

Create: `Dockerfile.dinoforge`

```dockerfile
# Multi-stage build: .NET 11 SDK → Runtime

# Build stage
FROM mcr.microsoft.com/dotnet/sdk:11.0-preview-bookworm as builder

WORKDIR /src

# Copy solution and projects
COPY DINOForge.sln ./
COPY src/ ./src/
COPY schemas/ ./schemas/
COPY packs/ ./packs/

# Restore dependencies
RUN dotnet restore src/DINOForge.sln

# Build release configuration
RUN dotnet build src/DINOForge.sln -c Release -o /app/build

# Publish stage
FROM mcr.microsoft.com/dotnet/runtime:11.0-preview-bookworm

LABEL maintainer="DINOForge Developers"
LABEL description="DINOForge Runtime & SDK Container"
LABEL version="0.14.0"

# Install additional runtime dependencies
RUN apt-get update && apt-get install -y \
    curl \
    jq \
    wget \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built artifacts from builder
COPY --from=builder /app/build .

# Create directories for mods, data, logs
RUN mkdir -p /app/mods /app/data /app/logs

# Environment variables
ENV DOTNET_SYSTEM_GLOBALIZATION_INVARIANT=1
ENV DOTNET_EnableDiagnostics=0

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8765/health || exit 1

# Expose MCP server port
EXPOSE 8765

# Default: run MCP server
CMD ["dotnet", "DINOForge.Tools.DinoforgeMcp.dll"]
```

## Docker Compose Configuration

Create: `docker-compose.yml`

```yaml
version: '3.9'

services:
  # DINOForge MCP Server
  dinoforge-mcp:
    build:
      context: .
      dockerfile: Dockerfile.dinoforge
    container_name: dinoforge-mcp
    ports:
      - "8765:8765"
    volumes:
      # Mod packs
      - ./packs:/app/packs:ro
      # Persistent data (configs, logs)
      - dinoforge-data:/app/data
      # Logs for inspection
      - dinoforge-logs:/app/logs
    environment:
      # .NET configuration
      DOTNET_SYSTEM_GLOBALIZATION_INVARIANT: "1"
      DOTNET_EnableDiagnostics: "0"
      # DINOForge configuration
      LOG_LEVEL: "Info"
      MCP_HOST: "0.0.0.0"
      MCP_PORT: "8765"
    restart: unless-stopped
    networks:
      - dinoforge-network
    depends_on:
      - dinoforge-cache

  # Redis cache (optional: for distributed pack compilation)
  dinoforge-cache:
    image: redis:7-alpine
    container_name: dinoforge-cache
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    restart: unless-stopped
    networks:
      - dinoforge-network

  # PostgreSQL database (optional: for game state tracking)
  dinoforge-db:
    image: postgres:15-alpine
    container_name: dinoforge-db
    environment:
      POSTGRES_USER: dinoforge
      POSTGRES_PASSWORD: changeme  # CHANGE THIS!
      POSTGRES_DB: dinoforge_data
    volumes:
      - postgres-data:/var/lib/postgresql/data
    ports:
      - "5432:5432"
    restart: unless-stopped
    networks:
      - dinoforge-network

  # (Optional) Wine container for game testing
  dinoforge-game-test:
    image: lutris/lutris:latest
    container_name: dinoforge-game-test
    volumes:
      # Game installation
      - game-install:/home/user/.steam/steamapps/common/Diplomacy\ is\ Not\ an\ Option
      # Mods
      - ./packs:/mods:ro
      # Shared data
      - dinoforge-data:/shared-data
    environment:
      DISPLAY: ":1"
    # Uncomment for X11 forwarding on Linux
    # ports:
    #   - "6000:6000"
    restart: "no"
    networks:
      - dinoforge-network
    profiles:
      - "with-game"  # Only enable with: docker-compose --profile with-game up

networks:
  dinoforge-network:
    driver: bridge

volumes:
  dinoforge-data:
    driver: local
  dinoforge-logs:
    driver: local
  redis-data:
    driver: local
  postgres-data:
    driver: local
  game-install:
    driver: local
```

## Building and Running

### Build the Docker Image

```bash
# From the DINOForge repository root
docker build -f Dockerfile.dinoforge -t dinoforge:latest .

# Or use docker-compose (builds automatically)
docker-compose build
```

### Start Services

```bash
# Start core services (MCP, Redis)
docker-compose up -d

# Or start with game testing container
docker-compose --profile with-game up -d

# View logs
docker-compose logs -f dinoforge-mcp

# Stop services
docker-compose down
```

### Verify Containers Are Running

```bash
docker-compose ps

# Expected output:
# NAME                    STATUS
# dinoforge-mcp           Up
# dinoforge-cache         Up
# dinoforge-db            Up
```

## Configuration via Environment Variables

Override settings without rebuilding:

```bash
docker-compose run --rm dinoforge-mcp env -i \
    LOG_LEVEL=Debug \
    MCP_HOST=127.0.0.1 \
    RUST_BACKTRACE=1 \
    dotnet DINOForge.Tools.DinoforgeMcp.dll
```

Or set in `.env` file:

```env
# .env
LOG_LEVEL=Info
MCP_HOST=0.0.0.0
MCP_PORT=8765
POSTGRES_PASSWORD=mysecurepassword
REDIS_HOST=dinoforge-cache
REDIS_PORT=6379
```

Then:

```bash
docker-compose --env-file .env up -d
```

## Volume Mounting for Development

### Mount Local Directories

```bash
# Mount source code for live development
docker-compose run --rm \
  -v $(pwd)/src:/app/src:ro \
  -v $(pwd)/schemas:/app/schemas:ro \
  dinoforge-mcp bash

# Inside container:
cd /app
dotnet run --project src/Tools/PackCompiler -- validate packs/
```

### Accessing Logs

```bash
# View container logs
docker-compose logs dinoforge-mcp

# Follow logs in real-time
docker-compose logs -f dinoforge-mcp

# Extract logs to file
docker-compose logs dinoforge-mcp > logs/mcp.log

# Access mounted logs directory
ls -la dinoforge-logs/
```

## Networking

### Accessing Services from Host

```bash
# MCP Server (from host)
curl http://localhost:8765/rpc \
  -d '{"jsonrpc":"2.0","id":1,"method":"game_status","params":{}}'

# Redis (from host)
redis-cli -h 127.0.0.1 -p 6379

# PostgreSQL (from host)
psql -h 127.0.0.1 -U dinoforge -d dinoforge_data
```

### Inter-container Communication

Containers can reach each other via service names:

```bash
# From dinoforge-mcp to Redis
redis://dinoforge-cache:6379

# From dinoforge-mcp to PostgreSQL
postgresql://dinoforge:password@dinoforge-db:5432/dinoforge_data
```

## Advanced: Multi-Stage Compilation

Scale pack compilation across multiple containers:

```yaml
services:
  pack-compiler:
    image: dinoforge:latest
    command: dotnet DINOForge.Tools.PackCompiler build ${PACK_NAME}
    volumes:
      - ./packs:/app/packs
      - ./dist:/app/dist
    environment:
      PACK_NAME: ${PACK_NAME:-example-balance}
    networks:
      - dinoforge-network

  pack-compiler-2:
    image: dinoforge:latest
    command: dotnet DINOForge.Tools.PackCompiler build ${PACK_NAME_2}
    volumes:
      - ./packs:/app/packs
      - ./dist:/app/dist
    networks:
      - dinoforge-network
```

Run parallel compilations:

```bash
PACK_NAME=warfare-modern PACK_NAME_2=example-balance \
docker-compose up pack-compiler pack-compiler-2
```

## Testing Connectivity

### Test MCP Server Health

```bash
# Inside container
docker-compose exec dinoforge-mcp curl -s http://localhost:8765/health | jq

# From host
curl -s http://localhost:8765/rpc \
  -d '{"jsonrpc":"2.0","id":1,"method":"game_status","params":{}}' | jq
```

### Test Redis Connection

```bash
docker-compose exec dinoforge-cache redis-cli ping
# Expected: PONG
```

### Test PostgreSQL Connection

```bash
docker-compose exec dinoforge-db psql -U dinoforge -d dinoforge_data -c "SELECT 1"
# Expected: 1
```

## Troubleshooting

### Container Fails to Start

**Solution**:
```bash
# Check logs
docker-compose logs dinoforge-mcp

# Rebuild image
docker-compose build --no-cache

# Restart
docker-compose up -d
```

### Port Already in Use

**Solution**:
```bash
# Find what's using the port
lsof -i :8765

# Change port in docker-compose.yml
# Or stop the conflicting service and restart
docker-compose down
```

### Volume Permission Issues

**Solution**:
```bash
# Fix permissions on host directories
chmod -R u+rwX packs/
chmod -R u+rwX dinoforge-logs/

# Or run container with user ID
docker-compose run --user $(id -u):$(id -g) dinoforge-mcp bash
```

### Container Out of Memory

**Solution**:
```bash
# Increase Docker memory limit
# Docker Desktop: Preferences → Resources → Memory: 8GB+ recommended

# Or limit specific container
# In docker-compose.yml, add:
# deploy:
#   resources:
#     limits:
#       memory: 2G
```

## Production Deployment

### Push to Docker Registry

```bash
# Tag image
docker tag dinoforge:latest myregistry.azurecr.io/dinoforge:v0.14.0

# Push to registry
docker push myregistry.azurecr.io/dinoforge:v0.14.0

# Deploy using helm, kubernetes, or Compose
```

### Security Best Practices

```dockerfile
# Use multi-stage builds (done above)
# Don't run as root
FROM mcr.microsoft.com/dotnet/runtime:11.0-preview-bookworm

RUN useradd -m -u 1000 dinoforge
USER dinoforge

# Set resource limits in docker-compose.yml
# Use environment variables for secrets (use .env.prod)
# Enable read-only root filesystem where possible
```

## Docker CLI Commands Reference

```bash
# Build
docker-compose build

# Start (detached)
docker-compose up -d

# Stop
docker-compose down

# View logs
docker-compose logs -f dinoforge-mcp

# Execute command in running container
docker-compose exec dinoforge-mcp bash

# Run one-off command
docker-compose run --rm dinoforge-mcp dotnet --version

# Remove volumes
docker-compose down -v

# Rebuild and restart
docker-compose down && docker-compose build && docker-compose up -d
```

## Next Steps

- [Explore example packs](/packs)
- [Windows deployment](/deploy/windows-deployment)
- [Linux deployment](/deploy/linux-deployment)
- [macOS deployment](/deploy/macos-deployment)
- [Troubleshooting guide](/deploy/troubleshooting)
