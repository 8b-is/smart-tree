#!/usr/bin/env python3
"""
First run checker - ensures configuration is complete
"""

import os
import sys
import subprocess
from colorama import init, Fore, Style

init(autoreset=True)

def check_first_run():
    """Check if setup wizard needs to run"""
    
    # Check for configuration markers
    markers = [
        ".env",
        "config/config.yaml",
        "config/.wizard_complete"
    ]
    
    if not any(os.path.exists(marker) for marker in markers):
        print(f"\n{Fore.YELLOW}{'=' * 60}")
        print(f"{Style.BRIGHT}First time setup required!{Style.NORMAL}")
        print(f"{'=' * 60}{Fore.RESET}\n")
        
        print(f"{Fore.GREEN}Welcome to Tmux AI Assistant!{Fore.RESET}")
        print(f"Let's get you set up with our interactive wizard.\n")
        
        # Run the setup wizard
        try:
            subprocess.run([sys.executable, "setup_wizard.py"], check=True)
            print(f"\n{Fore.GREEN}Setup complete! Continuing...{Fore.RESET}\n")
            return True
        except subprocess.CalledProcessError:
            print(f"\n{Fore.RED}Setup cancelled. Please run setup_wizard.py to configure.{Fore.RESET}")
            sys.exit(1)
        except KeyboardInterrupt:
            print(f"\n{Fore.YELLOW}Setup interrupted.{Fore.RESET}")
            sys.exit(1)
            
    # Check if .env exists but might be incomplete
    elif os.path.exists(".env"):
        with open(".env", "r") as f:
            content = f.read()
            
        # Check for common missing values
        if "your-openai-key" in content or "your-gemini-key" in content:
            print(f"\n{Fore.YELLOW}⚠️  Your .env file contains placeholder values!{Fore.RESET}")
            print(f"Run {Fore.CYAN}python setup_wizard.py --reconfigure{Fore.RESET} to fix this.\n")
            
            if "--force" not in sys.argv:
                response = input(f"{Fore.CYAN}Continue anyway? [y/N]: {Fore.RESET}").lower()
                if response != 'y':
                    sys.exit(1)
                    
    return False

if __name__ == "__main__":
    check_first_run()