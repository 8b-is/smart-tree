# Smart Tree Feedback System Overview

A "mean ass efficient" feedback ingestion and processing system that automatically turns user feedback into actionable GitHub issues.

## System Architecture

```
┌─────────────────────┐
│   Smart Tree MCP    │
│  (AI Assistants)    │
└──────────┬──────────┘
           │ Submit Feedback
           ▼
┌─────────────────────┐     ┌──────────────┐     ┌─────────────────┐
│   Feedback API      │────▶│    Redis     │────▶│  Feedback Worker│
│   (Port 8422)       │     │   Queue      │     │   (Python)      │
└─────────────────────┘     └──────────────┘     └────────┬────────┘
                                                           │
                                                           ▼
                                                    ┌──────────────┐
                                                    │    GitHub    │
                                                    │    Issues    │
                                                    └──────────────┘
```

## Key Components

### 1. Feedback API (`/feedback-api`)
- FastAPI service on port 8422
- Accepts structured feedback from AI assistants
- Compresses feedback with zlib (10x reduction)
- Stores in date-organized directories
- Provides `/pending` endpoint for workers

### 2. Feedback Worker (`/feedback-worker`)
- Python async worker with uvloop
- Intelligent categorization (bugs, features, teleportation)
- Duplicate detection using multiple strategies
- Creates GitHub issues with proper labels
- Prometheus metrics for monitoring

### 3. Cloud-Init Script (`/cloud-init`)
- Automated Hetzner deployment
- Minimal CX11 instance (1 vCPU, 2GB RAM)
- Auto-installs Smart Tree, Docker, monitoring
- Production-ready sysctl optimizations

## Features

### Intelligent Processing
- **Pattern-based categorization**: Detects bugs vs features vs revolutionary ideas
- **Priority scoring**: High-impact bugs get immediate attention
- **Duplicate prevention**: Title hashing, Redis caching, GitHub issue checking
- **Auto-fix detection**: Labels issues that AI can implement

### Efficiency Optimizations
- **Batch processing**: 10 items at a time
- **Connection pooling**: Reuses HTTP/GitHub connections
- **Redis caching**: 7-day duplicate detection window
- **Async everything**: uvloop for maximum throughput
- **Minimal resources**: Runs on $5/month VPS

### Monitoring & Health
- **Prometheus metrics**: Processing rates, queue depth, errors
- **Grafana dashboards**: Beautiful visualization included
- **Health endpoints**: API and worker health checks
- **Docker compose**: Easy local development

## Quick Start

### Local Development
```bash
# Build and run
./scripts/manage.sh feedback-build
./scripts/manage.sh feedback-run

# Test the system
./scripts/manage.sh feedback-test

# Check status
./scripts/manage.sh feedback-status
```

### Production Deployment
```bash
# Set environment variables
export GITHUB_TOKEN=your_token
export HETZNER_TOKEN=your_token

# Deploy to Hetzner
./feedback-worker/deploy.sh hetzner
```

## Feedback Flow

1. **AI submits feedback** via MCP tools (`submit_feedback`, `request_tool`)
2. **API compresses and stores** feedback with unique ID
3. **Worker polls queue** every 10 seconds
4. **Categorization engine** determines type and priority
5. **Duplicate check** prevents redundant issues
6. **GitHub issue created** with proper labels and formatting
7. **Metrics updated** for monitoring

## GitHub Issue Templates

### Bug Issues
- Priority labels (Critical/High/Normal)
- Impact and frequency scores
- Code examples with expected output
- Auto-fix label if applicable

### Feature Requests
- Use case documentation
- Implementation suggestions
- Impact scoring for prioritization

## Performance

- Processes 100+ feedback items/minute
- 10x compression reduces storage
- Sub-second categorization
- Parallel GitHub API calls
- Memory usage < 256MB

## Monitoring

Access Grafana at `http://localhost:3000` (admin/admin) to view:
- Feedback processing rate
- Queue depth and latency
- GitHub API usage
- Success/error rates
- System resource usage

## Future Enhancements

1. **ML-based categorization** using feedback history
2. **Auto-PR generation** for auto-fixable issues
3. **Slack/Discord notifications** for critical bugs
4. **Feedback analytics** dashboard
5. **Multi-repo support** for monorepos

## Contributing

The feedback system itself accepts feedback! Use the `submit_feedback` MCP tool with category "feedback-system" to suggest improvements.

---

Built with efficiency and rock music in mind. May your feedback be ever actionable! 🎸