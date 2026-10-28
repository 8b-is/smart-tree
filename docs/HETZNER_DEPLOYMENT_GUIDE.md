# Smart Tree Feedback System - Hetzner Deployment Guide 🚀🏗️

## Your Construction Helper is Going to the Cloud! ☁️

This guide walks through deploying the Smart Tree feedback system to Hetzner Cloud. Think of it as sending your construction helper to work on a remote site!

## Prerequisites 🛠️

### 1. Hetzner Account Setup
- Create account at [console.hetzner.cloud](https://console.hetzner.cloud)
- Generate API token: Project → Security → API Tokens → Generate API Token
- Save the token securely (you'll need it as `HETZNER_TOKEN`)

### 2. Install Hetzner CLI
```bash
# macOS
brew install hcloud

# Linux
wget -q -O- https://github.com/hetznercloud/cli/releases/latest/download/hcloud-linux-amd64.tar.gz | tar xz
sudo mv hcloud /usr/local/bin/

# Verify installation
hcloud version
```

### 3. Configure CLI
```bash
hcloud context create smart-tree-feedback
# Enter your API token when prompted
```

## Environment Setup 🔧

### Required Environment Variables
```bash
export HETZNER_TOKEN="your-hetzner-api-token"
export GITHUB_TOKEN="your-github-token"  # For creating issues from feedback
export DISCORD_WEBHOOK="https://discord.com/api/webhooks/..."  # Optional
```

### Create .env File
```bash
cd /Users/wraith/source/HomeAssistant/st-aygent/feedback-worker
cat > .env << EOF
HETZNER_TOKEN=${HETZNER_TOKEN}
GITHUB_TOKEN=${GITHUB_TOKEN}
FEEDBACK_API_URL=https://f.8b.is/api
DISCORD_WEBHOOK=${DISCORD_WEBHOOK}
EOF
```

## Deployment Options 🚀

### Option 1: Quick Deploy (Recommended) 
Your helper does all the work!

```bash
cd /Users/wraith/source/HomeAssistant/st-aygent/feedback-worker
./deploy.sh hetzner
```

This will:
1. Build Docker containers
2. Push to GitHub Container Registry
3. Create Hetzner server with cloud-init
4. Deploy feedback worker automatically
5. Setup monitoring with cAdvisor

### Option 2: Manual Deployment
For when you want more control over your construction site!

#### Step 1: Build and Push Containers
```bash
cd /Users/wraith/source/HomeAssistant/st-aygent/feedback-worker

# Build containers
docker build -t ghcr.io/8b-is/smart-tree-feedback-worker:latest .

# Login to GitHub Container Registry
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin

# Push to registry
docker push ghcr.io/8b-is/smart-tree-feedback-worker:latest
```

#### Step 2: Create Hetzner Server
```bash
# Prepare cloud-init with environment variables
cd /Users/wraith/source/HomeAssistant/st-aygent
envsubst < cloud-init/hetzner-feedback-worker.yaml > /tmp/cloud-init.yaml

# Create server
hcloud server create \
  --name smart-tree-feedback-prod \
  --type cx11 \
  --image ubuntu-22.04 \
  --location nbg1 \
  --user-data-from-file /tmp/cloud-init.yaml \
  --label service=smart-tree-feedback \
  --label environment=production

# Get server IP
hcloud server ip smart-tree-feedback-prod
```

#### Step 3: Verify Deployment
```bash
# Wait for server to initialize (about 2-3 minutes)
sleep 180

# SSH into server
ssh root@$(hcloud server ip smart-tree-feedback-prod)

# Check service status
systemctl status smart-tree-feedback
docker-compose ps
```

## Server Specifications 📊

### Recommended: CX11
- **vCPU**: 1 core
- **RAM**: 2 GB
- **Storage**: 20 GB NVMe
- **Cost**: ~€3.29/month
- **Perfect for**: Feedback worker with Redis

### Alternative: CPX11 (More Power)
- **vCPU**: 2 cores (shared)
- **RAM**: 2 GB
- **Storage**: 40 GB NVMe  
- **Cost**: ~€4.15/month
- **When to use**: Higher feedback volume

## Monitoring & Management 🔍

### Health Check
```bash
# From deployment machine
SERVER_IP=$(hcloud server ip smart-tree-feedback-prod)
ssh root@$SERVER_IP /opt/smart-tree-feedback/health-check.sh

# Or directly
curl http://$SERVER_IP:9090/metrics  # Prometheus metrics
curl http://$SERVER_IP:8080/         # cAdvisor dashboard
```

### View Logs
```bash
# SSH into server
ssh root@$SERVER_IP

# View worker logs
cd /opt/smart-tree-feedback
docker-compose logs -f feedback-worker

# View Redis logs
docker-compose logs -f redis
```

### Update Deployment
```bash
# Pull latest changes
ssh root@$SERVER_IP "cd /opt/smart-tree-feedback && docker-compose pull"

# Restart services
ssh root@$SERVER_IP "docker-compose restart"
```

## Firewall Configuration 🔐

The cloud-init script sets up basic firewall rules. To add more:

```bash
# SSH into server
ssh root@$SERVER_IP

# Allow specific ports
ufw allow 9090/tcp  # Prometheus metrics
ufw allow 8080/tcp  # cAdvisor

# Check status
ufw status
```

## Backup Strategy 💾

### Redis Data Backup
```bash
# Create backup
ssh root@$SERVER_IP "docker exec smart-tree-feedback_redis_1 redis-cli BGSAVE"

# Download backup
scp root@$SERVER_IP:/opt/smart-tree-feedback/redis-data/dump.rdb ./backup-$(date +%Y%m%d).rdb
```

### Automated Backups (Optional)
Add to crontab on server:
```bash
0 3 * * * docker exec smart-tree-feedback_redis_1 redis-cli BGSAVE
0 4 * * * rclone copy /opt/smart-tree-feedback/redis-data your-backup-destination:smart-tree-backups/
```

## Scaling 📈

### Horizontal Scaling
Deploy multiple workers:
```bash
# Create additional servers
for i in {2..3}; do
  hcloud server create \
    --name smart-tree-feedback-worker-$i \
    --type cx11 \
    --image ubuntu-22.04 \
    --location nbg1 \
    --user-data-from-file /tmp/cloud-init.yaml
done
```

### Load Balancer (Future)
```bash
# Create load balancer
hcloud load-balancer create \
  --name smart-tree-feedback-lb \
  --type lb11 \
  --location nbg1
```

## Troubleshooting 🔧

### Common Issues

#### 1. Container Won't Start
```bash
# Check logs
docker-compose logs feedback-worker
# Usually: Missing environment variables
```

#### 2. Redis Connection Failed
```bash
# Check Redis is running
docker-compose ps redis
# Restart if needed
docker-compose restart redis
```

#### 3. GitHub Token Invalid
```bash
# Update token in .env
vim /opt/smart-tree-feedback/.env
# Restart worker
docker-compose restart feedback-worker
```

#### 4. High Memory Usage
```bash
# Check memory
docker stats
# Adjust Redis max memory
docker exec redis redis-cli CONFIG SET maxmemory 150mb
```

## Cost Optimization 💰

### Tips from Trisha in Accounting
1. **Use Snapshots**: Create snapshots before major changes (€0.01/GB/month)
2. **Reserved Pricing**: Not available on Hetzner, but predictable monthly costs
3. **Monitor Usage**: Set up alerts for unusual activity
4. **Clean Old Servers**: Delete test servers promptly

### Monthly Budget
- **Server (CX11)**: €3.29
- **Backups**: ~€0.20
- **Total**: ~€3.50/month

As Trisha says: "That's less than a fancy coffee! And this helper works 24/7!" ☕

## Production Checklist ✅

Before going live:
- [ ] Environment variables configured
- [ ] GitHub token has necessary permissions
- [ ] Firewall rules configured
- [ ] Monitoring endpoints accessible
- [ ] Backup strategy in place
- [ ] Health checks passing
- [ ] Discord webhook tested (if using)
- [ ] DNS configured (if using custom domain)

## Quick Commands Reference 🎯

```bash
# Deploy
./deploy.sh hetzner

# Check status
hcloud server list
ssh root@$(hcloud server ip smart-tree-feedback-prod) docker-compose ps

# View logs
ssh root@$(hcloud server ip smart-tree-feedback-prod) docker-compose logs -f

# Restart
ssh root@$(hcloud server ip smart-tree-feedback-prod) docker-compose restart

# Delete server (careful!)
hcloud server delete smart-tree-feedback-prod
```

## Support & Help 🆘

- **Hetzner Status**: [status.hetzner.com](https://status.hetzner.com)
- **Hetzner Docs**: [docs.hetzner.com](https://docs.hetzner.com)
- **Smart Tree Issues**: [github.com/8b-is/smart-tree/issues](https://github.com/8b-is/smart-tree/issues)

---

Remember: Your construction helper is now working in the cloud, processing feedback to make Smart Tree even better! 🏗️☁️

Pro Tip: "Deploy early, deploy often, but always check your tokens!" - The Cheet, probably 🎸

Aye, Aye! 🚢