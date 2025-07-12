#!/usr/bin/env python
"""
TAI Remote CLI - Connect to remote tmux sessions with AI assistance!
This demonstrates how tai.is will work for remote monitoring.

Usage:
    ./tai_remote_cli.py add host1 server1.com username
    ./tai_remote_cli.py list
    ./tai_remote_cli.py monitor host1:session
    ./tai_remote_cli.py connect host1
"""

import click
import os
import yaml
from pathlib import Path
from typing import Dict, Optional
import logging
from remote_tmux import RemoteTmuxBridge, RemoteTmuxConnection
from tmux_monitor import TmuxAIMonitor, ConfigWatcher

# Set up colorful logging - Trisha insists! 
logging.basicConfig(
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    level=logging.INFO
)
logger = logging.getLogger(__name__)


class RemoteConfig:
    """Manage remote host configurations"""
    
    def __init__(self):
        self.config_dir = Path.home() / '.tai' / 'config'
        self.config_file = self.config_dir / 'remotes.yaml'
        self.config_dir.mkdir(parents=True, exist_ok=True)
        self.hosts = self._load_config()
        
    def _load_config(self) -> Dict:
        """Load remote hosts configuration"""
        if self.config_file.exists():
            with open(self.config_file, 'r') as f:
                return yaml.safe_load(f) or {'hosts': {}}
        return {'hosts': {}}
        
    def save_config(self):
        """Save configuration to disk"""
        with open(self.config_file, 'w') as f:
            yaml.dump(self.hosts, f, default_flow_style=False)
            
    def add_host(self, alias: str, hostname: str, username: str, 
                 port: int = 22, key_file: Optional[str] = None):
        """Add a remote host configuration"""
        
        # Expand key file path
        if key_file:
            key_file = str(Path(key_file).expanduser())
            
        self.hosts['hosts'][alias] = {
            'hostname': hostname,
            'username': username,
            'port': port,
            'key_file': key_file
        }
        self.save_config()
        
    def get_host(self, alias: str) -> Optional[Dict]:
        """Get host configuration by alias"""
        return self.hosts['hosts'].get(alias)
        
    def list_hosts(self) -> Dict:
        """List all configured hosts"""
        return self.hosts['hosts']


@click.group()
def cli():
    """TAI Remote - Monitor tmux sessions on any server! 🌐"""
    pass


@cli.command()
@click.argument('alias')
@click.argument('hostname')
@click.argument('username')
@click.option('--port', default=22, help='SSH port')
@click.option('--key', help='Path to SSH private key')
def add(alias, hostname, username, port, key):
    """Add a remote host configuration"""
    
    config = RemoteConfig()
    
    # Use default SSH key if not specified
    if not key:
        default_key = Path.home() / '.ssh' / 'id_rsa'
        if default_key.exists():
            key = str(default_key)
            click.echo(f"Using default SSH key: {key}")
            
    config.add_host(alias, hostname, username, port, key)
    
    click.echo(f"✅ Added remote host '{alias}' ({username}@{hostname})")
    click.echo(f"   You can now: tai remote monitor {alias}:session")


@cli.command()
def list():
    """List all configured remote hosts"""
    
    config = RemoteConfig()
    hosts = config.list_hosts()
    
    if not hosts:
        click.echo("No remote hosts configured yet!")
        click.echo("Add one with: tai remote add <alias> <hostname> <username>")
        return
        
    click.echo("🌐 Configured Remote Hosts:")
    click.echo("")
    
    for alias, info in hosts.items():
        click.echo(f"  📍 {alias}")
        click.echo(f"     Host: {info['username']}@{info['hostname']}")
        click.echo(f"     Port: {info.get('port', 22)}")
        if info.get('key_file'):
            click.echo(f"     Key:  {info['key_file']}")
        click.echo("")


