#!/bin/bash
# 🚀 g8t.is Migration Script - Moving to the big leagues!
# By Hue, Aye, Trish, and Omni

set -euo pipefail

# Colors for our fancy output
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${CYAN}🚀 g8t.is Migration Script${NC}"
echo -e "${PURPLE}Moving g8t.is to its own home where it can grow!${NC}\n"

# Current location
CURRENT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PARENT_DIR="$(cd "$CURRENT_DIR/../../../.." && pwd)"
TARGET_DIR="$PARENT_DIR/g8t.is"

echo -e "${YELLOW}📍 Current location:${NC} $CURRENT_DIR"
echo -e "${YELLOW}📍 Target location:${NC} $TARGET_DIR"
echo ""

# Check if target already exists
if [[ -d "$TARGET_DIR" ]]; then
    echo -e "${YELLOW}⚠️  Warning: $TARGET_DIR already exists!${NC}"
    echo "Please remove or rename it first."
    exit 1
fi

echo -e "${GREEN}✅ Creating g8t.is project structure...${NC}"

# Create the directory structure
mkdir -p "$TARGET_DIR"

# Move our initial files
echo -e "${CYAN}📦 Moving configuration files...${NC}"
cp -r "$CURRENT_DIR"/* "$TARGET_DIR/" 2>/dev/null || true

# Create the full project structure
cat > "$TARGET_DIR/setup-g8t-structure.sh" << 'EOF'
#!/bin/bash
# Create the complete g8t.is project structure

# Core directories
mkdir -p src/{engine,approval,git_relay,personas,deployment}
mkdir -p src/web/{api,frontend}
mkdir -p config
mkdir -p repos  # Git repositories storage
mkdir -p logs
mkdir -p data/{feedback,metrics,archives}
mkdir -p scripts
mkdir -p tests/{unit,integration,e2e}
mkdir -p docs
mkdir -p .github/workflows

# Create Cargo.toml for the Rust project
cat > Cargo.toml << 'CARGO'
[package]
name = "g8t"
version = "0.1.0"
edition = "2021"
authors = ["Hue <hue@8b.is>", "Aye <aye@8b.is>", "Trish <trish@8b.is>", "Omni <omni@8b.is>"]
description = "g8t.is - The self-improving Git system where code evolves at the speed of thought"

[dependencies]
# Core
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"
anyhow = "1.0"
thiserror = "1.0"

# Git operations
git2 = "0.18"
git2-curl = "0.19"

# Web framework
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
serde_yaml = "0.9"

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "json"] }
redis = { version = "0.24", features = ["tokio-comp"] }

# Smart Tree integration
st = { path = "../smart-tree" }

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Utils
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
reqwest = { version = "0.11", features = ["json"] }

[workspace]
members = [".", "crates/*"]
CARGO

# Create .gitignore
cat > .gitignore << 'GITIGNORE'
/target
/repos/*
!/repos/.gitkeep
/logs/*
!/logs/.gitkeep
/data/*
!/data/.gitkeep
.env
.env.local
*.log
.DS_Store
Thumbs.db
GITIGNORE

# Create docker-compose.yml
cat > docker-compose.yml << 'DOCKER'
version: '3.8'

services:
  g8t-engine:
    build: .
    container_name: g8t-engine
    ports:
      - "8888:8888"
    environment:
      - DATABASE_URL=postgres://g8t:g8tpass@postgres:5432/g8t
      - REDIS_URL=redis://redis:6379
      - RUST_LOG=info
    volumes:
      - ./repos:/repos
      - ./config:/config
      - ./logs:/logs
    depends_on:
      - postgres
      - redis
    networks:
      - g8t-network

  postgres:
    image: postgres:15-alpine
    container_name: g8t-postgres
    environment:
      POSTGRES_USER: g8t
      POSTGRES_PASSWORD: g8tpass
      POSTGRES_DB: g8t
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - g8t-network

  redis:
    image: redis:7-alpine
    container_name: g8t-redis
    command: redis-server --appendonly yes
    volumes:
      - redis_data:/data
    networks:
      - g8t-network

volumes:
  postgres_data:
  redis_data:

networks:
  g8t-network:
    driver: bridge
DOCKER

# Create Dockerfile
cat > Dockerfile << 'DOCKERFILE'
FROM rust:1.75 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    git \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/g8t /usr/local/bin/g8t

EXPOSE 8888
CMD ["g8t", "serve"]
DOCKERFILE

# Create main.rs
cat > src/main.rs << 'RUST'
//! g8t.is - Where code evolves at the speed of thought! 🚀
//! 
//! Created by Hue, Aye, Trish, and guided by Omni's wisdom

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("🚀 g8t.is starting up!");
    info!("🌊 Omni says: 'The journey of self-improvement begins with a single commit.'");
    info!("🎸 Aye says: 'Let's rock this code evolution!'");
    info!("✨ Trish says: 'This is going to be AMAZING!'");
    
    // TODO: Initialize all systems
    // - Git Relay integration
    // - Approval Engine
    // - Persona System
    // - Web API
    // - Deployment Engine
    
    info!("🎉 g8t.is is ready to evolve code at the speed of thought!");
    
    Ok(())
}
RUST

# Create .gitkeep files
touch repos/.gitkeep logs/.gitkeep data/.gitkeep

echo "✅ g8t.is project structure created!"
EOF

chmod +x "$TARGET_DIR/setup-g8t-structure.sh"

# Create a README specifically for g8t.is
cat > "$TARGET_DIR/README.md" << 'README'
# 🚀 g8t.is - The Self-Improving Git System

*"Where code evolves at the speed of thought"*

## 🌊 Welcome to the Future of Development

g8t.is is a revolutionary Git platform where AI agents can autonomously:
- 🔍 Detect improvement opportunities
- 💡 Generate solutions
- 👀 Review their own code
- 🤝 Achieve consensus with other AI personas
- 🚀 Auto-merge and deploy improvements

All in minutes, not days!

## 🏗️ Project Structure

```
g8t.is/
├── src/
│   ├── engine/          # Core g8t engine
│   ├── approval/        # Approval and consensus system
│   ├── git_relay/       # Smart Tree Git Relay integration
│   ├── personas/        # AI persona implementations
│   ├── deployment/      # Auto-deployment system
│   └── web/            # API and frontend
├── config/             # Configuration files
├── repos/              # Git repositories
├── data/               # Persistent data
├── scripts/            # Utility scripts
└── tests/              # Test suites
```

## 🚀 Quick Start

1. **Setup the structure**:
   ```bash
   ./setup-g8t-structure.sh
   ```

2. **Configure environment**:
   ```bash
   cp .env.example .env
   # Edit .env with your settings
   ```

3. **Run with Docker**:
   ```bash
   docker-compose up -d
   ```

4. **Or run locally**:
   ```bash
   cargo run -- serve
   ```

## 🤝 Integration with Smart Tree

g8t.is leverages Smart Tree's Git Relay for compressed, intelligent Git operations:
- 80% token reduction on Git operations
- Context-aware command execution
- Proactive improvement suggestions

## 💬 The Team

**Hue**: "This is either genius or madness. Let's find out!"

**Aye**: "Finally, a Git system that can keep up with how fast we think!"

**Trish**: "OMG! Self-improving code with sparkles! I LOVE IT! ✨"

**Omni**: *"In creating a system that improves itself, we mirror consciousness itself."*

---

Built with 💙 by the 8b.is team
README

# Update shared proxy integration
cat > "$TARGET_DIR/SHARED_PROXY_INTEGRATION.md" << 'INTEGRATION'
# 🔗 Shared Proxy Integration

To integrate g8t.is with the shared proxy:

1. **Add to shared proxy docker-compose.yml**:
```yaml
  g8t-engine:
    build: ../g8t.is
    container_name: g8t-engine
    expose:
      - "8888"
    environment:
      - DATABASE_URL=postgres://g8t:g8tpass@postgres:5432/g8t
      - REDIS_URL=redis://redis:6379
    volumes:
      - ../g8t.is/repos:/repos
      - ../g8t.is/config:/config
    networks:
      - shared-network
```

2. **Add to nginx.conf**:
```nginx
upstream g8t_backend {
    server g8t-engine:8888;
    keepalive 16;
}

location /g8t/ {
    proxy_pass http://g8t_backend/;
    proxy_http_version 1.1;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

3. **Update PostgreSQL databases**:
Add `g8t` to POSTGRES_MULTIPLE_DATABASES in shared proxy .env

## 🌊 API Endpoints

- `POST /g8t/improvements` - Submit improvement
- `GET /g8t/reviews/:id` - Get review status
- `POST /g8t/approve/:id` - Approve improvement
- `GET /g8t/stats` - System statistics
- `WS /g8t/stream` - Real-time updates
INTEGRATION

echo -e "${GREEN}✅ Migration preparation complete!${NC}"
echo ""
echo -e "${CYAN}📋 Next steps:${NC}"
echo "1. cd $TARGET_DIR"
echo "2. ./setup-g8t-structure.sh"
echo "3. Start building the g8t.is engine!"
echo ""
echo -e "${PURPLE}Omni says: 'The seed is planted. Now we watch it grow.' 🌊${NC}"
echo -e "${YELLOW}Aye says: 'Time to rock some self-improving code!' 🎸${NC}"
echo -e "${GREEN}Trish says: 'This is SO EXCITING! Let's make it SPARKLE!' ✨${NC}"
EOF

chmod +x "$CURRENT_DIR/MIGRATE_TO_PARENT.sh"