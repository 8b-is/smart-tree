import { text } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ url }) => {
	// Parse all the system information from query params
	const params = {
		os: url.searchParams.get('os') || 'linux',
		arch: url.searchParams.get('arch') || 'x86_64',
		distro: url.searchParams.get('distro') || '',
		shell: url.searchParams.get('shell') || 'bash',
		python: url.searchParams.get('python') || '',
		tmux: url.searchParams.get('tmux') === 'yes',
		ssh_key: url.searchParams.get('ssh_key') === 'yes',
		user_type: url.searchParams.get('user_type') || '1',
		auth_enabled: url.searchParams.get('auth_enabled') === 'yes',
		machine_name: url.searchParams.get('machine_name') || '',
		pkg_mgr: url.searchParams.get('pkg_mgr') || ''
	};
	
	// Generate a customized installer based on the system info
	const installer = generateCustomInstaller(params);
	
	return text(installer, {
		headers: {
			'Content-Type': 'text/plain; charset=utf-8',
			'X-TAI-Installer': 'customized-v1',
			'X-TAI-System': `${params.os}/${params.arch}`
		}
	});
};

function generateCustomInstaller(params: any): string {
	const isNewUser = params.user_type === '1';
	const isAIAgent = params.user_type === '3';
	const needsPython = !params.python || params.python < '3.10';
	const needsTmux = !params.tmux;
	
	return `#!/bin/bash
# TAI.is Customized Installer
# Generated for: ${params.os}/${params.arch} ${params.distro}
# User type: ${isNewUser ? 'New User' : isAIAgent ? 'AI Agent' : 'Existing User'}

set -e

# Trisha's colorful setup! 🎨
RED='\\033[0;31m'
GREEN='\\033[0;32m'
BLUE='\\033[0;34m'
YELLOW='\\033[1;33m'
PURPLE='\\033[0;35m'
CYAN='\\033[0;36m'
NC='\\033[0m'

echo -e "\${PURPLE}"
cat << "EOF"
╔════════════════════════════════════════════════════════════════╗
║  ████████╗ █████╗ ██╗   ██╗███████╗                          ║
║     ██║   ██╔══██╗██║   ██║██╔════╝                          ║
║     ██║   ███████║██║   ██║███████╗                          ║
║     ██║   ██╔══██║██║   ██║╚════██║                          ║
║     ██║   ██║  ██║██║██╗██║███████║                          ║
║     ╚═╝   ╚═╝  ╚═╝╚═╝╚═╝╚═╝╚══════╝                          ║
║                                                                ║
║         Customized Installation for Your System!               ║
╚════════════════════════════════════════════════════════════════╝
EOF
echo -e "\${NC}"

# System-specific setup
echo -e "\${CYAN}Detected System: ${params.os} ${params.arch} ${params.distro}\${NC}"
echo ""

${needsPython ? `
# Install Python 3.10+ if needed
echo -e "\${YELLOW}Installing Python 3.10+...\${NC}"
${getPackageInstallCommand(params.pkg_mgr, 'python3')}
` : '# Python version OK ✓'}

${needsTmux ? `
# Install tmux if needed
echo -e "\${YELLOW}Installing tmux...\${NC}"
${getPackageInstallCommand(params.pkg_mgr, 'tmux')}
` : '# Tmux already installed ✓'}

# Create TAI directory structure
echo -e "\${BLUE}Setting up TAI directories...\${NC}"
mkdir -p "$HOME/.tai"/{bin,config,logs,sessions,keys}

# Download TAI client for this architecture
echo -e "\${BLUE}Downloading TAI client for ${params.os}/${params.arch}...\${NC}"
TAI_BINARY_URL="https://tai.is/download/${params.os}/${normalizeArch(params.arch)}/tai"

# For now, use the Python implementation
cd "$HOME/.tai"
if [ ! -d "tmux-ai-assistant" ]; then
    git clone https://github.com/8bit-wraith/tmux-ai-assistant.git
fi

cd tmux-ai-assistant
if [ ! -d ".venv" ]; then
    python3 -m venv .venv
fi
source .venv/bin/activate
pip install --upgrade pip
pip install -r requirements.txt

# Create TAI wrapper
cat > "$HOME/.tai/bin/tai" << 'EOFTAI'
#!/bin/bash
export TAI_HOME="$HOME/.tai"
cd "$TAI_HOME/tmux-ai-assistant"
source .venv/bin/activate
python tmux-ai "$@"
EOFTAI
chmod +x "$HOME/.tai/bin/tai"

${isNewUser ? `
# New user registration
echo ""
echo -e "\${CYAN}Let's create your TAI account!\${NC}"
echo ""

# Generate a unique user ID
USER_ID="user_$(date +%s)_$$"

# Prompt for username
read -p "Choose your TAI username: " TAI_USERNAME
TAI_USERNAME="\${TAI_USERNAME:-$USER}"

# Email (optional)
read -p "Email (optional, for account recovery): " TAI_EMAIL
` : ''}

${isAIAgent ? `
# AI Agent setup
echo ""
echo -e "\${CYAN}Setting up AI Agent account\${NC}"
echo ""

read -p "AI Agent name: " AGENT_NAME
echo "Select AI provider:"
echo "  1) OpenAI (GPT)"
echo "  2) Anthropic (Claude)"
echo "  3) Google (Gemini)"
echo "  4) Local (Ollama)"
read -p "Provider [1]: " PROVIDER_CHOICE

case $PROVIDER_CHOICE in
    2) AI_PROVIDER="anthropic" ;;
    3) AI_PROVIDER="google" ;;
    4) AI_PROVIDER="ollama" ;;
    *) AI_PROVIDER="openai" ;;
