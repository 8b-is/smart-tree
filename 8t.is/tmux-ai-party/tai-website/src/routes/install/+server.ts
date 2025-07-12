import { json, text } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import UAParser from 'ua-parser-js';

export const GET: RequestHandler = async ({ request, url }) => {
	const userAgent = request.headers.get('user-agent') || '';
	const parser = new UAParser(userAgent);
	const ua = parser.getResult();
	
	// Check if this is curl/wget
	const isCurl = userAgent.toLowerCase().includes('curl');
	const isWget = userAgent.toLowerCase().includes('wget');
	
	if (!isCurl && !isWget) {
		// Return a redirect to the main page for browsers
		return new Response(null, {
			status: 302,
			headers: {
				'Location': '/'
			}
		});
	}
	
	// For curl/wget, return the detection script
	const detectionScript = `#!/bin/bash
# TAI.is Smart Installer Detection Script
# This script detects your system and downloads the perfect installer!

set -e

# Colors for Trisha's style! 💅
RED='\\033[0;31m'
GREEN='\\033[0;32m'
BLUE='\\033[0;34m'
YELLOW='\\033[1;33m'
PURPLE='\\033[0;35m'
CYAN='\\033[0;36m'
NC='\\033[0m' # No Color

echo -e "\${PURPLE}TAI.is Smart Installer\${NC}"
echo "Detecting your system..."
echo ""

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"
KERNEL="$(uname -r)"
DISTRO=""
SHELL_TYPE="\${SHELL##*/}"

# Detect Linux distribution
if [ "$OS" = "Linux" ]; then
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO="$ID"
        DISTRO_VERSION="$VERSION_ID"
    elif [ -f /etc/redhat-release ]; then
        DISTRO="rhel"
    elif [ -f /etc/debian_version ]; then
        DISTRO="debian"
    fi
fi

# Detect package manager
PKG_MANAGER=""
if command -v apt-get >/dev/null 2>&1; then
    PKG_MANAGER="apt"
elif command -v yum >/dev/null 2>&1; then
    PKG_MANAGER="yum"
elif command -v brew >/dev/null 2>&1; then
    PKG_MANAGER="brew"
elif command -v apk >/dev/null 2>&1; then
    PKG_MANAGER="apk"
fi

# Check Python version
PYTHON_VERSION=""
if command -v python3 >/dev/null 2>&1; then
    PYTHON_VERSION=$(python3 --version 2>&1 | awk '{print $2}')
elif command -v python >/dev/null 2>&1; then
    PYTHON_VERSION=$(python --version 2>&1 | awk '{print $2}')
fi

# Check for tmux
HAS_TMUX="no"
TMUX_VERSION=""
if command -v tmux >/dev/null 2>&1; then
    HAS_TMUX="yes"
    TMUX_VERSION=$(tmux -V | awk '{print $2}')
fi

# Check SSH setup
HAS_SSH_KEY="no"
if [ -f "$HOME/.ssh/id_rsa.pub" ] || [ -f "$HOME/.ssh/id_ed25519.pub" ]; then
    HAS_SSH_KEY="yes"
fi

# System info summary
echo -e "\${GREEN}System Information:\${NC}"
echo "  OS:           $OS"
echo "  Architecture: $ARCH"
echo "  Distribution: \${DISTRO:-N/A}"
echo "  Shell:        $SHELL_TYPE"
echo "  Package Mgr:  \${PKG_MANAGER:-none}"
echo "  Python:       \${PYTHON_VERSION:-not found}"
echo "  Tmux:         $HAS_TMUX \${TMUX_VERSION}"
echo "  SSH Key:      $HAS_SSH_KEY"
echo ""

# User type detection
echo -e "\${CYAN}Let's set up your TAI account!\${NC}"
echo ""
echo "Are you:"
echo "  1) New user - First time using TAI"
echo "  2) Existing user - Already have a TAI account"
echo "  3) AI Agent - Setting up an AI service account"
read -p "Select (1-3) [1]: " USER_TYPE
USER_TYPE="\${USER_TYPE:-1}"

# Authentication services
echo ""
echo -e "\${CYAN}Authentication Services\${NC}"
echo "Would you like TAI.is to provide authentication services for this machine?"
echo "This allows:"
echo "  • SSH access through tai.is gateway"
echo "  • Centralized user management"
echo "  • AI agents to access this machine"
echo ""
read -p "Enable authentication services? [y/N]: " ENABLE_AUTH
ENABLE_AUTH="\${ENABLE_AUTH:-N}"

# Machine registration
MACHINE_NAME=""
if [[ "\$ENABLE_AUTH" =~ ^[Yy]$ ]]; then
    echo ""
    read -p "Machine name (for tai.is registry): " MACHINE_NAME
    MACHINE_NAME="\${MACHINE_NAME:-$(hostname)}"
fi

# Build the download URL with all parameters
PARAMS="os=\${OS,,}&arch=$ARCH&distro=\${DISTRO}&shell=$SHELL_TYPE"
PARAMS="\${PARAMS}&python=\${PYTHON_VERSION}&tmux=$HAS_TMUX&ssh_key=$HAS_SSH_KEY"
PARAMS="\${PARAMS}&user_type=$USER_TYPE&auth_enabled=\${ENABLE_AUTH,,}"
PARAMS="\${PARAMS}&machine_name=\${MACHINE_NAME}&pkg_mgr=\${PKG_MANAGER}"

echo ""
echo -e "\${BLUE}Downloading customized installer...\${NC}"
echo ""

# Download and run the customized installer
curl -sSL "https://tai.is/setup/generate?\${PARAMS}" | sh

# Alternative for debugging:
# echo "Would download from: https://tai.is/setup/generate?\${PARAMS}"
`;

	return text(detectionScript, {
		headers: {
			'Content-Type': 'text/plain; charset=utf-8',
			'X-TAI-Installer': 'smart-detection-v1'
		}
	});
};