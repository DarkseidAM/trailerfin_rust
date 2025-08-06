#!/bin/bash

# Build script for trailerfin_rust Docker image

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
IMAGE_NAME="trailerfin_rust"
TAG="latest"
REGISTRY=""
PUSH=false

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  -t, --tag TAG         Docker image tag (default: latest)"
    echo "  -r, --registry REG    Docker registry (e.g., ghcr.io/username)"
    echo "  -p, --push            Push image to registry after build"
    echo "  -h, --help            Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Build with tag 'latest'"
    echo "  $0 -t v1.1.0                         # Build with tag 'v1.1.0'"
    echo "  $0 -r ghcr.io/username -t v1.1.0 -p  # Build and push to registry"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--tag)
            TAG="$2"
            shift 2
            ;;
        -r|--registry)
            REGISTRY="$2"
            shift 2
            ;;
        -p|--push)
            PUSH=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Build the full image name
if [[ -n "$REGISTRY" ]]; then
    FULL_IMAGE_NAME="${REGISTRY}/${IMAGE_NAME}:${TAG}"
else
    FULL_IMAGE_NAME="${IMAGE_NAME}:${TAG}"
fi

print_status "Building Docker image: $FULL_IMAGE_NAME"

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed or not in PATH"
    exit 1
fi

# Build the image
print_status "Starting Docker build..."
if docker build -t "$FULL_IMAGE_NAME" .; then
    print_status "Docker build completed successfully!"
    
    # Show image info
    print_status "Image details:"
    docker images "$FULL_IMAGE_NAME"
    
    # Push if requested
    if [[ "$PUSH" == true ]]; then
        if [[ -z "$REGISTRY" ]]; then
            print_error "Registry must be specified when pushing"
            exit 1
        fi
        
        print_status "Pushing image to registry..."
        if docker push "$FULL_IMAGE_NAME"; then
            print_status "Image pushed successfully!"
        else
            print_error "Failed to push image"
            exit 1
        fi
    fi
    
    print_status "Build completed successfully!"
else
    print_error "Docker build failed"
    exit 1
fi
