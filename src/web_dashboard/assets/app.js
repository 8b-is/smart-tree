// Smart Tree Web Dashboard - Main Application

class Dashboard {
    constructor() {
        this.terminal = null;
        this.fitAddon = null;
        this.ws = null;
        this.currentPath = null;
        this.selectedFile = null;
        this.sidebarWidth = 250;

        this.init();
    }

    async init() {
        this.initTerminal();
        this.initWebSocket();
        this.initFileBrowser();
        this.initResizer();
        this.initEventListeners();
        await this.loadHealth();
    }

    // Terminal Setup
    initTerminal() {
        this.terminal = new Terminal({
            cursorBlink: true,
            cursorStyle: 'block',
            fontSize: 14,
            fontFamily: "'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
            theme: {
                background: '#0a0a0a',
                foreground: '#00ff00',
                cursor: '#00ff00',
                cursorAccent: '#0a0a0a',
                selectionBackground: '#006600',
                black: '#0a0a0a',
                red: '#ff4444',
                green: '#00ff00',
                yellow: '#ffaa00',
                blue: '#4444ff',
                magenta: '#ff44ff',
                cyan: '#00ffff',
                white: '#ffffff',
                brightBlack: '#444444',
                brightRed: '#ff6666',
                brightGreen: '#66ff66',
                brightYellow: '#ffcc66',
                brightBlue: '#6666ff',
                brightMagenta: '#ff66ff',
                brightCyan: '#66ffff',
                brightWhite: '#ffffff'
            },
            allowTransparency: true,
            scrollback: 10000
        });

        this.fitAddon = new FitAddon.FitAddon();
        this.terminal.loadAddon(this.fitAddon);

        const container = document.getElementById('terminal');
        this.terminal.open(container);
        this.fitAddon.fit();

        // Handle resize
        window.addEventListener('resize', () => this.fitAddon.fit());

        // Terminal input
        this.terminal.onData(data => {
            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                this.ws.send(JSON.stringify({ type: 'input', data }));
            }
        });

