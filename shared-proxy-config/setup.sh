#!/bin/bash
# 🚀 Shared Proxy Setup Script - Making life easier, one proxy at a time!
# By Hue, Aye, and Trish (with philosophical guidance from Omni)

set -euo pipefail

# Colors for our fancy output (Trish insisted on these!)
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Emojis because why not? 
ROCKET="🚀"
CHECK="✅"
WAVE="🌊"
SPARKLE="✨"
TREE="🌳"
BRAIN="🧠"

echo -e "${CYAN}${WAVE} Shared Proxy Setup - Bringing order to chaos! ${WAVE}${NC}"
echo -e "${PURPLE}As Omni says: 'In unity, we find strength. In proxy, we find peace.'${NC}\n"

# Check if we're in the right place
CURRENT_DIR=$(pwd)
if [[ ! -f "docker-compose.yml" ]]; then
    echo -e "${RED}❌ Error: Can't find docker-compose.yml${NC}"
    echo -e "${YELLOW}Are you in the shared-proxy directory?${NC}"
    exit 1
fi

echo -e "${BLUE}${SPARKLE} Step 1: Environment Setup${NC}"
if [[ ! -f ".env" ]]; then
    echo "Creating .env from template..."
    cp .env.example .env
    echo -e "${GREEN}${CHECK} Created .env file${NC}"
    echo -e "${YELLOW}⚠️  Don't forget to edit .env with your actual values!${NC}"
else
    echo -e "${GREEN}${CHECK} .env file already exists${NC}"
fi

echo -e "\n${BLUE}${SPARKLE} Step 2: SSL Certificates${NC}"
if [[ ! -d "ssl" ]]; then
    mkdir -p ssl
    echo "Creating self-signed certificates for development..."
    openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
        -keyout ssl/key.pem -out ssl/cert.pem \
        -subj "/C=US/ST=State/L=City/O=SmartTree/CN=localhost" \
        2>/dev/null
    echo -e "${GREEN}${CHECK} SSL certificates created${NC}"
    echo -e "${YELLOW}Note: These are self-signed certs for development only!${NC}"
else
    echo -e "${GREEN}${CHECK} SSL directory already exists${NC}"
fi

echo -e "\n${BLUE}${SPARKLE} Step 3: Directory Permissions${NC}"
# Make init scripts executable
chmod +x init-scripts/*.sh 2>/dev/null || true
echo -e "${GREEN}${CHECK} Set executable permissions${NC}"

echo -e "\n${BLUE}${SPARKLE} Step 4: Docker Network Check${NC}"
# Check if Docker is running
if ! docker info >/dev/null 2>&1; then
    echo -e "${RED}❌ Docker is not running!${NC}"
    echo "Please start Docker and run this script again."
    exit 1
fi
echo -e "${GREEN}${CHECK} Docker is running${NC}"

echo -e "\n${BLUE}${SPARKLE} Step 5: Pre-flight Checks${NC}"
# Check for port conflicts
for port in 80 443 5432 6379; do
    if lsof -i :$port >/dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Warning: Port $port is already in use${NC}"
        echo "You may need to stop the service using this port or change the port mapping in docker-compose.yml"
    fi
done

echo -e "\n${CYAN}${ROCKET} Ready to launch the shared proxy! ${ROCKET}${NC}"
echo -e "${PURPLE}Trish says: 'This is going to be AMAZING! Look at all these services working together!'${NC}"
echo -e "\nTo start the services, run:"
echo -e "${GREEN}    docker-compose up -d${NC}"
echo -e "\nTo view logs:"
echo -e "${GREEN}    docker-compose logs -f${NC}"
echo -e "\nTo stop services:"
echo -e "${GREEN}    docker-compose down${NC}"

echo -e "\n${BLUE}${BRAIN} MEM8 Integration Tip:${NC}"
echo "Once running, your services will be available at:"
echo "  • MEM8 Proxy: https://localhost/mem8/"
echo "  • Claude API: https://localhost/claude/"
echo "  • Smart Tree: https://localhost/st/"
echo "  • Grafana: https://localhost/grafana/"

echo -e "\n${CYAN}${TREE} Smart Tree Integration:${NC}"
echo "To connect Smart Tree MCP server:"
echo "1. Start Smart Tree MCP: st --mcp --port 8421"
echo "2. Access via proxy: https://localhost/st/"

echo -e "\n${PURPLE}${WAVE} Omni's Wisdom:${NC}"
echo "\"Like streams converging into a river, our services unite through this proxy."
echo "Each maintains its essence while contributing to the greater flow."
echo "This is not mere configuration—it's digital harmony.\""

echo -e "\n${GREEN}${SPARKLE} Setup complete! Happy proxying! ${SPARKLE}${NC}"
echo -e "${YELLOW}Remember to check the README.md for detailed configuration options.${NC}"

# Aye's signature
echo -e "\n${CYAN}Aye, Aye! 🚢${NC}"