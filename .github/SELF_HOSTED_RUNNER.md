# Setting Up Self-Hosted macOS Runner 🏠🍎

This guide will help you set up your Mac as a self-hosted GitHub Actions runner for the smart-tree repository.

## Why Self-Hosted?

- 🚀 **Faster builds** - Uses your local hardware
- 💰 **Saves GitHub Actions minutes** - Free for private repos
- 🔧 **Full control** - Your environment, your tools
- ⚡ **Better caching** - Persistent between runs

## Prerequisites

- macOS (ARM64/M1/M2/M3 or Intel)
- GitHub repo admin access
- Homebrew (optional but recommended)
- Rust toolchain installed

## Quick Setup (5 minutes)

### 1. Get Registration Token

```bash
# From your smart-tree directory:
cd /aidata/ayeverse/smart-tree

# Get a registration token (valid for 1 hour):
gh api repos/8b-is/smart-tree/actions/runners/registration-token | jq -r .token
```

Copy the token that's displayed.

### 2. Download GitHub Actions Runner

```bash
# Create a directory for the runner:
mkdir -p ~/actions-runner && cd ~/actions-runner

# Download the latest runner (ARM64 for Apple Silicon):
curl -o actions-runner-osx-arm64-2.311.0.tar.gz -L \
  https://github.com/actions/runner/releases/download/v2.311.0/actions-runner-osx-arm64-2.311.0.tar.gz

# Extract:
tar xzf ./actions-runner-osx-arm64-2.311.0.tar.gz
```

**For Intel Macs**, use:
```bash
curl -o actions-runner-osx-x64-2.311.0.tar.gz -L \
  https://github.com/actions/runner/releases/download/v2.311.0/actions-runner-osx-x64-2.311.0.tar.gz
tar xzf ./actions-runner-osx-x64-2.311.0.tar.gz
```

### 3. Configure the Runner

```bash
# Run the configuration:
./config.sh \
  --url https://github.com/8b-is/smart-tree \
  --token PASTE_YOUR_TOKEN_HERE \
  --labels macOS,ARM64,self-hosted \
  --name "Hue-MacBook-Pro" \
  --work _work

# When prompted:
# - Runner group: Just press Enter (default)
# - Runner name: Enter a descriptive name
# - Runner labels: It will show macOS,ARM64,self-hosted
# - Work folder: Just press Enter (default: _work)
```

### 4. Install as a Service (Recommended)

This will start the runner automatically on boot:

```bash
# Install the service:
sudo ./svc.sh install

# Start the service:
sudo ./svc.sh start

# Check status:
sudo ./svc.sh status
```

**Alternative: Run interactively** (for testing):
```bash
./run.sh
```

## Verify It's Working

### Check Runner Status

```bash
# List runners:
gh api repos/8b-is/smart-tree/actions/runners | jq '.runners[] | {name, status, labels}'
```

You should see your runner listed with `status: "online"`.

### Test the Workflow

```bash
# Trigger the self-hosted workflow manually:
gh workflow run rust-self-hosted.yml

# Watch it run:
gh run watch
```

## Runner Management

### View Runner Status
```bash
gh api repos/8b-is/smart-tree/actions/runners
```

### Stop the Runner
```bash
sudo ~/actions-runner/svc.sh stop
```

### Start the Runner
```bash
sudo ~/actions-runner/svc.sh start
```

### Restart the Runner
```bash
sudo ~/actions-runner/svc.sh restart
```

### Remove the Runner
```bash
# Stop the service:
sudo ~/actions-runner/svc.sh stop
sudo ~/actions-runner/svc.sh uninstall

# Remove from GitHub:
cd ~/actions-runner
./config.sh remove --token $(gh api repos/8b-is/smart-tree/actions/runners/remove-token | jq -r .token)
```

## Workflow Configuration

The self-hosted workflow is at: `.github/workflows/rust-self-hosted.yml`

**Features:**
- ✅ Manual trigger via `workflow_dispatch`
- ✅ System info reporting (CPU, memory, disk)
- ✅ Automatic Rust version management
- ✅ Intelligent caching (registry, index, build artifacts)
- ✅ Full test suite
- ✅ Optional benchmarks
- ✅ Automatic cleanup

**To enable automatic triggers** (push/PR), uncomment lines 6-10 in the workflow file.

## Security Notes

⚠️ **Important**: Self-hosted runners execute code from PRs. Only enable for trusted repositories.

- ✅ **DO**: Use for your own repos
- ✅ **DO**: Review code before merging PRs
- ❌ **DON'T**: Use for public repos with untrusted contributors
- ❌ **DON'T**: Store secrets in the runner environment

## Troubleshooting

### Runner Shows Offline
```bash
# Check if service is running:
sudo ~/actions-runner/svc.sh status

# View logs:
tail -f ~/actions-runner/_diag/*.log
```

### Build Failures
```bash
# Check Rust installation:
rustc --version
cargo --version

# Update Rust:
rustup update stable
```

### Permission Issues
```bash
# Fix ownership:
sudo chown -R $(whoami) ~/actions-runner

# Fix permissions:
chmod +x ~/actions-runner/*.sh
```

## Performance Tips

1. **Enable SSD caching** - The workflow caches cargo artifacts
2. **Increase parallel jobs** - Set `RUST_TEST_THREADS` to your CPU count
3. **Use release builds** - Already configured in workflow
4. **Close other apps** - Free up RAM and CPU

## Monitoring

View real-time logs while a job runs:

```bash
# Watch the current run:
gh run watch

# View specific job logs:
gh run view --log
```

## Next Steps

Once your runner is set up:

1. ✅ Verify it appears in the GitHub repo → Settings → Actions → Runners
2. ✅ Run a manual workflow: `gh workflow run rust-self-hosted.yml`
3. ✅ Check the logs: `gh run watch`
4. ✅ Consider enabling automatic triggers

## Questions?

- Check runner logs: `~/actions-runner/_diag/`
- GitHub Actions docs: https://docs.github.com/en/actions
- smart-tree issues: https://github.com/8b-is/smart-tree/issues

---

**Need help?** Ask Hue or Claude! 🤖
