#!/usr/bin/env python3
"""
Simple SSE test server for Smart Tree
Demonstrates how to serve SSE events for directory monitoring
"""

import asyncio
import json
import time
from datetime import datetime
from pathlib import Path
from aiohttp import web
import aiohttp_sse

# Simulated directory events
SAMPLE_EVENTS = [
    {
        "type": "scan_complete",
        "path": "/home/user/project",
        "stats": {
            "total_files": 1234,
            "total_dirs": 56,
            "total_size": 78901234,
            "scan_time_ms": 1500
        }
    },
    {
        "type": "created",
        "path": "/home/user/project/new_file.rs",
        "node": {
            "path": "/home/user/project/new_file.rs",
            "is_dir": False,
            "size": 1024,
            "permissions": "644",
            "modified": datetime.utcnow().isoformat()
        }
    },
    {
        "type": "modified",
        "path": "/home/user/project/main.rs",
        "node": {
            "path": "/home/user/project/main.rs",
            "is_dir": False,
            "size": 2048,
            "permissions": "644",
            "modified": datetime.utcnow().isoformat()
        }
    },
    {
        "type": "analysis",
        "path": "/home/user/project",
        "format": "ai",
        "data": "TREE_HEX_V1:\n0 755 1000 1000 1000 0 📁 project/\n1 644 1000 1000 400 0 🦀 main.rs\n1 644 1000 1000 600 0 📋 Cargo.toml\nSTATS: F:2 D:1 S:1000 (0MB)\nTYPES: rs:1 toml:1\nEND_AI"
    },
    {
        "type": "stats",
        "path": "/home/user/project",
        "stats": {
            "total_files": 1235,
            "total_dirs": 56,
            "total_size": 78902258,
            "scan_time_ms": 500
        }
    },
    {
        "type": "deleted",
        "path": "/home/user/project/old_file.txt"
    }
]

async def sse_handler(request):
    """Handle SSE connections"""
    async with aiohttp_sse.sse_response(request) as resp:
        # Send initial connection event
        await resp.send(json.dumps({
            "type": "connection",
            "message": "Connected to Smart Tree SSE test server",
            "timestamp": datetime.utcnow().isoformat()
        }), event='init', id=str(int(time.time() * 1000)))
        
        # Send sample events with delays
        event_id = 1
        for event in SAMPLE_EVENTS:
            await asyncio.sleep(2)  # 2 second delay between events
            await resp.send(json.dumps(event), event=event['type'], id=str(event_id))
            event_id += 1
        
        # Send periodic heartbeats
        heartbeat_count = 0
        while True:
            await asyncio.sleep(5)  # Heartbeat every 5 seconds
            heartbeat_count += 1
            await resp.send(json.dumps({
                "type": "heartbeat",
                "count": heartbeat_count
            }), event='heartbeat', id=str(event_id))
            event_id += 1
            
            # Send a stats update every 3 heartbeats
            if heartbeat_count % 3 == 0:
                await resp.send(json.dumps({
                    "type": "stats",
                    "path": "/home/user/project",
                    "stats": {
                        "total_files": 1235 + heartbeat_count,
                        "total_dirs": 56,
                        "total_size": 78902258 + (heartbeat_count * 1024),
                        "scan_time_ms": 500
                    }
                }), event='stats', id=str(event_id))
                event_id += 1

async def index_handler(request):
    """Serve a simple HTML page for browser testing"""
    html = """
    <!DOCTYPE html>
    <html>
    <head>
        <title>Smart Tree SSE Test</title>
        <style>
            body { font-family: monospace; padding: 20px; }
            #events { 
                background: #f0f0f0; 
                padding: 10px; 
                height: 400px; 
                overflow-y: auto;
                white-space: pre-wrap;
            }
            .event { 
                margin: 5px 0; 
                padding: 5px; 
                background: white; 
                border-left: 3px solid #0066cc;
            }
            .heartbeat { border-color: #green; opacity: 0.7; }
            .created { border-color: #00cc00; }
            .modified { border-color: #ffcc00; }
            .deleted { border-color: #cc0000; }
        </style>
    </head>
    <body>
        <h1>Smart Tree SSE Test</h1>
        <div id="events"></div>
        <script>
            const eventsDiv = document.getElementById('events');
            const source = new EventSource('/sse');
            
            function addEvent(type, data) {
                const div = document.createElement('div');
                div.className = 'event ' + type;
                div.textContent = `[${new Date().toISOString()}] ${type}: ${JSON.stringify(data, null, 2)}`;
                eventsDiv.appendChild(div);
                eventsDiv.scrollTop = eventsDiv.scrollHeight;
            }
            
            source.addEventListener('message', (e) => {
                const data = JSON.parse(e.data);
                addEvent(data.type || 'message', data);
            });
            
            source.addEventListener('init', (e) => {
                const data = JSON.parse(e.data);
                addEvent('init', data);
            });
            
            ['scan_complete', 'created', 'modified', 'deleted', 'analysis', 'stats', 'heartbeat'].forEach(eventType => {
                source.addEventListener(eventType, (e) => {
                    const data = JSON.parse(e.data);
                    addEvent(eventType, data);
                });
            });
            
            source.addEventListener('error', (e) => {
                addEvent('error', { message: 'Connection error', readyState: source.readyState });
            });
        </script>
    </body>
    </html>
    """
    return web.Response(text=html, content_type='text/html')

def create_app():
    """Create the web application"""
    app = web.Application()
    app.router.add_get('/', index_handler)
    app.router.add_get('/sse', sse_handler)
    return app

if __name__ == '__main__':
    print("🚀 Smart Tree SSE Test Server")
    print("📍 Server running at http://localhost:8420")
    print("🔗 SSE endpoint: http://localhost:8420/sse")
    print("🌐 Browser test: http://localhost:8420")
    print("\nPress Ctrl+C to stop")
    
    app = create_app()
    web.run_app(app, host='0.0.0.0', port=8420)