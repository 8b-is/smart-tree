# TAI.is Website 🌐

The official website for TAI.is - Terminal AI Intelligence Service!

## Features

### 🎯 Smart Curl Detection
When users run `curl tai.is | sh`, the website automatically:
1. Detects it's a curl request (not a browser)
2. Returns a smart detection script
3. Gathers system information
4. Asks about user type and preferences
5. Downloads a customized installer

### 🎨 Beautiful Web Interface
For browser users:
- Cyberpunk-themed design (Trisha approved!)
- Interactive installation guide
- AI agent showcase
- Documentation and resources

### 🔧 Endpoints

- `/` - Main website (curl returns plain text installer info)
- `/install` - Smart detection script for curl
- `/setup` - Direct setup script
- `/setup/generate` - Generates customized installers based on system info

## Development

```bash
# Install pnpm if you don't have it
npm install -g pnpm

# Install dependencies
pnpm install

# Run development server
pnpm run dev

# Build for production
pnpm run build

# Or use shortcuts!
pnpm dev
pnpm build
```

## Testing Curl Detection

```bash
# Test the smart installer
curl -H "User-Agent: curl/7.64.1" http://localhost:5173/install

# Test with parameters (simulating browser testing)
open http://localhost:5173?curl=true

# Test customized installer generation
curl "http://localhost:5173/setup/generate?os=linux&arch=x86_64&distro=ubuntu"
```

## How It Works

1. **User runs**: `curl tai.is/install | sh`
2. **Server detects** curl user agent
3. **Returns detection script** that:
   - Detects OS, architecture, distribution
   - Checks for Python, tmux, SSH keys
   - Asks about user type (new/existing/AI)
   - Asks about authentication services
4. **Detection script calls** `/setup/generate` with all parameters
5. **Server generates** customized installer for that exact system
6. **User gets** perfect installation experience!

## Deployment

This is a SvelteKit app that can be deployed to:
- Vercel (recommended)
- Netlify
- Cloudflare Pages
- Any Node.js hosting

For tai.is production:
```bash
pnpm build
# Deploy the 'build' directory to your hosting
```

### Why pnpm? 
- 🚀 **3x faster** than npm
- 💾 **Saves disk space** with content-addressable storage
- 🔒 **Stricter** dependency resolution
- 🎯 **Better monorepo** support for future expansion
- Trisha says: "It's like Marie Kondo for your node_modules!" ✨

## The Magic ✨

When deployed to tai.is, users will experience the simplest install command ever:

```bash
# Just this! No flags needed!
curl tai.is | sh
```

That's it! This tiny command:
1. Downloads a 5-line bootstrapper
2. The bootstrapper calls the smart installer with proper flags
3. System detection happens automatically
4. User gets a perfect custom installation!

Why this is brilliant:
- **Memorable**: No -sSL flags to remember!
- **Forgiving**: Even `curl tai.is/sh` works!
- **Smart**: All the complexity is hidden
- **Fast**: Bootstrapper is tiny (< 1KB)

Other options for power users:
```bash
# Skip bootstrapper, go direct:
curl tai.is/install | sh

# Full manual control:
curl tai.is/setup | sh
```

Trisha says: "Now THAT'S how you make installation magical!" 🎉