#!/usr/bin/env python3
"""
Tmux AI Assistant v2 - Continuous Intelligent Monitoring
Now with queue processing, smart pauses, and interactive helpers!
Aye & Hue making terminal magic happen! ✨
"""

import asyncio
import time
import re
import os
import sys
import logging
from typing import List, Optional, Dict, Any, Deque
from collections import deque
from datetime import datetime, timedelta
import libtmux
import click
from openai import OpenAI
import google.generativeai as genai
import ollama
from dotenv import load_dotenv
import yaml
from colorama import init, Fore, Style
from dataclasses import dataclass
from enum import Enum
import hashlib
import json

# Initialize colorama for beautiful output
init(autoreset=True)

# Set up logger
logger = logging.getLogger(__name__)

# Load environment variables
load_dotenv()


class ProcessingReason(Enum):
    """Reasons why we're processing the queue"""
    PAUSE_DETECTED = "pause_detected"
    CONTEXT_LIMIT = "context_limit"
    PROMPT_DETECTED = "prompt_detected"
    TIMEOUT = "timeout"
    QUESTION_DETECTED = "question_detected"


@dataclass
class LineEntry:
    """A single line with metadata"""
    content: str
    timestamp: datetime
    line_number: int
    
    
@dataclass
class Summary:
    """A summary of processed lines"""
    content: str
    start_line: int
    end_line: int
    timestamp: datetime
    line_count: int


@dataclass
class InteractivePrompt:
    """Detected interactive prompt that needs a response"""
    pattern: str
    prompt_type: str  # password, confirmation, choice, etc.
    detected_at: datetime
    full_line: str
    

class SecureVault:
    """Secure storage for passwords and automated decisions
    Trisha loves security! 🔐
    """
    def __init__(self, vault_file: str = "config/vault.yaml"):
        self.vault_file = vault_file
        self.vault_data = self.load_vault()
        
    def load_vault(self) -> Dict[str, Any]:
        """Load vault data from encrypted file"""
        if os.path.exists(self.vault_file):
            with open(self.vault_file, 'r') as f:
                data = yaml.safe_load(f) or {}
                # In production, this would be encrypted!
                return data
        return {
            "passwords": {},
            "auto_responses": {},
            "patterns": {}
        }
        
    def save_vault(self):
        """Save vault data (in production, encrypt this!)"""
        os.makedirs(os.path.dirname(self.vault_file), exist_ok=True)
        with open(self.vault_file, 'w') as f:
            yaml.dump(self.vault_data, f)
            
    def get_password(self, context: str) -> Optional[str]:
        """Get password for a context"""
        # Hash the context for security
        context_hash = hashlib.sha256(context.encode()).hexdigest()[:16]
        return self.vault_data.get("passwords", {}).get(context_hash)
        
    def store_password(self, context: str, password: str):
        """Store password for a context"""
        context_hash = hashlib.sha256(context.encode()).hexdigest()[:16]
        if "passwords" not in self.vault_data:
            self.vault_data["passwords"] = {}
        self.vault_data["passwords"][context_hash] = password
        self.save_vault()
        
    def get_auto_response(self, pattern: str) -> Optional[str]:
        """Get automated response for a pattern"""
        return self.vault_data.get("auto_responses", {}).get(pattern)


