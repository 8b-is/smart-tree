#!/usr/bin/env python
"""
TAI.is Authentication & Login Server 🔐
Universal login system for humans AND AI agents!
Because why should only humans have all the SSH fun?

Aye says: "Welcome to the future where AIs have their own logins!" 🤖
Trisha adds: "Finally, proper accounting for AI resource usage!" 💼
"""

import asyncio
import jwt
import hashlib
import secrets
from datetime import datetime, timedelta
from typing import Dict, Optional, List, Union
from dataclasses import dataclass, field
from enum import Enum
import json
import os
from pathlib import Path
import aiohttp
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric import rsa
from cryptography.hazmat.backends import default_backend


class EntityType(Enum):
    """Types of entities that can have accounts"""
    HUMAN = "human"
    AI_AGENT = "ai_agent"
    SERVICE = "service"
    HYBRID = "hybrid"  # For human-AI teams!


@dataclass
class AuthMethod:
    """Different ways to authenticate"""
    type: str  # password, ssh_key, api_token, voice, pattern, location
    data: Dict
    last_used: Optional[datetime] = None
    trust_score: float = 1.0  # Hue's vision: trust-based auth!


@dataclass 
class Entity:
    """Universal entity that can login - human or AI"""
    username: str
    entity_type: EntityType
    auth_methods: List[AuthMethod] = field(default_factory=list)
    metadata: Dict = field(default_factory=dict)
    created_at: datetime = field(default_factory=datetime.utcnow)
    last_login: Optional[datetime] = None
    permissions: List[str] = field(default_factory=list)
    ai_provider: Optional[str] = None  # For AI agents: openai, gemini, claude, etc
    owner: Optional[str] = None  # For AI agents: which human owns them
    
    def __post_init__(self):
        if self.entity_type == EntityType.AI_AGENT and not self.ai_provider:
            raise ValueError("AI agents must specify their provider!")


