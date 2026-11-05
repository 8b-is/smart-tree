#!/bin/bash

# Welcome to the Grand Ole Opry of scripts!
# This is the manage.sh for "The King's Code," your one-stop-shop for managing this rock 'n' roll project.
# Brought to you by the letter 'A' for 'Aye' and 'H' for 'Hue'.
# Trisha in Accounting approved this message.

# --- Configuration: Tune Your Guitar ---
API_DIR="api"
WEB_DIR="web"
ENGINE_DIR="engine"
VENV_DIR=".venv"

# --- Colors: Because Life's Too Short for Black and White Terminals ---
C_RESET='\033[0m'
C_RED='\033[0;31m'
C_GREEN='\033[0;32m'
C_YELLOW='\033[0;33m'
C_BLUE='\033[0;34m'
C_PURPLE='\033[0;35m'
C_CYAN='\033[0;36m'

# --- Helper Functions: The Roadies ---
function print_rockstar() {
    echo -e "${C_CYAN}$1${C_RESET}"
}

function print_success() {
    echo -e "${C_GREEN}$1${C_RESET}"
}

function print_warning() {
    echo -e "${C_YELLOW}$1${C_RESET}"
}

function print_error() {
    echo -e "${C_RED}$1${C_RESET}"
}

function print_stage_direction() {
    echo -e "${C_PURPLE}$1${C_RESET}"
}

# --- The Main Acts: The Functions ---

# The Opening Act: Setup
function setup() {
    print_stage_direction "Setting the stage for The King's Code... 🎸"

    # API Setup (Python)
    print_rockstar "Setting up the API in '$API_DIR'..."
    if [ ! -d "$API_DIR/$VENV_DIR" ]; then
        python3 -m venv "$API_DIR/$VENV_DIR"
        print_success "Virtual environment created for the API."
    else
        print_warning "API virtual environment already exists. Skipping."
    fi
    source "$API_DIR/$VENV_DIR/bin/activate"
    pip install -r "$API_DIR/requirements.txt"
    deactivate
    print_success "API dependencies installed."

    # Web Setup (Svelte/Node.js)
    print_rockstar "Setting up the web frontend in '$WEB_DIR'..."
    (cd "$WEB_DIR" && npm install)
    print_success "Web dependencies installed."

    # Engine Setup (Rust)
    print_rockstar "Building the analysis engine in '$ENGINE_DIR'..."
    (cd "$ENGINE_DIR" && cargo build --release)
    print_success "Analysis engine built."

    print_stage_direction "Setup complete! You're ready to rock. 🤘"
}

# The Headliner: Start
function start() {
    print_stage_direction "Ladies and gentlemen, please welcome... The King's Code! 🎤"

    # Start API
    print_rockstar "Starting the API server..."
    (source "$API_DIR/$VENV_DIR/bin/activate" && uvicorn main:app --host 0.0.0.0 --port 8000 &)

    # Start Web
    print_rockstar "Starting the web server..."
    (cd "$WEB_DIR" && npm run dev &)

    print_success "All services are up and running! Check out the show at http://localhost:5173"
}

# The Encore: Stop
function stop() {
    print_stage_direction "Show's over, folks! You don't have to go home, but you can't stay here."
    pkill -f uvicorn
    pkill -f "npm run dev"
    print_success "All services have been stopped. The stage is clear."
}

# The Afterparty: Test
function test() {
    print_stage_direction "Sound check! 1, 2, 3... Is this thing on?"

    # Test API
    print_rockstar "Running API tests..."
    (source "$API_DIR/$VENV_DIR/bin/activate" && pytest)

    # Test Web
    print_rockstar "Running web tests..."
    (cd "$WEB_DIR" && npm test)

    # Test Engine
    print_rockstar "Running engine tests..."
    (cd "$ENGINE_DIR" && cargo test)

    print_success "All tests passed! We're ready for the big show."
}


# --- The Setlist: The Main Logic ---
case "$1" in
    setup)
        setup
        ;;
    start)
        start
        ;;
    stop)
        stop
        ;;
    restart)
        stop
        start
        ;;
    test)
        test
        ;;
    *)
        echo "Usage: $0 {setup|start|stop|restart|test}"
        exit 1
        ;;
esac

exit 0