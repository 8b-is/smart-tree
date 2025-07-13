# Smart Tree Feedback Worker 🌳

Mean ass efficient feedback processor that turns AI suggestions into GitHub issues!

## Overview

This worker:
- Polls f.8t.is for pending feedback
- Categorizes feedback (bug, feature, teleportation goal)
- Detects duplicates using fingerprinting
- Creates GitHub issues automatically
- Tracks everything with Prometheus metrics

## Quick Start

### Local Development

```bash
# Set GitHub token
export GITHUB_TOKEN=ghp_your_token_here

# Run locally
./scripts/manage.sh feedback-run

# Check logs
docker-compose logs -f feedback-worker

# View metrics
open http://localhost:9090/metrics
```

### Deploy to Hetzner

```bash
# Set tokens
export GITHUB_TOKEN=ghp_your_token_here
export HETZNER_TOKEN=your_hetzner_token

# Deploy
./scripts/manage.sh feedback-deploy hetzner

# The script will output the server IP
```

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   f.8t.is   │────▶│    Worker    │────▶│   GitHub    │
│ Feedback API│     │   (Python)   │     │   Issues    │
└─────────────┘     └──────────────┘     └─────────────┘
                           │
                           ▼
                    ┌─────────────┐
                    │    Redis     │
                    │ (Duplicate   │
                    │  Detection)  │
                    └─────────────┘
```

## Categorization Logic

- **Bug**: error, crash, fail, broken, doesn't work, exception, panic
- **Feature**: add, enhance, improve, support, suggestion, idea
- **Teleportation**: quantum, ai, machine learning, neural, consciousness

## Duplicate Detection

1. Generates fingerprint from title + description
2. Checks Redis cache for existing issues
3. Searches GitHub issues for similar titles
4. Prevents duplicate work automatically

## Metrics

Available at `/metrics` endpoint:

- `feedback_items_processed_total`
- `feedback_errors_total`
- `github_issues_created_total`
- `feedback_duplicates_detected_total`
- `feedback_processing_seconds`

## Configuration

Environment variables:

- `GITHUB_TOKEN` - Required for creating issues
- `FEEDBACK_API_URL` - Default: https://f.8t.is/api
- `REDIS_URL` - Default: redis://localhost:6379
- `GITHUB_REPO` - Default: 8b-is/smart-tree
- `PROMETHEUS_PORT` - Default: 9090

## Cloud-Init Deployment

The `cloud-init/hetzner-feedback-worker.yaml` file provides:
- Automatic Smart Tree installation
- Docker and Redis setup
- Performance optimizations
- Health monitoring
- Auto-updates via cron

## Monitoring

Grafana available at port 3000 (admin/admin) for visualizing:
- Processing rate
- Error rate
- Duplicate detection rate
- GitHub API usage

## Why It's Mean Ass Efficient

- Uses uvloop for faster async
- Redis for instant duplicate checks
- Fingerprinting reduces API calls
- Concurrent processing with limits
- Minimal resource usage (256MB RAM)
- Runs on $5/month VPS!

---

*Built for Smart Tree by the feedback-loving team!* 🚀