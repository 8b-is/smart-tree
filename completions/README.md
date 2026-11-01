# Smart Tree Shell Completions

This directory contains enhanced shell completion scripts for Smart Tree (`st`).

## Features

### Enhanced Zsh Completions (`_st_enhanced`)
- **Smart Tab Completion**: Complete all flags, options, and modes
- **Contextual Tips**: Shows helpful tips based on what you're typing
- **Mode Descriptions**: Each output mode shows its description
- **Auto-suggestions**: Integration with zsh-autosuggestions
- **Common Patterns**: Pre-defined aliases for frequent operations
- **Interactive Help**: `sttips` command shows tips and tricks

## Installation

### Quick Install (Zsh)
```bash
# Using manage.sh
./scripts/manage.sh completions

# Or run the setup script directly
zsh completions/setup_zsh.sh
```

### Manual Install
```bash
# Copy completion file to your completions directory
cp completions/_st_enhanced ~/.zsh/completions/_st

# Add to ~/.zshrc
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit && compinit
source ~/.config/st/config.zsh
```

## Usage Examples

### Tab Completion
```bash
st <TAB>                    # Shows path completions and options
st --mode <TAB>             # Shows all 18 output modes with descriptions
st . --find <TAB>           # Shows tip about using with --mode ls
st . --newer-than <TAB>     # Suggests date format
```

### Helpful Aliases (after installation)
```bash
stai                        # st . --mode summary-ai -z
stfind "pattern"            # st . --find "pattern"
stsearch "keyword"          # st . --search "keyword"
stwaste                     # st . --mode waste
stls                        # st . --mode ls
stquick                     # Quick 3-level AI summary
strecent                    # Files changed in last 7 days
sttips                      # Show tips and tricks
```

### Tips Examples
When you type certain combinations, you'll see contextual tips:
- After `--find`: "💡 TIP: Use --mode ls with --find to see full match context"
- After `--stream`: "💡 TIP: Streaming mode essential for dirs with >100k files"
- After `--mode`: "💡 TIP: Use 'summary-ai' for 10x compression when working with LLMs"

## Configuration

The setup creates `~/.config/st/config.zsh` with:
- Useful aliases
- Auto-suggestion patterns
- Tips function

## Auto-suggestions

If you have `zsh-autosuggestions` installed:
```bash
# Install on macOS
brew install zsh-autosuggestions

# Install on Ubuntu/Debian
sudo apt install zsh-autosuggestions

# Install on Arch
sudo pacman -S zsh-autosuggestions
```

The completion script will automatically seed your history with common Smart Tree patterns for better suggestions.

## Customization

Edit `~/.config/st/config.zsh` to:
- Add your own aliases
- Modify auto-suggestion patterns
- Customize tips

## Troubleshooting

### Completions not working
1. Ensure the completion file is in a directory in your `$fpath`:
   ```bash
   echo $fpath
   ```

2. Rebuild completion cache:
   ```bash
   rm -f ~/.zcompdump
   compinit
   ```

3. Check if completions are loaded:
   ```bash
   which _st
   ```

### Tips not showing
- Tips require a terminal that supports ANSI escape codes
- Check if `_message` function is available in your zsh

## Contributing

To add new completions or tips:
1. Edit `completions/_st_enhanced`
2. Add new patterns to the `args` array
3. Add contextual tips in the state machine
4. Test with `./scripts/manage.sh completions`