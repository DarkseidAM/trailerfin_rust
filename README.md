# Trailerfin: Rust

Trailerfin is a Rust-based tool designed to scan directories for IMDb IDs and update trailer links in your media files. It fetches the latest trailers or videos from IMDb, ensuring your media library is always up-to-date with the latest content.
This is a rewrite of the original [Trailerfin](https://github.com/Pukabyte/trailerfin) by [Pukabyte](https://github.com/Pukabyte).

## Features
* Scans directories for IMDb IDs or TMDb IDs and updates trailer links
* Fetches the latest trailer or video from IMDb
* Supports scheduled automatic refreshes
* Configurable via environment variables
* Docker and Docker Compose support
* Robust logging for monitoring and troubleshooting
* Written in rust for performance and safety

## Requirements
* IMDb IDs in your media folder structure, or
  TMDB IDs in your media folder structure

## Configuration via Env Variables

```yaml
# Number of concurrent items to process.
# Optional, Defaults to '1'
TRAILERFIN_THREADS: "4"

# Your media mount path.
# Optional, Defaults to '/mnt/plex'
TRAILERFIN_SCAN_PATH: "/data"

# The path to the cache directory where the local embedded db is stored.
# Optional, Defaults to '/config'
# The cache is currently only used to store a lookup of tmdb to imdb ids to reduce the need to requery for them.
TRAILERFIN_CACHE_PATH: "/config"

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

# The data source to use for fetching trailers.
# Optional, Defaults to 'imdb'. Can be 'tmdb' or 'imdb'
# Note: If you use 'tmdb', you must provide a valid TMDB API key in TRAILERFIN_TMDB_API_KEY.
TRAILERFIN_DATA_SOURCE: "imdb"

# The TMDB API key to use if TRAILERFIN_DATA_SOURCE is set to 'tmdb'.
# Optional, Required if TRAILERFIN_DATA_SOURCE is 'tmdb', Defaults to 'None'
# This is used to look up the external imdb ID for the movie or TV show.
# This data is cached locally in the TRAILERFIN_CACHE_PATH directory.
TRAILERFIN_TMDB_API_KEY: "69blahblahlolblah420"

# Sets the internal rate limit for requests through the imdb client.
# Optional, Defaults to '30/minute'
TRAILERFIN_IMDB_RATE_LIMIT: "30/minute"

# Sets the internal rate limit for requests through the tmdb client.
# Optional, Defaults to '50/second'
TRAILERFIN_TMDB_RATE_LIMIT: "50/second"

# The movie folders to maintain trailers for. This is relative to TRAILERFIN_SCAN_PATH.
# Required if you want movie trailers, Defaults to 'None'
TRAILERFIN_MOVIE_FOLDERS: "Movies,Movies 4k"
                            
# The TV folders to maintain trailers for. This is relative to TRAILERFIN_SCAN_PATH.
# Required if you want TV trailers, Defaults to 'None'
TRAILERFIN_TV_FOLDERS: "TV Shows"

# The regex pattern to match IMDb IDs in folder names.
# Optional, Defaults to '{imdb-(tt\d+)}'
# Must contain a capture group for the IMDb ID (e.g., (tt\d+))
TRAILERFIN_IMDB_ID_REGEX: "{imdb-(tt\\d+)}"

# The regex pattern to match TMDB IDs in folder names.
# Optional, Defaults to '{tmdb-(\d+)}'
# Must contain a capture group for the TMDB ID (e.g., (\d+))
TRAILERFIN_TMDB_ID_REGEX: "{tmdb-(\\d+)}"
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
      TRAILERFIN_MOVIE_FOLDERS: "Movies,Movies 4k"
      TRAILERFIN_TV_FOLDERS: "TV Shows"
    restart: always
    volumes:
      - /mnt/plex:/mnt/plex:rshared
      - config:/config

volumes:
  config:
```
