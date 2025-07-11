#!/usr/bin/env python3
"""
Tmux AI Assistant - Monitors tmux sessions and generates intelligent next steps
Aye here! Ready to make your tmux sessions smarter, Hue! 🚀
"""

import time
import re
import os
import sys
import asyncio
import logging
from typing import List, Optional, Tuple, Dict
from datetime import datetime
import libtmux
import click
from openai import OpenAI
import google.generativeai as genai  # Aye, welcome Gemini to the party!
import ollama  # Welcome Ollama - the local hero Trisha loves! 🏠
from dotenv import load_dotenv
import yaml
from colorama import init, Fore, Style
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

# Initialize colorama for cross-platform colored output
init(autoreset=True)

# Set up logger - Trisha loves keeping detailed records!
logger = logging.getLogger(__name__)

# Suppress libtmux debug logging unless verbose
logging.getLogger('libtmux').setLevel(logging.WARNING)
logging.getLogger('libtmux.common').setLevel(logging.WARNING)

# Load environment variables - because secrets are like Trish's favorite snacks, best kept hidden!
load_dotenv()


class ConfigWatcher(FileSystemEventHandler):
    """Watches for changes to configuration files - hot reload baby!
    This ensures our AI assistant is always up-to-date without a restart.
    """

    def __init__(self, monitor):
        self.monitor = monitor

    def on_modified(self, event):
        # Only react to changes in markdown or yaml config files
        if event.src_path.endswith(".md") or event.src_path.endswith(".yaml"):
            print(f"{Fore.YELLOW}Config file changed: {event.src_path}")
            self.monitor.reload_config()


