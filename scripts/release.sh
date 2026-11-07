#!/usr/bin/env bash
# ============================================================================
# Smart Tree Unified Release Pipeline
# Builds Linux x86_64 (glibc 2.35 Docker) + macOS ARM64 (ssh s), packages
# immutable i1 bundles, uploads to GitHub Releases, and promotes i1.is.
# ============================================================================
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MAC_HOST="${MAC_HOST:-s}"
MAC_CACHE_TARGET_DIR="${MAC_CACHE_TARGET_DIR:-/Users/hue/source/8b-is/smart-tree/target}"
LINUX_CACHE_TARGET_DIR="${LINUX_CACHE_TARGET_DIR:-/tmp/st-docker-target}"
I1_NODES=(ns3 ns1 ns2 ns4 8s.is)
I1_ROOT="/srv/i1-releases/smart-tree"
GITHUB_REPO="8b-is/smart-tree"

G='\033[0;32m'
C='\033[0;36m'
N='\033[0m'
info() { printf "${C}→${N} %s\n" "$*"; }
ok()   { printf "${G}✓${N} %s\n" "$*"; }

cd "$PROJECT_DIR"
unset GH_TOKEN GITHUB_TOKEN

STAGE_ROOT="$(mktemp -d /tmp/st-i1-release.XXXXXX)"
trap 'rm -rf "$STAGE_ROOT"' EXIT

# 1. Create source snapshot
info "Creating immutable source snapshot..."
RELEASE_DIR="$(python3 scripts/package_i1_release.py snapshot --output "$STAGE_ROOT")"
BUILD_ID="$(cat "$RELEASE_DIR/build-id.txt" | tr -d '[:space:]')"
VERSION="$(cat "$RELEASE_DIR/version.txt" | tr -d '[:space:]')"
TAG="v${VERSION}"
ok "Snapshot ready: ${TAG} (${BUILD_ID})"

# 2. Ensure smart-tree-cli-build Docker image exists
if ! docker image inspect smart-tree-cli-build >/dev/null 2>&1; then
    info "Building smart-tree-cli-build container (Ubuntu 22.04 glibc 2.35)..."
    docker build -t smart-tree-cli-build - < deploy/hub/Dockerfile.cli
fi

# 3. Build Linux x86_64 and macOS ARM64 (ssh s) in parallel
info "Starting parallel builds: Linux x86_64 (Docker) + macOS aarch64 (${MAC_HOST})..."
mkdir -p "$LINUX_CACHE_TARGET_DIR"
find "$RELEASE_DIR/source/smart-tree" -exec touch {} +

(
    docker run --rm \
        -u "$(id -u):$(id -g)" \
        -e ST_BUILD_ID="$BUILD_ID" \
        -e SMART_TREE_NO_UPDATE_CHECK=1 \
        -e CARGO_TARGET_DIR="/target" \
        -e CARGO_HOME="/cargo" \
        -v "$RELEASE_DIR/source/smart-tree:/build/source" \
        -v "$LINUX_CACHE_TARGET_DIR:/target" \
        -v "$HOME/.cargo/registry:/cargo/registry" \
        -v "$HOME/.cargo/git:/cargo/git" \
        smart-tree-cli-build \
        cargo build --locked --release --features full --bin st --bin std --bin m8 --bin n8x
) &
LINUX_PID=$!

REMOTE_WORK="/tmp/st-mac-release-${BUILD_ID}"
(
    ssh -o BatchMode=yes "$MAC_HOST" "rm -rf '$REMOTE_WORK' && mkdir -p '$REMOTE_WORK'"
    scp -q "$RELEASE_DIR/source.tar.gz" "${MAC_HOST}:${REMOTE_WORK}/source.tar.gz"
    ssh -o BatchMode=yes "$MAC_HOST" "zsh -lic '
        set -euo pipefail
        export PATH=\"/opt/homebrew/opt/rustup/bin:/opt/homebrew/bin:\$HOME/.cargo/bin:\$PATH\"
        cd \"$REMOTE_WORK\"
        tar -xzf source.tar.gz
        cd smart-tree
        export ST_BUILD_ID=\"$BUILD_ID\"
        export SMART_TREE_NO_UPDATE_CHECK=1
        export CARGO_TARGET_DIR=\"$MAC_CACHE_TARGET_DIR\"
        touch src/main.rs build.rs
        cargo build --locked --release --features full --bin st --bin std --bin m8 --bin n8x
        mkdir -p \"$REMOTE_WORK/out\" \"\$HOME/.local/bin\"
        for bin in st std m8 n8x; do
            cp \"\$CARGO_TARGET_DIR/release/\$bin\" \"$REMOTE_WORK/out/\$bin\"
            install -m 0755 \"\$CARGO_TARGET_DIR/release/\$bin\" \"\$HOME/.local/bin/\$bin\"
        done
    '"
) &
MAC_PID=$!

wait "$LINUX_PID"
ok "Linux x86_64 build complete"
wait "$MAC_PID"
ok "macOS aarch64 build complete on ${MAC_HOST}"

# 4. Package Linux and macOS archives into RELEASE_DIR
python3 scripts/package_i1_release.py binaries \
    --release "$RELEASE_DIR" \
    --binaries "$LINUX_CACHE_TARGET_DIR/release" \
    --target x86_64-unknown-linux-gnu \
    --force
