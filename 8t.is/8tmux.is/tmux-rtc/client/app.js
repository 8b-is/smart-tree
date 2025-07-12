// 🚀 TMUX Hot Tub Client - Where Aye & Hue collaborate in style!
// Trisha's Note: This code sparkles with efficiency! ✨

class TMUXHotTub {
    constructor() {
        this.ws = null;
        this.terminal = null;
        this.fitAddon = null;
        this.webLinksAddon = null;
        this.sessionId = null;
        this.isConnected = false;
        this.ttsEnabled = true;
        this.ttsVoices = null;
        
        this.initializeTerminal();
        this.initializeEventListeners();
        this.loadTTSVoices();
        this.showConnectionModal();
    }
    
    initializeTerminal() {
        // Create the terminal with Hot Tub theming
        this.terminal = new Terminal({
            theme: {
                background: '#0a0e27',
                foreground: '#e4e7ff',
                cursor: '#00ffff',
                cursorAccent: '#0a0e27',
                selection: 'rgba(0, 255, 255, 0.3)',
                black: '#0a0e27',
                red: '#ff0044',
                green: '#00ff88',
                yellow: '#ffff00',
                blue: '#0088ff',
                magenta: '#ff00ff',
                cyan: '#00ffff',
                white: '#e4e7ff',
                brightBlack: '#3a4570',
                brightRed: '#ff4488',
                brightGreen: '#44ffaa',
                brightYellow: '#ffff88',
                brightBlue: '#44aaff',
                brightMagenta: '#ff88ff',
                brightCyan: '#88ffff',
                brightWhite: '#ffffff'
            },
            fontFamily: 'Menlo, Monaco, "Courier New", monospace',
            fontSize: 14,
            cursorBlink: true,
            cursorStyle: 'block'
        });
        
        // Add addons for better functionality
        this.fitAddon = new FitAddon.FitAddon();
        this.webLinksAddon = new WebLinksAddon.WebLinksAddon();
        
        this.terminal.loadAddon(this.fitAddon);
        this.terminal.loadAddon(this.webLinksAddon);
        
        // Open terminal in the container
        this.terminal.open(document.getElementById('terminal'));
        this.fitAddon.fit();
        
        // Handle terminal input
        this.terminal.onData((data) => {
            if (this.isConnected && this.ws) {
                // Check for TTS markers
                this.checkForTTSMarkers(data);
                
                this.ws.send(JSON.stringify({
                    type: 'terminal-input',
                    data: btoa(data)
                }));
            }
        });
        
        // Handle resize
        this.terminal.onResize(({ cols, rows }) => {
            if (this.isConnected && this.ws) {
                this.ws.send(JSON.stringify({
                    type: 'resize',
                    cols,
                    rows
                }));
            }
        });
        
        // Fit terminal on window resize
        window.addEventListener('resize', () => {
            this.fitAddon.fit();
        });
    }
    
    initializeEventListeners() {
        // Connection form
        document.getElementById('connection-form').addEventListener('submit', (e) => {
            e.preventDefault();
            this.connect();
        });
        
        // Terminal controls
        document.getElementById('clear-terminal').addEventListener('click', () => {
            this.terminal.clear();
            this.showNotification('🧹 Terminal cleared! Fresh start!');
        });
        
        document.getElementById('copy-selected').addEventListener('click', () => {
            const selection = this.terminal.getSelection();
            if (selection) {
                navigator.clipboard.writeText(selection);
                this.showNotification('📋 Copied to clipboard!');
            }
        });
        
        document.getElementById('paste-text').addEventListener('click', async () => {
            try {
                const text = await navigator.clipboard.readText();
                this.terminal.paste(text);
            } catch (err) {
                this.showNotification('📌 Unable to paste - check permissions');
            }
        });
        
        // Layout toggle for mobile
        document.getElementById('toggle-layout').addEventListener('click', () => {
            const container = document.getElementById('split-container');
            container.classList.toggle('vertical-layout');
            this.fitAddon.fit();
            this.showNotification('📱 Layout toggled!');
        });
        
        // Preview mode selector
        document.getElementById('preview-mode').addEventListener('change', (e) => {
            this.updatePreviewMode(e.target.value);
        });
    }
    
    async loadTTSVoices() {
        // Load available TTS voices
        if ('speechSynthesis' in window) {
            this.ttsVoices = await new Promise((resolve) => {
                const voices = speechSynthesis.getVoices();
                if (voices.length) {
                    resolve(voices);
                } else {
                    speechSynthesis.onvoiceschanged = () => {
                        resolve(speechSynthesis.getVoices());
                    };
                }
            });
            console.log('🎤 TTS voices loaded! Trisha has', this.ttsVoices.length, 'voices to choose from!');
        }
    }
    
