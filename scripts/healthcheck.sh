#!/bin/sh
# =============================================================================
# Health Check Script for fusion-rs
# =============================================================================
# This script is designed for Docker HEALTHCHECK and general monitoring.
# It checks the application's health endpoints and returns appropriate exit codes.
#
# Exit Codes:
#   0 - Healthy
#   1 - Unhealthy
#
# Environment Variables:
#   FUSION_SERVER__HOST - Server host (default: localhost)
#   FUSION_SERVER__PORT - Server port (default: 8080)
#   HEALTHCHECK_TIMEOUT - Request timeout in seconds (default: 5)
#   HEALTHCHECK_RETRIES - Number of retry attempts (default: 3)
#   HEALTHCHECK_RETRY_DELAY - Delay between retries in seconds (default: 1)
#
# Usage:
#   ./healthcheck.sh [endpoint] [host] [port]
#
# Examples:
#   ./healthcheck.sh                      # Uses default: /health/live
#   ./healthcheck.sh /health              # Full health check
#   ./healthcheck.sh /health/ready        # Readiness check
#   ./healthcheck.sh /health/live         # Liveness check (default)
#   ./healthcheck.sh /health/ready 0.0.0.0 8080  # Override host and port
# =============================================================================

set -e

# Configuration - Read from environment variables with fallbacks
ENDPOINT="${1:-/health/live}"                          # Default to liveness probe
HOST="${2:-${FUSION_SERVER__HOST:-localhost}}"        # From env or default to localhost
PORT="${3:-${FUSION_SERVER__PORT:-8080}}"             # From env or default to 8080
TIMEOUT="${HEALTHCHECK_TIMEOUT:-5}"                   # Timeout in seconds
MAX_RETRIES="${HEALTHCHECK_RETRIES:-3}"               # Number of retries
RETRY_DELAY="${HEALTHCHECK_RETRY_DELAY:-1}"           # Delay between retries

URL="http://${HOST}:${PORT}${ENDPOINT}"

# Function to check health with curl (preferred)
check_with_curl() {
    curl -f -s -o /dev/null -w "%{http_code}" \
        --max-time "${TIMEOUT}" \
        "${URL}"
}

# Function to check health with wget (fallback)
check_with_wget() {
    wget -q -O /dev/null -T "${TIMEOUT}" \
        --server-response \
        "${URL}" 2>&1 | grep -q "200 OK"
    return $?
}

# Function to perform health check
perform_check() {
    # Try curl first
    if command -v curl >/dev/null 2>&1; then
        HTTP_CODE=$(check_with_curl)
        if [ "$HTTP_CODE" = "200" ]; then
            return 0
        else
            echo "Health check failed with HTTP code: ${HTTP_CODE}" >&2
            return 1
        fi
    # Fallback to wget
    elif command -v wget >/dev/null 2>&1; then
        if check_with_wget; then
            return 0
        else
            echo "Health check failed with wget" >&2
            return 1
        fi
    else
        echo "Error: Neither curl nor wget is available" >&2
        return 1
    fi
}

# Main execution with retry logic
attempt=1
while [ $attempt -le $MAX_RETRIES ]; do
    if [ $attempt -gt 1 ]; then
        echo "Retry attempt ${attempt}/${MAX_RETRIES}..." >&2
        sleep "${RETRY_DELAY}"
    fi

    if perform_check; then
        echo "Health check passed: ${URL}" >&2
        exit 0
    fi

    attempt=$((attempt + 1))
done

echo "Health check failed after ${MAX_RETRIES} attempts: ${URL}" >&2
exit 1
