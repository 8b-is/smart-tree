#!/usr/bin/env python3
"""
Tmux AI Attach - Simple tmux client attachment with AI assistance
Attaches to tmux properly so you can kick it out if needed!
"""

import os
import sys
import subprocess
import select
import termios
import tty
import signal
import asyncio
import queue
import threading
from typing import Optional
from datetime import datetime
import click
from colorama import init, Fore, Style
from openai import OpenAI
from dotenv import load_dotenv

# Initialize colorama
init(autoreset=True)

# Load environment variables
load_dotenv()


class TmuxAttachClient:
    """Simple tmux attachment client with AI assistance"""
    
    def __init__(self, session_name: str, ai_assist: bool = True):
        self.session_name = session_name
        self.ai_assist = ai_assist
        
        # AI setup
        if ai_assist:
            self.ai_client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))
            
        # Process and terminal state
        self.tmux_process = None
        self.original_tty = None
        self.running = False
        
        # Command queue for AI suggestions
        self.suggestion_queue = queue.Queue()
        self.buffer = []
        self.last_activity = datetime.now()
        
    def save_terminal_state(self):
        """Save current terminal state"""
        self.original_tty = termios.tcgetattr(sys.stdin)
        
    def restore_terminal_state(self):
        """Restore terminal state"""
        if self.original_tty:
            termios.tcsetattr(sys.stdin, termios.TCSADRAIN, self.original_tty)
            
    def attach_to_session(self):
        """Attach to tmux session as a normal client"""
        try:
            # Save terminal state
            self.save_terminal_state()
            
            # Put terminal in raw mode
            tty.setraw(sys.stdin)
            
            # Start tmux attach
            cmd = ['tmux', 'attach-session', '-t', self.session_name]
            
            print(f"\r\n{Fore.GREEN}Attaching to tmux session: {self.session_name}{Fore.RESET}\r\n")
            print(f"{Fore.YELLOW}Press Ctrl+B D to detach properly{Fore.RESET}\r\n")
            
            # Start tmux process
            self.tmux_process = subprocess.Popen(
                cmd,
                stdin=sys.stdin,
                stdout=sys.stdout,
                stderr=sys.stderr
            )
            
            self.running = True
            
            # Start AI assistant thread if enabled
            if self.ai_assist:
                assistant_thread = threading.Thread(target=self.ai_assistant_loop)
                assistant_thread.daemon = True
                assistant_thread.start()
                
            # Wait for tmux to exit
            self.tmux_process.wait()
            
        except Exception as e:
            print(f"\r\n{Fore.RED}Error: {e}{Fore.RESET}\r\n")
            
        finally:
            self.running = False
            self.restore_terminal_state()
            print(f"\r\n{Fore.CYAN}Detached from tmux session{Fore.RESET}\r\n")
            
    def ai_assistant_loop(self):
        """Background thread for AI assistance"""
        import libtmux
        
        server = libtmux.Server()
        session = None
        
        # Find our session
        for s in server.sessions:
            if s.name == self.session_name:
                session = s
                break
                
        if not session:
            return
            
        last_content = []
        
        while self.running:
            try:
                # Get current pane content
                pane = session.active_pane
                content = pane.capture_pane()
                
                # Check for changes
                if content != last_content:
                    # Look for errors or help requests
                    recent_lines = content[-10:]  # Last 10 lines
                    
                    for line in recent_lines:
                        if any(trigger in line.lower() for trigger in ['error:', 'failed', 'help', 'not found']):
                            # Generate AI suggestion
                            suggestion = self.get_ai_suggestion('\n'.join(recent_lines))
                            if suggestion:
                                self.suggestion_queue.put(suggestion)
                                # Note: We don't automatically send - user can see it in logs
                                
                    last_content = content
                    
                time.sleep(1)  # Check every second
                
            except Exception as e:
                # Silently continue - don't interrupt the session
                pass
                
    def get_ai_suggestion(self, context: str) -> Optional[str]:
        """Get AI suggestion for the current context"""
        try:
            response = self.ai_client.chat.completions.create(
                model="gpt-4",
                messages=[
                    {
                        "role": "system",
                        "content": "You are a helpful terminal assistant. Provide brief, specific command suggestions."
                    },
                    {
                        "role": "user",
                        "content": f"Based on this terminal output, suggest a helpful command:\n\n{context}"
                    }
                ],
                max_tokens=100,
                temperature=0.3
            )
            return response.choices[0].message.content.strip()
        except:
            return None
            

@click.command()
@click.argument("session_name")
@click.option("--no-ai", is_flag=True, help="Disable AI assistance")
def main(session_name: str, no_ai: bool):
    """
    Attach to tmux session with optional AI assistance
    
    This creates a proper tmux client connection that can be:
    - Detached normally with Ctrl+B D
    - Kicked out by the session owner
    - Enhanced with AI suggestions (in logs)
    """
    
    print(f"{Fore.CYAN}{'='*50}")
    print(f"🎹 Tmux AI Attach Client")
    print(f"{'='*50}{Fore.RESET}\n")
    
    # Check if session exists
    result = subprocess.run(
        ['tmux', 'has-session', '-t', session_name],
        capture_output=True
    )
    
    if result.returncode != 0:
        print(f"{Fore.RED}Session '{session_name}' not found!{Fore.RESET}")
        
        # List available sessions
        list_result = subprocess.run(
            ['tmux', 'list-sessions'],
            capture_output=True,
            text=True
        )
        
        if list_result.returncode == 0:
            print(f"\n{Fore.YELLOW}Available sessions:{Fore.RESET}")
            print(list_result.stdout)
        else:
            print(f"{Fore.YELLOW}No tmux sessions found{Fore.RESET}")
            
        return
        
    # Create and run client
    client = TmuxAttachClient(
        session_name=session_name,
        ai_assist=not no_ai
    )
    
    # Handle signals
    def signal_handler(sig, frame):
        client.running = False
        if client.tmux_process:
            client.tmux_process.terminate()
            
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)
    
    # Attach to session
    client.attach_to_session()
    
    # Show any AI suggestions that were generated
    if not no_ai and not client.suggestion_queue.empty():
        print(f"\n{Fore.GREEN}AI Suggestions generated during session:{Fore.RESET}")
        while not client.suggestion_queue.empty():
            suggestion = client.suggestion_queue.get()
            print(f"  💡 {suggestion}")
            

if __name__ == "__main__":
    main()