#!/usr/bin/env python
"""
Remote Tmux Connection Module 🌐
Allows tmux-ai-assistant to connect to remote tmux sessions via SSH!
Part of the tai.is cloud vision - bringing AI assistance to tmux everywhere!

Aye says: "Why be limited to local when we can go global?" 🚀
"""

import subprocess
import json
import re
from typing import List, Optional, Dict, Any
import paramiko
from dataclasses import dataclass
import logging

logger = logging.getLogger(__name__)


@dataclass
class RemoteSession:
    """Represents a tmux session on a remote host"""
    name: str
    windows: int
    created: str
    attached: bool
    host: str
    
    def __str__(self):
        return f"{self.name}@{self.host} ({self.windows} windows)"


class RemoteTmuxConnection:
    """
    Connects to tmux sessions on remote servers via SSH
    This is the magic that will power tai.is! 🎩✨
    """
    
    def __init__(self, host: str, username: str, port: int = 22, 
                 key_filename: Optional[str] = None, password: Optional[str] = None):
        """
        Initialize remote tmux connection
        
        Args:
            host: Remote hostname or IP
            username: SSH username
            port: SSH port (default 22)
            key_filename: Path to SSH private key
            password: SSH password (if not using key)
        """
        self.host = host
        self.username = username
        self.port = port
        self.key_filename = key_filename
        self.password = password
        self.ssh_client = None
        
        # Trisha says: "Always have a backup plan!" 💼
        self._connection_methods = []
        if key_filename:
            self._connection_methods.append(('key', key_filename))
        if password:
            self._connection_methods.append(('password', password))
            
    def connect(self) -> bool:
        """
        Establish SSH connection to remote host
        
        Returns:
            bool: True if connection successful
        """
        if self.ssh_client and self.ssh_client.get_transport() and self.ssh_client.get_transport().is_active():
            return True
            
        self.ssh_client = paramiko.SSHClient()
        self.ssh_client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
        
        # Try each connection method
        for method, credential in self._connection_methods:
            try:
                if method == 'key':
                    self.ssh_client.connect(
                        self.host, 
                        port=self.port, 
                        username=self.username,
                        key_filename=credential,
                        timeout=10
                    )
                else:  # password
                    self.ssh_client.connect(
                        self.host,
                        port=self.port,
                        username=self.username,
                        password=credential,
                        timeout=10
                    )
                
                logger.info(f"🎉 Connected to {self.host} via SSH!")
                return True
                
            except Exception as e:
                logger.warning(f"Failed to connect with {method}: {e}")
                continue
                
        logger.error(f"😢 Could not connect to {self.host}")
        return False
        
    def disconnect(self):
        """Close SSH connection"""
        if self.ssh_client:
            self.ssh_client.close()
            logger.info(f"Disconnected from {self.host}")
            
    def execute_tmux_command(self, command: str) -> Optional[str]:
        """
        Execute a tmux command on the remote host
        
        Args:
            command: tmux command to execute
            
        Returns:
            Command output or None if failed
        """
        if not self.connect():
            return None
            
        try:
            # Ensure we're running tmux command
            if not command.startswith('tmux'):
                command = f'tmux {command}'
                
            stdin, stdout, stderr = self.ssh_client.exec_command(command)
            output = stdout.read().decode('utf-8')
            error = stderr.read().decode('utf-8')
            
            if error and 'no server running' not in error:
                logger.error(f"Remote tmux error: {error}")
                return None
                
            return output
            
        except Exception as e:
            logger.error(f"Failed to execute tmux command: {e}")
            return None
            
    def list_sessions(self) -> List[RemoteSession]:
        """
        List all tmux sessions on remote host
        
        Returns:
            List of RemoteSession objects
        """
        output = self.execute_tmux_command('list-sessions -F "#{session_name}:#{session_windows}:#{session_created}:#{session_attached}"')
        
        if not output:
            return []
            
        sessions = []
        for line in output.strip().split('\n'):
            if not line:
                continue
                
            parts = line.split(':')
            if len(parts) >= 4:
                sessions.append(RemoteSession(
                    name=parts[0],
                    windows=int(parts[1]),
                    created=parts[2],
                    attached=parts[3] == '1',
                    host=self.host
                ))
                
        return sessions
        
    def capture_pane(self, session_name: str, window: int = 0, pane: int = 0) -> Optional[str]:
        """
        Capture content from a specific pane
        
        Args:
            session_name: Name of tmux session
            window: Window index (default 0)
            pane: Pane index (default 0)
            
        Returns:
            Pane content or None if failed
        """
        target = f"{session_name}:{window}.{pane}"
        return self.execute_tmux_command(f'capture-pane -t {target} -p')
        
    def send_keys(self, session_name: str, keys: str, window: int = 0, pane: int = 0):
        """
        Send keys to a specific pane
        
        Args:
            session_name: Name of tmux session  
            keys: Keys to send
            window: Window index (default 0)
            pane: Pane index (default 0)
        """
        target = f"{session_name}:{window}.{pane}"
        # Escape single quotes in the keys
        escaped_keys = keys.replace("'", "'\"'\"'")
        self.execute_tmux_command(f"send-keys -t {target} '{escaped_keys}'")
        
    def get_pane_info(self, session_name: str, window: int = 0, pane: int = 0) -> Optional[Dict[str, Any]]:
        """
        Get detailed information about a pane
        
        Returns:
            Dict with pane information
        """
        target = f"{session_name}:{window}.{pane}"
        format_str = '{"width": #{pane_width}, "height": #{pane_height}, "pid": #{pane_pid}, "current_path": "#{pane_current_path}"}'
        
        output = self.execute_tmux_command(f'display-message -t {target} -p \'{format_str}\'')
        
        if output:
            try:
                return json.loads(output.strip())
            except json.JSONDecodeError:
                logger.error(f"Failed to parse pane info: {output}")
                
        return None
        
    def monitor_session(self, session_name: str, callback=None):
        """
        Monitor a remote tmux session for changes
        Similar to the local monitoring but for remote sessions!
        
        Args:
            session_name: Name of session to monitor
            callback: Function to call with new content
        """
        import time
        
        last_content = ""
        logger.info(f"🔍 Starting to monitor {session_name}@{self.host}")
        
        while True:
            try:
                content = self.capture_pane(session_name)
                
                if content and content != last_content:
                    if callback:
                        callback(content, self.host, session_name)
                    last_content = content
                    
                time.sleep(1)  # Check every second
                
            except KeyboardInterrupt:
                logger.info("Monitoring stopped by user")
                break
            except Exception as e:
                logger.error(f"Error monitoring session: {e}")
                time.sleep(5)  # Wait before retrying


