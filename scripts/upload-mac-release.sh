#!/usr/bin/env bash
# ============================================================================
# Build & Upload Signed macOS (aarch64-apple-darwin) Release Bundle
# Builds on Mac Studio (default: 172.16.47.91) and publishes to i1.is + GitHub
# ============================================================================
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAC_HOST="${MAC_HOST:-172.16.47.91}"
MAC_CACHE_TARGET_DIR="${MAC_CACHE_TARGET_DIR:-/Users/hue/source/8b-is/smart-tree/target}"
I1_HOST="${I1_HOST:-i1.is}"
I1_ROOT="${I1_ROOT:-/srv/i1-releases/smart-tree}"
GITHUB_REPO="${GITHUB_REPO:-8b-is/smart-tree}"
BUILD_ID="${1:-}"

G='\033[0;32m'
C='\033[0;36m'
Y='\033[1;33m'
N='\033[0m'

info() { printf "${C}→${N} %s\n" "$*"; }
ok()   { printf "${G}✓${N} %s\n" "$*"; }
warn() { printf "${Y}!${N} %s\n" "$*"; }

STAGE_DIR="$(mktemp -d)"
trap 'rm -rf "$STAGE_DIR"' EXIT

# 1. Resolve active BUILD_ID from i1.is if not provided
if [[ -z "$BUILD_ID" ]]; then
    info "Resolving current release build_id from ${I1_HOST}:${I1_ROOT}/current ..."
    BUILD_ID="$(ssh -o BatchMode=yes "$I1_HOST" "cat ${I1_ROOT}/current/build-id.txt")"
fi

RELEASE_DIR="${STAGE_DIR}/${BUILD_ID}"
mkdir -p "$RELEASE_DIR"

info "Fetching release snapshot ${BUILD_ID} from ${I1_HOST}..."
scp -q "${I1_HOST}:${I1_ROOT}/${BUILD_ID}/manifest.json" \
       "${I1_HOST}:${I1_ROOT}/${BUILD_ID}/source.tar.gz" \
       "${I1_HOST}:${I1_ROOT}/${BUILD_ID}/source.tar.gz.sha256" \
       "${I1_HOST}:${I1_ROOT}/${BUILD_ID}/build-id.txt" \
       "${I1_HOST}:${I1_ROOT}/${BUILD_ID}/version.txt" \
       "${I1_HOST}:${I1_ROOT}/${BUILD_ID}/BUILD_FEATURES.txt" \
       "$RELEASE_DIR/"

VERSION="$(cat "$RELEASE_DIR/version.txt" | tr -d '[:space:]')"
TAG="v${VERSION}"
info "Target release: ${TAG} (build_id: ${BUILD_ID})"

# 2. Stage source snapshot onto Mac and compile native aarch64-apple-darwin binaries
REMOTE_WORK="/tmp/st-mac-release-${BUILD_ID}"
info "Syncing source snapshot to Mac builder (${MAC_HOST}:${REMOTE_WORK})..."
ssh -o BatchMode=yes "$MAC_HOST" "rm -rf '$REMOTE_WORK' && mkdir -p '$REMOTE_WORK'"
scp -q "$RELEASE_DIR/source.tar.gz" "${MAC_HOST}:${REMOTE_WORK}/source.tar.gz"
# Also sync local repo files to /Users/hue/source/8b-is/smart-tree so Mac workspace stays fresh
rsync -az --exclude 'target/' --exclude '.git/' "$PROJECT_DIR/" "${MAC_HOST}:/Users/hue/source/8b-is/smart-tree/"

info "Building native aarch64-apple-darwin release binaries (st, std, m8, n8x) on ${MAC_HOST}..."
ssh -o BatchMode=yes "$MAC_HOST" "zsh -lic '
    set -euo pipefail
    export PATH=\"/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:\$HOME/.cargo/bin:\$PATH\"
    cd \"$REMOTE_WORK\"
    tar -xzf source.tar.gz
    cd smart-tree
    export ST_BUILD_ID=\"$BUILD_ID\"
    export SMART_TREE_NO_UPDATE_CHECK=1
    export CARGO_TARGET_DIR=\"$MAC_CACHE_TARGET_DIR\"
    # Touch src/main.rs so ST_BUILD_ID is guaranteed to be baked into st
    touch src/main.rs build.rs
    cargo build --locked --release --features full --bin st --bin std --bin m8 --bin n8x
    \"\$CARGO_TARGET_DIR/release/st\" --version
    \"\$CARGO_TARGET_DIR/release/st\" --help >/dev/null
    \"\$CARGO_TARGET_DIR/release/std\" --help >/dev/null
    \"\$CARGO_TARGET_DIR/release/m8\" --help >/dev/null
    \"\$CARGO_TARGET_DIR/release/n8x\" --help >/dev/null
    mkdir -p \"$REMOTE_WORK/out\" \"\$HOME/.local/bin\"
    for bin in st std m8 n8x; do
        cp \"\$CARGO_TARGET_DIR/release/\$bin\" \"$REMOTE_WORK/out/\$bin\"
        install -m 0755 \"\$CARGO_TARGET_DIR/release/\$bin\" \"\$HOME/.local/bin/\$bin\"
        if [[ -d /opt/homebrew/bin && -w /opt/homebrew/bin ]]; then
            install -m 0755 \"\$CARGO_TARGET_DIR/release/\$bin\" \"/opt/homebrew/bin/\$bin\"
        fi
    done
