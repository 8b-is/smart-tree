#!/usr/bin/env bash
# Smart Tree Feedback Worker Deployment Script
# Mean ass efficient deployment to various targets

set -euo pipefail

# Colors
RED=$'\033[0;31m'
GREEN=$'\033[0;32m'
YELLOW=$'\033[1;33m'
BLUE=$'\033[0;34m'
NC=$'\033[0m'

# Config
REGISTRY=${REGISTRY:-ghcr.io/8b-is}
WORKER_IMAGE="smart-tree-feedback-worker"
API_IMAGE="smart-tree-feedback-api"
VERSION=$(git describe --tags --always --dirty)

print_info() { echo "${BLUE}[INFO]${NC} $1"; }
print_success() { echo "${GREEN}[SUCCESS]${NC} $1"; }
print_error() { echo "${RED}[ERROR]${NC} $1" >&2; }
print_warning() { echo "${YELLOW}[WARNING]${NC} $1"; }

usage() {
    cat << EOF
Usage: $0 [deploy-type] [options]

Deploy Types:
  local       Deploy to local Docker (default)
  hetzner     Deploy to Hetzner Cloud
  registry    Push to container registry only

Options:
  --no-build  Skip building containers
  --tag TAG   Use specific tag (default: git describe)

Examples:
  $0 local
  $0 hetzner --tag v1.0.0
  $0 registry --no-build
EOF
}

build_containers() {
    print_info "Building containers..."
    
    # Build worker
    docker build -t ${REGISTRY}/${WORKER_IMAGE}:${VERSION} \
                 -t ${REGISTRY}/${WORKER_IMAGE}:latest \
                 .
    
    # Build API if exists
    if [[ -d "../feedback-api" ]]; then
        docker build -t ${REGISTRY}/${API_IMAGE}:${VERSION} \
                     -t ${REGISTRY}/${API_IMAGE}:latest \
                     ../feedback-api/
    fi
    
    print_success "Containers built!"
}

deploy_local() {
    print_info "Deploying to local Docker..."
    
    # Check if docker-compose exists
    if [[ ! -f "docker-compose.yml" ]]; then
        print_error "docker-compose.yml not found!"
        exit 1
    fi
    
    # Start services
    docker-compose up -d
    
    # Wait for health
    sleep 5
    
    # Check status
    docker-compose ps
    
    print_success "Local deployment complete!"
    print_info "Worker logs: docker-compose logs -f feedback-worker"
}

deploy_hetzner() {
    print_info "Deploying to Hetzner Cloud..."
    
    # Check for hcloud CLI
    if ! command -v hcloud &> /dev/null; then
        print_error "hcloud CLI not found! Install from: https://github.com/hetznercloud/cli"
        exit 1
    fi
    
    # Check for required env vars
    if [[ -z "${HETZNER_TOKEN:-}" ]]; then
        print_error "HETZNER_TOKEN environment variable required!"
        exit 1
    fi
    
    # Server config
    SERVER_NAME="smart-tree-feedback-${VERSION//\./-}"
    SERVER_TYPE="cx11"  # 1 vCPU, 2GB RAM - perfect for feedback worker
    IMAGE="ubuntu-22.04"
    LOCATION="nbg1"
    
    print_info "Creating server: $SERVER_NAME"
    
    # Create cloud-init file with env vars substituted
    CLOUD_INIT_FILE="/tmp/cloud-init-${VERSION}.yaml"
    envsubst < ../cloud-init/hetzner-feedback-worker.yaml > "$CLOUD_INIT_FILE"
    
    # Create server
    hcloud server create \
        --name "$SERVER_NAME" \
        --type "$SERVER_TYPE" \
        --image "$IMAGE" \
        --location "$LOCATION" \
        --user-data-from-file "$CLOUD_INIT_FILE" \
        --label "service=smart-tree-feedback" \
        --label "version=$VERSION"
    
    # Get server IP
    SERVER_IP=$(hcloud server ip "$SERVER_NAME")
    
    print_success "Hetzner deployment complete!"
    print_info "Server: $SERVER_NAME"
    print_info "IP: $SERVER_IP"
    print_info "Metrics: http://$SERVER_IP:8080/"
    
    # Cleanup
    rm -f "$CLOUD_INIT_FILE"
}

deploy_registry() {
    print_info "Pushing to container registry..."
    
    # Check if logged in to registry
    if ! docker pull ${REGISTRY}/test 2>/dev/null; then
        print_warning "Not logged in to ${REGISTRY}"
        print_info "Run: docker login ${REGISTRY}"
    fi
    
    # Push images
    docker push ${REGISTRY}/${WORKER_IMAGE}:${VERSION}
    docker push ${REGISTRY}/${WORKER_IMAGE}:latest
    
    if [[ -d "../feedback-api" ]]; then
        docker push ${REGISTRY}/${API_IMAGE}:${VERSION}
        docker push ${REGISTRY}/${API_IMAGE}:latest
    fi
    
    print_success "Images pushed to registry!"
}

# Main
main() {
    local deploy_type="${1:-local}"
    local skip_build=false
    
    # Parse options
    shift || true
    while [[ $# -gt 0 ]]; do
        case $1 in
            --no-build)
                skip_build=true
                shift
                ;;
            --tag)
                VERSION="$2"
                shift 2
                ;;
            --help|-h)
                usage
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
    
    print_info "Deployment version: $VERSION"
    
    # Build containers unless skipped
    if [[ "$skip_build" != "true" ]]; then
        build_containers
    fi
    
    # Deploy based on type
    case "$deploy_type" in
        local)
            deploy_local
            ;;
        hetzner)
            deploy_registry  # Push to registry first
            deploy_hetzner
            ;;
        registry)
            deploy_registry
            ;;
        *)
            print_error "Unknown deploy type: $deploy_type"
            usage
            exit 1
            ;;
    esac
}

# Run main
main "$@"