class ContinuousTmuxMonitor:
    """The brain of our continuous monitoring operation!
    Now with smart queue processing and interactive helpers.
    """
    
    def __init__(
        self,
        session_name: str,
        config_dir: str = "config",
        max_context_lines: int = 500,
        pause_threshold: float = 15.0,
        dead_threshold: float = 120.0,
    ):
        self.session_name = session_name
        self.config_dir = config_dir
        self.max_context_lines = max_context_lines
        self.pause_threshold = pause_threshold
        self.dead_threshold = dead_threshold
        
        # Load configuration
        self.config = self.load_config()
        
        # Initialize AI providers
        self.init_ai_providers()
        
        # Initialize tmux connection
        self.server = libtmux.Server()
        self.session = None
        self.pane = None
        
        # Queue and processing state
        self.line_queue: Deque[LineEntry] = deque(maxlen=10000)
        self.summaries: List[Summary] = []
        self.last_processed_line = 0
        self.last_activity_time = datetime.now()
        self.processing_lock = asyncio.Lock()
        
        # Pattern detection
        self.prompt_patterns = self.config.get("prompt_patterns", [])
        self.question_patterns = [
            r"Are you sure.*\?",
            r"Do you want to.*\?",
            r"Continue\?.*\(y/n\)",
            r"Overwrite.*\?",
            r"Delete.*\?",
            r"Save changes.*\?",
            r"Password:",
            r"Enter passphrase",
            r"Username:",
            r"\[sudo\] password for",
        ]
        
        # Interactive helpers
        self.vault = SecureVault()
        self.automation_enabled = self.config.get("automation_enabled", False)
        self.verbose = self.config.get("verbose", False)
        
        # Statistics
        self.stats = {
            "lines_processed": 0,
            "summaries_created": 0,
            "prompts_detected": 0,
            "questions_answered": 0,
            "auto_responses": 0
        }
        
    def load_config(self) -> dict:
        """Load configuration from YAML file"""
        config_file = os.path.join(self.config_dir, "config.yaml")
        if os.path.exists(config_file):
            with open(config_file, 'r') as f:
                return yaml.safe_load(f) or {}
        return {}
        
    def init_ai_providers(self):
        """Initialize AI providers based on config"""
        # Similar to original but simplified for this example
        self.summarization_provider = self.config.get("providers", {}).get("summarization", "openai")
        self.next_step_provider = self.config.get("providers", {}).get("next_step", "openai")
        
        # Initialize clients as needed
        if "openai" in [self.summarization_provider, self.next_step_provider]:
            self.openai_client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))
            
        if "gemini" in [self.summarization_provider, self.next_step_provider]:
            genai.configure(api_key=os.getenv("GEMINI_API_KEY"))
            
    def connect_to_session(self) -> bool:
        """Connect to tmux session"""
        try:
            self.session = self.server.find_where({"session_name": self.session_name})
            if self.session:
                self.pane = self.session.active_pane
                print(f"{Fore.GREEN}Connected to tmux session: {self.session_name}")
                return True
            else:
                print(f"{Fore.RED}Session '{self.session_name}' not found!")
                return False
        except Exception as e:
            print(f"{Fore.RED}Error connecting to tmux: {e}")
            return False
            
    async def capture_lines_continuously(self):
        """Continuously capture new lines from the pane"""
        last_content = []
        
        while True:
            try:
                # Capture current pane content
                current_content = self.pane.capture_pane()
                
                # Find new lines
                if len(current_content) > len(last_content):
                    new_lines = current_content[len(last_content):]
                    
                    # Add to queue with metadata
                    for i, line in enumerate(new_lines):
                        entry = LineEntry(
                            content=line,
                            timestamp=datetime.now(),
                            line_number=len(last_content) + i
                        )
                        self.line_queue.append(entry)
                        self.last_activity_time = datetime.now()
                        
                        if self.verbose:
                            logger.debug(f"New line: {line[:80]}...")
                            
                        # Check for interactive prompts
                        if self.detect_interactive_prompt(line):
                            await self.handle_interactive_prompt(line)
                            
                elif len(current_content) < len(last_content):
                    # Pane was cleared or scrolled
                    logger.info("Pane content changed significantly, resetting")
                    
                last_content = current_content
                
                # Small delay to avoid hammering tmux
                await asyncio.sleep(0.1)
                
            except Exception as e:
                logger.error(f"Error capturing lines: {e}")
                await asyncio.sleep(1)
                
    def detect_interactive_prompt(self, line: str) -> Optional[InteractivePrompt]:
        """Detect if a line is an interactive prompt"""
        for pattern in self.question_patterns:
            if re.search(pattern, line, re.IGNORECASE):
                return InteractivePrompt(
                    pattern=pattern,
                    prompt_type=self.classify_prompt(pattern),
                    detected_at=datetime.now(),
                    full_line=line
                )
        return None
        
    def classify_prompt(self, pattern: str) -> str:
        """Classify the type of prompt"""
        if "password" in pattern.lower() or "passphrase" in pattern.lower():
            return "password"
        elif "y/n" in pattern.lower() or "yes/no" in pattern.lower():
            return "confirmation"
        elif "username" in pattern.lower():
            return "username"
        else:
            return "general"
            
    async def handle_interactive_prompt(self, line: str):
        """Handle detected interactive prompts"""
        prompt = self.detect_interactive_prompt(line)
        if not prompt:
            return
            
        print(f"{Fore.YELLOW}Interactive prompt detected: {prompt.prompt_type}")
        self.stats["prompts_detected"] += 1
        
        if self.automation_enabled:
            # Try to auto-respond
            response = self.vault.get_auto_response(prompt.pattern)
            if response:
                print(f"{Fore.GREEN}Auto-responding with: {'*' * len(response) if prompt.prompt_type == 'password' else response}")
                self.pane.send_keys(response)
                self.stats["auto_responses"] += 1
            else:
                print(f"{Fore.YELLOW}No auto-response configured for this prompt")
                # Trigger AI assistance
                await self.process_queue(ProcessingReason.QUESTION_DETECTED)
        else:
            # Just notify the user
            print(f"{Fore.CYAN}Manual response needed for: {line}")
            
    async def process_queue(self, reason: ProcessingReason):
        """Process the current queue and generate summary/next steps"""
        async with self.processing_lock:
            if not self.line_queue:
                return
                
            print(f"{Fore.BLUE}Processing queue - Reason: {reason.value}")
            
            # Get lines to process
            lines_to_process = list(self.line_queue)
            
            # If we have previous summaries and hit context limit, use rolling summary
            if reason == ProcessingReason.CONTEXT_LIMIT and self.summaries:
                # Keep last summary and recent lines
                context = self._build_rolling_context(lines_to_process)
            else:
                # Process all lines
                context = "\n".join([line.content for line in lines_to_process])
                
            # Generate summary
            summary_text = await self.summarize_activity(context)
            
            # Create summary object
            if lines_to_process:
                summary = Summary(
                    content=summary_text,
                    start_line=lines_to_process[0].line_number,
                    end_line=lines_to_process[-1].line_number,
                    timestamp=datetime.now(),
                    line_count=len(lines_to_process)
                )
                self.summaries.append(summary)
                self.stats["summaries_created"] += 1
                
                # Clear processed lines from queue
                self.line_queue.clear()
                
                # Generate next steps if at prompt
                if reason in [ProcessingReason.PROMPT_DETECTED, ProcessingReason.TIMEOUT]:
                    next_steps = await self.generate_next_step(summary_text)
                    print(f"{Fore.GREEN}Suggested next step: {next_steps}")
                    
                    # Log the interaction
                    self.log_interaction(context, summary_text, next_steps)
                    
    def _build_rolling_context(self, recent_lines: List[LineEntry]) -> str:
        """Build context using previous summary + recent lines"""
        context_parts = []
        
        # Add last 2 summaries for context
        if len(self.summaries) >= 2:
            context_parts.append(f"[Previous Summary]: {self.summaries[-2].content}")
        if self.summaries:
            context_parts.append(f"[Last Summary]: {self.summaries[-1].content}")
            
        # Add recent lines
        recent_text = "\n".join([line.content for line in recent_lines[-self.max_context_lines:]])
        context_parts.append(f"[Recent Activity]:\n{recent_text}")
        
        return "\n\n".join(context_parts)
        
    async def monitor_activity(self):
        """Monitor for pauses and trigger processing"""
        while True:
            try:
                time_since_activity = (datetime.now() - self.last_activity_time).total_seconds()
                
                # Check queue size
                if len(self.line_queue) >= self.max_context_lines:
                    await self.process_queue(ProcessingReason.CONTEXT_LIMIT)
                    
                # Check for pause
                elif time_since_activity >= self.pause_threshold and self.line_queue:
                    # Check if we're at a prompt
                    if self.line_queue and self.is_at_prompt([line.content for line in self.line_queue]):
                        await self.process_queue(ProcessingReason.PROMPT_DETECTED)
                    else:
                        await self.process_queue(ProcessingReason.PAUSE_DETECTED)
                        
                # Check for dead/timeout
                elif time_since_activity >= self.dead_threshold and self.line_queue:
                    print(f"{Fore.YELLOW}Session appears inactive for {self.dead_threshold}s")
                    await self.process_queue(ProcessingReason.TIMEOUT)
                    
                await asyncio.sleep(1)
                
            except Exception as e:
                logger.error(f"Error in monitor loop: {e}")
                await asyncio.sleep(1)
                
    def is_at_prompt(self, lines: List[str]) -> bool:
        """Check if we're at a prompt"""
        for line in reversed(lines):
            if line.strip():
                for pattern in self.prompt_patterns:
                    try:
                        if re.search(pattern, line):
                            return True
                    except re.error:
                        pass
                return False
        return False
        
    async def summarize_activity(self, activity: str) -> str:
        """Summarize activity using configured AI provider"""
        # Simplified version - in real implementation, use provider-specific methods
        if self.summarization_provider == "openai":
            response = self.openai_client.chat.completions.create(
                model="gpt-4o",
                messages=[
                    {"role": "system", "content": "Summarize this terminal activity concisely."},
                    {"role": "user", "content": activity}
                ],
                temperature=0.3,
                max_tokens=300
            )
            return response.choices[0].message.content
        else:
            return "Summary not implemented for this provider"
            
    async def generate_next_step(self, summary: str) -> str:
        """Generate next step suggestion"""
        if self.next_step_provider == "openai":
            response = self.openai_client.chat.completions.create(
                model="gpt-4o",
                messages=[
                    {"role": "system", "content": "Suggest the next helpful terminal command based on this summary."},
                    {"role": "user", "content": summary}
                ],
                temperature=0.7,
                max_tokens=200
            )
            return response.choices[0].message.content
        else:
            return "Next step generation not implemented for this provider"
            
    def log_interaction(self, activity: str, summary: str, next_step: str):
        """Log interaction to file"""
        log_dir = "logs"
        os.makedirs(log_dir, exist_ok=True)
        
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        log_file = os.path.join(log_dir, f"continuous_{timestamp}.log")
        
        with open(log_file, 'w') as f:
            f.write(f"Timestamp: {datetime.now()}\n")
            f.write(f"Session: {self.session_name}\n")
            f.write(f"Stats: {json.dumps(self.stats, indent=2)}\n")
            f.write(f"\n--- ACTIVITY ---\n{activity}\n")
            f.write(f"\n--- SUMMARY ---\n{summary}\n")
            f.write(f"\n--- NEXT STEP ---\n{next_step}\n")
            
    def print_stats(self):
        """Print current statistics"""
        print(f"\n{Fore.CYAN}=== Session Statistics ===")
        print(f"Lines processed: {self.stats['lines_processed']}")
        print(f"Summaries created: {self.stats['summaries_created']}")
        print(f"Prompts detected: {self.stats['prompts_detected']}")
        print(f"Questions answered: {self.stats['questions_answered']}")
        print(f"Auto responses: {self.stats['auto_responses']}")
        print(f"Queue size: {len(self.line_queue)}")
        print(f"========================\n")
        
    async def run(self):
        """Run the continuous monitor"""
        print(f"{Fore.CYAN}{Style.BRIGHT}")
        print("=" * 50)
        print("   🚀 Tmux AI Assistant v2 🚀")
        print("   Continuous Intelligent Monitoring")
        print("=" * 50)
        print(f"{Style.RESET_ALL}")
        
        if not self.connect_to_session():
            return
            
        print(f"{Fore.GREEN}Starting continuous monitoring...")
        print(f"  - Pause threshold: {self.pause_threshold}s")
        print(f"  - Dead threshold: {self.dead_threshold}s")
        print(f"  - Max context lines: {self.max_context_lines}")
        print(f"  - Automation: {'ENABLED' if self.automation_enabled else 'DISABLED'}")
        print(f"{Fore.YELLOW}Press Ctrl+C to stop\n")
        
        try:
            # Start concurrent tasks
            tasks = [
                asyncio.create_task(self.capture_lines_continuously()),
                asyncio.create_task(self.monitor_activity()),
            ]
            
            # Add periodic stats printing
            async def print_stats_periodically():
                while True:
                    await asyncio.sleep(60)
                    self.print_stats()
                    
            tasks.append(asyncio.create_task(print_stats_periodically()))
            
            # Run until cancelled
            await asyncio.gather(*tasks)
            
        except KeyboardInterrupt:
            print(f"\n{Fore.YELLOW}Monitoring stopped by user")
            self.print_stats()
            