class TmuxAIMonitor:
    """Main monitor class - the brain of our operation!
    This class orchestrates the tmux session monitoring and AI interactions.
    """

    def __init__(
        self,
        session_name: str,
        ai_provider: Optional[str] = None,
        openai_api_key: Optional[str] = None,
        gemini_api_key: Optional[str] = None,
        config_dir: str = "config",
    ):
        """
        Initialize the AI-powered tmux monitor.

        Args:
            session_name: The name of the tmux session to monitor.
            ai_provider: The AI model provider to use (deprecated - use config.yaml).
            openai_api_key: API key for OpenAI.
            gemini_api_key: API key for Google Gemini.
            config_dir: Directory where configuration files are stored.
        """
        self.session_name = session_name
        self.config_dir = config_dir

        supported_providers = {"openai", "gemini", "ollama"}
        if ai_provider and ai_provider.lower() not in supported_providers:
            raise ValueError(f"Unsupported AI provider: {ai_provider}. Supported providers are {supported_providers}")


        # Load configuration first to get provider settings
        self.config = self.load_config()

        # Determine providers from config (allows mixed providers!)
        if ai_provider:
            # Legacy support - single provider for both tasks
            print(
                f"{Fore.YELLOW}Note: Single provider mode is deprecated. Use config.yaml for mixed providers!"
            )
            self.summarization_provider = ai_provider.lower()
            self.next_step_provider = ai_provider.lower()
        else:
            # New way - read from config!
            self.summarization_provider = self.config.get("providers", {}).get(
                "summarization", "openai"
            )
            self.next_step_provider = self.config.get("providers", {}).get(
                "next_step", "openai"
            )

        print(f"{Fore.CYAN}Providers configured:")
        print(f"  - Summarization: {self.summarization_provider}")
        print(f"  - Next Steps: {self.next_step_provider}")

        # Initialize AI clients based on the providers needed
        self.openai_client = None
        self.gemini_client = None
        self.ollama_client = None

        # Initialize clients for each unique provider
        providers_needed = set([self.summarization_provider, self.next_step_provider])

        if "openai" in providers_needed:
            if not openai_api_key:
                raise ValueError(
                    "OpenAI API key is required when using OpenAI provider."
                )
            self.openai_client = OpenAI(api_key=openai_api_key)
            print(f"{Fore.GREEN}Initialized OpenAI client. Ready to chat! 💬")

        if "gemini" in providers_needed:
            if not gemini_api_key:
                raise ValueError(
                    "Google Gemini API key is required when using Gemini provider."
                )
            genai.configure(api_key=gemini_api_key)
            print(
                f"{Fore.GREEN}Initialized Google Gemini client. Let's get generative! ✨"
            )

        if "ollama" in providers_needed:
            try:
                models = ollama.list()
                print(
                    f"{Fore.GREEN}Initialized Ollama client. Running locally - no cloud fees! 💰"
                )
                print(
                    f"{Fore.CYAN}Available models: {', '.join([m['name'] for m in models['models']])}"
                )
            except Exception as e:
                raise ValueError(
                    f"Ollama is not running or not installed. Please start Ollama first: {e}"
                )

        # Initialize tmux server connection
        self.server = libtmux.Server()
        self.session = None
        self.pane = None

        # Monitoring state
        self.last_interaction_line = 0
        self.learned_prompts = set()  # Store dynamically learned prompts
        self.prompt_learning_enabled = self.config.get("prompt_learning", True)
        self.min_prompt_confidence = self.config.get("min_prompt_confidence", 0.8)
        
        # Base prompt patterns
        self.base_prompt_patterns = [
            r" > $",  # Default prompt pattern from user example
            r"\$ $",  # Standard shell prompt
            r">>> $",  # Python prompt
            r"In \[\d+\]: $",  # IPython prompt
            r"mysql> $",  # MySQL prompt
            r": $",  # Colon prompt
            r"# $",  # Root prompt
        ]
        
        # Combined patterns (base + learned)
        self.prompt_patterns = self.base_prompt_patterns.copy()

        # Load system prompts
        self.system_prompts = self.load_system_prompts()
        # Keep legacy single prompt for backward compatibility
        self.system_prompt = self.system_prompts.get(
            "next_step", self.system_prompts.get("default", "")
        )
        
        # Load any previously learned prompts
        self.load_learned_prompts()

        # Start config file watcher
        self.start_config_watcher()

    def start_config_watcher(self):
        """Start watching config directory for changes.
        This allows for dynamic updates to prompts and settings.
        """
        self.observer = Observer()
        event_handler = ConfigWatcher(self)
        self.observer.schedule(event_handler, self.config_dir, recursive=True)
        self.observer.start()

    def load_system_prompts(self) -> dict:
        """Load system prompts from config or markdown files.
        Now supports separate prompts for summarization and next steps!
        """
        prompts = {}

        # First, check if prompts are defined in config.yaml
        if hasattr(self, "config") and "system_prompts" in self.config:
            prompts = self.config["system_prompts"].copy()

        # Then check for markdown files (these override config)
        prompt_files = {
            "summarization": "summarization_prompt.md",
            "next_step": "next_step_prompt.md",
            "default": "system_prompt.md",  # Legacy support
        }

        default_prompts = {
            "summarization": """You are an expert at analyzing terminal session activity.
Provide concise, technical summaries focusing on:
- Key commands executed
- Important outputs or errors
- Overall progress or issues
Keep summaries brief but informative.""",
            "next_step": """You are a helpful terminal assistant providing actionable next steps.
Based on the session summary, suggest ONE clear next action.
Be specific and practical. Format suggestions as commands when appropriate.
Consider context and help users progress efficiently.""",
            "default": """You are an AI assistant helping with terminal sessions.
Your task is to analyze terminal output and suggest helpful next steps.
Be concise and practical in your suggestions.""",
        }

        # Load from files if they exist
        for key, filename in prompt_files.items():
            prompt_file = os.path.join(self.config_dir, filename)
            if os.path.exists(prompt_file):
                with open(prompt_file, "r") as f:
                    prompts[key] = f.read()
            elif key not in prompts:
                # Use default if not in config and file doesn't exist
                prompts[key] = default_prompts[key]
                # Create the file for future editing
                if key == "default":  # Only create the legacy file
                    os.makedirs(self.config_dir, exist_ok=True)
                    with open(prompt_file, "w") as f:
                        f.write(default_prompts[key])

        return prompts

    def load_config(self) -> dict:
        """Load configuration from YAML file.
        Now supports mixed providers and separate system prompts!
        """
        config_file = os.path.join(self.config_dir, "config.yaml")

        # Legacy defaults for backward compatibility
        legacy_defaults = {
            "openai_summarization_model": "gpt-4o",
            "openai_next_step_model": "gpt-4o",
            "gemini_summarization_model": "gemini-1.5-flash",
            "gemini_next_step_model": "gemini-1.5-flash",
            "ollama_summarization_model": "llama3.2:3b",
            "ollama_next_step_model": "llama3.2:3b",
            "max_history_lines": 1000,
            "check_interval": 1.0,
            "temperature": 0.7,
            "max_tokens": 500,
        }

        if os.path.exists(config_file):
            with open(config_file, "r") as f:
                loaded_config = yaml.safe_load(f) or {}

                # Handle new config structure
                if "providers" in loaded_config:
                    # New enhanced config - merge carefully
                    config = loaded_config

                    # Add legacy keys for backward compatibility
                    if "models" in config:
                        for provider in ["openai", "gemini", "ollama"]:
                            if provider in config["models"]:
                                for task in ["summarization", "next_step"]:
                                    if task in config["models"][provider]:
                                        legacy_key = f"{provider}_{task}_model"
                                        config[legacy_key] = config["models"][provider][
                                            task
                                        ]

                    # Extract settings
                    if "settings" in config:
                        config.update(config["settings"])

                    # Extract prompt patterns
                    if "prompt_patterns" in config:
                        self.prompt_patterns = config["prompt_patterns"]

                    # Ensure all legacy defaults are present
                    for key, value in legacy_defaults.items():
                        if key not in config:
                            config[key] = value

                    return config
                else:
                    # Old config format - merge with defaults
                    return {**legacy_defaults, **loaded_config}
        else:
            # No config file - return legacy defaults
            return legacy_defaults

    def reload_config(self):
        """Reload configuration files - hot reload functionality!
        Keeps our AI assistant nimble and responsive to changes.
        """
        print(f"{Fore.GREEN}Reloading configuration...")
        self.config = self.load_config()
        self.system_prompts = self.load_system_prompts()
        self.system_prompt = self.system_prompts.get(
            "next_step", self.system_prompts.get("default", "")
        )

        # Update providers if changed
        if "providers" in self.config:
            old_summarization = self.summarization_provider
            old_next_step = self.next_step_provider

            self.summarization_provider = self.config["providers"].get(
                "summarization", "openai"
            )
            self.next_step_provider = self.config["providers"].get(
                "next_step", "openai"
            )

            if (
                old_summarization != self.summarization_provider
                or old_next_step != self.next_step_provider
            ):
                print(f"{Fore.YELLOW}Provider configuration changed!")
                print(
                    f"  - Summarization: {old_summarization} → {self.summarization_provider}"
                )
                print(f"  - Next Steps: {old_next_step} → {self.next_step_provider}")

    def connect_to_session(self) -> bool:
        """Connect to the specified tmux session.
        Essential for monitoring and interacting with the terminal.
        """
        try:
            # Get session using newer API
            sessions = self.server.sessions
            self.session = None
            
            for session in sessions:
                if session.name == self.session_name:
                    self.session = session
                    break
                    
            if self.session:
                # Get the currently active pane
                self.pane = self.session.active_pane
                print(f"{Fore.GREEN}Connected to tmux session: {self.session_name}")
                return True
            else:
                # List available sessions for user info
                session_names = [s.name for s in sessions]
                print(f"{Fore.RED}Session '{self.session_name}' not found!")
                print(f"Available sessions: {session_names}")
                return False

        except Exception as e:
            print(f"{Fore.RED}Error connecting to tmux: {e}")
            return False

    def capture_pane_content(self) -> List[str]:
        """Capture the current content of the pane.
        This is how we get the raw terminal output for AI analysis.
        """
        if not self.pane:
            return []

        try:
            # Capture pane content (capture-pane -p prints to stdout)
            content = self.pane.capture_pane()
            return content
        except Exception as e:
            print(f"{Fore.RED}Error capturing pane content: {e}")
            return []

    def is_at_prompt(self, lines: List[str]) -> bool:
        """Check if the last non-empty line matches a prompt pattern.
        This helps us know when the user is ready for AI assistance.
        """
        # Find the last non-empty line
        for line in reversed(lines):
            if line.strip():
                # Check against all prompt patterns
                for pattern in self.prompt_patterns:
                    try:
                        if re.search(pattern, line):
                            return True
                    except re.error as e:
                        print(f"{Fore.YELLOW}Invalid regex pattern '{pattern}': {e}")
                return False  # No prompt found on the last non-empty line
        return False  # No non-empty lines found

    def get_activity_since_last_interaction(self, lines: List[str]) -> str:
        """Extract activity since the last interaction.
        We only want to analyze new output, not the whole history every time.
        """
        # If we haven't tracked any interaction yet, take the last N lines
        if self.last_interaction_line == 0:
            relevant_lines = lines[-self.config["max_history_lines"] :]
        else:
            relevant_lines = lines[self.last_interaction_line :]

        # Update the last interaction line for next time
        self.last_interaction_line = len(lines)

        # Join the lines and clean up
        activity = "\n".join(relevant_lines)
        return activity.strip()

    async def _get_openai_summary(self, activity: str) -> str:
        """Internal method to summarize activity using OpenAI."""
        if not self.openai_client:
            return "OpenAI client not initialized."

        try:
            if hasattr(self, 'verbose') and self.verbose:
                logger.debug(f"Sending {len(activity)} chars to OpenAI for summarization")
            response = self.openai_client.chat.completions.create(
                model=self.config["openai_summarization_model"],
                messages=[
                    {
                        "role": "system",
                        "content": self.system_prompts.get(
                            "summarization", self.system_prompts.get("default", "")
                        ),
                    },
                    {"role": "user", "content": f"Terminal activity:\n\n{activity}"},
                ],
                temperature=self.config.get(
                    "summarization_temperature", self.config.get("temperature", 0.7)
                ),
                max_tokens=self.config.get(
                    "summarization_max_tokens", self.config.get("max_tokens", 500)
                ),
            )
            return response.choices[0].message.content
        except Exception as e:
            print(f"{Fore.RED}Error during OpenAI summarization: {e}")
            return f"Error summarizing activity with OpenAI: {str(e)}"

    async def _get_gemini_summary(self, activity: str) -> str:
        """Internal method to summarize activity using Google Gemini."""
        # Check if Gemini API is properly configured
        try:
            # Simple check - if we can create a model, we're good
            test_model = genai.GenerativeModel(self.config["gemini_summarization_model"])
            if not test_model:
                logger.warning("Gemini API not properly configured")
                return "Gemini client not initialized."
        except Exception as e:
            logger.error(f"Error checking Gemini configuration: {e}")
            return "Gemini client initialization error."

        try:
            if hasattr(self, 'verbose') and self.verbose:
                logger.debug(f"Sending {len(activity)} chars to Gemini for summarization")
            model = genai.GenerativeModel(self.config["gemini_summarization_model"])
            response = await model.generate_content_async(
                contents=[
                    {
                        "role": "user",
                        "parts": [
                            self.system_prompts.get(
                                "summarization", self.system_prompts.get("default", "")
                            ),
                            f"Terminal activity:\n\n{activity}",
                        ],
                    }
                ],
                generation_config=genai.GenerationConfig(
                    temperature=self.config.get(
                        "summarization_temperature", self.config.get("temperature", 0.7)
                    ),
                    max_output_tokens=self.config.get(
                        "summarization_max_tokens", self.config.get("max_tokens", 500)
                    ),
                ),
            )
            return response.text
        except Exception as e:
            print(f"{Fore.RED}Error during Gemini summarization: {e}")
            return f"Error summarizing activity with Gemini: {str(e)}"

    async def _get_ollama_summary(self, activity: str) -> str:
        """Internal method to summarize activity using Ollama - locally and budget-friendly!"""
        try:
            if hasattr(self, 'verbose') and self.verbose:
                logger.debug(f"Sending {len(activity)} chars to Ollama for summarization")
            # Ollama uses a simpler API - perfect for Trisha's accounting!
            response = await ollama.AsyncClient().chat(
                model=self.config["ollama_summarization_model"],
                messages=[
                    {
                        "role": "system",
                        "content": self.system_prompts.get(
                            "summarization", self.system_prompts.get("default", "")
                        ),
                    },
                    {"role": "user", "content": f"Terminal activity:\n\n{activity}"},
                ],
                options={
                    "temperature": self.config.get(
                        "summarization_temperature", self.config.get("temperature", 0.7)
                    ),
                    "num_predict": self.config.get(
                        "summarization_max_tokens", self.config.get("max_tokens", 500)
                    ),
                },
            )
            return response["message"]["content"]
        except Exception as e:
            print(f"{Fore.RED}Error during Ollama summarization: {e}")
            return f"Error summarizing activity with Ollama: {str(e)}"

    async def summarize_activity(self, activity: str) -> str:
        """Summarize the terminal activity using the configured summarization provider."""
        if not activity:
            return "No activity to summarize."

        if self.summarization_provider == "openai":
            return await self._get_openai_summary(activity)
        elif self.summarization_provider == "gemini":
            return await self._get_gemini_summary(activity)
        elif self.summarization_provider == "ollama":
            return await self._get_ollama_summary(activity)
        else:
            return f"Unsupported AI provider for summarization: {self.summarization_provider}"

    async def _get_openai_next_steps(self, summary: str) -> str:
        """Internal method to generate next steps using OpenAI."""
        if not self.openai_client:
            return "OpenAI client not initialized."

        try:
            response = self.openai_client.chat.completions.create(
                model=self.config["openai_next_step_model"],
                messages=[
                    {
                        "role": "system",
                        "content": self.system_prompts.get(
                            "next_step", self.system_prompt
                        ),
                    },
                    {
                        "role": "user",
                        "content": f"Based on this terminal session summary, suggest the next step:\n\n{summary}",
                    },
                ],
                temperature=self.config.get(
                    "next_step_temperature", self.config.get("temperature", 0.7)
                ),
                max_tokens=self.config.get(
                    "next_step_max_tokens", self.config.get("max_tokens", 500)
                ),
            )
            return response.choices[0].message.content
        except Exception as e:
            print(f"{Fore.RED}Error generating OpenAI next step: {e}")
            return f"Error generating next step with OpenAI: {str(e)}"

    async def _get_gemini_next_steps(self, summary: str) -> str:
        """Internal method to generate next steps using Google Gemini."""
        try:
            model = genai.GenerativeModel(self.config["gemini_next_step_model"])
            response = await model.generate_content_async(
                contents=[
                    {
                        "role": "user",
                        "parts": [
                            self.system_prompts.get("next_step", self.system_prompt),
                            f"Based on this terminal session summary, suggest the next step:\n\n{summary}",
                        ],
                    }
                ],
                generation_config=genai.GenerationConfig(
                    temperature=self.config.get(
                        "next_step_temperature", self.config.get("temperature", 0.7)
                    ),
                    max_output_tokens=self.config.get(
                        "next_step_max_tokens", self.config.get("max_tokens", 500)
                    ),
                ),
            )
            return response.text
        except Exception as e:
            print(f"{Fore.RED}Error generating Gemini next step: {e}")
            return f"Error generating next step with Gemini: {str(e)}"

    async def _get_ollama_next_steps(self, summary: str) -> str:
        """Internal method to generate next steps using Ollama - fiscally responsible AI!"""
        try:
            response = await ollama.AsyncClient().chat(
                model=self.config["ollama_next_step_model"],
                messages=[
                    {
                        "role": "system",
                        "content": self.system_prompts.get(
                            "next_step", self.system_prompt
                        ),
                    },
                    {
                        "role": "user",
                        "content": f"Based on this terminal session summary, suggest the next step:\n\n{summary}",
                    },
                ],
                options={
                    "temperature": self.config.get(
                        "next_step_temperature", self.config.get("temperature", 0.7)
                    ),
                    "num_predict": self.config.get(
                        "next_step_max_tokens", self.config.get("max_tokens", 500)
                    ),
                },
            )
            return response["message"]["content"]
        except Exception as e:
            print(f"{Fore.RED}Error generating Ollama next step: {e}")
            return f"Error generating next step with Ollama: {str(e)}"

    async def generate_next_step(self, summary: str) -> str:
        """Generate the next suggested step based on the summary using the configured next step provider."""
        if self.next_step_provider == "openai":
            return await self._get_openai_next_steps(summary)
        elif self.next_step_provider == "gemini":
            return await self._get_gemini_next_steps(summary)
        elif self.next_step_provider == "ollama":
            return await self._get_ollama_next_steps(summary)
        else:
            return f"Unsupported AI provider for next step generation: {self.next_step_provider}"

    def analyze_for_prompt_pattern(self, lines: List[str]) -> Optional[str]:
        """Analyze recent lines to detect potential prompt patterns.
        Uses heuristics to identify repeating patterns that might be prompts.
        """
        if not self.prompt_learning_enabled:
            return None
            
        # Look for patterns in the last 10 non-empty lines
        recent_lines = []
        for line in reversed(lines):
            if line.strip():
                recent_lines.append(line)
                if len(recent_lines) >= 10:
                    break
        
        # Count line endings that appear multiple times
        line_endings = {}
        for line in recent_lines:
            # Extract last 20 chars as potential prompt
            ending = line[-20:].strip()
            if len(ending) > 0 and len(ending) < 15:  # Reasonable prompt length
                # Look for common prompt indicators
                if any(char in ending for char in ['$', '>', '#', ':', ']', '»', '❯', '➜']):
                    line_endings[ending] = line_endings.get(ending, 0) + 1
        
        # Find most common ending that appears at least 3 times
        for ending, count in sorted(line_endings.items(), key=lambda x: x[1], reverse=True):
            if count >= 3:
                # Create a regex pattern for this prompt
                # Escape special regex chars and add end anchor
                escaped = re.escape(ending)
                pattern = escaped + r"\s*$"
                
                # Test if it's a valid pattern and not already known
                if pattern not in self.prompt_patterns and pattern not in self.learned_prompts:
                    try:
                        re.compile(pattern)
                        return pattern
                    except re.error:
                        pass
        
        return None
    
    def add_prompt_pattern(self, pattern: str, learned: bool = False):
        """Add a new prompt pattern, optionally marking it as learned.
        Validates the pattern before adding.
        """
        try:
            # Validate regex
            re.compile(pattern)
            
            if pattern not in self.prompt_patterns:
                self.prompt_patterns.append(pattern)
                if learned:
                    self.learned_prompts.add(pattern)
                    print(f"{Fore.GREEN}Learned new prompt pattern: {pattern}")
                    # Save to config for persistence
                    self.save_learned_prompts()
                return True
        except re.error as e:
            print(f"{Fore.RED}Invalid regex pattern: {e}")
            return False
            
    def save_learned_prompts(self):
        """Save learned prompts to a file for persistence."""
        learned_file = os.path.join(self.config_dir, "learned_prompts.yaml")
        data = {
            "learned_prompts": list(self.learned_prompts),
            "last_updated": datetime.now().isoformat()
        }
        with open(learned_file, "w") as f:
            yaml.dump(data, f)
            
    def load_learned_prompts(self):
        """Load previously learned prompts."""
        learned_file = os.path.join(self.config_dir, "learned_prompts.yaml")
        if os.path.exists(learned_file):
            with open(learned_file, "r") as f:
                data = yaml.safe_load(f) or {}
                learned = data.get("learned_prompts", [])
                for pattern in learned:
                    self.add_prompt_pattern(pattern, learned=True)
                    
    def send_to_pane(self, text: str):
        """Send text to the tmux pane.
        This is how the AI can directly interact with the terminal.
        """
        if not self.pane:
            return

        try:
            # Send the text to the pane
            self.pane.send_keys(text)
            print(f"{Fore.GREEN}Sent to pane: {text[:50]}...")
        except Exception as e:
            print(f"{Fore.RED}Error sending to pane: {e}")

    async def run_monitoring_loop(self):
        """Main monitoring loop - where the magic happens!
        Continuously monitors the tmux session and provides AI assistance.
        """
        print(f"\n{Fore.CYAN}Monitoring Configuration:{Fore.RESET}")
        print(f"  📊 Summarization: {Fore.YELLOW}{self.summarization_provider.upper()}{Fore.RESET}")
        print(f"  💡 Next Steps: {Fore.YELLOW}{self.next_step_provider.upper()}{Fore.RESET}")
        print(f"  ⏱️  Check interval: {self.config['check_interval']}s")
        print(f"\n{Fore.GREEN}Ready! Press Ctrl+C to stop{Fore.RESET}")

        try:
            while True:
                # Capture current pane content
                lines = self.capture_pane_content()

                # Check if we're at a prompt
                if self.is_at_prompt(lines):
                    print(f"\n{Fore.BLUE}🎯 Detected prompt! Analyzing activity...{Fore.RESET}")
                elif self.prompt_learning_enabled:
                    # Try to learn new prompt patterns
                    new_pattern = self.analyze_for_prompt_pattern(lines)
                    if new_pattern:
                        self.add_prompt_pattern(new_pattern, learned=True)

                    # Get activity since last interaction
                    activity = self.get_activity_since_last_interaction(lines)

                    if activity:
                        # Step 1: Summarize the activity
                        print(
                            f"{Fore.YELLOW}📝 Summarizing activity using {self.summarization_provider.upper()}...{Fore.RESET}",
                            end='', flush=True
                        )
                        summary = await self.summarize_activity(
                            activity
                        )  # Await the async call
                        print(f" {Fore.GREEN}✓{Fore.RESET}")
                        print(f"{Fore.CYAN}Summary: {summary[:100]}...{Fore.RESET}")

                        # Step 2: Generate next step
                        print(
                            f"{Fore.YELLOW}🤔 Generating next step using {self.next_step_provider.upper()}...{Fore.RESET}",
                            end='', flush=True
                        )
                        next_step = await self.generate_next_step(
                            summary
                        )  # Await the async call
                        print(f" {Fore.GREEN}✓{Fore.RESET}")
                        print(f"\n{Fore.GREEN}💡 Next step:{Fore.RESET} {next_step}")

                        # Optionally send to pane (commented out for safety)
                        # self.send_to_pane(next_step)

                        # Log to file for review - because good records are always a plus!
                        self.log_interaction(activity, summary, next_step)

                # Sleep before next check
                await asyncio.sleep(
                    self.config["check_interval"]
                )  # Use asyncio.sleep for async

        except KeyboardInterrupt:
            print(f"\n{Fore.YELLOW}Monitoring stopped by user")
            self.observer.stop()
            self.observer.join()

    def log_interaction(self, activity: str, summary: str, next_step: str):
        """Log interactions to a file for review.
        This creates a historical record of AI assistance.
        """
        log_dir = "logs"
        os.makedirs(log_dir, exist_ok=True)

        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        log_file = os.path.join(log_dir, f"interaction_{timestamp}.log")

        with open(log_file, "w") as f:
            f.write(f"Timestamp: {datetime.now()}\n")
            f.write(f"Session: {self.session_name}\n")
            f.write(
                f"AI Providers: Summarization={self.summarization_provider.upper()}, Next Steps={self.next_step_provider.upper()}\n"
            )  # Log both AI providers used
            f.write(f"\n--- ACTIVITY ---\n{activity}\n")
            f.write(f"\n--- SUMMARY ---\n{summary}\n")
            f.write(f"\n--- NEXT STEP ---\n{next_step}\n")
            
        if hasattr(self, 'verbose') and self.verbose:
            logger.info(f"Interaction logged to {log_file}")


