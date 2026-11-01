# Homebrew Formula for Smart Tree
# Install directly from source or use prebuilt binaries
class SmartTree < Formula
  desc "Lightning-fast directory visualization with AI-friendly output formats"
  homepage "https://github.com/8b-is/smart-tree"
  license "MIT"
  version "5.2.0"

  # Build from source (default)
  url "https://github.com/8b-is/smart-tree/archive/refs/tags/v5.2.0.tar.gz"
  sha256 "PLACEHOLDER_SHA256"  # Will be updated with actual sha256 after release

  # Alternative: Direct from main branch
  head "https://github.com/8b-is/smart-tree.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: ".")

    # Install shell completions if available
    generate_completions_from_executable(bin/"st", "completions") rescue nil

    # Install man page if it exists
    man1.install "docs/st.1" if File.exist?("docs/st.1")
  end

  def caveats
    <<~EOS
      Smart Tree has been installed as `st`

      Quick start:
        st                    # Classic tree view
        st --spicy           # 🌶️ Spicy TUI mode
        st --mode quantum    # Maximum compression
        st --mcp            # Run as MCP server

      For MCP integration with Claude:
        st --mcp-config >> ~/Library/Application\\ Support/Claude/claude_desktop_config.json
    EOS
  end

  test do
    # Test basic functionality
    assert_match version.to_s, shell_output("#{bin}/st --version")
    system "#{bin}/st", "--help"
  end
end