    connect() {
        let serverUrl = document.getElementById('server-url').value;
        const sessionId = document.getElementById('session-id').value;
        const tmuxSession = document.getElementById('tmux-session').value;
        
        // Auto-detect protocol based on current page
        if (!serverUrl.startsWith('ws://') && !serverUrl.startsWith('wss://')) {
            const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
            serverUrl = `${protocol}//${serverUrl}`;
        }
        
        // Close existing connection
        if (this.ws) {
            this.ws.close();
        }
        
        this.ws = new WebSocket(serverUrl);
        
        this.ws.onopen = () => {
            console.log('🌊 Connected to the Hot Tub!');
            this.isConnected = true;
            this.updateConnectionStatus(true);
            
            // Join session
            this.ws.send(JSON.stringify({
                type: 'join-session',
                sessionId: sessionId || null,
                tmuxSession: tmuxSession,
                role: 'collaborator'
            }));
            
            // Hide modal
            document.getElementById('connection-modal').classList.add('hidden');
        };
        
        this.ws.onmessage = (event) => {
            const data = JSON.parse(event.data);
            this.handleServerMessage(data);
        };
        
        this.ws.onerror = (error) => {
            console.error('💥 WebSocket error:', error);
            this.showNotification('🚫 Connection error! Check server.');
        };
        
        this.ws.onclose = () => {
            console.log('👋 Disconnected from Hot Tub');
            this.isConnected = false;
            this.updateConnectionStatus(false);
        };
    }
    
    handleServerMessage(data) {
        switch (data.type) {
            case 'session-joined':
                this.sessionId = data.sessionId;
                document.getElementById('session-info').textContent = `Session: ${this.sessionId}`;
                this.showNotification(data.message);
                break;
                
            case 'terminal-data':
                // Decode and write to terminal
                const termData = atob(data.data);
                this.terminal.write(termData);
                
                // Check for TTS markers in output
                this.scanForTTSMarkers(termData);
                break;
                
            case 'tts-speak':
                this.speak(data.text, data.voice);
                break;
                
            case 'preview-content':
                this.updatePreview(data.content, data.contentType);
                break;
                
            case 'error':
                this.showNotification(`❌ ${data.message}`);
                break;
        }
    }
    
    checkForTTSMarkers(input) {
        // Look for ~~ markers in user input
        const ttsPattern = /~~(.+?)~~/g;
        const matches = input.match(ttsPattern);
        
        if (matches) {
            matches.forEach(match => {
                const text = match.replace(/~~/g, '').trim();
                this.speak(text, 'user');
            });
        }
    }
    
    scanForTTSMarkers(output) {
        // Scan terminal output for TTS markers
        const ttsPattern = /~~(.+?)~~/g;
        let match;
        
        while ((match = ttsPattern.exec(output)) !== null) {
            const text = match[1].trim();
            // ~~ Hue, Check this out ~~
            if (text.toLowerCase().includes('hue')) {
                this.speak(text, 'aye');
            } else if (text.toLowerCase().includes('trisha')) {
                this.speak(text, 'trisha');
            } else {
                this.speak(text, 'system');
            }
        }
    }
    
    speak(text, voice = 'system') {
        if (!this.ttsEnabled || !this.ttsVoices) return;
        
        const utterance = new SpeechSynthesisUtterance(text);
        
        // Select voice based on who's speaking
        switch (voice) {
            case 'aye':
                // Aye gets a friendly, knowledgeable voice
                utterance.voice = this.ttsVoices.find(v => v.name.includes('Daniel')) || this.ttsVoices[0];
                utterance.pitch = 0.9;
                utterance.rate = 1.1;
                break;
            case 'trisha':
                // Trisha gets an enthusiastic, bubbly voice
                utterance.voice = this.ttsVoices.find(v => v.name.includes('Samantha') || v.name.includes('Victoria')) || this.ttsVoices[1];
                utterance.pitch = 1.2;
                utterance.rate = 1.2;
                break;
            case 'user':
                // Hue gets a calm, thoughtful voice
                utterance.voice = this.ttsVoices.find(v => v.name.includes('Alex')) || this.ttsVoices[2];
                utterance.pitch = 1.0;
                utterance.rate = 1.0;
                break;
            default:
                // System messages
                utterance.voice = this.ttsVoices[0];
                utterance.pitch = 1.0;
                utterance.rate = 1.1;
        }
        
        speechSynthesis.speak(utterance);
        
        // Visual indicator
        this.showNotification(`🔊 ${voice}: ${text.substring(0, 30)}...`);
    }
    
