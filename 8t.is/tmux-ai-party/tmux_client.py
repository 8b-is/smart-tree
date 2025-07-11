#!/usr/bin/env python3
"""
Tmux AI Client - Attaches as a real tmux client for interactive collaboration!
🎹 "Sing us a song, you're the piano man..." 🎪
Let's make a coding carnival where everyone can join!
"""

import asyncio
import os
import sys
import time
import signal
import threading
import queue
from typing import Optional, List, Dict, Any, Callable
from datetime import datetime
import libtmux
import click
from openai import OpenAI
import google.generativeai as genai
import ollama
from dotenv import load_dotenv
import yaml
from colorama import init, Fore, Style, Back
import subprocess
import pty
import select
import termios
import tty
import fcntl
import struct
from dataclasses import dataclass
from enum import Enum
import websockets
import json
from aiohttp import web
import aiohttp_cors

# Initialize colorama
init(autoreset=True)

# Load environment variables
load_dotenv()


class ClientMode(Enum):
    """Different modes for the AI client"""
    OBSERVE = "observe"       # Just watch and suggest
    ASSIST = "assist"         # Watch and queue commands
    COLLABORATE = "collaborate"  # Full interactive mode
    SPECTATE = "spectate"     # Web-based spectator mode


@dataclass
class Command:
    """A command to be sent to the terminal"""
    text: str
    timestamp: datetime
    source: str  # 'ai', 'user', 'web'
    approved: bool = False
    

@dataclass
class SessionState:
    """Current state of the tmux session"""
    content: List[str]
    cursor_position: tuple
    size: tuple
    mode: str
    last_activity: datetime


