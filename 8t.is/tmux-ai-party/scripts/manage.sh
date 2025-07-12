#!/bin/bash

# scripts/manage.sh
# The Grand Orchestrator of the MCP Server Enlightenment Project! 🎶
# Managed by Aye, with a little help from Hue and the ever-sparkling Trish!

# --- Configuration & Setup ---
# Define our project's main Python server file.
# This is where the magic happens, like Elvis's stage!
MCP_SERVER_FILE="mcp_server.py"
# The virtual environment directory. Keep things tidy!
VENV_DIR=".venv" # Standard Python virtual environment name
# The directory where our main HTML documentation lives.
DOCS_DIR="docs"
# The main HTML file we're improving.
TARGET_HTML_FILE="MCP servers - OpenAI API.html"
# The improved HTML file (once we create it!)
IMPROVED_HTML_FILE="MCP servers - OpenAI API_improved.html"
# Our custom CSS file for that extra sparkle!
CUSTOM_CSS_FILE="styles.css"
# Our custom JS file for dynamic flair!
CUSTOM_JS_FILE="scripts.js"

# --- Helper Functions: Because Repetition is for Robots, Not Us! ---

# Function to display a snazzy header for each action.
# Makes the terminal output look like a concert marquee!
display_header() {
    echo -e "\n✨✨✨ $1 ✨✨✨"
    echo "--------------------------------------------------"
}

# Function to check if the virtual environment is set up.
# We don't want to run without our proper stage setup!
check_venv() {
    if [ ! -d "$VENV_DIR" ]; then
        display_header "Setting up Python Virtual Environment 🐍"
        python3 -m venv "$VENV_DIR" || { echo "Error: Failed to create virtual environment. Is Python 3 installed?"; exit 1; }
        echo "Virtual environment created at $VENV_DIR."
    fi
    # Activate the virtual environment
    source "$VENV_DIR/bin/activate" || { echo "Error: Failed to activate virtual environment."; exit 1; }
    echo "Virtual environment activated!"
}

# Function to install Python dependencies.
# Gotta get all our band members ready!
install_dependencies() {
    display_header "Installing Python Dependencies 📦"
    if [ -f "requirements.txt" ]; then
        pip install -r requirements.txt || { echo "Error: Failed to install Python dependencies."; exit 1; }
        echo "Python dependencies installed!"
    else
        echo "No requirements.txt found. Skipping dependency installation."
    fi
}

# --- Core Commands: The Main Acts! ---

# Start the MCP server. Let the show begin!
# Start the MCP server. Let the show begin!
# Arguments: [session_name] [ai_provider] [openai_api_key] [gemini_api_key] [port]
start_mcp_server() {
    display_header "Starting Unified MCP Server 🚀"
    check_venv
    install_dependencies # Ensure dependencies are installed before running

    local session_name="$1"
    local ai_provider="${2:-openai}" # Default to openai if not provided
    local openai_api_key="$3"
    local gemini_api_key="$4"
    local port="${5:-8000}" # Default to 8000 if not provided

    if [ -z "$session_name" ]; then
        echo -e "\e[31mError: Session name is required to start the server.\e[0m"
        echo "Usage: ./scripts/manage.sh start <session_name> [ai_provider] [openai_api_key] [gemini_api_key] [port]"
        exit 1
    fi

    echo -e "\e[34mStarting MCP Server for session: '$session_name' with AI Provider: '$ai_provider' on port: '$port'\e[0m"
    echo -e "\e[34mRemember to set OPENAI_API_KEY and/or GEMINI_API_KEY in your .env file or pass them as arguments!\e[0m"

    # Construct the command dynamically
    SERVER_COMMAND="python $MCP_SERVER_FILE $session_name --ai-provider $ai_provider --port $port"

    if [ "$ai_provider" == "openai" ] && [ -n "$openai_api_key" ]; then
        SERVER_COMMAND="$SERVER_COMMAND --openai-api-key $openai_api_key"
    elif [ "$ai_provider" == "gemini" ] && [ -n "$gemini_api_key" ]; then
        SERVER_COMMAND="$SERVER_COMMAND --gemini-api-key $gemini_api_key"
    fi

    # Using 'exec' to replace the current shell process with the server,
    # allowing it to receive signals directly. This is more robust than nohup.
    # For backgrounding, Hue can use '&' when calling this script.
    echo -e "\e[35mExecuting: $SERVER_COMMAND\e[0m"
    exec $SERVER_COMMAND
}

# Stop the MCP server. Time for a break!
stop_mcp_server() {
    display_header "Stopping MCP Server 🛑"
    # Find the PID of the mcp_server.py process
    SERVER_PID=$(pgrep -f "python $MCP_SERVER_FILE")
    if [ -n "$SERVER_PID" ]; then
        echo -e "\e[33mStopping MCP Server (PID: $SERVER_PID)...\e[0m"
        kill "$SERVER_PID"
        echo -e "\e[32mMCP Server stopped!\e[0m"
    else
        echo -e "\e[33mNo MCP Server found running.\e[0m"
    fi
}

# Restart the MCP server. Encore!
restart_mcp_server() {
    display_header "Restarting MCP Server 🔄"
    stop_mcp_server
    sleep 2 # Give it a moment to catch its breath, like a true performer!
    # Re-use the arguments passed to restart, if any
    start_mcp_server "$2" "$3" "$4" "$5" "$6"
}

# Build the project (e.g., process HTML, CSS, JS).
# This is where we make things pretty and performant!
# Build the project (e.g., install dependencies).
# This is where we make things ready for action!
build_project() {
    display_header "Building Project 🏗️"
    check_venv
    install_dependencies
    echo -e "\e[32mProject build complete! Ready to rock and roll! 🎸\e[0m"
}

# Run tests. Hue's sanity check!
# Run tests. Hue's sanity check!
run_tests() {
    display_header "Running Tests 🧪"
    check_venv
    install_dependencies # Ensure test dependencies are met
    echo -e "\e[34mRunning pytest for our Python friends...\e[0m"
    pytest -v --color=yes || { echo -e "\e[31mTests failed! Time to debug, Hue!\e[0m"; exit 1; }
    echo -e "\e[32mAll tests passed! You're a testing superstar, Hue! ⭐\e[0m"
}

# Clean up generated files. Keep it spotless!
clean_project() {
    display_header "Cleaning Project 🧹"
    echo -e "\e[33mRemoving virtual environment and log files...\e[0m"
    rm -rf "$VENV_DIR"
    rm -rf "logs/" # Remove the entire logs directory
    rm -f "mcp_server.log" # In case it was created by nohup
    echo -e "\e[32mCleanup complete! Fresh as a daisy! 🌼\e[0m"
}

# --- Main Script Logic: The Conductor's Baton ---

case "$1" in
    start_mcp_server)
        start_mcp_server
        ;;
    stop_mcp_server)
        stop_mcp_server
        ;;
    restart_mcp_server)
        restart_mcp_server
        ;;
    build)
        build_project
        ;;
    test)
        run_tests
        ;;
    clean)
        clean_project
        ;;
    *)
        echo "Usage: ./scripts/manage.sh {start_mcp_server|stop_mcp_server|restart_mcp_server|build|test|clean}"
        echo "Aye, Aye! Choose your adventure!"
        ;;
esac

echo -e "\n--- Operation Complete! Aye, Aye! 🚢 ---"