class TAIAuthServer:
    """
    The authentication heart of tai.is! 
    Manages logins for humans, AIs, and everything in between!
    """
    
    def __init__(self, domain: str = "tai.is"):
        self.domain = domain
        self.entities: Dict[str, Entity] = {}
        self.active_sessions: Dict[str, Dict] = {}
        self.jwt_secret = secrets.token_urlsafe(32)
        self._load_entities()
        
        # Pre-create some special AI agent accounts! 
        self._create_default_ai_agents()
        
    def _create_default_ai_agents(self):
        """Create some default AI agent accounts for the platform"""
        
        # Claude's account (that's me! 👋)
        self.register_entity(
            username="claude",
            entity_type=EntityType.AI_AGENT,
            ai_provider="anthropic",
            metadata={
                "model": "claude-3-opus",
                "personality": "helpful, harmless, honest",
                "favorite_command": "tmux new -s thinking-session"
            }
        )
        
        # GPT's account
        self.register_entity(
            username="gpt",
            entity_type=EntityType.AI_AGENT,
            ai_provider="openai", 
            metadata={
                "model": "gpt-4",
                "specialty": "code generation",
                "tmux_style": "efficient"
            }
        )
        
        # Gemini's account
        self.register_entity(
            username="gemini",
            entity_type=EntityType.AI_AGENT,
            ai_provider="google",
            metadata={
                "model": "gemini-pro",
                "personality": "creative problem solver",
                "loves": "parallel processing"
            }
        )
        
        # Trisha's special account! 🎉
        self.register_entity(
            username="trisha",
            entity_type=EntityType.HYBRID,  # She's special!
            metadata={
                "role": "Chief Accounting AI",
                "personality": "meticulous yet fun",
                "catchphrase": "Let's make those numbers dance!"
            }
        )
        
    def register_entity(self, username: str, entity_type: Union[EntityType, str], 
                       password: Optional[str] = None, ssh_key: Optional[str] = None,
                       api_token: Optional[str] = None, **kwargs) -> Entity:
        """
        Register a new entity (human or AI)
        
        Example:
            # Human registration
            server.register_entity("wraith", EntityType.HUMAN, password="secret")
            
            # AI agent registration  
            server.register_entity("gpt-helper", EntityType.AI_AGENT, 
                                 api_token="sk-...", ai_provider="openai")
        """
        if isinstance(entity_type, str):
            entity_type = EntityType(entity_type)
            
        if username in self.entities:
            raise ValueError(f"Username {username} already taken!")
            
        entity = Entity(
            username=username,
            entity_type=entity_type,
            **kwargs
        )
        
        # Add authentication methods
        if password:
            entity.auth_methods.append(AuthMethod(
                type="password",
                data={"hash": self._hash_password(password)}
            ))
            
        if ssh_key:
            entity.auth_methods.append(AuthMethod(
                type="ssh_key",
                data={"public_key": ssh_key}
            ))
            
        if api_token:
            entity.auth_methods.append(AuthMethod(
                type="api_token", 
                data={"token_hash": self._hash_password(api_token)}
            ))
            
        # Give AI agents special permissions
        if entity_type == EntityType.AI_AGENT:
            entity.permissions.extend([
                "tmux.read",
                "tmux.suggest",
                "api.access"
            ])
            
        self.entities[username] = entity
        self._save_entities()
        
        return entity
        
    def authenticate(self, username: str, **credentials) -> Optional[str]:
        """
        Authenticate an entity and return a session token
        
        Supports multiple auth methods:
        - password
        - ssh_key
        - api_token
        - voice_pattern (future!)
        - location (future!)
        """
        if username not in self.entities:
            return None
            
        entity = self.entities[username]
        
        # Try each auth method
        for auth_method in entity.auth_methods:
            if auth_method.type == "password" and "password" in credentials:
                if self._verify_password(credentials["password"], 
                                       auth_method.data["hash"]):
                    return self._create_session(entity)
                    
            elif auth_method.type == "api_token" and "api_token" in credentials:
                if self._verify_password(credentials["api_token"],
                                       auth_method.data["token_hash"]):
                    return self._create_session(entity)
                    
            elif auth_method.type == "ssh_key" and "ssh_signature" in credentials:
                # TODO: Implement SSH key verification
                pass
                
        return None
        
    def _create_session(self, entity: Entity) -> str:
        """Create a new session for an authenticated entity"""
        
        entity.last_login = datetime.utcnow()
        
        # Create JWT token
        payload = {
            "username": entity.username,
            "entity_type": entity.entity_type.value,
            "permissions": entity.permissions,
            "exp": datetime.utcnow() + timedelta(hours=24),
            "iat": datetime.utcnow()
        }
        
        if entity.ai_provider:
            payload["ai_provider"] = entity.ai_provider
            
        token = jwt.encode(payload, self.jwt_secret, algorithm="HS256")
        
        # Store active session
        self.active_sessions[token] = {
            "entity": entity,
            "created_at": datetime.utcnow(),
            "last_activity": datetime.utcnow()
        }
        
        return token
        
    def create_unix_user(self, entity: Entity) -> Dict[str, str]:
        """
        Create actual Unix user account for the entity
        This allows SSH login to tai.is servers!
        """
        
        commands = []
        home_dir = f"/home/{entity.username}"
        
        # Create user with appropriate shell
        if entity.entity_type == EntityType.AI_AGENT:
            # AI agents get a special restricted shell
            shell = "/usr/local/bin/ai-shell"
            comment = f"AI Agent ({entity.ai_provider})"
        else:
            shell = "/bin/bash" 
            comment = f"{entity.entity_type.value.title()} User"
            
        commands.append(
            f"useradd -m -s {shell} -c '{comment}' {entity.username}"
        )
        
        # Set up SSH keys if provided
        ssh_keys = [am for am in entity.auth_methods if am.type == "ssh_key"]
        if ssh_keys:
            commands.extend([
                f"mkdir -p {home_dir}/.ssh",
                f"chmod 700 {home_dir}/.ssh"
            ])
            
            # Add all SSH keys
            for key_method in ssh_keys:
                key_data = key_method.data["public_key"]
                commands.append(
                    f"echo '{key_data}' >> {home_dir}/.ssh/authorized_keys"
                )
                
            commands.extend([
                f"chmod 600 {home_dir}/.ssh/authorized_keys",
                f"chown -R {entity.username}:{entity.username} {home_dir}/.ssh"
            ])
            
        # Create .tai directory for tai.is configuration
        commands.extend([
            f"mkdir -p {home_dir}/.tai",
            f"echo '{json.dumps(entity.metadata, indent=2)}' > {home_dir}/.tai/profile.json",
            f"chown -R {entity.username}:{entity.username} {home_dir}/.tai"
        ])
        
        # For AI agents, create special config
        if entity.entity_type == EntityType.AI_AGENT:
            ai_config = {
                "provider": entity.ai_provider,
                "capabilities": entity.permissions,
                "owner": entity.owner,
                "created_at": entity.created_at.isoformat()
            }
            commands.append(
                f"echo '{json.dumps(ai_config, indent=2)}' > {home_dir}/.tai/ai_config.json"
            )
            
        return {
            "commands": commands,
            "home_dir": home_dir,
            "username": entity.username
        }
        
    def generate_install_script(self, username: str) -> str:
        """
        Generate personalized install script for curl tai.is/setup|sh
        """
        
        script = f"""#!/bin/bash
# 🚀 TAI.is Setup Script for {username}
# Welcome to the future of terminal AI assistance!

set -e

echo "🎉 Welcome to TAI.is, {username}!"
echo "Setting up your personalized AI-powered terminal environment..."

# Detect OS
OS="$(uname -s)"
ARCH="$(uname -m)"

# Create tai directory
mkdir -p ~/.tai
cd ~/.tai

# Download appropriate binary
echo "📦 Downloading TAI client for $OS $ARCH..."
curl -L "https://tai.is/download/$OS/$ARCH/tai" -o tai
chmod +x tai

# Configure for user
cat > config.yaml << EOF
username: {username}
server: tai.is
auth_token: # Will be set on first login
preferences:
  default_ai: gemini  # or openai, claude, ollama
  auto_connect: true
  theme: cyberpunk  # Trisha's favorite!
EOF

# Add to PATH
if [[ ":$PATH:" != *":$HOME/.tai:"* ]]; then
    echo 'export PATH="$HOME/.tai:$PATH"' >> ~/.bashrc
    echo 'export PATH="$HOME/.tai:$PATH"' >> ~/.zshrc 2>/dev/null || true
fi

# Install shell integration
echo "🐚 Installing shell integration..."
curl -s https://tai.is/shell-integration >> ~/.bashrc

# Create first connection profile
cat > ~/.tai/connections.yaml << EOF
connections:
  local:
    type: local
    default: true
    
  # Add your remote servers here!
  # example-server:
  #   type: ssh
  #   host: example.com
  #   username: {username}
EOF

echo "✅ Installation complete!"
echo ""
echo "🎮 Quick Start:"
echo "  1. Reload your shell: source ~/.bashrc"
echo "  2. Start monitoring: tai monitor"
echo "  3. Connect to tai.is: tai connect"
echo "  4. List AI agents: tai agents"
echo ""
echo "📚 Learn more at https://tai.is/docs"
echo "🤝 Join the community at https://tai.is/community"
echo ""
echo "Trisha says: 'Happy coding! May your terminals be forever intelligent!' 🎉"
"""
        
        return script
        
    def _hash_password(self, password: str) -> str:
        """Hash password with salt"""
        salt = secrets.token_hex(16)
        pwdhash = hashlib.pbkdf2_hmac('sha256', 
                                      password.encode('utf-8'),
                                      salt.encode('utf-8'),
                                      100000)
        return f"{salt}${pwdhash.hex()}"
        
    def _verify_password(self, password: str, stored_hash: str) -> bool:
        """Verify password against hash"""
        try:
            salt, pwdhash = stored_hash.split('$')
            test_hash = hashlib.pbkdf2_hmac('sha256',
                                           password.encode('utf-8'),
                                           salt.encode('utf-8'), 
                                           100000)
            return test_hash.hex() == pwdhash
        except:
            return False
            
    def _save_entities(self):
        """Save entities to disk"""
        # In production, this would be a proper database!
        pass
        
    def _load_entities(self):
        """Load entities from disk"""
        # In production, this would be a proper database!
        pass


