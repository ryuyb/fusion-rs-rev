#!/bin/bash

# =============================================================================
# Multi-architecture build script using Zig cross-compilation
# =============================================================================

set -e

IMAGE_NAME=${IMAGE_NAME:-"fusion-rs"}
VERSION=${VERSION:-"latest"}

echo "🚀 Building multi-architecture images..."

# Build for AMD64
echo "📦 Building AMD64 image..."
docker build --target amd64 -t ${IMAGE_NAME}:${VERSION}-amd64 .

# Build for ARM64
echo "📦 Building ARM64 image..."
docker build --target arm64 -t ${IMAGE_NAME}:${VERSION}-arm64 .

echo "✅ Multi-architecture build completed!"
echo "Images created:"
echo "  - ${IMAGE_NAME}:${VERSION}-amd64"
echo "  - ${IMAGE_NAME}:${VERSION}-arm64"

# Optional: Create and push manifest (requires registry)
if [ -n "$REGISTRY" ]; then
    echo "📤 Creating and pushing multi-arch manifest..."
    
    # Tag images with registry
    docker tag ${IMAGE_NAME}:${VERSION}-amd64 ${REGISTRY}/${IMAGE_NAME}:${VERSION}-amd64
    docker tag ${IMAGE_NAME}:${VERSION}-arm64 ${REGISTRY}/${IMAGE_NAME}:${VERSION}-arm64
    
    # Push individual images
    docker push ${REGISTRY}/${IMAGE_NAME}:${VERSION}-amd64
    docker push ${REGISTRY}/${IMAGE_NAME}:${VERSION}-arm64
    
    # Create and push manifest
    docker manifest create ${REGISTRY}/${IMAGE_NAME}:${VERSION} \
        ${REGISTRY}/${IMAGE_NAME}:${VERSION}-amd64 \
        ${REGISTRY}/${IMAGE_NAME}:${VERSION}-arm64
    
    docker manifest push ${REGISTRY}/${IMAGE_NAME}:${VERSION}
    
    echo "✅ Multi-arch manifest pushed to ${REGISTRY}/${IMAGE_NAME}:${VERSION}"
fi

echo "🎉 Build process completed!"