    updatePreviewMode(mode) {
        const content = document.getElementById('preview-content');
        
        switch (mode) {
            case 'markdown':
                content.innerHTML = '<div class="markdown-preview"><p>📝 Waiting for markdown content...</p></div>';
                break;
            case 'webpage':
                content.innerHTML = '<iframe src="about:blank" style="width: 100%; height: 100%; border: none;"></iframe>';
                break;
            case 'stats':
                this.showStats();
                break;
            case 'notes':
                content.innerHTML = `
                    <div class="notes-section">
                        <h3>📓 Session Notes</h3>
                        <textarea id="session-notes" placeholder="Type your notes here..." 
                                  style="width: 100%; height: 300px; background: var(--bg-tertiary); 
                                         color: var(--text-primary); border: 1px solid var(--border-color); 
                                         border-radius: 0.5rem; padding: 1rem; font-size: 1rem;"></textarea>
                        <button onclick="hotTub.saveNotes()" class="btn-primary" style="margin-top: 1rem;">
                            💾 Save Notes
                        </button>
                    </div>`;
                break;
        }
    }
    
    updatePreview(content, contentType) {
        const previewContent = document.getElementById('preview-content');
        const previewMode = document.getElementById('preview-mode');
        
        switch (contentType) {
            case 'markdown':
                previewMode.value = 'markdown';
                previewContent.innerHTML = `<div class="markdown-preview">${marked.parse(content)}</div>`;
                break;
            case 'html':
                previewMode.value = 'webpage';
                const iframe = previewContent.querySelector('iframe') || document.createElement('iframe');
                iframe.srcdoc = content;
                if (!previewContent.contains(iframe)) {
                    previewContent.innerHTML = '';
                    previewContent.appendChild(iframe);
                }
                break;
            case 'stats':
                previewMode.value = 'stats';
                this.updateStats(content);
                break;
        }
    }
    
    showStats() {
        const content = document.getElementById('preview-content');
        content.innerHTML = `
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-value">--</div>
                    <div class="stat-label">Commands Run</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">--</div>
                    <div class="stat-label">Session Time</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">--</div>
                    <div class="stat-label">Characters Typed</div>
                </div>
                <div class="stat-card">
                    <div class="stat-value">--</div>
                    <div class="stat-label">TTS Messages</div>
                </div>
            </div>
            <div style="margin-top: 2rem; text-align: center;">
                <p style="color: var(--text-secondary);">
                    📊 Trisha's Accounting Tip: "Every keystroke counts!"
                </p>
            </div>`;
    }
    
    updateStats(stats) {
        // Update stats display with real data
        const cards = document.querySelectorAll('.stat-card');
        if (cards.length >= 4 && stats) {
            cards[0].querySelector('.stat-value').textContent = stats.commands || '0';
            cards[1].querySelector('.stat-value').textContent = stats.sessionTime || '00:00';
            cards[2].querySelector('.stat-value').textContent = stats.characters || '0';
            cards[3].querySelector('.stat-value').textContent = stats.ttsCount || '0';
        }
    }
    
    saveNotes() {
        const notes = document.getElementById('session-notes').value;
        localStorage.setItem(`tmux-notes-${this.sessionId}`, notes);
        this.showNotification('📓 Notes saved locally!');
    }
    
    updateConnectionStatus(connected) {
        const status = document.getElementById('connection-status');
        if (connected) {
            status.textContent = '🟢 Connected';
            status.style.color = 'var(--success)';
        } else {
            status.textContent = '🔴 Disconnected';
            status.style.color = 'var(--error)';
        }
    }
    
    showConnectionModal() {
        document.getElementById('connection-modal').classList.remove('hidden');
    }
    
    showNotification(message) {
        // Create a temporary notification
        const notification = document.createElement('div');
        notification.className = 'notification';
        notification.textContent = message;
        notification.style.cssText = `
            position: fixed;
            top: 20px;
            right: 20px;
            background: var(--bg-tertiary);
            color: var(--text-primary);
            padding: 1rem 1.5rem;
            border-radius: 0.5rem;
            border: 1px solid var(--accent-cyan);
            box-shadow: 0 0 20px rgba(0, 255, 255, 0.5);
            animation: slideIn 0.3s ease-out;
            z-index: 2000;
        `;
        
        document.body.appendChild(notification);
        
        setTimeout(() => {
            notification.style.animation = 'slideOut 0.3s ease-out';
            setTimeout(() => notification.remove(), 300);
        }, 3000);
    }
}

// Initialize the Hot Tub when DOM is ready
let hotTub;
document.addEventListener('DOMContentLoaded', () => {
    hotTub = new TMUXHotTub();
    console.log('🛁 Hot Tub is ready! Aye & Hue, let\'s collaborate!');
});

// Add CSS animations
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from {
            transform: translateX(100%);
            opacity: 0;
        }
        to {
            transform: translateX(0);
            opacity: 1;
        }
    }
    
    @keyframes slideOut {
        from {
            transform: translateX(0);
            opacity: 1;
        }
        to {
            transform: translateX(100%);
            opacity: 0;
        }
    }
    
    .vertical-layout {
        flex-direction: column !important;
    }
`;
document.head.appendChild(style);