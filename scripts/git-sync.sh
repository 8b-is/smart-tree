#!/bin/bash
# 🎸 Smart Tree Multi-Remote Git Sync Manager
# "In the franchise wars, all git hosts are Smart Tree repos!" - The Quantum Cheet
#
# This script manages synchronization across multiple git remotes, allowing:
# - Selective pushes to specific remotes
# - Experimental branch management
# - Fork synchronization
# - Temperature control (conservative vs aggressive sync)

set -euo pipefail

# Colors for our rock concert
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
REMOTES=(
    "origin:github.com:Primary GitHub repository"
    "forgejo:g.8b.is:8b.is Forgejo instance"
    "gitlab:gitlab.com:GitLab mirror"
)

# Default temperature (0-10, higher = more aggressive)
TEMP=${GIT_TEMP:-5}

# Function to print with style
print_header() {
    echo -e "\n${PURPLE}═══════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}🎸 $1${NC}"
    echo -e "${PURPLE}═══════════════════════════════════════════════════════${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Show current remotes
show_remotes() {
    print_header "Current Git Remotes"
    git remote -v | column -t
}

# Check remote status
check_remote_status() {
    local remote=$1
    print_info "Checking $remote..."
    
    # Fetch remote refs
    if git fetch "$remote" --dry-run 2>&1 | grep -q "fatal"; then
        print_error "$remote is not accessible"
        return 1
    else
        print_success "$remote is accessible"
        
        # Show branch differences
        local branches=$(git branch -r | grep "^  $remote/" | sed "s/  $remote\///")
        if [ -n "$branches" ]; then
            echo "  Branches: $(echo $branches | tr '\n' ' ')"
        fi
        return 0
    fi
}

# Sync to specific remote
sync_to_remote() {
    local remote=$1
    local branch=${2:-$(git branch --show-current)}
    local force=${3:-false}
    
    print_info "Syncing branch '$branch' to $remote..."
    
    if [ "$force" = "true" ] || [ "$TEMP" -ge 8 ]; then
        print_warning "Force pushing to $remote (temperature: $TEMP)"
        git push "$remote" "$branch" --force-with-lease
    else
        git push "$remote" "$branch"
    fi
    
    print_success "Synced to $remote"
}

# Selective push based on patterns
selective_push() {
    local pattern=$1
    local remote=$2
    
    print_header "Selective Push: $pattern → $remote"
    
    # Find branches matching pattern
    local branches=$(git branch | grep -E "$pattern" | sed 's/\* //g' | tr -d ' ')
    
    if [ -z "$branches" ]; then
        print_warning "No branches match pattern: $pattern"
        return
    fi
    
    echo "Found branches:"
    echo "$branches" | sed 's/^/  - /'
    
    if [ "$TEMP" -lt 3 ]; then
        read -p "Continue with push? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_info "Push cancelled"
            return
        fi
    fi
    
    for branch in $branches; do
        sync_to_remote "$remote" "$branch"
    done
}

# Create experimental branch on specific remote
create_experimental() {
    local branch_name=$1
    local remote=${2:-forgejo}
    local base=${3:-main}
    
    print_header "Creating Experimental Branch"
    
    # Create branch
    git checkout -b "$branch_name" "$base"
    print_success "Created branch: $branch_name"
    
    # Push only to specified remote
    git push -u "$remote" "$branch_name"
    print_success "Pushed to $remote only"
    
    # Set up tracking
    git branch --set-upstream-to="$remote/$branch_name"
    
    print_info "Branch '$branch_name' is now tracking $remote"
    print_info "Use 'git push' to push to $remote only"
    print_info "Use './git-sync.sh push-all' to sync everywhere"
}

# Fork sync - pull from one remote, push to another
fork_sync() {
    local source=$1
    local target=$2
    local branch=${3:-main}
    
    print_header "Fork Sync: $source → $target"
    
    # Fetch from source
    print_info "Fetching from $source..."
    git fetch "$source"
    
    # Checkout and update
    git checkout "$branch"
    git merge "$source/$branch" --ff-only || {
        print_warning "Cannot fast-forward, attempting rebase..."
        if [ "$TEMP" -ge 7 ]; then
            git rebase "$source/$branch"
        else
            print_error "Merge conflict detected. Increase temperature or resolve manually."
            return 1
        fi
    }
    
    # Push to target
    sync_to_remote "$target" "$branch"
}

# Temperature-based sync strategy
temp_sync() {
    print_header "Temperature-Based Sync (Current: $TEMP)"
    
    case $TEMP in
        0|1|2)
            print_info "Conservative mode - only syncing current branch to origin"
            sync_to_remote "origin" "$(git branch --show-current)"
            ;;
        3|4|5)
            print_info "Moderate mode - syncing main branches to all remotes"
            for remote in origin forgejo gitlab; do
                if git remote | grep -q "^$remote$"; then
                    sync_to_remote "$remote" "main"
                fi
            done
            ;;
        6|7|8)
            print_info "Warm mode - syncing all branches to primary remotes"
            for branch in $(git branch | sed 's/\* //g' | tr -d ' '); do
                sync_to_remote "origin" "$branch"
                sync_to_remote "forgejo" "$branch"
            done
            ;;
        9|10)
            print_info "Hot mode - full sync to all remotes!"
            print_warning "🔥 Maximum temperature! Syncing everything everywhere!"
            for remote in $(git remote); do
                for branch in $(git branch | sed 's/\* //g' | tr -d ' '); do
                    sync_to_remote "$remote" "$branch" true
                done
            done
            ;;
        *)
            print_error "Invalid temperature: $TEMP (use 0-10)"
            exit 1
            ;;
    esac
}

