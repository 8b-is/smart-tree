#!/bin/bash
# Quick setup script for multi-remote configuration

echo "🎸 Setting up Smart Tree multi-remote configuration..."

# Check current remotes
echo "Current remotes:"
git remote -v

# Add Forgejo if not exists
if ! git remote | grep -q "forgejo"; then
    echo "Adding Forgejo remote..."
    git remote add forgejo git@g.8b.is:8b-is/smart-tree.git
fi

# Add GitLab if not exists
if ! git remote | grep -q "gitlab"; then
    echo "Adding GitLab remote..."
    git remote add gitlab git@gitlab.com:8b-is/smart-tree.git
fi

# Set up fetch all
git config remote.origin.fetch "+refs/heads/*:refs/remotes/origin/*"
git config remote.forgejo.fetch "+refs/heads/*:refs/remotes/forgejo/*"
git config remote.gitlab.fetch "+refs/heads/*:refs/remotes/gitlab/*"

# Enable fetching from all remotes
git config fetch.parallel 3

# Set up push default
git config push.default current

# Create testing remotes (optional)
read -p "Set up testing remotes? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git remote add ci-test git@ci.8b.is:testing/smart-tree.git 2>/dev/null || true
    git remote add quantum-test git@quantum.8b.is:test/smart-tree.git 2>/dev/null || true
fi

echo "✅ Multi-remote setup complete!"
echo
echo "Quick commands:"
echo "  ./scripts/git-sync.sh status     - Check all remotes"
echo "  ./scripts/git-sync.sh push-all   - Push to all remotes"
echo "  ./scripts/git-sync.sh menu       - Interactive menu"
echo
echo "Set temperature with: export GIT_TEMP=7  (0-10 scale)"
echo "Current temperature: ${GIT_TEMP:-5}"