'"

# 3. Pull compiled macOS binaries back and package signed archive
MAC_BIN_DIR="${STAGE_DIR}/mac-bins"
mkdir -p "$MAC_BIN_DIR"
info "Retrieving compiled macOS binaries from ${MAC_HOST}..."
scp -q "${MAC_HOST}:${REMOTE_WORK}/out/st" \
       "${MAC_HOST}:${REMOTE_WORK}/out/std" \
       "${MAC_HOST}:${REMOTE_WORK}/out/m8" \
       "${MAC_HOST}:${REMOTE_WORK}/out/n8x" \
       "$MAC_BIN_DIR/"
chmod 0755 "$MAC_BIN_DIR"/*

info "Packaging st-aarch64-apple-darwin.tar.gz and updating manifest.json..."
python3 "$PROJECT_DIR/scripts/package_i1_release.py" binaries \
    --release "$RELEASE_DIR" \
    --binaries "$MAC_BIN_DIR" \
    --target aarch64-apple-darwin \
    --force

# Create smart-tree-aarch64-apple-darwin.tar.gz alias (matching Linux release layout)
cp "$RELEASE_DIR/st-aarch64-apple-darwin.tar.gz" "$RELEASE_DIR/smart-tree-aarch64-apple-darwin.tar.gz"
(cd "$RELEASE_DIR" && sha256sum "smart-tree-aarch64-apple-darwin.tar.gz" > "smart-tree-aarch64-apple-darwin.tar.gz.sha256")

# 4. Upload to i1.is release server
info "Uploading st-aarch64-apple-darwin.tar.gz and updated manifest.json to ${I1_HOST}:${I1_ROOT}/${BUILD_ID}/ ..."
scp -q "$RELEASE_DIR/st-aarch64-apple-darwin.tar.gz" \
       "$RELEASE_DIR/st-aarch64-apple-darwin.tar.gz.sha256" \
       "$RELEASE_DIR/smart-tree-aarch64-apple-darwin.tar.gz" \
       "$RELEASE_DIR/smart-tree-aarch64-apple-darwin.tar.gz.sha256" \
       "$RELEASE_DIR/manifest.json" \
       "${I1_HOST}:${I1_ROOT}/${BUILD_ID}/"

ssh -o BatchMode=yes "$I1_HOST" "chmod 0644 ${I1_ROOT}/${BUILD_ID}/st-aarch64-apple-darwin.tar.gz* ${I1_ROOT}/${BUILD_ID}/smart-tree-aarch64-apple-darwin.tar.gz* ${I1_ROOT}/${BUILD_ID}/manifest.json"
ok "Published st-aarch64-apple-darwin.tar.gz to https://i1.is/releases/smart-tree/${BUILD_ID}/"

# 5. Upload to GitHub Releases
if command -v gh >/dev/null 2>&1; then
    if gh release view "$TAG" -R "$GITHUB_REPO" >/dev/null 2>&1; then
        info "Uploading macOS assets to GitHub Release ${TAG} (${GITHUB_REPO})..."
        gh release upload "$TAG" \
            "$RELEASE_DIR/st-aarch64-apple-darwin.tar.gz" \
            "$RELEASE_DIR/st-aarch64-apple-darwin.tar.gz.sha256" \
            "$RELEASE_DIR/smart-tree-aarch64-apple-darwin.tar.gz" \
            "$RELEASE_DIR/smart-tree-aarch64-apple-darwin.tar.gz.sha256" \
            "$RELEASE_DIR/manifest.json" \
            -R "$GITHUB_REPO" \
            --clobber
        ok "Uploaded st-aarch64-apple-darwin.tar.gz to GitHub Release ${TAG}"
    else
        warn "GitHub release ${TAG} not found; skipped GitHub upload."
    fi
fi

# Clean up temporary remote build folder
ssh -o BatchMode=yes "$MAC_HOST" "rm -rf '$REMOTE_WORK'"
ok "macOS release build & upload complete for ${BUILD_ID}!"
