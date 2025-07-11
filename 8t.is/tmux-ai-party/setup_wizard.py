#!/usr/bin/env python3
"""
🎉 Tmux AI Assistant - Interactive Setup Wizard 🎉
Because configuration should be fun, not frustrating!
Aye, Hue, and Trisha's configuration extravaganza! 🌟
"""

import os
import sys
import yaml
import click
from typing import Dict, Any, Optional, List
from colorama import init, Fore, Back, Style
import re
from pathlib import Path
import subprocess
import shutil
from datetime import datetime

# Initialize colorama for beautiful colors
init(autoreset=True)

# Fun ASCII art banner
BANNER = f"""{Fore.CYAN}{Style.BRIGHT}
╔══════════════════════════════════════════════════════════════════╗
║                                                                  ║
║      🤖 Tmux AI Assistant - Configuration Wizard 🤖              ║
║                                                                  ║
║      Welcome to the most fun setup experience ever!              ║
║      Let's get you configured in style!                          ║
║                                                                  ║
╚══════════════════════════════════════════════════════════════════╝
{Style.RESET_ALL}"""

class SetupWizard:
    """The magical configuration wizard that makes setup a breeze!"""
    
    def __init__(self):
        self.config = {}
        self.env_vars = {}
        self.vault_data = {"passwords": {}, "auto_responses": {}, "patterns": {}}
        self.config_dir = "config"
        self.first_run = self.detect_first_run()
        self.existing_config = {}
        self.existing_env = {}
        
        # Load existing configuration if reconfiguring
        self.load_existing_config()
        
    def detect_first_run(self) -> bool:
        """Detect if this is the first time running"""
        markers = [
            ".env",
            "config/config.yaml",
            "config/.wizard_complete"
        ]
        return not any(os.path.exists(marker) for marker in markers)
        
    def load_existing_config(self):
        """Load existing configuration for reconfigure mode"""
        # Load existing .env
        if os.path.exists(".env"):
            from dotenv import dotenv_values
            self.existing_env = dotenv_values(".env")
            
        # Load existing config.yaml
        config_file = os.path.join(self.config_dir, "config.yaml")
        if os.path.exists(config_file):
            with open(config_file, 'r') as f:
                self.existing_config = yaml.safe_load(f) or {}
                
        # Load existing vault
        vault_file = os.path.join(self.config_dir, "vault.yaml")
        if os.path.exists(vault_file):
            with open(vault_file, 'r') as f:
                self.vault_data = yaml.safe_load(f) or {
                    "passwords": {},
                    "auto_responses": {},
                    "patterns": {}
                }
        
    def print_section(self, title: str, icon: str = "🎯"):
        """Print a beautiful section header"""
        print(f"\n{Fore.YELLOW}{'=' * 60}")
        print(f"{icon} {Style.BRIGHT}{title}{Style.NORMAL}")
        print(f"{'=' * 60}{Fore.RESET}\n")
        
    def get_existing_value(self, *path):
        """Get a value from existing config using a path"""
        # Try existing_config first
        obj = self.existing_config
        for key in path:
            if isinstance(obj, dict) and key in obj:
                obj = obj[key]
            else:
                return None
        return obj
        
    def get_input(self, prompt: str, default: Optional[str] = None, 
                  secret: bool = False, required: bool = True,
                  validator: Optional[callable] = None,
                  existing_key: Optional[str] = None) -> str:
        """Get user input with validation and pretty formatting"""
        
        # Check for existing value
        if existing_key and not default:
            if existing_key in self.existing_env:
                default = self.existing_env[existing_key]
                if secret and default:
                    # Mask the existing value for security
                    default = default[:4] + "*" * (len(default) - 8) + default[-4:] if len(default) > 8 else "*" * len(default)
        
        # Build the prompt
        full_prompt = f"{Fore.CYAN}→ {prompt}"
        if default and not secret:
            full_prompt += f" {Fore.YELLOW}[{default}]"
        elif default and secret:
            full_prompt += f" {Fore.YELLOW}[{default}]"
        full_prompt += f": {Fore.RESET}"
        
        while True:
            if secret:
                import getpass
                value = getpass.getpass(full_prompt)
            else:
                value = input(full_prompt)
                
            # Use default if empty
            if not value and default:
                # If it's a masked secret, use the original value from env
                if secret and existing_key and existing_key in self.existing_env:
                    value = self.existing_env[existing_key]
                else:
                    value = default
                
            # Check required
            if required and not value:
                print(f"{Fore.RED}✗ This field is required!{Fore.RESET}")
                continue
                
            # Validate
            if validator and value:
                valid, message = validator(value)
                if not valid:
                    print(f"{Fore.RED}✗ {message}{Fore.RESET}")
                    continue
                    
            return value
            
    def get_choice(self, prompt: str, choices: List[str], 
                   default: Optional[str] = None,
                   existing_value: Optional[str] = None) -> str:
        """Get a choice from user with pretty menu"""
        # Use existing value as default if available
        if existing_value and not default:
            default = existing_value
            
        print(f"{Fore.CYAN}{prompt}{Fore.RESET}")
        if default:
            print(f"{Style.DIM}Current: {default}{Style.RESET_ALL}")
        
        for i, choice in enumerate(choices, 1):
            marker = "→" if choice == default else " "
            print(f"  {Fore.YELLOW}{i}{Fore.RESET}{marker} {choice}")
            
        while True:
            prompt_text = f"\n{Fore.CYAN}Select (1-{len(choices)})"
            if default:
                prompt_text += f" or Enter to keep [{default}]"
            prompt_text += f": {Fore.RESET}"
            
            selection = input(prompt_text)
            
            # Use default if empty
            if not selection and default:
                return default
            
            try:
                idx = int(selection) - 1
                if 0 <= idx < len(choices):
                    return choices[idx]
            except ValueError:
                pass
                
            # Check if they typed the choice directly
            if selection in choices:
                return selection
                
            print(f"{Fore.RED}✗ Invalid selection!{Fore.RESET}")
            
    def get_yes_no(self, prompt: str, default: bool = True) -> bool:
        """Get yes/no answer with default"""
        default_str = "Y/n" if default else "y/N"
        
        while True:
            response = input(f"{Fore.CYAN}→ {prompt} [{default_str}]: {Fore.RESET}").lower()
            
            if not response:
                return default
            elif response in ['y', 'yes']:
                return True
            elif response in ['n', 'no']:
                return False
            else:
                print(f"{Fore.RED}✗ Please answer yes or no{Fore.RESET}")
                
    def validate_api_key(self, key: str) -> tuple[bool, str]:
        """Validate API key format"""
        if key.startswith("sk-") and len(key) > 20:
            return True, ""
        return False, "Invalid API key format"
        
    def validate_session_name(self, name: str) -> tuple[bool, str]:
        """Validate tmux session name"""
        if re.match(r'^[a-zA-Z0-9_-]+$', name):
            return True, ""
        return False, "Session name can only contain letters, numbers, hyphens, and underscores"
        
    def setup_mandatory(self):
        """Setup mandatory configuration"""
        self.print_section("Mandatory Configuration", "🔑")
        
        print(f"{Fore.GREEN}First, let's set up the essentials!{Fore.RESET}\n")
        
        # AI Provider selection
        print(f"{Fore.YELLOW}1. Choose your AI providers:{Fore.RESET}")
        print(f"   You can use different AIs for different tasks!")
        print(f"   - {Fore.CYAN}OpenAI{Fore.RESET}: Most powerful, great for complex tasks")
        print(f"   - {Fore.CYAN}Gemini{Fore.RESET}: Cost-effective, good for summarization")
        print(f"   - {Fore.CYAN}Ollama{Fore.RESET}: Local, free, perfect for privacy!\n")
        
        providers = ["openai", "gemini", "ollama", "mixed"]
        
        # Determine existing choice
        existing_providers = self.get_existing_value("providers")
        existing_choice = "mixed" if existing_providers else None
        if existing_providers and isinstance(existing_providers, dict):
            # Check if it's single provider or mixed
            summ = existing_providers.get("summarization")
            next = existing_providers.get("next_step")
            if summ == next and summ in providers:
                existing_choice = summ
        
        provider_choice = self.get_choice(
            "Select AI provider strategy:",
            providers,
            default="mixed",
            existing_value=existing_choice
        )
        
        if provider_choice == "mixed":
            # Mixed provider setup
            existing_summ = self.get_existing_value("providers", "summarization")
            existing_next = self.get_existing_value("providers", "next_step")
            
            self.config["providers"] = {
                "summarization": self.get_choice(
                    "\nWhich AI for summarization (processing terminal output)?",
                    ["openai", "gemini", "ollama"],
                    default="gemini",
                    existing_value=existing_summ
                ),
                "next_step": self.get_choice(
                    "\nWhich AI for next step suggestions?",
                    ["openai", "gemini", "ollama"],
                    default="openai",
                    existing_value=existing_next
                )
            }
        else:
            # Single provider
            self.config["providers"] = {
                "summarization": provider_choice,
                "next_step": provider_choice
            }
            
        # Collect needed API keys
        needed_providers = set()
        needed_providers.add(self.config["providers"]["summarization"])
        needed_providers.add(self.config["providers"]["next_step"])
        
        print(f"\n{Fore.YELLOW}2. API Keys:{Fore.RESET}")
        
        if "openai" in needed_providers:
            self.env_vars["OPENAI_API_KEY"] = self.get_input(
                "OpenAI API key",
                secret=True,
                validator=self.validate_api_key,
                existing_key="OPENAI_API_KEY"
            )
            
        if "gemini" in needed_providers:
            self.env_vars["GEMINI_API_KEY"] = self.get_input(
                "Google Gemini API key",
                secret=True,
                existing_key="GEMINI_API_KEY"
            )
            
        if "ollama" in needed_providers:
            print(f"{Fore.GREEN}✓ Ollama doesn't need an API key! Just make sure it's running.{Fore.RESET}")
            
        # Default session name
        print(f"\n{Fore.YELLOW}3. Default tmux session:{Fore.RESET}")
        existing_session = self.get_existing_value("default_session")
        self.config["default_session"] = self.get_input(
            "Default session name to monitor",
            default=existing_session or "main",
            validator=self.validate_session_name
        )
        
    def setup_monitoring(self):
        """Setup monitoring configuration"""
        self.print_section("Monitoring Configuration", "👁️")
        
        if not self.get_yes_no("Configure monitoring settings?", default=True):
            return
            
        print(f"\n{Fore.GREEN}Let's tune how the monitor behaves!{Fore.RESET}\n")
        
        # Monitoring mode
        existing_mode = self.get_existing_value("monitoring_mode")
        mode_choices = [
            "prompt-based (original) - Wait for shell prompts",
            "continuous (v2) - Process activity continuously"
        ]
        existing_mode_choice = mode_choices[1] if existing_mode == "continuous" else mode_choices[0] if existing_mode else None
        
        mode = self.get_choice(
            "Monitoring mode:",
            mode_choices,
            default="continuous (v2) - Process activity continuously",
            existing_value=existing_mode_choice
        )
        
        self.config["monitoring_mode"] = "continuous" if "continuous" in mode else "prompt"
        
        if self.config["monitoring_mode"] == "continuous":
            # Continuous mode settings
            existing_monitoring = self.get_existing_value("monitoring") or {}
            self.config["monitoring"] = {
                "pause_threshold": float(self.get_input(
                    "Seconds of inactivity before processing",
                    default=str(existing_monitoring.get("pause_threshold", 15))
                )),
                "dead_threshold": float(self.get_input(
                    "Seconds before considering session dead",
                    default=str(existing_monitoring.get("dead_threshold", 120))
                )),
                "max_context_lines": int(self.get_input(
                    "Maximum lines before forced summarization",
                    default=str(existing_monitoring.get("max_context_lines", 500))
                ))
            }
        else:
            # Prompt-based settings
            existing_settings = self.get_existing_value("settings") or {}
            self.config["settings"] = {
                "check_interval": float(self.get_input(
                    "How often to check for prompts (seconds)",
                    default=str(existing_settings.get("check_interval", 1.0))
                )),
                "max_history_lines": int(self.get_input(
                    "Maximum lines to analyze",
                    default=str(existing_settings.get("max_history_lines", 1000))
                ))
            }
            
    def setup_prompts(self):
        """Setup system prompts"""
        self.print_section("System Prompts", "💭")
        
        if not self.get_yes_no("Customize AI system prompts?", default=False):
            return
            
        print(f"\n{Fore.GREEN}Let's personalize how the AI responds!{Fore.RESET}\n")
        
        # Summarization prompt
        print(f"{Fore.YELLOW}Summarization prompt (how to analyze terminal activity):{Fore.RESET}")
        print(f"{Style.DIM}Default: Concise technical summaries focusing on commands and errors{Style.RESET_ALL}")
        
        if self.get_yes_no("\nUse custom summarization prompt?", default=False):
            self.config.setdefault("system_prompts", {})["summarization"] = self.get_multiline_input(
                "Enter your custom summarization prompt"
            )
            
        # Next step prompt
        print(f"\n{Fore.YELLOW}Next step prompt (how to suggest commands):{Fore.RESET}")
        print(f"{Style.DIM}Default: Helpful suggestions with specific commands{Style.RESET_ALL}")
        
        if self.get_yes_no("\nUse custom next step prompt?", default=False):
            self.config.setdefault("system_prompts", {})["next_step"] = self.get_multiline_input(
                "Enter your custom next step prompt"
            )
            
    def get_multiline_input(self, prompt: str) -> str:
        """Get multiline input from user"""
        print(f"{Fore.CYAN}{prompt} (Enter '---' on a new line to finish):{Fore.RESET}")
        lines = []
        while True:
            line = input()
            if line == "---":
                break
            lines.append(line)
        return "\n".join(lines)
        
    def setup_automation(self):
        """Setup automation and vault"""
        self.print_section("Automation & Security", "🔐")
        
        if not self.get_yes_no("Configure automation features?", default=True):
            return
            
        print(f"\n{Fore.GREEN}Let's set up automated responses!{Fore.RESET}")
        print(f"{Fore.YELLOW}⚠️  Use with caution - automation can be powerful!{Fore.RESET}\n")
        
        # Enable automation
        existing_automation = self.get_existing_value("interactive", "automation_enabled")
        self.config["interactive"] = {
            "automation_enabled": self.get_yes_no(
                "Enable automation mode?",
                default=existing_automation if existing_automation is not None else False
            )
        }
        
        if not self.config["interactive"]["automation_enabled"]:
            print(f"{Fore.GREEN}✓ Automation disabled - you'll handle all prompts manually{Fore.RESET}")
            return
            
        # Common auto-responses
        print(f"\n{Fore.YELLOW}Configure common auto-responses:{Fore.RESET}")
        
        responses = {
            "Continue? (y/n)": self.get_yes_no("Auto-confirm generic continue prompts?"),
            "Save changes?": self.get_yes_no("Auto-save changes?"),
            "Overwrite existing?": self.get_yes_no("Auto-overwrite files?", default=False),
            "Delete?": self.get_yes_no("Auto-confirm deletions?", default=False),
        }
        
        for pattern, enabled in responses.items():
            if enabled:
                response = "y" if "?" in pattern else "Y"
                self.vault_data["auto_responses"][pattern] = response
                
        # Password storage
        if self.get_yes_no("\nStore any passwords for automation?", default=False):
            print(f"{Fore.YELLOW}Password storage (encrypted in production):{Fore.RESET}")
            
            while True:
                context = self.get_input(
                    "Password context (e.g., 'sudo on myserver')",
                    required=False
                )
                if not context:
                    break
                    
                password = self.get_input(
                    f"Password for '{context}'",
                    secret=True
                )
                
                # Store with context hash
                import hashlib
                context_hash = hashlib.sha256(context.encode()).hexdigest()[:16]
                self.vault_data["passwords"][context_hash] = password
                
                print(f"{Fore.GREEN}✓ Password stored for context: {context}{Fore.RESET}")
                
                if not self.get_yes_no("Add another password?", default=False):
                    break
                    
    def setup_advanced(self):
        """Setup advanced features"""
        self.print_section("Advanced Features", "🚀")
        
        if not self.get_yes_no("Configure advanced features?", default=False):
            return
            
        # Model selection
        print(f"\n{Fore.YELLOW}AI Model Selection:{Fore.RESET}")
        
        if "openai" in [self.config["providers"]["summarization"], 
                       self.config["providers"]["next_step"]]:
            existing_models = self.get_existing_value("models", "openai") or {}
            self.config.setdefault("models", {}).setdefault("openai", {})
            self.config["models"]["openai"]["summarization"] = self.get_input(
                "OpenAI model for summarization",
                default=existing_models.get("summarization", "gpt-4o")
            )
            self.config["models"]["openai"]["next_step"] = self.get_input(
                "OpenAI model for next steps",
                default=existing_models.get("next_step", "gpt-4o")
            )
            
        # Temperature settings
        print(f"\n{Fore.YELLOW}Temperature (creativity) settings:{Fore.RESET}")
        existing_temp = self.get_existing_value("settings", "temperature") or self.get_existing_value("temperature")
        self.config.setdefault("settings", {})["temperature"] = float(
            self.get_input(
                "Temperature (0.0-1.0, higher = more creative)",
                default=str(existing_temp or 0.7)
            )
        )
        
        # Logging
        existing_logging = self.get_existing_value("logging") or {}
        self.config["logging"] = {
            "verbose": self.get_yes_no(
                "Enable verbose logging?", 
                default=existing_logging.get("verbose", False)
            ),
            "log_interactions": self.get_yes_no(
                "Log all interactions?", 
                default=existing_logging.get("log_interactions", True)
            )
        }
        
    def create_tmux_session_check(self):
        """Check if user has tmux sessions"""
        try:
            result = subprocess.run(
                ["tmux", "list-sessions"],
                capture_output=True,
                text=True
            )
            
            if result.returncode == 0:
                sessions = result.stdout.strip().split('\n')
                print(f"\n{Fore.GREEN}Found {len(sessions)} tmux session(s):{Fore.RESET}")
                for session in sessions[:5]:  # Show first 5
                    print(f"  • {session}")
                if len(sessions) > 5:
                    print(f"  ... and {len(sessions) - 5} more")
            else:
                print(f"\n{Fore.YELLOW}No tmux sessions found.{Fore.RESET}")
                print(f"Start one with: {Fore.CYAN}tmux new -s {self.config.get('default_session', 'main')}{Fore.RESET}")
                
        except FileNotFoundError:
            print(f"\n{Fore.RED}tmux not found! Please install tmux first.{Fore.RESET}")
            
    def save_configuration(self):
        """Save all configuration files"""
        self.print_section("Saving Configuration", "💾")
        
        # Create config directory
        os.makedirs(self.config_dir, exist_ok=True)
        
        # Save .env file
        print(f"Saving API keys to .env...")
        # Merge with existing env vars (preserve any we didn't touch)
        final_env = self.existing_env.copy()
        final_env.update(self.env_vars)
        
        with open(".env", "w") as f:
            f.write("# Tmux AI Assistant Configuration\n")
            f.write(f"# Generated by setup wizard on {datetime.now()}\n\n")
            for key, value in final_env.items():
                f.write(f"{key}={value}\n")
        print(f"{Fore.GREEN}✓ .env file updated{Fore.RESET}")
        
        # Save config.yaml
        config_file = os.path.join(self.config_dir, "config.yaml")
        print(f"Saving configuration to {config_file}...")
        with open(config_file, "w") as f:
            f.write("# Tmux AI Assistant Configuration\n")
            f.write(f"# Generated by setup wizard on {datetime.now()}\n")
            f.write("# Run setup_wizard.py to reconfigure\n\n")
            yaml.dump(self.config, f, default_flow_style=False, sort_keys=False)
        print(f"{Fore.GREEN}✓ config.yaml created{Fore.RESET}")
        
        # Save vault.yaml if automation is enabled
        if self.vault_data["passwords"] or self.vault_data["auto_responses"]:
            vault_file = os.path.join(self.config_dir, "vault.yaml")
            print(f"Saving automation data to {vault_file}...")
            with open(vault_file, "w") as f:
                f.write("# Tmux AI Assistant - Secure Vault\n")
                f.write("# ⚠️  Contains sensitive data - DO NOT COMMIT!\n")
                f.write(f"# Generated on {datetime.now()}\n\n")
                yaml.dump(self.vault_data, f, default_flow_style=False)
            print(f"{Fore.GREEN}✓ vault.yaml created{Fore.RESET}")
            
        # Create wizard completion marker
        marker_file = os.path.join(self.config_dir, ".wizard_complete")
        with open(marker_file, "w") as f:
            f.write(f"Setup completed on {datetime.now()}\n")
            f.write(f"Mode: {self.config.get('monitoring_mode', 'prompt')}\n")
            
    def show_next_steps(self):
        """Show what to do next"""
        self.print_section("Setup Complete! 🎉", "✅")
        
        mode = self.config.get("monitoring_mode", "prompt")
        session = self.config.get("default_session", "main")
        
        print(f"{Fore.GREEN}Your Tmux AI Assistant is ready to go!{Fore.RESET}\n")
        
        print(f"{Fore.YELLOW}Next steps:{Fore.RESET}")
        print(f"\n1. Start a tmux session (if you haven't already):")
        print(f"   {Fore.CYAN}tmux new -s {session}{Fore.RESET}")
        
        if mode == "continuous":
            print(f"\n2. Run the continuous monitor:")
            print(f"   {Fore.CYAN}./scripts/run-continuous-monitor.sh {session}{Fore.RESET}")
            if self.config.get("interactive", {}).get("automation_enabled"):
                print(f"   {Fore.YELLOW}Or with automation:{Fore.RESET}")
                print(f"   {Fore.CYAN}./scripts/run-continuous-monitor.sh {session} --auto{Fore.RESET}")
        else:
            print(f"\n2. Run the monitor:")
            print(f"   {Fore.CYAN}./tmux_monitor.py {session}{Fore.RESET}")
            
        print(f"\n3. For MCP server integration:")
        print(f"   {Fore.CYAN}./scripts/run-openai-mcp.sh {session}{Fore.RESET}")
        
        print(f"\n{Fore.GREEN}Happy coding with your new AI assistant! 🚀{Fore.RESET}")
        print(f"\n{Style.DIM}Run {Fore.CYAN}python setup_wizard.py{Style.RESET_ALL} anytime to reconfigure")
        
    def run(self):
        """Run the complete setup wizard"""
        print(BANNER)
        
        if self.first_run:
            print(f"{Fore.GREEN}Welcome to your first setup! Let's get you configured.{Fore.RESET}")
        else:
            print(f"{Fore.YELLOW}Welcome back! Let's update your configuration.{Fore.RESET}")
            if not self.get_yes_no("\nProceed with reconfiguration?", default=True):
                print(f"{Fore.YELLOW}Setup cancelled.{Fore.RESET}")
                return
                
        try:
            # Run through all setup sections
            self.setup_mandatory()
            self.setup_monitoring()
            self.setup_prompts()
            self.setup_automation()
            self.setup_advanced()
            
            # Check tmux
            self.create_tmux_session_check()
            
            # Save everything
            self.save_configuration()
            
            # Show next steps
            self.show_next_steps()
            
        except KeyboardInterrupt:
            print(f"\n\n{Fore.YELLOW}Setup cancelled by user.{Fore.RESET}")
            sys.exit(1)
        except Exception as e:
            print(f"\n\n{Fore.RED}Error during setup: {e}{Fore.RESET}")
            sys.exit(1)


@click.command()
@click.option("--reconfigure", is_flag=True, help="Force reconfiguration even if already set up")
def main(reconfigure):
    """
    Interactive setup wizard for Tmux AI Assistant
    
    This wizard will guide you through:
    - API key configuration
    - AI provider selection
    - Monitoring preferences
    - Automation setup
    - Advanced features
    """
    wizard = SetupWizard()
    
    if not wizard.first_run and not reconfigure:
        print(f"{Fore.YELLOW}Setup already complete!{Fore.RESET}")
        print(f"Run with {Fore.CYAN}--reconfigure{Fore.RESET} to change settings")
        
        # Show current config summary
        if os.path.exists("config/config.yaml"):
            with open("config/config.yaml", "r") as f:
                config = yaml.safe_load(f)
                
            print(f"\n{Fore.GREEN}Current configuration:{Fore.RESET}")
            providers = config.get("providers", {})
            print(f"  • Summarization: {providers.get('summarization', 'not set')}")
            print(f"  • Next steps: {providers.get('next_step', 'not set')}")
            print(f"  • Mode: {config.get('monitoring_mode', 'prompt-based')}")
        return
        
    wizard.run()


if __name__ == "__main__":
    main()