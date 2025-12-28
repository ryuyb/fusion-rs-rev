#!/usr/bin/env bash
# =============================================================================
# Local Docker Build Script for fusion-rs
# Optimized multi-arch build with caching support
# =============================================================================

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
IMAGE_NAME="${IMAGE_NAME:-fusion-rs}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
DOCKERFILE="${DOCKERFILE:-Dockerfile.optimized}"
CACHE_DIR="${CACHE_DIR:-/tmp/.buildx-cache}"
PLATFORMS="${PLATFORMS:-linux/amd64}"

# Print colored message
print_msg() {
    local color=$1
    shift
    echo -e "${color}$*${NC}"
}

print_info() {
    print_msg "$BLUE" "ℹ️  $*"
}

print_success() {
    print_msg "$GREEN" "✅ $*"
}

print_warning() {
    print_msg "$YELLOW" "⚠️  $*"
}

print_error() {
    print_msg "$RED" "❌ $*"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
check_prerequisites() {
    print_info "Checking prerequisites..."

    if ! command_exists docker; then
        print_error "Docker is not installed"
        exit 1
    fi

    if ! docker buildx version >/dev/null 2>&1; then
        print_error "Docker Buildx is not available"
        exit 1
    fi

    print_success "Prerequisites check passed"
}

# Create buildx builder if needed
setup_builder() {
    print_info "Setting up Docker Buildx builder..."

    if ! docker buildx inspect multiarch >/dev/null 2>&1; then
        print_info "Creating new builder 'multiarch'..."
        docker buildx create --name multiarch --use
        docker buildx inspect --bootstrap
    else
        print_info "Using existing builder 'multiarch'..."
        docker buildx use multiarch
    fi

    print_success "Builder setup complete"
}

# Build image
build_image() {
    local push_flag=""
    local load_flag=""

    print_info "Building image: ${IMAGE_NAME}:${IMAGE_TAG}"
    print_info "Platforms: ${PLATFORMS}"
    print_info "Dockerfile: ${DOCKERFILE}"

    # Determine output mode
    if [[ "${PLATFORMS}" == *","* ]]; then
        # Multi-platform build requires push
        if [[ "${PUSH:-false}" == "true" ]]; then
            push_flag="--push"
            print_info "Multi-platform build: pushing to registry"
        else
            print_warning "Multi-platform build without push (image won't be loaded locally)"
        fi
    else
        # Single platform can be loaded locally
        load_flag="--load"
        print_info "Single platform build: loading to local Docker"
    fi

    # Build command
    print_info "Starting build..."
    docker buildx build \
        --platform "${PLATFORMS}" \
        --file "${DOCKERFILE}" \
        --tag "${IMAGE_NAME}:${IMAGE_TAG}" \
        --cache-from type=local,src="${CACHE_DIR}" \
        --cache-to type=local,dest="${CACHE_DIR}-new,mode=max" \
        ${push_flag} \
        ${load_flag} \
        .

    # Move cache (workaround for cache growing issue)
    if [[ -d "${CACHE_DIR}-new" ]]; then
        print_info "Updating cache..."
        rm -rf "${CACHE_DIR}"
        mv "${CACHE_DIR}-new" "${CACHE_DIR}"
    fi

    print_success "Build complete!"
}

# Show image info
show_image_info() {
    if docker image inspect "${IMAGE_NAME}:${IMAGE_TAG}" >/dev/null 2>&1; then
        print_info "Image information:"
        docker image inspect "${IMAGE_NAME}:${IMAGE_TAG}" \
            --format '  Size: {{.Size | printf "%.2f MB" | div 1048576}}'
        docker image inspect "${IMAGE_NAME}:${IMAGE_TAG}" \
            --format '  Created: {{.Created}}'
        docker image inspect "${IMAGE_NAME}:${IMAGE_TAG}" \
            --format '  Architecture: {{.Architecture}}'
    fi
}

# Run container for testing
run_container() {
    print_info "Starting container for testing..."

    docker run -d \
        --name fusion-rs-test \
        -p 8080:8080 \
        -e RUST_LOG=debug \
        "${IMAGE_NAME}:${IMAGE_TAG}"

    print_success "Container started: fusion-rs-test"
    print_info "Test the service: curl http://localhost:8080/health"
    print_info "View logs: docker logs -f fusion-rs-test"
    print_info "Stop container: docker stop fusion-rs-test && docker rm fusion-rs-test"
}

# Show usage
usage() {
    cat <<EOF
Usage: $0 [OPTIONS] [COMMAND]

Commands:
    build           Build Docker image (default)
    run             Build and run container for testing
    multiarch       Build multi-arch image (amd64 + arm64)
    clean           Clean build cache

Options:
    -t, --tag TAG       Image tag (default: latest)
    -n, --name NAME     Image name (default: fusion-rs)
    -p, --push          Push image to registry
    -h, --help          Show this help message

Environment Variables:
    IMAGE_NAME          Image name (default: fusion-rs)
    IMAGE_TAG           Image tag (default: latest)
    DOCKERFILE          Dockerfile path (default: Dockerfile.optimized)
    CACHE_DIR           Cache directory (default: /tmp/.buildx-cache)
    PLATFORMS           Target platforms (default: linux/amd64)
    PUSH                Push to registry (default: false)

Examples:
    # Build for current platform
    $0 build

    # Build with custom tag
    $0 -t v1.0.0 build

    # Build and test
    $0 run

    # Build multi-arch and push
    $0 -p multiarch

    # Clean cache
    $0 clean
EOF
}

# Clean cache
clean_cache() {
    print_info "Cleaning build cache..."
    rm -rf "${CACHE_DIR}" "${CACHE_DIR}-new"
    docker builder prune -f
    print_success "Cache cleaned"
}

# Main function
main() {
    local command="build"

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -t|--tag)
                IMAGE_TAG="$2"
                shift 2
                ;;
            -n|--name)
                IMAGE_NAME="$2"
                shift 2
                ;;
            -p|--push)
                PUSH="true"
                shift
                ;;
            build|run|multiarch|clean)
                command="$1"
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done

    # Execute command
    case $command in
        build)
            check_prerequisites
            setup_builder
            build_image
            show_image_info
            ;;
        run)
            check_prerequisites
            setup_builder
            build_image
            show_image_info
            run_container
            ;;
        multiarch)
            PLATFORMS="linux/amd64,linux/arm64"
            check_prerequisites
            setup_builder
            build_image
            ;;
        clean)
            clean_cache
            ;;
    esac
}

# Run main function
main "$@"
