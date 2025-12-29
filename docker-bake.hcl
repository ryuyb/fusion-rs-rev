// Docker Bake configuration for fusion-rs multi-arch builds

// ==== Variables ====

variable "REGISTRY" {
  default = "ghcr.io"
}

variable "IMAGE_NAME" {
  default = "fusion-rs"
}

variable "SOURCE_REPOSITORY_URL" {
  default = null
}

variable "SOURCE_COMMIT" {
  default = null
}

variable "SOURCE_VERSION" {
  default = null
}

// ==== Functions ====

function "labels" {
  params = []
  result = {
    "org.opencontainers.image.source" = "${SOURCE_REPOSITORY_URL}"
    "org.opencontainers.image.revision" = "${SOURCE_COMMIT}"
    "org.opencontainers.image.version" = "${SOURCE_VERSION}"
    "org.opencontainers.image.created" = "${formatdate("YYYY-MM-DD'T'hh:mm:ssZZZZZ", timestamp())}"
  }
}

// ==== Targets ====

// Default target attributes
target "_default_attributes" {
  dockerfile = "Dockerfile"
  context = "."
  labels = labels()
  args = {
    BUILDKIT_INLINE_CACHE = "1"
  }
}

// Multi-platform target for CI/CD - full image build
target "multi" {
  inherits = ["_default_attributes"]
  tags = ["${REGISTRY}/${IMAGE_NAME}"]
}

// Builder target for exporting binaries
// This target builds only the builder stage
target "builder" {
  inherits = ["_default_attributes"]
  target = "builder"
}
