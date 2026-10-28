# Smart Tree Multi-Remote Git Workflow 🎸

This document describes the multi-remote git setup for Smart Tree, enabling distributed development across GitHub, Forgejo, and GitLab.

## Current Remotes

- **origin** (GitHub): `git@github.com:8b-is/smart-tree.git` - Primary public repository
- **forgejo** (8b.is): `git@g.8b.is:8b-is/smart-tree.git` - Experimental features & private development
- **gitlab**: `git@gitlab.com:8b-is/smart-tree.git` - Mirror for CI/CD and backup

## Temperature-Based Sync Control 🌡️

The `GIT_TEMP` environment variable (0-10) controls sync aggressiveness:

- **0-2 (Cold)**: Conservative - GitHub only, manual approval
- **3-4 (Cool)**: Careful - GitHub primary, Forgejo experimental
- **5-6 (Moderate)**: Balanced - All remotes, stable branches only
- **7-8 (Warm)**: Active - All remotes, all branches
- **9-10 (Hot)**: Aggressive - Force sync everywhere

```bash
# Set temperature
export GIT_TEMP=7

# Check current temperature
echo $GIT_TEMP
```

## Quick Commands

### Status & Info
```bash
# Check all remotes
./scripts/git-sync.sh status

# Show current configuration
git remote -v
```

### Basic Operations
```bash
# Push current branch to all remotes
./scripts/git-sync.sh push-all

# Push specific branch to all
./scripts/git-sync.sh push-all feature/quantum-api

# Temperature-based automatic sync
./scripts/git-sync.sh temp
```

### Selective Pushing
```bash
# Push branches matching pattern to specific remote
./scripts/git-sync.sh selective 'experimental-.*' forgejo
./scripts/git-sync.sh selective 'release-.*' gitlab

# Create experimental branch (only on Forgejo)
./scripts/git-sync.sh experimental quantum-feature forgejo
```

### Fork Synchronization
```bash
# Sync from one remote to another
./scripts/git-sync.sh fork-sync origin forgejo main
./scripts/git-sync.sh fork-sync forgejo gitlab experimental/quantum
```

### Interactive Menu
```bash
# Launch interactive menu
./scripts/git-sync.sh menu
# or just
./scripts/git-sync.sh
```

## Branch Routing Rules

Based on `.git-remotes.yaml` configuration:

- **experimental/*** → Forgejo only (exclusive)
- **hue/*** → Forgejo only (personal branches)
- **quantum/*** → Forgejo & GitLab
- **security/*** → All remotes (high priority)
- **main, release/*** → All remotes

## Pre-Push Hooks

The `.git-hooks/pre-push` script enforces:

1. **Temperature checks**: 
   - Forgejo requires temp ≥ 3
   - GitLab requires temp ≥ 5

2. **Branch restrictions**:
   - Personal branches (hue/*) only go to Forgejo
   - Experimental branches prompt before GitHub

3. **Testing remotes**: Run tests before push

## Common Workflows

### Starting Experimental Work
```bash
# Create experimental branch on Forgejo
./scripts/git-sync.sh experimental my-experiment

# Work on it...
git add .
git commit -m "Experimental changes"
git push  # Goes only to Forgejo

# When ready for wider testing
GIT_TEMP=7 ./scripts/git-sync.sh push-all
```

### Syncing Stable Releases
```bash
# Ensure all remotes have latest stable
git checkout main
./scripts/git-sync.sh push-all main

# Tag release on all remotes
git tag v3.2.0
GIT_TEMP=8 ./scripts/git-sync.sh temp  # Pushes tags
```

### Private Development
```bash
# Create personal branch
git checkout -b hue/my-feature
git push -u forgejo hue/my-feature  # Only Forgejo accepts hue/* branches
```

### Testing Before Public Push
```bash
# Use testing remotes
git remote add ci-test git@ci.8b.is:testing/smart-tree.git
git push ci-test feature/new-thing  # Tests run automatically
```

## Tips & Best Practices

1. **Start Low, Go High**: Begin with low temperature (3-5) for daily work
2. **Experimental First**: Test new features on Forgejo before GitHub
3. **Use Branch Patterns**: Name branches to match routing rules
4. **Check Before Push**: Run `./scripts/git-sync.sh status` regularly
5. **Document Experiments**: Update .git-remotes.yaml for new patterns

## Troubleshooting

### Remote Not Accessible
```bash
# Check SSH keys
ssh -T git@g.8b.is
ssh -T git@gitlab.com
ssh -T git@github.com

# Verify remote URL
git remote get-url forgejo
```

### Temperature Too Low
```bash
# Temporary increase
GIT_TEMP=5 git push forgejo

# Permanent for session
export GIT_TEMP=7
```

### Push Rejected
```bash
# Check pre-push hook logs
cat .git/push-log/history.log

# Bypass hooks (use carefully!)
git push --no-verify
```

## Advanced Configuration

### Add New Remote
```bash
# Add custom remote
git remote add quantum git@quantum.8b.is:labs/smart-tree.git

# Update .git-remotes.yaml
# Add to REMOTES array in git-sync.sh
```

### Custom Temperature Profile
Edit `.git-remotes.yaml` to add new temperature profiles or modify thresholds.

### Webhook Integration
Configure webhooks in `.git-remotes.yaml` for automated notifications.

## Philosophy

"In the franchise wars, all git hosts are Smart Tree repos!" - The multi-remote setup ensures no single point of failure and enables experimental development without affecting stable users.

Rock on with distributed version control! 🎸