        // Terminal resize
        this.terminal.onResize(({ cols, rows }) => {
            if (this.ws && this.ws.readyState === WebSocket.OPEN) {
                this.ws.send(JSON.stringify({ type: 'resize', cols, rows }));
            }
        });
    }

    // WebSocket Connection
    initWebSocket() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws/terminal`;

        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
            this.updateConnectionStatus(true);
            // Send initial size
            const { cols, rows } = this.terminal;
            this.ws.send(JSON.stringify({ type: 'resize', cols, rows }));
        };

        this.ws.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data);
                switch (msg.type) {
                    case 'output':
                        this.terminal.write(msg.data);
                        break;
                    case 'exit':
                        this.terminal.write(`\r\n[Process exited with code ${msg.code}]\r\n`);
                        break;
                    case 'error':
                        this.terminal.write(`\r\n\x1b[31m[Error: ${msg.message}]\x1b[0m\r\n`);
                        break;
                }
            } catch (e) {
                console.error('Failed to parse message:', e);
            }
        };

        this.ws.onclose = () => {
            this.updateConnectionStatus(false);
            this.terminal.write('\r\n\x1b[33m[Disconnected - Reconnecting...]\x1b[0m\r\n');
            setTimeout(() => this.initWebSocket(), 2000);
        };

        this.ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };
    }

    updateConnectionStatus(connected) {
        const status = document.getElementById('connectionStatus');
        const dot = status.querySelector('.status-dot');
        const text = status.querySelector('.status-text');

        if (connected) {
            dot.classList.add('connected');
            dot.classList.remove('disconnected');
            text.textContent = 'Connected';
        } else {
            dot.classList.remove('connected');
            dot.classList.add('disconnected');
            text.textContent = 'Disconnected';
        }
    }

    // File Browser
    async initFileBrowser() {
        await this.loadFiles();
        document.getElementById('refreshFiles').addEventListener('click', () => this.loadFiles());
    }

    async loadFiles(path = null) {
        try {
            const url = path ? `/api/files?path=${encodeURIComponent(path)}` : '/api/files';
            const response = await fetch(url);
            const files = await response.json();

            this.currentPath = path || '.';
            this.renderFileTree(files);
            document.getElementById('cwdDisplay').textContent = this.currentPath;
        } catch (e) {
            console.error('Failed to load files:', e);
        }
    }

    renderFileTree(files) {
        const container = document.getElementById('fileTree');
        container.innerHTML = '';

        // Add parent directory link if not at root
        if (this.currentPath && this.currentPath !== '.') {
            const parentItem = this.createFileItem({
                name: '..',
                is_dir: true,
                path: this.getParentPath(this.currentPath)
            }, true);
            container.appendChild(parentItem);
        }

        files.forEach(file => {
            const item = this.createFileItem(file);
            container.appendChild(item);
        });
    }

    createFileItem(file, isParent = false) {
        const item = document.createElement('div');
        item.className = 'file-item' + (file.is_dir ? ' directory' : '');

        const icon = document.createElement('span');
        icon.className = 'file-icon ' + this.getIconClass(file);

        const name = document.createElement('span');
        name.className = 'file-name';
        name.textContent = file.name;

        item.appendChild(icon);
        item.appendChild(name);

        if (!file.is_dir && file.size !== undefined) {
            const size = document.createElement('span');
            size.className = 'file-size';
            size.textContent = this.formatSize(file.size);
            item.appendChild(size);
        }

        item.addEventListener('click', () => this.handleFileClick(file));
        item.addEventListener('dblclick', () => this.handleFileDoubleClick(file));

        return item;
    }

    getIconClass(file) {
        if (file.is_dir) return 'icon-folder';

        const type = file.file_type || 'file';
        switch (type) {
            case 'rust': return 'icon-rust';
            case 'python': return 'icon-python';
            case 'javascript': return 'icon-javascript';
            case 'typescript': return 'icon-typescript';
            case 'markdown': return 'icon-markdown';
            case 'json': return 'icon-json';
            case 'html': return 'icon-html';
            case 'css': return 'icon-css';
            case 'shell': return 'icon-shell';
            case 'lock': return 'icon-lock';
            case 'toml':
            case 'yaml': return 'icon-config';
            default: return 'icon-file';
        }
    }

    formatSize(bytes) {
        if (bytes < 1024) return bytes + ' B';
        if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' K';
        if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + ' M';
        return (bytes / (1024 * 1024 * 1024)).toFixed(1) + ' G';
    }

    getParentPath(path) {
        const parts = path.split('/');
        parts.pop();
        return parts.join('/') || '.';
    }

    async handleFileClick(file) {
        // Update selection
        document.querySelectorAll('.file-item.selected').forEach(el => el.classList.remove('selected'));
        event.currentTarget.classList.add('selected');
        this.selectedFile = file;

        if (!file.is_dir) {
            await this.previewFile(file);
        }
    }

    handleFileDoubleClick(file) {
        if (file.is_dir) {
            this.loadFiles(file.path);
        } else {
            // Could open in editor
            this.previewFile(file);
        }
    }

    async previewFile(file) {
        const container = document.getElementById('previewContainer');
        const content = document.getElementById('previewContent');
        const title = document.getElementById('previewTitle');

        title.textContent = file.name;
        container.classList.add('visible');

        try {
            const response = await fetch(`/api/file?path=${encodeURIComponent(file.path)}`);
            const data = await response.json();

            if (data.is_binary) {
                content.innerHTML = '<div class="preview-placeholder">[Binary file]</div>';
            } else if (file.file_type === 'markdown') {
                content.innerHTML = marked.parse(data.content);
            } else {
                content.innerHTML = `<pre class="code-preview">${this.escapeHtml(data.content)}</pre>`;
            }
        } catch (e) {
            content.innerHTML = `<div class="preview-placeholder">Failed to load file</div>`;
        }

        // Resize terminal to fit
        setTimeout(() => this.fitAddon.fit(), 100);
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // Sidebar Resizer
    initResizer() {
        const handle = document.getElementById('resizeHandle');
        const sidebar = document.getElementById('sidebar');
        let isResizing = false;

        handle.addEventListener('mousedown', (e) => {
            isResizing = true;
            document.body.style.cursor = 'col-resize';
            document.body.style.userSelect = 'none';
        });

        document.addEventListener('mousemove', (e) => {
            if (!isResizing) return;

            const newWidth = e.clientX;
            if (newWidth >= 150 && newWidth <= 400) {
                sidebar.style.width = newWidth + 'px';
                this.sidebarWidth = newWidth;
            }
        });

        document.addEventListener('mouseup', () => {
            if (isResizing) {
                isResizing = false;
                document.body.style.cursor = '';
                document.body.style.userSelect = '';
                this.fitAddon.fit();
            }
        });
    }

    // Event Listeners
    initEventListeners() {
        document.getElementById('closePreview').addEventListener('click', () => {
            document.getElementById('previewContainer').classList.remove('visible');
            setTimeout(() => this.fitAddon.fit(), 100);
        });

        document.getElementById('newTerminal').addEventListener('click', () => {
            // For now, just clear and reconnect
            this.terminal.clear();
            if (this.ws) this.ws.close();
            this.initWebSocket();
        });
    }

    // Health Check
    async loadHealth() {
        try {
            const response = await fetch('/api/health');
            const data = await response.json();
            document.getElementById('versionDisplay').textContent = data.version;
            document.getElementById('connectionCount').textContent = `${data.connections} connection${data.connections !== 1 ? 's' : ''}`;
        } catch (e) {
            console.error('Health check failed:', e);
        }
    }
}

// Initialize on DOM ready
document.addEventListener('DOMContentLoaded', () => {
    window.dashboard = new Dashboard();
});
