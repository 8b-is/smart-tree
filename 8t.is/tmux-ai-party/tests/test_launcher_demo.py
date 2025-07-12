#!/usr/bin/env python3
"""
Test the launcher's functionality programmatically
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import importlib.util
spec = importlib.util.spec_from_file_location("launcher", "tmux-ai-launcher.py")
launcher_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(launcher_module)
InteractiveLauncher = launcher_module.InteractiveLauncher
from colorama import init, Fore, Style

init(autoreset=True)

def test_launcher():
    """Test the launcher's core functionality"""
    
    print(f"{Fore.CYAN}🎪 Testing Tmux AI Assistant Launcher 🎪{Style.RESET_ALL}\n")
    
    launcher = InteractiveLauncher()
    
    # Test 1: Load configuration
    print(f"{Fore.GREEN}Test 1: Loading configuration{Fore.RESET}")
    config = launcher.load_config()
    assert config, "Configuration should be loaded"
    print(f"  ✓ Config loaded successfully")
    
    # Test 2: List tmux sessions
    print(f"\n{Fore.GREEN}Test 2: Listing tmux sessions{Fore.RESET}")
    sessions = launcher.list_tmux_sessions()
    assert isinstance(sessions, list), "list_tmux_sessions should return a list"
    print(f"  ✓ Found {len(sessions)} sessions: {sessions}")
    
    # Test 3: Building commands for different modes
    print(f"\n{Fore.GREEN}Test 3: Building commands for different modes{Fore.RESET}")
    
    # Simulate choices for local monitoring
    launcher.choices = {'location': 'local', 'mode': 'monitor'}
    launcher.session_info = {'session_name': 'coding'}
    cmd = launcher.build_command()
    cmd_str = ' '.join(cmd)
    assert 'tmux_monitor.py' in cmd_str
    assert '--session-name coding' in cmd_str
    print(f"  ✓ Monitor mode command: {cmd_str}")
    
    # Simulate choices for attach mode
    launcher.choices = {'location': 'local', 'mode': 'attach'}
    cmd = launcher.build_command()
    cmd_str = ' '.join(cmd)
    assert 'tmux_attach.py' in cmd_str
    print(f"  ✓ Attach mode command: {cmd_str}")
    
    # Simulate choices for web mode
    launcher.choices = {'location': 'local', 'mode': 'web'}
    launcher.session_info = {'session_name': 'coding', 'web_port': '8080'}
    cmd = launcher.build_command()
    cmd_str = ' '.join(cmd)
    assert '--mode web' in cmd_str
    assert '--web-port 8080' in cmd_str
    print(f"  ✓ Web mode command: {cmd_str}")
    
    # Simulate choices for MCP API mode
    launcher.choices = {'location': 'local', 'mode': 'api'}
    launcher.session_info = {'session_name': 'coding'}
    cmd = launcher.build_command()
    cmd_str = ' '.join(cmd)
    assert '--mode api' in cmd_str
    print(f"  ✓ API/MCP mode command: {cmd_str}")
    
    # Test 4: Remote session scenarios
    print(f"\n{Fore.GREEN}Test 4: Remote session scenarios{Fore.RESET}")
    
    # SSH session
    launcher.choices = {'location': 'ssh', 'mode': 'monitor'}
    launcher.session_info = {
        'session_name': 'remote-coding',
        'ssh_host': 'example.com',
        'ssh_user': 'hue',
        'ssh_key': '~/.ssh/id_rsa'
    }
    cmd = launcher.build_command()
    cmd_str = ' '.join(cmd)
    assert 'ssh' in cmd_str and 'hue@example.com' in cmd_str and 'remote-coding' in cmd_str
    print(f"  ✓ SSH mode command: {cmd_str}")
    
    # Docker session
    launcher.choices = {'location': 'docker', 'mode': 'monitor'}
    launcher.session_info = {
        'session_name': 'container-session',
        'container': 'my-dev-container'
    }
    cmd = launcher.build_command()
    cmd_str = ' '.join(cmd)
    assert 'docker exec' in cmd_str and 'my-dev-container' in cmd_str and 'container-session' in cmd_str
    print(f"  ✓ Docker mode command: {cmd_str}")
    
    # Test 5: Feature detection
    print(f"\n{Fore.GREEN}Test 5: Interactive features simulation{Fore.RESET}")
    
    # Test the banner
    launcher.print_banner()
    
    # Test animated print
    launcher.animated_print("🎹 Testing typewriter effect... ", delay=0.01)
    
    print(f"\n{Fore.YELLOW}✨ All tests completed! The launcher is ready for interactive use.{Fore.RESET}")
    
    # Show example interactive usage
    print(f"\n{Fore.CYAN}Example interactive usage:{Fore.RESET}")
    print("  1. Run: ./tmux-ai-launcher.py")
    print("  2. Choose session location (local/SSH/Docker/K8s)")
    print("  3. Select or create tmux session")
    print("  4. Pick interaction mode (monitor/attach/web/etc)")
    print("  5. Configure sharing and AI options")
    print("  6. Save configuration for quick reuse")
    print("  7. Launch! 🚀")
    
    print(f"\n{Fore.GREEN}Quick launch saved configs:{Fore.RESET}")
    print("  ./tmux-ai-launcher.py --load my-dev-setup")

if __name__ == "__main__":
    try:
        test_launcher()
    except Exception as e:
        print(f"{Fore.RED}Error: {e}{Fore.RESET}")
        sys.exit(1)