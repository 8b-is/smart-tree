# 🔒 Proxy Setup for TMUX Hot Tub

Since you have a local proxy, here's the optimal setup:

## 1. Run the Hot Tub Server (HTTP)

```bash
cd tmux-rtc
./scripts/manage.sh install
./scripts/manage.sh start
# Server runs on http://localhost:8888
```

## 2. Configure Your Proxy

Add this to your proxy configuration:

### For NGINX:
```nginx
location /tmux/ {
    proxy_pass http://YOUR_LOCAL_IP:8888/;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    proxy_set_header X-Forwarded-Proto $scheme;
}
```

### For Caddy:
```caddy
tmux.yourdomain.com {
    reverse_proxy localhost:8888 {
        header_up Host {host}
        header_up X-Real-IP {remote}
        header_up X-Forwarded-Proto {scheme}
    }
}
```

### For HAProxy:
```
backend tmux_backend
    mode http
    server tmux1 YOUR_LOCAL_IP:8888 check
    timeout tunnel 1h
    option http-server-close
    option forwardfor
```

## 3. Access from Devices

- **Local Network**: `http://YOUR_LOCAL_IP:8888`
- **Through Proxy**: `https://your.proxy.domain/tmux/`
- **iPhone/iPad**: Use the proxy URL for secure access

## 4. Tips for Your Setup

Since you have a local proxy, you can:

1. **Keep it simple**: Run the standard HTTP server
2. **Let proxy handle SSL**: No need for certificates on the app
3. **Use subdomains**: `tmux.local.domain` → `localhost:8888`
4. **Add authentication**: Most proxies support basic auth

## 5. Quick Test

```bash
# From the machine running the server:
curl http://localhost:8888

# From another device on your network:
curl http://YOUR_LOCAL_IP:8888

# Through your proxy:
curl https://your.proxy.domain/tmux/
```

~~ Hue, your proxy is handling the security perfectly! ~~