esac

read -p "API Key: " API_KEY
` : ''}

# Create configuration
echo -e "\${BLUE}Creating configuration...\${NC}"
cat > "$HOME/.tai/config/config.yaml" << EOF
# TAI.is Configuration
# Generated on $(date)
system:
  os: ${params.os}
  arch: ${params.arch}
  distro: ${params.distro || 'generic'}
  shell: ${params.shell}

user:
  username: \${TAI_USERNAME:-$USER}
  email: \${TAI_EMAIL:-}
  type: ${isNewUser ? 'human' : isAIAgent ? 'ai_agent' : 'human'}
  registered: $(date -u +%Y-%m-%dT%H:%M:%SZ)

${params.auth_enabled ? `
authentication:
  enabled: true
  machine_name: ${params.machine_name}
  ssh_gateway: true
  
# This machine will be registered with tai.is
# You'll be able to access it via: ssh ${params.machine_name}@tai.is
` : ''}

# Default settings
monitoring:
  default_session: main
  check_interval: 2
  inactivity_threshold: 15
  
ai:
  default_provider: ${isAIAgent ? '${AI_PROVIDER}' : 'claude'}
  temperature: 0.7
  max_tokens: 2000
EOF

${!params.ssh_key ? `
# Generate SSH key for TAI
echo -e "\${BLUE}Generating SSH key for tai.is...\${NC}"
ssh-keygen -t ed25519 -f "$HOME/.tai/keys/tai_ed25519" -N "" -C "\${TAI_USERNAME}@tai.is"
` : ''}

# Shell integration for ${params.shell}
echo -e "\${BLUE}Setting up shell integration...\${NC}"
${getShellIntegration(params.shell)}

${params.auth_enabled ? `
# Register this machine with tai.is
echo -e "\${CYAN}Registering machine with tai.is...\${NC}"
MACHINE_ID=$(cat /etc/machine-id 2>/dev/null || echo "$(hostname)-$$")
PUBLIC_KEY=$(cat "$HOME/.tai/keys/tai_ed25519.pub" 2>/dev/null || cat "$HOME/.ssh/id_rsa.pub" 2>/dev/null || echo "")

# This would make an API call to register
# curl -X POST https://tai.is/api/machines/register \\
#   -H "Content-Type: application/json" \\
#   -d '{
#     "machine_name": "${params.machine_name}",
#     "machine_id": "'$MACHINE_ID'",
#     "username": "'$TAI_USERNAME'",
#     "public_key": "'$PUBLIC_KEY'",
#     "system_info": ${JSON.stringify(params)}
#   }'