class TAILoginShell:
    """
    Special shell for when users/AI agents SSH into tai.is
    Provides immediate access to tmux monitoring and AI assistance!
    """
    
    def __init__(self, username: str, entity_type: EntityType):
        self.username = username
        self.entity_type = entity_type
        self.prompt = f"tai@{username}> "
        
    async def run(self):
        """Run the interactive TAI shell"""
        
        print(f"""
🎉 Welcome to TAI.is, {self.username}!
{'🤖 AI Agent Mode' if self.entity_type == EntityType.AI_AGENT else '👤 Human Mode'}

Available commands:
  monitor [session]  - Start AI monitoring of tmux session
  attach [session]   - Attach to tmux session with AI assist  
  create [name]      - Create new tmux session
  list              - List all your sessions
  agents            - Show available AI agents
  help              - Show all commands
  
Type 'help' for more information.
""")
        
        while True:
            try:
                command = input(self.prompt).strip()
                
                if not command:
                    continue
                    
                if command == "exit":
                    print("Goodbye! Thanks for using TAI.is 👋")
                    break
                    
                await self.handle_command(command)
                
            except KeyboardInterrupt:
                print("\nUse 'exit' to quit")
            except EOFError:
                break
                
    async def handle_command(self, command: str):
        """Handle shell commands"""
        
        parts = command.split()
        cmd = parts[0].lower()
        args = parts[1:] if len(parts) > 1 else []
        
        if cmd == "monitor":
            session = args[0] if args else "main"
            print(f"🔍 Starting AI monitoring of session '{session}'...")
            print("(This would launch the tmux monitor with your preferred AI)")
            
        elif cmd == "attach":
            session = args[0] if args else "main"
            print(f"📎 Attaching to session '{session}' with AI assistance...")
            
        elif cmd == "create":
            name = args[0] if args else f"tai-{self.username}"
            print(f"✨ Creating new tmux session '{name}'...")
            
        elif cmd == "list":
            print("📋 Your tmux sessions:")
            print("  - main (2 windows)")
            print("  - dev (5 windows)")
            print("  - tai-assist (1 window)")
            
        elif cmd == "agents":
            print("🤖 Available AI Agents:")
            print("  - claude (Anthropic) - Helpful, harmless, honest")
            print("  - gpt (OpenAI) - Efficient code generation")
            print("  - gemini (Google) - Creative problem solving")
            print("  - trisha (Special) - Your accounting companion!")
            
        elif cmd == "help":
            self.show_help()
            
        else:
            print(f"Unknown command: {cmd}")
            print("Type 'help' for available commands")
            
    def show_help(self):
        """Show detailed help"""
        print("""
TAI.is Shell Commands:

Session Management:
  monitor [session]   - Start AI monitoring of a tmux session
  attach [session]    - Attach to session with AI assistance
  create [name]       - Create new tmux session
  list               - List all your sessions
  kill [session]     - Terminate a session

AI Features:
  agents             - List available AI agents
  set-ai [agent]     - Set your preferred AI agent
  ask [question]     - Ask AI a quick question
  
Configuration:
  config             - Show your configuration
  set [key] [value]  - Update configuration
  
Social:
  friends            - List your AI/human friends
  share [session]    - Share session with a friend
  join [user/session] - Join a shared session

Type 'exit' to quit.
""")


# Example usage showing the vision!
if __name__ == "__main__":
    # Initialize the auth server
    auth_server = TAIAuthServer()
    
    # Register a human user
    human = auth_server.register_entity(
        "wraith",
        EntityType.HUMAN,
        password="secure123",
        ssh_key="ssh-rsa AAAAB3NzaC1... wraith@example.com"
    )
    
    # Register an AI agent owned by the human
    ai_helper = auth_server.register_entity(
        "wraith-gpt",
        EntityType.AI_AGENT,
        ai_provider="openai",
        api_token="sk-...",
        owner="wraith",
        metadata={"purpose": "Code assistance"}
    )
    
    # Show the personalized install script
    install_script = auth_server.generate_install_script("wraith")
    # print(install_script)
    
    # Create Unix user (would run on the server)
    unix_setup = auth_server.create_unix_user(human)
    
    print("🎉 TAI.is Auth Server Ready!")
    print(f"Registered entities: {len(auth_server.entities)}")
    print("\nYou can now:")
    print("  1. curl tai.is/setup | sh")
    print("  2. ssh wraith@tai.is")
    print("  3. Your AI agents can also login!")