@click.command()
@click.argument("session_name")
@click.option("--config-dir", default="config", help="Configuration directory")
@click.option("--pause-threshold", default=15.0, help="Seconds of inactivity before processing")
@click.option("--dead-threshold", default=120.0, help="Seconds before considering session dead")
@click.option("--max-lines", default=500, help="Maximum context lines before summarizing")
@click.option("--enable-automation", is_flag=True, help="Enable automated responses")
@click.option("--add-auto-response", nargs=2, multiple=True, help="Add auto-response pattern")
@click.option("--verbose", is_flag=True, help="Enable verbose logging")
def main(
    session_name: str,
    config_dir: str,
    pause_threshold: float,
    dead_threshold: float,
    max_lines: int,
    enable_automation: bool,
    add_auto_response: List[tuple],
    verbose: bool,
):
    """
    Tmux AI Assistant v2 - Continuous Intelligent Monitoring
    
    Monitor SESSION_NAME with smart queue processing and interactive helpers.
    """
    
    # Set up logging
    if verbose:
        logging.basicConfig(
            level=logging.DEBUG,
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
        )
    
    # Create monitor
    monitor = ContinuousTmuxMonitor(
        session_name=session_name,
        config_dir=config_dir,
        max_context_lines=max_lines,
        pause_threshold=pause_threshold,
        dead_threshold=dead_threshold,
    )
    
    # Configure automation
    monitor.automation_enabled = enable_automation
    monitor.verbose = verbose
    
    # Add any auto-responses
    for pattern, response in add_auto_response:
        monitor.vault.vault_data.setdefault("auto_responses", {})[pattern] = response
        monitor.vault.save_vault()
        print(f"{Fore.GREEN}Added auto-response: {pattern} -> {'*' * len(response) if 'password' in pattern.lower() else response}")
    
    # Run the monitor
    asyncio.run(monitor.run())
    

if __name__ == "__main__":
    main()