@click.command()
@click.argument("session_name")
@click.option("--config-dir", default="config", help="Configuration directory")
@click.option(
    "--ai-provider",
    default=None,
    type=click.Choice(["openai", "gemini", "ollama"], case_sensitive=False),
    help="AI provider to use (deprecated - use config.yaml for mixed providers)",
)
@click.option(
    "--openai-api-key",
    envvar="OPENAI_API_KEY",
    help="OpenAI API key (if using openai provider)",
)
@click.option(
    "--gemini-api-key",
    envvar="GEMINI_API_KEY",
    help="Google Gemini API key (if using gemini provider)",
)
@click.option("--add-prompt", multiple=True, help="Add custom prompt patterns (regex supported)")
@click.option("--list-prompts", is_flag=True, help="List all current prompt patterns")
@click.option("--test-prompt", help="Test if a line matches any prompt pattern")
@click.option("--verbose", "-v", is_flag=True, help="Enable verbose logging to logs folder")
@click.option(
    "--port",
    default=8000,
    type=int,
    help="Port for the OpenAI MCP server (if running in HTTP mode)",
)
def main(
    session_name: str,
    config_dir: str,
    ai_provider: str,
    openai_api_key: Optional[str],
    gemini_api_key: Optional[str],
    add_prompt: Tuple[str],
    list_prompts: bool,
    test_prompt: Optional[str],
    verbose: bool,
    port: int,
):
    """
    Tmux AI Assistant - Your intelligent terminal companion!

    Monitors a tmux SESSION_NAME and provides AI-powered assistance.
    """

    # Set up verbose logging if requested
    if verbose:
        log_dir = "logs"
        os.makedirs(log_dir, exist_ok=True)
        log_file = os.path.join(log_dir, f"tmux_monitor_{datetime.now().strftime('%Y%m%d_%H%M%S')}.log")
        
        # Configure file logging
        file_handler = logging.FileHandler(log_file)
        file_handler.setLevel(logging.DEBUG)
        formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
        file_handler.setFormatter(formatter)
        
        # Add to root logger
        root_logger = logging.getLogger()
        root_logger.setLevel(logging.DEBUG)
        root_logger.addHandler(file_handler)
        
        # Enable libtmux debug logging in verbose mode
        logging.getLogger('libtmux').setLevel(logging.DEBUG)
        logging.getLogger('libtmux.common').setLevel(logging.DEBUG)
        
        print(f"{Fore.YELLOW}Verbose logging enabled. Log file: {log_file}")
    else:
        # Make sure console output is clean
        logging.basicConfig(level=logging.WARNING, format='%(message)s')
    
    # Fancy banner - because why not? Trisha loves these!
    print(f"{Fore.CYAN}{Style.BRIGHT}")
    print("=" * 50)
    print("   🤖 Tmux AI Assistant 🤖")
    print("   Your Intelligent Terminal Companion")
    print("=" * 50)
    print(f"{Style.RESET_ALL}")

    # Handle utility commands first
    if list_prompts or test_prompt:
        # Create a temporary monitor just to load prompts
        try:
            temp_monitor = TmuxAIMonitor(
                "dummy", None, openai_api_key, gemini_api_key, config_dir
            )
        except:
            # Just load the config to get patterns
            config_file = os.path.join(config_dir, "config.yaml")
            if os.path.exists(config_file):
                with open(config_file, "r") as f:
                    config = yaml.safe_load(f) or {}
                    prompt_patterns = config.get("prompt_patterns", [])
            else:
                prompt_patterns = []
                
        if list_prompts:
            print(f"{Fore.CYAN}Current prompt patterns:")
            if 'temp_monitor' in locals() and temp_monitor is not None:
                for i, pattern in enumerate(temp_monitor.prompt_patterns, 1):
                    learned = pattern in temp_monitor.learned_prompts
                    print(f"  {i}. {pattern} {Fore.GREEN if learned else ''}{'(learned)' if learned else ''}")
            else:
                for i, pattern in enumerate(prompt_patterns, 1):
                    print(f"  {i}. {pattern}")
            sys.exit(0)
            
        if test_prompt:
            print(f"{Fore.CYAN}Testing line: '{test_prompt}'")
            matched = False
            if 'temp_monitor' in locals() and temp_monitor is not None:
                patterns = temp_monitor.prompt_patterns
            else:
                patterns = prompt_patterns
                
            for pattern in patterns:
                try:
                    if re.search(pattern, test_prompt):
                        print(f"{Fore.GREEN}✓ Matches pattern: {pattern}")
                        matched = True
                except re.error as e:
                    print(f"{Fore.RED}✗ Invalid pattern '{pattern}': {e}")
                    
            if not matched:
                print(f"{Fore.YELLOW}No patterns matched")
            sys.exit(0)
    
    # Don't check for keys here - let the monitor initialization handle it
    # based on which providers are actually needed from config

    # Create monitor instance
    try:
        monitor = TmuxAIMonitor(
            session_name, ai_provider, openai_api_key, gemini_api_key, config_dir
        )
        
        # Set verbose mode
        monitor.verbose = verbose
        
    except ValueError as e:
        print(f"{Fore.RED}Initialization Error: {e}")
        sys.exit(1)

    # Add any custom prompt patterns
    for pattern in add_prompt:
        if monitor.add_prompt_pattern(pattern):
            print(f"{Fore.GREEN}Added custom prompt pattern: {pattern}")

    # Connect to tmux session
    if not monitor.connect_to_session():
        sys.exit(1)

    # Start monitoring
    asyncio.run(monitor.run_monitoring_loop())  # Run the async loop


if __name__ == "__main__":
    # This part is for running tmux_monitor.py directly, not the MCP server
    # The MCP server will call run_unified_server from mcp_server.py
    main()
