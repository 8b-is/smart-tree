#!/usr/bin/env python3
"""
🎪 Tmux AI Assistant - Interactive Launcher 🎪
The friendliest way to start your coding carnival!
"""

import os
import sys
import subprocess
import click
from colorama import init, Fore, Style, Back
import yaml
from typing import Optional, List, Dict
import time
import random
from pathlib import Path

# Initialize colorama
init(autoreset=True)

# Fun ASCII art
BANNER = f"""{Fore.CYAN}{Style.BRIGHT}
╔═══════════════════════════════════════════════════════════════╗
║                                                               ║
║        🎹 Tmux AI Assistant - Coding Carnival 🎪              ║
║                                                               ║
║     "Sing us a song, you're the piano man..." 🎵             ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝
{Style.RESET_ALL}"""

PIANO_KEYS = "🎹🎹🎹🎹🎹🎹🎹🎹🎹🎹"


class InteractiveLauncher:
    """The friendly interactive launcher for all your tmux AI needs!"""
    
    def __init__(self):
        self.config = self.load_config()
        self.session_info = {}
        self.choices = {}
        
    def load_config(self) -> dict:
        """Load existing configuration"""
        config_file = "config/config.yaml"
        if os.path.exists(config_file):
            with open(config_file, 'r') as f:
                return yaml.safe_load(f) or {}
        return {}
        
    def print_banner(self):
        """Print the welcome banner"""
        print(BANNER)
        
    def animated_print(self, text: str, delay: float = 0.02):
        """Print text with typewriter effect"""
        for char in text:
            print(char, end='', flush=True)
            time.sleep(delay)
        print()
        
    def get_choice(self, prompt: str, options: List[Dict[str, str]], 
                   allow_back: bool = True) -> str:
        """Get user choice with nice formatting"""
        print(f"\n{Fore.YELLOW}{prompt}{Fore.RESET}")
        
        # Add options
        for i, opt in enumerate(options, 1):
            icon = opt.get('icon', '•')
            print(f"  {Fore.CYAN}{i}){Fore.RESET} {icon} {opt['text']}")
            if 'desc' in opt:
                print(f"     {Style.DIM}{opt['desc']}{Style.RESET_ALL}")
                
        if allow_back:
            print(f"  {Fore.CYAN}0){Fore.RESET} ← Back")
            
        while True:
            try:
                choice = input(f"\n{Fore.GREEN}Your choice: {Fore.RESET}")
                
                if allow_back and choice == '0':
                    return 'back'
                    
                idx = int(choice) - 1
                if 0 <= idx < len(options):
                    return options[idx]['value']
                    
            except (ValueError, IndexError):
                pass
                
            print(f"{Fore.RED}Please enter a valid number{Fore.RESET}")
            
    def get_input(self, prompt: str, default: Optional[str] = None,
                  validator: Optional[callable] = None) -> str:
        """Get text input from user"""
        full_prompt = f"{Fore.YELLOW}{prompt}"
        if default:
            full_prompt += f" [{default}]"
        full_prompt += f": {Fore.RESET}"
        
        while True:
            value = input(full_prompt).strip()
            
            if not value and default:
                value = default
                
            if validator:
                valid, msg = validator(value)
                if not valid:
                    print(f"{Fore.RED}{msg}{Fore.RESET}")
                    continue
                    
            return value
            
    def get_yes_no(self, prompt: str, default: bool = True) -> bool:
        """Get yes/no answer"""
        default_str = "Y/n" if default else "y/N"
        
        while True:
            response = input(f"{Fore.YELLOW}{prompt} [{default_str}]: {Fore.RESET}").lower()
            
            if not response:
                return default
            elif response in ['y', 'yes']:
                return True
            elif response in ['n', 'no']:
                return False
            else:
                print(f"{Fore.RED}Please answer yes or no{Fore.RESET}")
                
    def choose_session_location(self):
        """Step 1: Where is your tmux session?"""
        self.animated_print(f"\n{PIANO_KEYS}\n", 0.05)
        
        options = [
            {
                'value': 'local',
                'text': 'Local tmux session',
                'desc': 'Running on this machine',
                'icon': '💻'
            },
            {
                'value': 'ssh',
                'text': 'Remote SSH tmux session',
                'desc': 'Connect to a remote server',
                'icon': '🌐'
            },
            {
                'value': 'docker',
                'text': 'Docker container tmux',
                'desc': 'Running inside a container',
                'icon': '🐳'
            },
            {
                'value': 'kubernetes',
                'text': 'Kubernetes pod tmux',
                'desc': 'Running in a K8s pod',
                'icon': '☸️'
            }
        ]
        
        choice = self.get_choice("Where is your tmux session?", options, allow_back=False)
        self.choices['location'] = choice
        
        # Get location-specific details
        if choice == 'ssh':
            self.get_ssh_details()
        elif choice == 'docker':
            self.get_docker_details()
        elif choice == 'kubernetes':
            self.get_kubernetes_details()
            
    def get_ssh_details(self):
        """Get SSH connection details"""
        print(f"\n{Fore.CYAN}SSH Connection Details:{Fore.RESET}")
        
        # Check for saved connections
        saved_hosts = self.config.get('saved_hosts', [])
        if saved_hosts:
            print(f"\n{Fore.GREEN}Saved connections:{Fore.RESET}")
            for i, host in enumerate(saved_hosts, 1):
                print(f"  {i}) {host['name']} ({host['user']}@{host['host']})")
                
            use_saved = self.get_yes_no("\nUse a saved connection?")
            if use_saved:
                # Let them pick one
                pass  # Simplified for this example
                
        self.session_info['ssh_host'] = self.get_input("SSH Host", validator=self.validate_hostname)
        self.session_info['ssh_user'] = self.get_input("SSH User", default=os.getenv('USER'))
        self.session_info['ssh_port'] = self.get_input("SSH Port", default="22")
        
        # SSH key or password?
        auth_method = self.get_choice("Authentication method:", [
            {'value': 'key', 'text': 'SSH Key', 'icon': '🔑'},
            {'value': 'password', 'text': 'Password', 'icon': '🔒'}
        ])
        
        if auth_method == 'key':
            default_key = os.path.expanduser("~/.ssh/id_rsa")
            self.session_info['ssh_key'] = self.get_input("SSH Key path", default=default_key)
            
        # Save this connection?
        if self.get_yes_no("\nSave this connection for future use?"):
            self.save_ssh_connection()
            
    def get_docker_details(self):
        """Get Docker container details"""
        print(f"\n{Fore.CYAN}Docker Container Details:{Fore.RESET}")
        
        # List running containers
        try:
            result = subprocess.run(['docker', 'ps', '--format', '{{.Names}}'], 
                                  capture_output=True, text=True)
            if result.returncode == 0:
                containers = result.stdout.strip().split('\n')
                if containers:
                    print(f"\n{Fore.GREEN}Running containers:{Fore.RESET}")
                    for i, container in enumerate(containers, 1):
                        print(f"  {i}) {container}")
                        
        except FileNotFoundError:
            print(f"{Fore.YELLOW}Docker not found{Fore.RESET}")
            
        self.session_info['container'] = self.get_input("Container name/ID")
        
    def get_kubernetes_details(self):
        """Get Kubernetes pod details"""
        print(f"\n{Fore.CYAN}Kubernetes Pod Details:{Fore.RESET}")
        
        self.session_info['namespace'] = self.get_input("Namespace", default="default")
        self.session_info['pod'] = self.get_input("Pod name")
        self.session_info['container'] = self.get_input("Container name (optional)")
        
    def choose_session(self):
        """Step 2: Choose or create tmux session"""
        print(f"\n{Fore.CYAN}Tmux Session Selection:{Fore.RESET}")
        
        # List existing sessions
        sessions = self.list_tmux_sessions()
        
        options = []
        if sessions:
            print(f"\n{Fore.GREEN}Existing sessions:{Fore.RESET}")
            for session in sessions:
                options.append({
                    'value': f'existing:{session}',
                    'text': f'Use existing: {session}',
                    'icon': '📂'
                })
                
        options.append({
            'value': 'new',
            'text': 'Create new session',
            'icon': '✨'
        })
        
        choice = self.get_choice("Which tmux session?", options)
        
        if choice == 'new':
            session_name = self.get_input("New session name", default="coding-carnival")
            self.session_info['session_name'] = session_name
            self.session_info['create_new'] = True
        else:
            self.session_info['session_name'] = choice.split(':')[1]
            self.session_info['create_new'] = False
            
    def list_tmux_sessions(self) -> List[str]:
        """List tmux sessions based on location"""
        location = self.choices.get('location', 'local')
        
        if location == 'local':
            try:
                result = subprocess.run(['tmux', 'list-sessions', '-F', '#{session_name}'],
                                      capture_output=True, text=True)
                if result.returncode == 0:
                    return result.stdout.strip().split('\n')
            except:
                pass
                
        # For remote sessions, would need to SSH and check
        return []
        
    def choose_mode(self):
        """Step 3: How do you want to interact?"""
        print(f"\n{Fore.CYAN}Interaction Mode:{Fore.RESET}")
        
        options = [
            {
                'value': 'monitor',
                'text': 'Monitor only',
                'desc': 'Watch and get AI suggestions',
                'icon': '👁️'
            },
            {
                'value': 'attach',
                'text': 'Attach as client',
                'desc': 'Join the session directly',
                'icon': '🔗'
            },
            {
                'value': 'collaborate',
                'text': 'Collaborative mode',
                'desc': 'AI assists with commands',
                'icon': '🤝'
            },
            {
                'value': 'web',
                'text': 'Web interface',
                'desc': 'Start web-based carnival',
                'icon': '🌐'
            },
            {
                'value': 'api',
                'text': 'API/MCP mode',
                'desc': 'For ChatGPT/Claude integration',
                'icon': '🔌'
            }
        ]
        
        self.choices['mode'] = self.get_choice("How would you like to interact?", options)
        
    def configure_sharing(self):
        """Step 4: Who can join the carnival?"""
        if self.choices['mode'] not in ['web', 'collaborate']:
            return
            
        print(f"\n{Fore.CYAN}Sharing & Collaboration:{Fore.RESET}")
        
        options = [
            {
                'value': 'private',
                'text': 'Private (just me)',
                'icon': '🔒'
            },
            {
                'value': 'team',
                'text': 'Team members only',
                'desc': 'Requires authentication',
                'icon': '👥'
            },
            {
                'value': 'public_view',
                'text': 'Public viewing',
                'desc': 'Anyone can watch',
                'icon': '👀'
            },
            {
                'value': 'public_suggest',
                'text': 'Public with suggestions',
                'desc': 'Anyone can suggest commands',
                'icon': '💡'
            }
        ]
        
        self.choices['sharing'] = self.get_choice("Who can join your coding carnival?", options)
        
        if self.choices['sharing'] in ['team', 'public_view', 'public_suggest']:
            self.configure_access()
            
    def configure_access(self):
        """Configure access control"""
        print(f"\n{Fore.CYAN}Access Configuration:{Fore.RESET}")
        
        if self.choices['sharing'] == 'team':
            # Get team members
            print("\nAdd team members (email or username, one per line, empty to finish):")
            members = []
            while True:
                member = input("  > ").strip()
                if not member:
                    break
                members.append(member)
                
            self.session_info['team_members'] = members
            
        # Port configuration for web
        if self.choices['mode'] == 'web':
            self.session_info['web_port'] = self.get_input(
                "Web interface port", 
                default="8080"
            )
            
            # External access?
            if self.get_yes_no("Allow external access? (not just localhost)"):
                self.session_info['bind_address'] = '0.0.0.0'
                print(f"{Fore.YELLOW}⚠️  Remember to configure firewall rules!{Fore.RESET}")
            else:
                self.session_info['bind_address'] = 'localhost'
                
    def configure_ai(self):
        """Step 5: AI configuration"""
        print(f"\n{Fore.CYAN}AI Assistant Configuration:{Fore.RESET}")
        
        # Check current config
        providers = self.config.get('providers', {})
        if providers:
            print(f"\nCurrent configuration:")
            print(f"  • Summarization: {providers.get('summarization', 'not set')}")
            print(f"  • Suggestions: {providers.get('next_step', 'not set')}")
            
            if not self.get_yes_no("\nChange AI configuration?", default=False):
                return
                
        options = [
            {
                'value': 'openai',
                'text': 'OpenAI (GPT-4)',
                'desc': 'Most powerful, requires API key',
                'icon': '🧠'
            },
            {
                'value': 'gemini',
                'text': 'Google Gemini',
                'desc': 'Fast and cost-effective',
                'icon': '✨'
            },
            {
                'value': 'ollama',
                'text': 'Ollama (Local)',
                'desc': 'Free, runs locally',
                'icon': '🏠'
            },
            {
                'value': 'mixed',
                'text': 'Mixed providers',
                'desc': 'Different AIs for different tasks',
                'icon': '🎨'
            },
            {
                'value': 'none',
                'text': 'No AI assistance',
                'icon': '🚫'
            }
        ]
        
        self.choices['ai_mode'] = self.get_choice("AI assistant preference:", options)
        
    def configure_features(self):
        """Step 6: Additional features"""
        print(f"\n{Fore.CYAN}Additional Features:{Fore.RESET}")
        
        features = []
        
        if self.get_yes_no("Enable command history recording?", default=True):
            features.append('history')
            
        if self.get_yes_no("Enable session recording/replay?", default=False):
            features.append('recording')
            
        if self.get_yes_no("Enable automatic error detection?", default=True):
            features.append('error_detection')
            
        if self.get_yes_no("Enable command suggestions?", default=True):
            features.append('suggestions')
            
        if self.choices['mode'] in ['collaborate', 'web']:
            if self.get_yes_no("Enable voting on suggestions?", default=True):
                features.append('voting')
                
        self.session_info['features'] = features
        
    def build_command(self) -> List[str]:
        """Build the command to run based on all choices"""
        cmd = []
        
        location = self.choices['location']
        mode = self.choices['mode']
        
        # Base command depends on mode
        if mode == 'monitor':
            if location == 'local':
                cmd = ['python', 'tmux_monitor.py']
            else:
                cmd = ['python', 'remote_tmux.py']
                
        elif mode == 'attach':
            if location == 'local':
                cmd = ['python', 'tmux_attach.py']
            else:
                cmd = ['python', 'remote_tmux.py', '--attach']
                
        elif mode == 'collaborate':
            cmd = ['python', 'tmux_client.py', '--mode', 'collaborate']
            
        elif mode == 'web':
            cmd = ['python', 'tmux_client.py', '--mode', 'spectate']
            cmd.extend(['--web-port', self.session_info.get('web_port', '8080')])
            
        elif mode == 'api':
            cmd = ['python', 'mcp_server.py']
            
        # Add session name
        cmd.append(self.session_info['session_name'])
        
        # Add location-specific options
        if location == 'ssh':
            cmd.extend(['--ssh-host', self.session_info['ssh_host']])
            cmd.extend(['--ssh-user', self.session_info['ssh_user']])
            if 'ssh_key' in self.session_info:
                cmd.extend(['--ssh-key', self.session_info['ssh_key']])
                
        elif location == 'docker':
            cmd.extend(['--docker', self.session_info['container']])
            
        elif location == 'kubernetes':
            cmd.extend(['--k8s-namespace', self.session_info['namespace']])
            cmd.extend(['--k8s-pod', self.session_info['pod']])
            
        # Add AI options
        if self.choices.get('ai_mode') == 'none':
            cmd.append('--no-ai')
            
        return cmd
        
    def save_session_config(self):
        """Save this session configuration for easy reuse"""
        print(f"\n{Fore.CYAN}Save Configuration:{Fore.RESET}")
        
        if self.get_yes_no("Save this configuration for easy reuse?"):
            name = self.get_input("Configuration name", 
                                default=self.session_info['session_name'])
            
            # Save to config
            saved_configs = self.config.get('saved_sessions', {})
            saved_configs[name] = {
                'choices': self.choices,
                'session_info': self.session_info,
                'timestamp': time.time()
            }
            
            # Write back
            config_file = "config/launcher_configs.yaml"
            os.makedirs("config", exist_ok=True)
            with open(config_file, 'w') as f:
                yaml.dump({'saved_sessions': saved_configs}, f)
                
            print(f"{Fore.GREEN}✓ Configuration saved as '{name}'")
            print(f"Next time, just run: ./tmux-ai-launcher.py --load {name}{Fore.RESET}")
            
    def show_summary(self):
        """Show a summary before launching"""
        print(f"\n{Fore.CYAN}{'='*60}")
        print("🎪 Ready to start your coding carnival! 🎪")
        print(f"{'='*60}{Fore.RESET}\n")
        
        print(f"{Fore.GREEN}Configuration Summary:{Fore.RESET}")
        print(f"  📍 Location: {self.choices['location']}")
        print(f"  📂 Session: {self.session_info['session_name']}")
        print(f"  🎮 Mode: {self.choices['mode']}")
        
        if 'sharing' in self.choices:
            print(f"  👥 Sharing: {self.choices['sharing']}")
            
        if 'web_port' in self.session_info:
            bind = self.session_info.get('bind_address', 'localhost')
            print(f"  🌐 Web URL: http://{bind}:{self.session_info['web_port']}")
            
        if self.choices.get('ai_mode'):
            print(f"  🤖 AI: {self.choices['ai_mode']}")
            
        features = self.session_info.get('features', [])
        if features:
            print(f"  ✨ Features: {', '.join(features)}")
            
        print(f"\n{Fore.YELLOW}Command to run:{Fore.RESET}")
        cmd = self.build_command()
        print(f"  {' '.join(cmd)}")
        
    def launch(self):
        """Actually launch the system"""
        if not self.get_yes_no("\n🚀 Ready to launch?", default=True):
            print(f"{Fore.YELLOW}Launch cancelled{Fore.RESET}")
            return
            
        cmd = self.build_command()
        
        print(f"\n{Fore.GREEN}Launching...{Fore.RESET}")
        self.animated_print(PIANO_KEYS, 0.05)
        
        # Create new session if needed
        if self.session_info.get('create_new'):
            if self.choices['location'] == 'local':
                subprocess.run(['tmux', 'new-session', '-d', '-s', 
                              self.session_info['session_name']])
                
        # Launch the command
        try:
            subprocess.run(cmd)
        except KeyboardInterrupt:
            print(f"\n{Fore.YELLOW}Thanks for joining the carnival! 🎪{Fore.RESET}")
            
    def validate_hostname(self, value: str) -> tuple[bool, str]:
        """Validate hostname/IP"""
        if not value:
            return False, "Hostname cannot be empty"
        # Simple validation - could be more thorough
        return True, ""
        
    def run_interactive(self):
        """Run the full interactive flow"""
        self.print_banner()
        
        # Check for saved sessions
        saved = self.config.get('saved_sessions', {})
        if saved:
            print(f"\n{Fore.GREEN}Quick launch saved configurations:{Fore.RESET}")
            for name, info in saved.items():
                print(f"  • {name}")
                
            if self.get_yes_no("\nUse a saved configuration?", default=False):
                # Load saved config
                pass  # Simplified for this example
                
        # Walk through the steps
        self.choose_session_location()
        self.choose_session()
        self.choose_mode()
        self.configure_sharing()
        self.configure_ai()
        self.configure_features()
        
        # Save config?
        self.save_session_config()
        
        # Show summary
        self.show_summary()
        
        # Launch!
        self.launch()
        
    def run_quick_launch(self, config_name: str):
        """Quick launch a saved configuration"""
        saved = self.config.get('saved_sessions', {})
        if config_name not in saved:
            print(f"{Fore.RED}Configuration '{config_name}' not found{Fore.RESET}")
            print("Available configurations:")
            for name in saved:
                print(f"  • {name}")
            return
            
        # Load the saved config
        config = saved[config_name]
        self.choices = config['choices']
        self.session_info = config['session_info']
        
        # Show summary and launch
        self.show_summary()
        self.launch()


@click.command()
@click.option('--load', help='Load a saved configuration')
@click.option('--list', 'list_configs', is_flag=True, help='List saved configurations')
def main(load, list_configs):
    """
    🎪 Tmux AI Assistant - Interactive Launcher
    
    The friendliest way to start your coding carnival!
    """
    
    launcher = InteractiveLauncher()
    
    if list_configs:
        saved = launcher.config.get('saved_sessions', {})
        if saved:
            print(f"{Fore.GREEN}Saved configurations:{Fore.RESET}")
            for name, info in saved.items():
                print(f"  • {name} - {info['choices']['mode']} mode")
        else:
            print("No saved configurations")
            
    elif load:
        launcher.run_quick_launch(load)
        
    else:
        launcher.run_interactive()


if __name__ == "__main__":
    main()