cp "$RELEASE_DIR/st-x86_64-unknown-linux-gnu.tar.gz" "$RELEASE_DIR/smart-tree-x86_64-unknown-linux-gnu.tar.gz"
(cd "$RELEASE_DIR" && sha256sum "smart-tree-x86_64-unknown-linux-gnu.tar.gz" > "smart-tree-x86_64-unknown-linux-gnu.tar.gz.sha256")

MAC_BIN_DIR="${STAGE_ROOT}/mac-bins"
mkdir -p "$MAC_BIN_DIR"
scp -q "${MAC_HOST}:${REMOTE_WORK}/out/st" \
       "${MAC_HOST}:${REMOTE_WORK}/out/std" \
       "${MAC_HOST}:${REMOTE_WORK}/out/m8" \
       "${MAC_HOST}:${REMOTE_WORK}/out/n8x" \
       "$MAC_BIN_DIR/"
chmod 0755 "$MAC_BIN_DIR"/*
ssh -o BatchMode=yes "$MAC_HOST" "rm -rf '$REMOTE_WORK'"

python3 scripts/package_i1_release.py binaries \
    --release "$RELEASE_DIR" \
    --binaries "$MAC_BIN_DIR" \
    --target aarch64-apple-darwin \
    --force
cp "$RELEASE_DIR/st-aarch64-apple-darwin.tar.gz" "$RELEASE_DIR/smart-tree-aarch64-apple-darwin.tar.gz"
(cd "$RELEASE_DIR" && sha256sum "smart-tree-aarch64-apple-darwin.tar.gz" > "smart-tree-aarch64-apple-darwin.tar.gz.sha256")
ok "Packaged Linux & macOS bundles into ${RELEASE_DIR}"

# 5. Publish to GitHub Release & i1.is nodes
if gh release view "$TAG" -R "$GITHUB_REPO" >/dev/null 2>&1; then
    gh release upload "$TAG" \
        "$RELEASE_DIR/manifest.json" \
        "$RELEASE_DIR/source.tar.gz" \
        "$RELEASE_DIR/source.tar.gz.sha256" \
        "$RELEASE_DIR/st-x86_64-unknown-linux-gnu.tar.gz" \
        "$RELEASE_DIR/st-x86_64-unknown-linux-gnu.tar.gz.sha256" \
        "$RELEASE_DIR/smart-tree-x86_64-unknown-linux-gnu.tar.gz" \
        "$RELEASE_DIR/smart-tree-x86_64-unknown-linux-gnu.tar.gz.sha256" \
        "$RELEASE_DIR/st-aarch64-apple-darwin.tar.gz" \
        "$RELEASE_DIR/st-aarch64-apple-darwin.tar.gz.sha256" \
        "$RELEASE_DIR/smart-tree-aarch64-apple-darwin.tar.gz" \
        "$RELEASE_DIR/smart-tree-aarch64-apple-darwin.tar.gz.sha256" \
        -R "$GITHUB_REPO" --clobber
else
    gh release create "$TAG" \
        "$RELEASE_DIR/manifest.json" \
        "$RELEASE_DIR/source.tar.gz" \
        "$RELEASE_DIR/source.tar.gz.sha256" \
        "$RELEASE_DIR/st-x86_64-unknown-linux-gnu.tar.gz" \
        "$RELEASE_DIR/st-x86_64-unknown-linux-gnu.tar.gz.sha256" \
        "$RELEASE_DIR/smart-tree-x86_64-unknown-linux-gnu.tar.gz" \
        "$RELEASE_DIR/smart-tree-x86_64-unknown-linux-gnu.tar.gz.sha256" \
        "$RELEASE_DIR/st-aarch64-apple-darwin.tar.gz" \
        "$RELEASE_DIR/st-aarch64-apple-darwin.tar.gz.sha256" \
        "$RELEASE_DIR/smart-tree-aarch64-apple-darwin.tar.gz" \
        "$RELEASE_DIR/smart-tree-aarch64-apple-darwin.tar.gz.sha256" \
        -R "$GITHUB_REPO" \
        --title "Smart Tree ${TAG}" \
        --notes "Smart Tree ${TAG} (${BUILD_ID})"
fi
ok "Published ${TAG} to GitHub Releases"

for host in "${I1_NODES[@]}"; do
    info "Promoting ${BUILD_ID} on ${host}..."
    ssh -o BatchMode=yes "$host" "rm -rf /tmp/st-pub-${BUILD_ID} && mkdir -p /tmp/st-pub-${BUILD_ID}"
    rsync -az --exclude 'source/' "$RELEASE_DIR/" "${host}:/tmp/st-pub-${BUILD_ID}/"
    scp -q "$PROJECT_DIR/scripts/publish_i1_release.py" "${host}:/tmp/publish_i1_release.py"
    ssh -o BatchMode=yes "$host" "
        python3 /tmp/publish_i1_release.py --release /tmp/st-pub-${BUILD_ID} --root ${I1_ROOT}
        cp -f /tmp/st-pub-${BUILD_ID}/smart-tree-*.tar.gz* ${I1_ROOT}/${BUILD_ID}/
        chmod 0644 ${I1_ROOT}/${BUILD_ID}/smart-tree-*.tar.gz*
        rm -rf /tmp/st-pub-${BUILD_ID}
    "
done
ok "Promoted ${TAG} (${BUILD_ID}) across all i1.is nodes!"
