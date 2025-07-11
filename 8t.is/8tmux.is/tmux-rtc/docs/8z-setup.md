# 🌊 TMUX Hot Tub @ 8z.is Setup

Perfect! With your *.8z.is domain and local proxy, here's the ideal setup:

## Quick Deploy

1. **Start the Hot Tub locally:**
```bash
cd tmux-rtc
./scripts/manage.sh start
# Runs on http://localhost:8888
```

2. **Access Options:**
- **Direct Local**: `http://localhost:8888`
- **Local Network**: `http://192.168.x.x:8888`
- **Through Proxy**: `https://tmux.8z.is`

## Suggested Proxy Config

For subdomain `tmux.8z.is`:

```nginx
server {
    server_name tmux.8z.is;
    
    location / {
        proxy_pass http://localhost:8888;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_buffering off;
        proxy_request_buffering off;
    }
}
```

## Client Connection

When connecting through your proxy, just use:
- Server URL: `tmux.8z.is` (auto-detects wss://)
- Session: `aye-hue-collab`

## Mobile Access

Share with your devices:
- `https://tmux.8z.is` - Works perfectly on iPhone/iPad!
- Add to Home Screen for app-like experience

~~ Hue, we're ready to dive into the 8z.is Hot Tub! ~~

🎉 Aye & Hue @ 8z.is - Making waves together! 🌊