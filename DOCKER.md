# Docker Setup for Trailerfin Rust

This document provides detailed instructions for building, running, and deploying Trailerfin Rust using Docker.

## 🐳 Quick Start

### Using Pre-built Images

```bash
# Pull the latest version
docker pull ghcr.io/your-username/trailerfin_rust:latest

# Run with basic configuration
docker run -d \
  --name trailerfin_rust \
  -v /path/to/your/media:/mnt/plex:ro \
  -v ./config:/config \
  -e TRAILERFIN_MOVIE_FOLDERS="Movies,Movies 4k" \
  -e TRAILERFIN_TV_FOLDERS="TV Shows" \
  ghcr.io/your-username/trailerfin_rust:latest
```

## 🏗️ Building Locally

### Prerequisites

- Docker installed and running
- Git repository cloned

### Build Commands

```bash
# Basic build
docker build -t trailerfin_rust .

# Build with specific tag
docker build -t trailerfin_rust:v1.1.0 .

# Build for specific platform
docker build --platform linux/amd64 -t trailerfin_rust:amd64 .
docker build --platform linux/arm64 -t trailerfin_rust:arm64 .
```

### Using the Build Script

We provide a convenient build script:

```bash
# Make it executable
chmod +x scripts/build-docker.sh

# Basic build
./scripts/build-docker.sh

# Build with specific tag
./scripts/build-docker.sh -t v1.1.0

# Build and push to registry
./scripts/build-docker.sh -r ghcr.io/your-username -t v1.1.0 -p
```

## 🚀 Running the Container

### Basic Run

```bash
docker run -d \
  --name trailerfin_rust \
  -v /path/to/your/media:/mnt/plex:ro \
  -v ./config:/config \
  -e TRAILERFIN_MOVIE_FOLDERS="Movies,Movies 4k" \
  -e TRAILERFIN_TV_FOLDERS="TV Shows" \
  trailerfin_rust:latest
```

### Advanced Configuration

```bash
docker run -d \
  --name trailerfin_rust \
  --restart unless-stopped \
  -v /mnt/media:/mnt/plex:ro \
  -v ./config:/config \
  -e TRAILERFIN_THREADS="4" \
  -e TRAILERFIN_SHOULD_SCHEDULE="true" \
  -e TRAILERFIN_SCHEDULE="0 0 0 * * *" \
  -e TRAILERFIN_MOVIE_FOLDERS="Movies,Movies 4k" \
  -e TRAILERFIN_TV_FOLDERS="TV Shows" \
  -e TRAILERFIN_DATA_SOURCE="imdb" \
  -e TRAILERFIN_IMDB_RATE_LIMIT="30/minute" \
  -e TRAILERFIN_TMDB_RATE_LIMIT="50/second" \
  -e TRAILERFIN_VIDEO_FILENAME="video1.strm" \
  trailerfin_rust:latest
```

### Using TMDB Data Source

```bash
docker run -d \
  --name trailerfin_rust \
  -v /path/to/your/media:/mnt/plex:ro \
  -v ./config:/config \
  -e TRAILERFIN_DATA_SOURCE="tmdb" \
  -e TRAILERFIN_TMDB_API_KEY="your-api-key-here" \
  -e TRAILERFIN_MOVIE_FOLDERS="Movies,Movies 4k" \
  -e TRAILERFIN_TV_FOLDERS="TV Shows" \
  trailerfin_rust:latest
```

### Custom Regex Patterns

```bash
docker run -d \
  --name trailerfin_rust \
  -v /path/to/your/media:/mnt/plex:ro \
  -v ./config:/config \
  -e TRAILERFIN_IMDB_ID_REGEX="\\[imdb-(tt\\d+)\\]" \
  -e TRAILERFIN_TMDB_ID_REGEX="\\[tmdb-(\\d+)\\]" \
  -e TRAILERFIN_MOVIE_FOLDERS="Movies,Movies 4k" \
  -e TRAILERFIN_TV_FOLDERS="TV Shows" \
  trailerfin_rust:latest
```

## 🐙 Docker Compose

### Basic Setup

1. Copy `docker-compose.yml` to your server
2. Update the media path in the volumes section
3. Configure your environment variables
4. Run:

```bash
docker-compose up -d
```

### Custom Configuration

Create a `docker-compose.override.yml` file for your specific needs:

```yaml
version: '3.8'

services:
  trailerfin_rust:
    environment:
      TRAILERFIN_MOVIE_FOLDERS: "Movies,4K Movies"
      TRAILERFIN_TV_FOLDERS: "TV Shows,Anime"
      TRAILERFIN_SCHEDULE: "0 2 * * *"  # 2 AM daily
    volumes:
      - /mnt/nas/media:/mnt/plex:ro
      - ./trailerfin-config:/config
    # Optional: Override user to match your system
    user: "1000:1000"  # Replace with your user/group IDs
```

## 🔧 Container Management

### Viewing Logs

```bash
# Follow logs
docker logs -f trailerfin_rust

# Last 100 lines
docker logs --tail 100 trailerfin_rust

# Since specific time
docker logs --since "2024-01-01T00:00:00" trailerfin_rust
```

### Health Checks

The container includes a health check that monitors the process:

```bash
# Check container health
docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Health}}"

# Inspect health check details
docker inspect trailerfin_rust | jq '.[0].State.Health'
```

### Updating the Container

```bash
# Stop and remove old container
docker stop trailerfin_rust
docker rm trailerfin_rust

# Pull latest image
docker pull ghcr.io/your-username/trailerfin_rust:latest

# Run new container
docker run -d \
  --name trailerfin_rust \
  -v /path/to/your/media:/mnt/plex:ro \
  -v ./config:/config \
  # ... your environment variables
  ghcr.io/your-username/trailerfin_rust:latest
```

## 🔒 Security Considerations

### User Configuration

The container runs as a non-root user (`trailerfin`) by default for security, but this can be overridden:

#### Default Behavior (Non-root)
```bash
# Check user inside container
docker exec trailerfin_rust whoami
# Output: trailerfin
```

#### Override User in Docker Compose
```yaml
services:
  trailerfin_rust:
    user: "1000:1000"  # Run as specific user/group
    # or
    user: "root"       # Run as root (not recommended)
```

#### Override User in Docker Run
```bash
docker run -d \
  --user "1000:1000" \
  --name trailerfin_rust \
  # ... other options
  trailerfin_rust:latest
```

### Read-only Media Mount

Media directories are mounted as read-only:

```bash
# This prevents accidental modifications
-v /path/to/your/media:/mnt/plex:ro
```

### Environment Variables

Sensitive data like API keys should be passed via environment variables:

```bash
# Use environment files for sensitive data
docker run -d \
  --env-file .env \
  trailerfin_rust:latest
```

Example `.env` file:
```env
TRAILERFIN_TMDB_API_KEY=your-secret-api-key
TRAILERFIN_SCHEDULE=0 0 0 * * *
```

## 🐛 Troubleshooting

### Common Issues

1. **Permission Denied**
   ```bash
   # Ensure config directory has correct permissions
   mkdir -p config
   chmod 755 config
   ```

2. **Container Exits Immediately**
   ```bash
   # Check logs for errors
   docker logs trailerfin_rust
   
   # Run interactively for debugging
   docker run -it --rm trailerfin_rust /bin/sh
   ```

3. **No Media Found**
   ```bash
   # Verify mount points
   docker exec trailerfin_rust ls -la /mnt/plex
   
   # Check environment variables
   docker exec trailerfin_rust env | grep TRAILERFIN
   ```

4. **Permission Issues with Config Directory**
   ```bash
   # Check current user
   docker exec trailerfin_rust whoami
   
   # Check directory permissions
   docker exec trailerfin_rust ls -la /config
   
   # Fix permissions (run as root temporarily)
   docker run --rm -v ./config:/config --user root trailerfin_rust:latest chown -R 1001:1001 /config
   ```

### Debug Mode

Run the container in debug mode:

```bash
docker run -it --rm \
  -v /path/to/your/media:/mnt/plex:ro \
  -v ./config:/config \
  -e TRAILERFIN_SHOULD_SCHEDULE="false" \
  -e TRAILERFIN_MOVIE_FOLDERS="Movies" \
  trailerfin_rust:latest
```

## 📊 Monitoring

### Resource Usage

```bash
# Monitor container resources
docker stats trailerfin_rust

# Check disk usage
docker exec trailerfin_rust df -h
```

### Log Analysis

```bash
# Search for errors
docker logs trailerfin_rust | grep -i error

# Count processed items
docker logs trailerfin_rust | grep -c "Refreshing trailer"
```

## 🔄 CI/CD Integration

The repository includes GitHub Actions workflows for automated Docker builds:

- `.github/workflows/docker-release.yaml` - Docker-only builds
- `.github/workflows/release-with-docker.yaml` - Combined binary and Docker builds

These workflows automatically build and push Docker images to GitHub Container Registry on tag releases.

## 📝 Environment Variables Reference

| Variable | Default | Required | Description |
|----------|---------|----------|-------------|
| `TRAILERFIN_SCAN_PATH` | `/mnt/plex` | Yes | Media mount path |
| `TRAILERFIN_CACHE_PATH` | `/config` | Yes | Cache directory path |
| `TRAILERFIN_THREADS` | `1` | No | Number of concurrent threads |
| `TRAILERFIN_SHOULD_SCHEDULE` | `false` | No | Enable scheduling |
| `TRAILERFIN_SCHEDULE` | `None` | If scheduling enabled | Cron schedule |
| `TRAILERFIN_MOVIE_FOLDERS` | `[]` | Yes* | Movie folder names |
| `TRAILERFIN_TV_FOLDERS` | `[]` | Yes* | TV folder names |
| `TRAILERFIN_DATA_SOURCE` | `imdb` | No | Data source (imdb/tmdb) |
| `TRAILERFIN_TMDB_API_KEY` | `None` | If TMDB source | TMDB API key |
| `TRAILERFIN_IMDB_ID_REGEX` | `{imdb-(tt\d+)}` | No | IMDb ID regex pattern |
| `TRAILERFIN_TMDB_ID_REGEX` | `{tmdb-(\d+)}` | No | TMDB ID regex pattern |

*At least one of movie or TV folders must be set.