# Multi-remote push
push_all() {
    local branch=${1:-$(git branch --show-current)}
    
    print_header "Pushing to All Remotes"
    
    for remote in $(git remote); do
        if check_remote_status "$remote" >/dev/null 2>&1; then
            sync_to_remote "$remote" "$branch"
        fi
    done
}

# Setup remote tracking for existing branch
setup_multi_tracking() {
    local branch=${1:-$(git branch --show-current)}
    
    print_header "Setting up Multi-Remote Tracking"
    
    # Create a custom push configuration
    git config --local "branch.$branch.pushRemote" "origin"
    
    # Add push URLs for simultaneous push
    for remote in $(git remote | grep -v origin); do
        local url=$(git remote get-url "$remote")
        git remote set-url --add --push origin "$url"
    done
    
    print_success "Branch '$branch' configured for multi-remote push"
    print_info "Use 'git push' to push to all remotes simultaneously"
}

# Interactive menu
interactive_menu() {
    while true; do
        print_header "Smart Tree Git Sync Manager"
        echo "1) Show remotes"
        echo "2) Check remote status"
        echo "3) Push current branch to all"
        echo "4) Selective push"
        echo "5) Create experimental branch"
        echo "6) Fork sync"
        echo "7) Temperature-based sync"
        echo "8) Setup multi-tracking"
        echo "9) Set temperature (current: $TEMP)"
        echo "0) Exit"
        
        read -p "Choose an option: " choice
        
        case $choice in
            1) show_remotes ;;
            2) 
                for remote in $(git remote); do
                    check_remote_status "$remote"
                done
                ;;
            3) push_all ;;
            4)
                read -p "Branch pattern (regex): " pattern
                read -p "Target remote: " remote
                selective_push "$pattern" "$remote"
                ;;
            5)
                read -p "Branch name: " branch
                read -p "Remote (default: forgejo): " remote
                create_experimental "$branch" "${remote:-forgejo}"
                ;;
            6)
                read -p "Source remote: " source
                read -p "Target remote: " target
                fork_sync "$source" "$target"
                ;;
            7) temp_sync ;;
            8) setup_multi_tracking ;;
            9)
                read -p "New temperature (0-10): " new_temp
                export TEMP=$new_temp
                ;;
            0) break ;;
            *) print_error "Invalid option" ;;
        esac
        
        echo
        read -p "Press Enter to continue..."
    done
}

# Main command handler
case "${1:-menu}" in
    status)
        show_remotes
        echo
        for remote in $(git remote); do
            check_remote_status "$remote"
        done
        ;;
    push-all)
        push_all "${2:-}"
        ;;
    selective)
        selective_push "${2:-.}" "${3:-origin}"
        ;;
    experimental)
        create_experimental "${2:-exp-$(date +%Y%m%d-%H%M%S)}" "${3:-forgejo}"
        ;;
    fork-sync)
        fork_sync "${2:-origin}" "${3:-forgejo}" "${4:-main}"
        ;;
    temp)
        temp_sync
        ;;
    temp-set)
        export TEMP="${2:-5}"
        print_info "Temperature set to: $TEMP"
        ;;
    setup-multi)
        setup_multi_tracking "${2:-}"
        ;;
    menu)
        interactive_menu
        ;;
    help|--help|-h)
        print_header "Smart Tree Git Sync Manager"
        echo "Usage: $0 [command] [options]"
        echo
        echo "Commands:"
        echo "  status              Show all remotes and their status"
        echo "  push-all [branch]   Push branch to all remotes"
        echo "  selective <pattern> <remote>  Push branches matching pattern"
        echo "  experimental <name> [remote]  Create experimental branch"
        echo "  fork-sync <from> <to> [branch]  Sync between remotes"
        echo "  temp                Temperature-based sync"
        echo "  temp-set <0-10>     Set sync temperature"
        echo "  setup-multi [branch]  Setup multi-remote tracking"
        echo "  menu                Interactive menu (default)"
        echo
        echo "Environment:"
        echo "  GIT_TEMP=<0-10>    Sync aggressiveness (default: 5)"
        echo
        echo "Examples:"
        echo "  $0 push-all main"
        echo "  $0 selective 'feature-.*' forgejo"
        echo "  $0 experimental quantum-api forgejo"
        echo "  GIT_TEMP=9 $0 temp"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for usage"
        exit 1
        ;;
esac

print_info "🎸 Rock on with distributed version control!"