@cli.command()
@click.argument('target')  # format: host:session or just host
def monitor(target):
    """Monitor a remote tmux session with AI assistance"""
    
    # Parse target
    if ':' in target:
        host_alias, session_name = target.split(':', 1)
    else:
        host_alias = target
        session_name = None
        
    config = RemoteConfig()
    host_info = config.get_host(host_alias)
    
    if not host_info:
        click.echo(f"❌ Unknown host: {host_alias}")
        click.echo("Use 'tai remote list' to see configured hosts")
        return
        
    # Create remote connection
    conn = RemoteTmuxConnection(
        host=host_info['hostname'],
        username=host_info['username'],
        port=host_info.get('port', 22),
        key_filename=host_info.get('key_file')
    )
    
    # Connect to remote
    click.echo(f"🔌 Connecting to {host_alias}...")
    if not conn.connect():
        click.echo(f"❌ Failed to connect to {host_alias}")
        return
        
    # List sessions if none specified
    if not session_name:
        sessions = conn.list_sessions()
        if not sessions:
            click.echo(f"No tmux sessions found on {host_alias}")
            return
            
        click.echo(f"\n📋 Tmux sessions on {host_alias}:")
        for i, session in enumerate(sessions):
            click.echo(f"  {i+1}. {session}")
            
        # Let user choose
        choice = click.prompt("Select session number", type=int)
        if 1 <= choice <= len(sessions):
            session_name = sessions[choice-1].name
        else:
            click.echo("Invalid choice!")
            return
            
    # Start monitoring!
    click.echo(f"\n🔍 Monitoring {session_name}@{host_alias}")
    click.echo("Press Ctrl+C to stop monitoring")
    click.echo("")
    
    # This is where we'd integrate with the AI monitor
    # For now, just show the content
    def on_content_change(content, host, session):
        click.echo(f"\n--- Update from {session}@{host} ---")
        # Here we would send to AI for processing
        # For demo, just show last 10 lines
        lines = content.strip().split('\n')
        for line in lines[-10:]:
            click.echo(line)
        click.echo("--- End Update ---\n")
        
    try:
        conn.monitor_session(session_name, on_content_change)
    except KeyboardInterrupt:
        click.echo("\n👋 Monitoring stopped")
        
    conn.disconnect()


@cli.command()
@click.argument('host_alias')
def connect(host_alias):
    """SSH directly to a remote host"""
    
    config = RemoteConfig()
    host_info = config.get_host(host_alias)
    
    if not host_info:
        click.echo(f"❌ Unknown host: {host_alias}")
        return
        
    # Build SSH command
    ssh_cmd = ['ssh']
    
    if host_info.get('key_file'):
        ssh_cmd.extend(['-i', host_info['key_file']])
        
    if host_info.get('port') and host_info['port'] != 22:
        ssh_cmd.extend(['-p', str(host_info['port'])])
        
    ssh_cmd.append(f"{host_info['username']}@{host_info['hostname']}")
    
    # Execute SSH
    import subprocess
    click.echo(f"🚀 Connecting to {host_alias}...")
    subprocess.run(ssh_cmd)


@cli.command()
@click.argument('host_alias')
def sessions(host_alias):
    """List tmux sessions on a remote host"""
    
    config = RemoteConfig()
    host_info = config.get_host(host_alias)
    
    if not host_info:
        click.echo(f"❌ Unknown host: {host_alias}")
        return
        
    conn = RemoteTmuxConnection(
        host=host_info['hostname'],
        username=host_info['username'],
        port=host_info.get('port', 22),
        key_filename=host_info.get('key_file')
    )
    
    click.echo(f"🔌 Connecting to {host_alias}...")
    if not conn.connect():
        click.echo(f"❌ Failed to connect to {host_alias}")
        return
        
    sessions = conn.list_sessions()
    
    if not sessions:
        click.echo(f"No tmux sessions found on {host_alias}")
    else:
        click.echo(f"\n📋 Tmux sessions on {host_alias}:")
        for session in sessions:
            status = "📍 attached" if session.attached else "⚪ detached"
            click.echo(f"  - {session.name} ({session.windows} windows) {status}")
            
    conn.disconnect()


@cli.command()
def bridge():
    """Show all sessions from all configured hosts (tai.is style!)"""
    
    config = RemoteConfig()
    hosts = config.list_hosts()
    
    if not hosts:
        click.echo("No remote hosts configured!")
        return
        
    bridge = RemoteTmuxBridge()
    
    # Add all configured hosts
    for alias, info in hosts.items():
        bridge.add_host(
            alias,
            info['hostname'],
            info['username'],
            port=info.get('port', 22),
            key_filename=info.get('key_file')
        )
        
    click.echo("🌐 Scanning all remote hosts for tmux sessions...")
    click.echo("")
    
    all_sessions = bridge.get_all_sessions()
    
    total_sessions = 0
    for host_alias, sessions in all_sessions.items():
        if sessions:
            click.echo(f"📍 {host_alias}:")
            for session in sessions:
                status = "attached" if session.attached else "detached"
                click.echo(f"   - {session.name} ({session.windows} windows) [{status}]")
            click.echo("")
            total_sessions += len(sessions)
        else:
            click.echo(f"📍 {host_alias}: No sessions")
            
    click.echo(f"\n✨ Total: {total_sessions} sessions across {len(hosts)} hosts")
    click.echo("\nTip: Monitor any session with: tai remote monitor <host>:<session>")


if __name__ == '__main__':
    # Make it feel like tai.is!
    import sys
    
    if len(sys.argv) == 1:
        print("""
🌐 TAI Remote - Monitor tmux sessions anywhere!

Quick Start:
  1. Add a host:    ./tai_remote_cli.py add myserver server.com username
  2. List hosts:    ./tai_remote_cli.py list  
  3. Monitor:       ./tai_remote_cli.py monitor myserver:main
  
This is a preview of tai.is remote functionality!
""")
    
    cli()