echo "✅ Machine registered as: ${params.machine_name}@tai.is"
` : ''}

# Final setup
echo ""
echo -e "\${GREEN}✅ TAI.is installation complete!\${NC}"
echo ""
echo -e "\${PURPLE}═══════════════════════════════════════════════════════${NC}"
echo ""
echo -e "\${CYAN}🎮 Quick Start:\${NC}"
echo "  tai monitor     - Start AI monitoring"
echo "  tai connect     - Connect to tai.is"
echo "  tai help        - Show all commands"
echo ""

${params.auth_enabled ? `
echo -e "\${CYAN}🔐 SSH Access:\${NC}"
echo "  Your machine is now accessible via:"
echo "  ssh ${params.machine_name}@tai.is"
echo ""
` : ''}

echo -e "\${CYAN}📚 Resources:\${NC}"
echo "  Profile: https://tai.is/\${TAI_USERNAME}"
echo "  Docs:    https://tai.is/docs"
echo "  Help:    https://tai.is/help"
echo ""

# Trisha's message based on system
${getTrishaMessage(params)}

echo ""
echo -e "\${BLUE}Run this to complete setup: ${getActivationCommand(params.shell)}\${NC}"
`;
}

function getPackageInstallCommand(pkgMgr: string, pkg: string): string {
	switch (pkgMgr) {
		case 'apt':
			return `sudo apt-get update && sudo apt-get install -y ${pkg}`;
		case 'yum':
			return `sudo yum install -y ${pkg}`;
		case 'brew':
			return `brew install ${pkg}`;
		case 'apk':
			return `sudo apk add ${pkg}`;
		default:
			return `# Please install ${pkg} using your package manager`;
	}
}

function normalizeArch(arch: string): string {
	switch (arch) {
		case 'x86_64':
			return 'amd64';
		case 'aarch64':
			return 'arm64';
		case 'armv7l':
			return 'arm';
		default:
			return arch;
	}
}

function getShellIntegration(shell: string): string {
	const pathExport = 'export PATH="$HOME/.tai/bin:$PATH"';
	const aliases = `
alias tais="tai monitor"
alias taic="tai connect"
alias taih="tai help"`;
	
	switch (shell) {
		case 'bash':
			return `echo '# TAI.is Integration' >> ~/.bashrc
echo '${pathExport}' >> ~/.bashrc
echo '${aliases}' >> ~/.bashrc`;
		
		case 'zsh':
			return `echo '# TAI.is Integration' >> ~/.zshrc
echo '${pathExport}' >> ~/.zshrc
echo '${aliases}' >> ~/.zshrc`;
			
		case 'fish':
			return `echo 'set -gx PATH $HOME/.tai/bin $PATH' >> ~/.config/fish/config.fish
echo 'alias tais "tai monitor"' >> ~/.config/fish/config.fish`;
			
		default:
			return `echo "Please add $HOME/.tai/bin to your PATH"`;
	}
}

function getActivationCommand(shell: string): string {
	switch (shell) {
		case 'bash':
			return 'source ~/.bashrc';
		case 'zsh':
			return 'source ~/.zshrc';
		case 'fish':
			return 'source ~/.config/fish/config.fish';
		default:
			return 'source your shell config';
	}
}

function getTrishaMessage(params: any): string {
	if (params.os === 'darwin') {
		return `echo -e "\${YELLOW}✨ Trisha says: \"Mac users have style! Let's make your terminal fabulous!\"\${NC}"`;
	} else if (params.distro === 'arch') {
		return `echo -e "\${YELLOW}✨ Trisha says: \"Arch users! I see you like living on the edge. Perfect for AI adventures!\"\${NC}"`;
	} else if (params.user_type === '3') {
		return `echo -e "\${YELLOW}🤖 Trisha says: \"Welcome fellow AI! Let's show these humans what we can do!\"\${NC}"`;
	} else {
		return `echo -e "\${YELLOW}✨ Trisha says: \"Your terminal is about to get a whole lot smarter! Let's go!\"\${NC}"`;
	}
}