class TmuxAIClient:
    """
    The Tmux AI Client - A real tmux client that can interact!
    Like a player piano that knows all the songs! 🎹
    """
    
    def __init__(
        self,
        session_name: str,
        mode: ClientMode = ClientMode.OBSERVE,
        config_dir: str = "config",
        web_port: Optional[int] = None
    ):
        self.session_name = session_name
        self.mode = mode
        self.config_dir = config_dir
        self.web_port = web_port
        
        # Load configuration
        self.config = self.load_config()
        
        # Initialize AI
        self.init_ai_providers()
        
        # Tmux connection
        self.server = libtmux.Server()
        self.session = None
        self.client_session = None
        self.pty_master = None
        self.pty_slave = None
        
        # Command queue and buffer
        self.command_queue = queue.Queue()
        self.send_buffer = []
        self.feedback_callback: Optional[Callable] = None
        
        # Session state
        self.session_state = SessionState(
            content=[],
            cursor_position=(0, 0),
            size=(80, 24),
            mode="normal",
            last_activity=datetime.now()
        )
        
        # Web interface
        self.web_app = None
        self.websocket_clients = set()
        
        # Control flags
        self.running = False
        self.attached = False
        
    def load_config(self) -> dict:
        """Load configuration"""
        config_file = os.path.join(self.config_dir, "config.yaml")
        if os.path.exists(config_file):
            with open(config_file, 'r') as f:
                return yaml.safe_load(f) or {}
        return {}
        
    def init_ai_providers(self):
        """Initialize AI providers"""
        # Similar to monitor, but focused on interactive assistance
        self.summarization_provider = self.config.get("providers", {}).get("summarization", "openai")
        self.next_step_provider = self.config.get("providers", {}).get("next_step", "openai")
        
        if "openai" in [self.summarization_provider, self.next_step_provider]:
            self.openai_client = OpenAI(api_key=os.getenv("OPENAI_API_KEY"))
            
        if "gemini" in [self.summarization_provider, self.next_step_provider]:
            genai.configure(api_key=os.getenv("GEMINI_API_KEY"))
            
    def connect_to_session(self) -> bool:
        """Connect to tmux session"""
        try:
            sessions = self.server.sessions
            for session in sessions:
                if session.name == self.session_name:
                    self.session = session
                    print(f"{Fore.GREEN}✓ Found tmux session: {self.session_name}{Fore.RESET}")
                    return True
                    
            print(f"{Fore.RED}Session '{self.session_name}' not found!{Fore.RESET}")
            return False
            
        except Exception as e:
            print(f"{Fore.RED}Error connecting: {e}{Fore.RESET}")
            return False
            
    def attach_as_client(self) -> bool:
        """Attach to tmux session as a real client using PTY"""
        try:
            # Create a pseudo-terminal
            self.pty_master, self.pty_slave = pty.openpty()
            
            # Get terminal size
            rows, cols = 24, 80
            try:
                size = struct.unpack('hh', fcntl.ioctl(sys.stdout, termios.TIOCGWINSZ, '1234'))
                rows, cols = size
            except:
                pass
                
            # Set PTY size
            fcntl.ioctl(self.pty_slave, termios.TIOCSWINSZ, struct.pack('hh', rows, cols))
            
            # Start tmux attach command
            cmd = ['tmux', 'attach-session', '-t', self.session_name]
            
            # If in read-only mode for observe/spectate
            if self.mode in [ClientMode.OBSERVE, ClientMode.SPECTATE]:
                cmd.append('-r')  # Read-only mode
                
            self.tmux_process = subprocess.Popen(
                cmd,
                stdin=self.pty_slave,
                stdout=self.pty_slave,
                stderr=self.pty_slave,
                preexec_fn=os.setsid
            )
            
            # Make master non-blocking
            flags = fcntl.fcntl(self.pty_master, fcntl.F_GETFL)
            fcntl.fcntl(self.pty_master, fcntl.F_SETFL, flags | os.O_NONBLOCK)
            
            self.attached = True
            print(f"{Fore.GREEN}✓ Attached to session as {self.mode.value} client{Fore.RESET}")
            return True
            
        except Exception as e:
            print(f"{Fore.RED}Failed to attach: {e}{Fore.RESET}")
            return False
            
    def read_terminal_output(self) -> Optional[bytes]:
        """Read output from the PTY"""
        try:
            # Check if data is available
            if select.select([self.pty_master], [], [], 0)[0]:
                data = os.read(self.pty_master, 4096)
                return data
        except OSError:
            pass
        return None
        
    def send_to_terminal(self, text: str):
        """Send text to the terminal"""
        if self.attached and self.mode != ClientMode.OBSERVE:
            try:
                os.write(self.pty_master, text.encode())
                print(f"{Fore.CYAN}→ Sent: {repr(text)}{Fore.RESET}")
            except OSError as e:
                print(f"{Fore.RED}Failed to send: {e}{Fore.RESET}")
                
    def queue_command(self, command: str, source: str = "ai"):
        """Queue a command for sending"""
        cmd = Command(
            text=command,
            timestamp=datetime.now(),
            source=source,
            approved=(self.mode == ClientMode.COLLABORATE)
        )
        self.command_queue.put(cmd)
        
        if self.mode == ClientMode.ASSIST:
            print(f"{Fore.YELLOW}📋 Queued: {command}{Fore.RESET}")
            print(f"{Fore.GRAY}   (Press 'y' to approve, 'n' to reject){Fore.RESET}")
            
    async def process_terminal_output(self, data: bytes):
        """Process output from terminal and generate AI suggestions"""
        try:
            # Decode terminal output
            text = data.decode('utf-8', errors='replace')
            
            # Update session state
            self.session_state.last_activity = datetime.now()
            
            # Broadcast to web clients
            if self.websocket_clients:
                await self.broadcast_to_web({
                    'type': 'terminal_output',
                    'data': text,
                    'timestamp': self.session_state.last_activity.isoformat()
                })
                
            # Analyze for AI assistance
            if self.should_provide_assistance(text):
                suggestion = await self.generate_suggestion(text)
                if suggestion:
                    print(f"\n{Fore.GREEN}💡 AI Suggestion:{Fore.RESET} {suggestion}")
                    
                    if self.mode == ClientMode.COLLABORATE:
                        self.queue_command(suggestion, source="ai")
                        
        except Exception as e:
            print(f"{Fore.RED}Error processing output: {e}{Fore.RESET}")
            
    def should_provide_assistance(self, text: str) -> bool:
        """Determine if AI assistance is needed"""
        # Look for error messages, prompts, or specific patterns
        assistance_triggers = [
            "error:",
            "failed",
            "not found",
            "permission denied",
            "syntax error",
            "?",  # Help prompt
            ">>",  # REPL prompt
            "$",   # Shell prompt
        ]
        
        text_lower = text.lower()
        return any(trigger in text_lower for trigger in assistance_triggers)
        
    async def generate_suggestion(self, context: str) -> Optional[str]:
        """Generate AI suggestion based on context"""
        # Simplified version - in real implementation, use the AI providers
        prompt = f"""Based on this terminal output, suggest the next helpful command:

Context:
{context}

Provide a single, specific command that would help."""

        try:
            if self.next_step_provider == "openai":
                response = self.openai_client.chat.completions.create(
                    model="gpt-4",
                    messages=[
                        {"role": "system", "content": "You are a helpful terminal assistant."},
                        {"role": "user", "content": prompt}
                    ],
                    max_tokens=100,
                    temperature=0.3
                )
                return response.choices[0].message.content.strip()
        except Exception as e:
            print(f"{Fore.RED}AI suggestion failed: {e}{Fore.RESET}")
            
        return None
        
    async def start_web_interface(self):
        """Start web interface for spectator mode"""
        if not self.web_port:
            return
            
        self.web_app = web.Application()
        
        # Configure CORS
        cors = aiohttp_cors.setup(self.web_app, defaults={
            "*": aiohttp_cors.ResourceOptions(
                allow_credentials=True,
                expose_headers="*",
                allow_headers="*"
            )
        })
        
        # Routes
        self.web_app.router.add_get('/', self.handle_index)
        self.web_app.router.add_get('/ws', self.handle_websocket)
        self.web_app.router.add_post('/command', self.handle_command)
        
        # Apply CORS to routes
        for route in list(self.web_app.router.routes()):
            cors.add(route)
            
        # Start server
        runner = web.AppRunner(self.web_app)
        await runner.setup()
        site = web.TCPSite(runner, 'localhost', self.web_port)
        await site.start()
        
        print(f"{Fore.CYAN}🌐 Web interface: http://localhost:{self.web_port}{Fore.RESET}")
        
    async def handle_index(self, request):
        """Serve the web interface"""
        html = """
<!DOCTYPE html>
<html>
<head>
    <title>Tmux AI Coding Carnival 🎪</title>
    <style>
        body {
            background: #1e1e1e;
            color: #d4d4d4;
            font-family: 'Consolas', 'Monaco', monospace;
            margin: 0;
            padding: 20px;
        }
        #terminal {
            background: #000;
            padding: 10px;
            border-radius: 5px;
            height: 500px;
            overflow-y: auto;
            white-space: pre;
            font-size: 14px;
        }
        #controls {
            margin-top: 20px;
        }
        input {
            background: #2d2d2d;
            color: #d4d4d4;
            border: 1px solid #3e3e3e;
            padding: 5px;
            width: 70%;
        }
        button {
            background: #007acc;
            color: white;
            border: none;
            padding: 5px 15px;
            cursor: pointer;
        }
        .suggestion {
            color: #4ec9b0;
            margin: 5px 0;
        }
        .status {
            color: #608b4e;
            margin: 10px 0;
        }
        h1 {
            text-align: center;
        }
        .jar {
            text-align: center;
            font-size: 20px;
            margin: 20px;
        }
    </style>
</head>
<body>
    <h1>🎹 Tmux AI Coding Carnival 🎪</h1>
    <div class="jar">☕ Put bread in my jar and say "Man, what are you doing here?" 🎵</div>
    
    <div id="terminal"></div>
    
    <div id="controls">
        <input type="text" id="commandInput" placeholder="Suggest a command...">
        <button onclick="sendCommand()">Send</button>
        <button onclick="clearTerminal()">Clear</button>
    </div>
    
    <div class="status" id="status">Connecting...</div>
    
    <script>
        const terminal = document.getElementById('terminal');
        const status = document.getElementById('status');
        const commandInput = document.getElementById('commandInput');
        
        // WebSocket connection
        const ws = new WebSocket(`ws://localhost:${PORT}/ws`);
        
        ws.onopen = () => {
            status.textContent = '🟢 Connected to the carnival!';
        };
        
        ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            
            if (data.type === 'terminal_output') {
                terminal.textContent += data.data;
                terminal.scrollTop = terminal.scrollHeight;
            } else if (data.type === 'suggestion') {
                const suggestion = document.createElement('div');
                suggestion.className = 'suggestion';
                suggestion.textContent = '💡 ' + data.text;
                terminal.appendChild(suggestion);
            }
        };
        
        ws.onclose = () => {
            status.textContent = '🔴 Show\'s over folks!';
        };
        
        function sendCommand() {
            const command = commandInput.value;
            if (command) {
                fetch('/command', {
                    method: 'POST',
                    headers: {'Content-Type': 'application/json'},
                    body: JSON.stringify({command: command})
                });
                commandInput.value = '';
            }
        }
        
        function clearTerminal() {
            terminal.textContent = '';
        }
        
        commandInput.addEventListener('keypress', (e) => {
            if (e.key === 'Enter') sendCommand();
        });
    </script>
</body>
</html>
        """.replace('${PORT}', str(self.web_port))
        
        return web.Response(text=html, content_type='text/html')
        
    async def handle_websocket(self, request):
        """Handle WebSocket connections"""
        ws = web.WebSocketResponse()
        await ws.prepare(request)
        
        self.websocket_clients.add(ws)
        
        try:
            async for msg in ws:
                if msg.type == web.WSMsgType.TEXT:
                    data = json.loads(msg.data)
                    # Handle incoming messages
                    
        finally:
            self.websocket_clients.remove(ws)
            
        return ws
        
    async def handle_command(self, request):
        """Handle command suggestions from web"""
        data = await request.json()
        command = data.get('command')
        
        if command:
            self.queue_command(command, source='web')
            
        return web.json_response({'status': 'queued'})
        
    async def broadcast_to_web(self, data: dict):
        """Broadcast to all websocket clients"""
        if self.websocket_clients:
            message = json.dumps(data)
            await asyncio.gather(
                *[ws.send_str(message) for ws in self.websocket_clients],
                return_exceptions=True
            )
            
    async def run_client_loop(self):
        """Main client loop"""
        print(f"\n{Fore.CYAN}{'='*50}")
        print(f"🎹 Tmux AI Client - {self.mode.value.title()} Mode")
        print(f"{'='*50}{Fore.RESET}\n")
        
        # Connect and attach
        if not self.connect_to_session():
            return
            
        if not self.attach_as_client():
            return
            
        # Start web interface if needed
        if self.web_port:
            await self.start_web_interface()
            
        self.running = True
        
        # Process loop
        try:
            while self.running:
                # Read terminal output
                data = self.read_terminal_output()
                if data:
                    await self.process_terminal_output(data)
                    
                # Process command queue
                try:
                    cmd = self.command_queue.get_nowait()
                    if cmd.approved or self.mode == ClientMode.COLLABORATE:
                        self.send_to_terminal(cmd.text + '\n')
                except queue.Empty:
                    pass
                    
                await asyncio.sleep(0.01)
                
        except KeyboardInterrupt:
            print(f"\n{Fore.YELLOW}🎹 The show's over folks!{Fore.RESET}")
            
        finally:
            self.cleanup()
            
    def cleanup(self):
        """Clean up resources"""
        self.running = False
        
        if self.tmux_process:
            self.tmux_process.terminate()
            
        if self.pty_master:
            os.close(self.pty_master)
        if self.pty_slave:
            os.close(self.pty_slave)
            

@click.command()
@click.argument("session_name")
@click.option(
    "--mode",
    type=click.Choice(["observe", "assist", "collaborate", "spectate"]),
    default="observe",
    help="Client mode"
)
@click.option("--web-port", type=int, help="Port for web interface")
@click.option("--config-dir", default="config", help="Configuration directory")
def main(session_name: str, mode: str, web_port: Optional[int], config_dir: str):
    """
    Tmux AI Client - Attach as a real tmux client!
    
    Modes:
    - observe: Watch and provide suggestions only
    - assist: Queue commands for approval
    - collaborate: Automatically execute AI suggestions
    - spectate: Web-based viewer with suggestion capability
    """
    
    client = TmuxAIClient(
        session_name=session_name,
        mode=ClientMode(mode),
        config_dir=config_dir,
        web_port=web_port
    )
    
    # Run the client
    asyncio.run(client.run_client_loop())


if __name__ == "__main__":
    main()