class RemoteTmuxBridge:
    """
    Bridge between local tmux-ai-assistant and remote sessions
    This will be the core of tai.is! 🌉
    """
    
    def __init__(self):
        self.connections: Dict[str, RemoteTmuxConnection] = {}
        
    def add_host(self, alias: str, host: str, username: str, **kwargs):
        """
        Add a remote host configuration
        
        Args:
            alias: Friendly name for the host
            host: Hostname or IP
            username: SSH username
            **kwargs: Additional SSH parameters
        """
        self.connections[alias] = RemoteTmuxConnection(host, username, **kwargs)
        logger.info(f"Added remote host '{alias}' ({username}@{host})")
        
    def get_all_sessions(self) -> Dict[str, List[RemoteSession]]:
        """
        Get all sessions from all configured hosts
        
        Returns:
            Dict mapping host alias to list of sessions
        """
        all_sessions = {}
        
        for alias, conn in self.connections.items():
            try:
                sessions = conn.list_sessions()
                all_sessions[alias] = sessions
                logger.info(f"Found {len(sessions)} sessions on {alias}")
            except Exception as e:
                logger.error(f"Failed to get sessions from {alias}: {e}")
                all_sessions[alias] = []
                
        return all_sessions
        
    def capture_from_remote(self, host_alias: str, session_name: str) -> Optional[str]:
        """
        Capture pane content from a remote session
        
        Args:
            host_alias: Alias of the remote host
            session_name: Name of the tmux session
            
        Returns:
            Pane content or None
        """
        if host_alias not in self.connections:
            logger.error(f"Unknown host: {host_alias}")
            return None
            
        return self.connections[host_alias].capture_pane(session_name)


# Example usage for tai.is integration
if __name__ == "__main__":
    # This is how tai.is could work! 
    bridge = RemoteTmuxBridge()
    
    # Add some remote hosts (in tai.is, these would come from user config)
    bridge.add_host(
        "dev-server",
        "dev.example.com",
        "wraith",
        key_filename="~/.ssh/id_rsa"
    )
    
    # Get all sessions
    sessions = bridge.get_all_sessions()
    
    for host, host_sessions in sessions.items():
        print(f"\n🖥️  {host}:")
        for session in host_sessions:
            print(f"  - {session}")
            
    # Monitor a specific remote session
    # This would integrate with the AI processing!
    # conn = bridge.connections["dev-server"]
    # conn.monitor_session("main", lambda content, host, session: 
    #     print(f"New content from {session}@{host}: {len(content)} chars"))