# Trailerfin: Rust

Trailerfin is a Rust-based tool designed to scan directories for IMDb IDs and update trailer links in your media files. It fetches the latest trailers or videos from IMDb, ensuring your media library is always up-to-date with the latest content.
This is a rewrite of the original [Trailerfin](https://github.com/Pukabyte/trailerfin) by [Pukabyte](https://github.com/Pukabyte).

## Features
* Scans directories for IMDb IDs and updates trailer links
* Fetches the latest trailer or video from IMDb
* Supports scheduled automatic refreshes
* Configurable via environment variables
* Docker and Docker Compose support
* Robust logging for monitoring and troubleshooting
* Written in rust for performance and safety

## Requirements
* IMDb IDs in your media folder structure

## Configuration via Env Variables

```yaml
# Number of concurrent items to process.
# Optional, Defaults to '1'
TRAILERFIN_THREADS: "4"

# Your media mount path.
# Optional, Defaults to '/mnt/plex'
TRAILERFIN_SCAN_PATH: "/data"

# The useragent to use when fetching trailers.
# Optional, Defaults to 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/124.0.0.0'.
TRAILERFIN_USER_AGENT: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/124.0.0.0"

# The filename to use for the trailer file.
# Optional, Defaults to 'video1.strm'
TRAILERFIN_VIDEO_FILENAME: "video.strm"

# Indicates whether to run on a schedule or not.
# Optional, Defaults to 'false'
TRAILERFIN_SHOULD_SCHEDULE: "true"

# The cron schedule for the trailer generation if TRAILERFIN_SHOULD_SCHEDULE is true.
# Optional, Required if TRAILERFIN_SHOULD_SCHEDULE is true, Defaults to 'None'
# Cron format for scheduling the trailer generation, 6 fields: seconds, minutes, hours, day of month, month, day of week
TRAILERFIN_SCHEDULE: "0 * * * * *"
```

## Docker

A container for this can be found in the repository [here](https://github.com/iPromKnight/containers/tree/main/apps/trailerfin_rust) and can be pulled from my github packages feed [here](https://github.com/users/iPromKnight/packages/container/package/trailerfin_rust)

A distroless docker container can be found: `ghcr.io/ipromknight/trailerfin_rust:rolling`

You can pull the image with:
```bash
docker pull ghcr.io/ipromknight/trailerfin_rust:rolling
```

This Image supports AMD64 and ARM64.

## Docker Compose

```yaml
services:
  trailerfin_rust:
    container_name: trailerfin_rust
    image: ghcr.io/ipromknight/trailerfin_rust:rolling
    environment:
      TRAILERFIN_THREADS: "4"
      TRAILERFIN_SHOULD_SCHEDULE: "true"
      TRAILERFIN_SCHEDULE: "0 0 0 * * *" # Midnight nightly.
    restart: always
    volumes:
      - /mnt/plex:/mnt/plex:rshared
```
