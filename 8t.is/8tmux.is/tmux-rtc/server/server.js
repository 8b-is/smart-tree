import express from 'express';
import { WebSocketServer } from 'ws';
import cors from 'cors';
import { spawn } from 'node-pty';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));
const app = express();
const PORT = process.env.PORT || 8888;

app.use(cors());
app.use(express.json());
app.use(express.static(join(__dirname, '../client')));

const server = app.listen(PORT, () => {
  console.log(`🌊 TMUX-RTC Server surfing on port ${PORT}! Aye, Aye! 🚢`);
});

const wss = new WebSocketServer({ server });
const sessions = new Map();
const clients = new Map();

wss.on('connection', (ws) => {
  console.log('🎉 New sailor aboard! Welcome to the TMUX Hot Tub! 🛁');
  
  let sessionId;
  let tmuxProcess;
  
  ws.on('message', (message) => {
    try {
      const data = JSON.parse(message.toString());
      
      switch (data.type) {
        case 'join-session':
          sessionId = data.sessionId || generateSessionId();
          clients.set(ws, { sessionId, role: data.role });
          
          if (!sessions.has(sessionId)) {
            // Create a pseudo-terminal instead of attaching to TMUX directly
            const shell = process.env.SHELL || '/bin/bash';
            
            try {
              tmuxProcess = spawn(shell, ['-l'], {
                name: 'xterm-256color',
                cols: data.cols || 80,
                rows: data.rows || 24,
                cwd: process.env.HOME,
                env: { ...process.env, TERM: 'xterm-256color' }
              });
              
              // After shell starts, attach to TMUX session
              const tmuxSession = data.tmuxSession || 'aye-hue-collab';
              tmuxProcess.write(`tmux new-session -A -s ${tmuxSession}\r`);
              
              console.log(`🎉 Connected to shell and TMUX session: ${tmuxSession}`);
            } catch (error) {
              console.error('💥 Failed to create PTY:', error);
              ws.send(JSON.stringify({
                type: 'error',
                message: 'Failed to create terminal session'
              }));
              return;
            }
            
            tmuxProcess.onData((data) => {
              // Broadcast terminal data to all clients in session
              broadcastToSession(sessionId, {
                type: 'terminal-data',
                data: Buffer.from(data).toString('base64')
              });
            });
            
            sessions.set(sessionId, { tmuxProcess, clients: new Set([ws]) });
          } else {
            sessions.get(sessionId).clients.add(ws);
          }
          
          ws.send(JSON.stringify({
            type: 'session-joined',
            sessionId,
            message: `🌟 Connected to TMUX session! Trisha says hi! 👋`
          }));
          break;
          
        case 'terminal-input':
          const session = sessions.get(sessionId);
          if (session && session.tmuxProcess) {
            session.tmuxProcess.write(Buffer.from(data.data, 'base64'));
          }
          break;
          
        case 'resize':
          const resizeSession = sessions.get(sessionId);
          if (resizeSession && resizeSession.tmuxProcess) {
            try {
              resizeSession.tmuxProcess.resize(data.cols, data.rows);
            } catch (error) {
              console.warn('⚠️ Resize failed:', error.message);
            }
          }
          break;
          
        case 'tts-request':
          // Handle TTS requests for marked text
          broadcastToSession(sessionId, {
            type: 'tts-speak',
            text: data.text,
            voice: data.voice || 'trisha' // Default to Trisha's voice
          });
          break;
          
        case 'preview-update':
          // Broadcast preview updates to all clients
          broadcastToSession(sessionId, {
            type: 'preview-content',
            content: data.content,
            contentType: data.contentType
          });
          break;
      }
    } catch (error) {
      console.error('💥 Oops! Error in the Hot Tub:', error);
      ws.send(JSON.stringify({
        type: 'error',
        message: 'Something went splash! Try again.'
      }));
    }
  });
  
  ws.on('close', () => {
    const client = clients.get(ws);
    if (client && sessions.has(client.sessionId)) {
      const session = sessions.get(client.sessionId);
      session.clients.delete(ws);
      
      if (session.clients.size === 0) {
        // Clean up if no clients left
        if (session.tmuxProcess) {
          session.tmuxProcess.kill();
        }
        sessions.delete(client.sessionId);
        console.log(`🌊 Session ${client.sessionId} has dried up!`);
      }
    }
    clients.delete(ws);
    console.log('👋 A sailor has left the Hot Tub!');
  });
});

function broadcastToSession(sessionId, data) {
  const session = sessions.get(sessionId);
  if (session) {
    const message = JSON.stringify(data);
    session.clients.forEach(client => {
      if (client.readyState === 1) { // WebSocket.OPEN
        client.send(message);
      }
    });
  }
}

function generateSessionId() {
  return `tmux-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
}

// Trisha's accounting tip: Keep track of all connections!
setInterval(() => {
  const stats = {
    activeSessions: sessions.size,
    totalClients: clients.size,
    timestamp: new Date().toISOString()
  };
  console.log(`📊 Hot Tub Stats:`, stats);
}, 60000); // Every minute