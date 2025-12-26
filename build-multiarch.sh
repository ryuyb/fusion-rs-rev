#!/bin/bash

# =============================================================================
# Multi-architecture build script using TARGETARCH and Docker Buildx
# =============================================================================

set -e

IMAGE_NAME=${IMAGE_NAME:-"fusion-rs"}
VERSION=${VERSION:-"latest"}

echo "🚀 Building multi-architecture images using Docker Buildx..."

# Ensure buildx is available
if ! docker buildx version > /dev/null 2>&1; then
    echo "❌ Docker buildx is not available. Please install Docker buildx."
    exit 1
fi

# Create builder if it doesn't exist
BUILDER_NAME="fusion-builder"
if ! docker buildx ls | grep -q "$BUILDER_NAME"; then
    echo "📦 Creating buildx builder: $BUILDER_NAME"
    docker buildx create --name "$BUILDER_NAME" --driver docker-container --bootstrap
fi

echo "🔧 Using buildx builder: $BUILDER_NAME"
docker buildx use "$BUILDER_NAME"

# Build for multiple platforms
echo "📦 Building for linux/amd64 and linux/arm64..."

if [ -n "$REGISTRY" ]; then
    # Build and push to registry
    echo "📤 Building and pushing to registry: $REGISTRY"
    docker buildx build \
        --platform linux/amd64,linux/arm64 \
        --tag "${REGISTRY}/${IMAGE_NAME}:${VERSION}" \
        --push \
        .
    
    echo "✅ Multi-arch image pushed to ${REGISTRY}/${IMAGE_NAME}:${VERSION}"
    
    # Inspect the manifest
    echo "🔍 Image manifest:"
    docker buildx imagetools inspect "${REGISTRY}/${IMAGE_NAME}:${VERSION}"
else
    # Build locally (can only load one platform at a time)
    echo "📦 Building for local use..."
    
    # Build for current platform and load
    docker buildx build \
        --platform linux/amd64 \
        --tag "${IMAGE_NAME}:${VERSION}-amd64" \
        --load \
        .
    
    docker buildx build \
        --platform linux/arm64 \
        --tag "${IMAGE_NAME}:${VERSION}-arm64" \
        --load \
        .
    
    echo "✅ Local images created:"
    echo "  - ${IMAGE_NAME}:${VERSION}-amd64"
    echo "  - ${IMAGE_NAME}:${VERSION}-arm64"
fi

echo "🎉 Build process completed!"

# Optional: Test the images
if [ "$TEST_IMAGES" = "true" ]; then
    echo "🧪 Testing images..."
    
    if [ -n "$REGISTRY" ]; then
        # Test multi-arch image
        docker run --rm --platform linux/amd64 "${REGISTRY}/${IMAGE_NAME}:${VERSION}" /app/fusion-rs --version || echo "AMD64 test failed"
        docker run --rm --platform linux/arm64 "${REGISTRY}/${IMAGE_NAME}:${VERSION}" /app/fusion-rs --version || echo "ARM64 test failed"
    else
        # Test local images
        docker run --rm "${IMAGE_NAME}:${VERSION}-amd64" /app/fusion-rs --version || echo "AMD64 test failed"
        docker run --rm "${IMAGE_NAME}:${VERSION}-arm64" /app/fusion-rs --version || echo "ARM64 test failed"
    fi
fi