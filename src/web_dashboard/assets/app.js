// Smart Tree Web Dashboard - Main Application

class Dashboard {
    constructor() {
        this.terminals = [];
        this.activeTerminalId = null;
        this.terminalCounter = 0;
        this.currentPath = null;
        this.selectedFile = null;
        this.sidebarWidth = 250;
        this.previewWidth = 400;
        this.terminalHeight = 300;
        this.layout = 'side'; // 'side' or 'bottom'
        this.debouncedSaveLayout = this.debounce(this.saveLayoutConfig, 500);

        this.init();
    }

    async init() {
        await this.loadThemeConfig(); // Load theme first
        this.initMobile();
        this.initVoice();
        await this.loadLayoutConfig(); // Load layout
        this.createTerminal(); // Create first terminal
        this.initFileBrowser();
        this.initResizers();
        this.initEventListeners();
        this.initKeyboardShortcuts();
        await this.loadHealth();

        // Refresh health periodically
        setInterval(() => this.loadHealth(), 30000);
    }

    // --- Config Persistence ---

    debounce(func, delay) {
        let timeout;
        return (...args) => {
            clearTimeout(timeout);
            timeout = setTimeout(() => func.apply(this, args), delay);
        };
    }
    
    async loadThemeConfig() {
        try {
            const response = await fetch('/api/config/theme');
            if (response.ok) {
                const theme = await response.json();
                for (const [key, value] of Object.entries(theme)) {
                    if (value) {
                        // Convert snake_case to --kebab-case
                        const cssVar = `--${key.replace(/_/g, '-')}`;
                        document.documentElement.style.setProperty(cssVar, value);
                    }
                }
            }
        } catch (e) {
            console.error('Failed to load theme config.', e);
        }
    }

    async loadLayoutConfig() {
        try {
            const response = await fetch('/api/config/layout');
            if (response.ok) {
                const config = await response.json();
                this.sidebarWidth = config.sidebar_width || 250;
                this.terminalHeight = config.terminal_height || 300;
                this.previewWidth = config.preview_width || 400;
                this.setLayout(config.layout_mode || 'side', false);
                
                // Apply loaded sizes
                document.getElementById('sidebar').style.width = `${this.sidebarWidth}px`;
                document.documentElement.style.setProperty('--terminal-height', `${this.terminalHeight}px`);
                const previewContainer = document.getElementById('previewContainer');
                if (previewContainer.classList.contains('visible')) {
                    previewContainer.style.width = `${this.previewWidth}px`;
                }

            } else {
                this.initLayout(); // Fallback to default
            }
        } catch (e) {
            console.error('Failed to load layout config, using defaults.', e);
            this.initLayout(); // Fallback to default
        }
    }

    async saveLayoutConfig() {
        const config = {
            sidebar_width: this.sidebarWidth,
            terminal_height: this.terminalHeight,
            preview_width: this.previewWidth,
            layout_mode: this.layout
        };

        try {
            await fetch('/api/config/layout', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(config)
            });
        } catch (e) {
            console.error('Failed to save layout config.', e);
        }
    }
    
    // Layout Management
    initLayout() {
        // This is now a fallback for when API fails
        this.setLayout('side', false);

        document.getElementById('toggleLayout').addEventListener('click', () => {
            this.toggleLayout();
        });
    }

    setLayout(layout, shouldSave = true) {
        this.layout = layout;
        const dashboard = document.getElementById('dashboard');

        if (layout === 'bottom') {
            dashboard.classList.add('layout-bottom');
        } else {
            dashboard.classList.remove('layout-bottom');
        }

        if (shouldSave) {
            this.saveLayoutConfig();
        }

        // Refit all terminals
        setTimeout(() => {
            this.terminals.forEach(t => t.fitAddon.fit());
        }, 100);
    }

    // Mobile Support
    initMobile() {
        this.isMobile = window.innerWidth <= 768;

        const menuBtn = document.getElementById('mobileMenuBtn');
        const backdrop = document.getElementById('sidebarBackdrop');
        const sidebar = document.getElementById('sidebar');

        menuBtn.addEventListener('click', () => this.toggleMobileSidebar());
        backdrop.addEventListener('click', () => this.closeMobileSidebar());

        // Force bottom layout on mobile
        if (this.isMobile) {
            this.setLayout('bottom');
        }

        // Handle orientation change
        window.addEventListener('resize', () => {
            const wasMobile = this.isMobile;
            this.isMobile = window.innerWidth <= 768;

            if (this.isMobile && !wasMobile) {
                this.setLayout('bottom');
                this.closeMobileSidebar();
            }
        });
    }

    toggleMobileSidebar() {
        const sidebar = document.getElementById('sidebar');
        const backdrop = document.getElementById('sidebarBackdrop');

        if (sidebar.classList.contains('mobile-open')) {
            this.closeMobileSidebar();
        } else {
            sidebar.classList.add('mobile-open');
            sidebar.classList.remove('collapsed');
            backdrop.classList.add('visible');
        }
    }

    closeMobileSidebar() {
        const sidebar = document.getElementById('sidebar');
        const backdrop = document.getElementById('sidebarBackdrop');

        sidebar.classList.remove('mobile-open');
        backdrop.classList.remove('visible');
    }

    // Voice Support (Text-to-Speech)
    initVoice() {
        this.voiceEnabled = false;
        this.speechSynth = window.speechSynthesis;
        this.voiceBtn = document.getElementById('voiceBtn');

        if (!this.speechSynth) {
            this.voiceBtn.style.display = 'none';
            return;
        }

        this.voiceBtn.addEventListener('click', () => this.toggleVoice());

        // Load saved preference
        this.voiceEnabled = localStorage.getItem('st-voice') === 'true';
        this.updateVoiceButton();
    }

    toggleVoice() {
        this.voiceEnabled = !this.voiceEnabled;
        localStorage.setItem('st-voice', this.voiceEnabled);
        this.updateVoiceButton();

        if (this.voiceEnabled) {
            this.speak('Voice output enabled');
        } else {
            this.speechSynth.cancel();
        }
    }

    updateVoiceButton() {
        if (this.voiceEnabled) {
            this.voiceBtn.classList.add('speaking');
            this.voiceBtn.title = 'Voice output ON (click to disable)';
        } else {
            this.voiceBtn.classList.remove('speaking');
            this.voiceBtn.title = 'Voice output OFF (click to enable)';
        }
    }

    speak(text) {
        if (!this.voiceEnabled || !this.speechSynth) return;

        // Cancel any ongoing speech
        this.speechSynth.cancel();

        const utterance = new SpeechSynthesisUtterance(text);
        utterance.rate = 1.0;
        utterance.pitch = 1.0;
        utterance.volume = 0.8;

        // Try to use a nice voice
        const voices = this.speechSynth.getVoices();
        const preferredVoice = voices.find(v =>
            v.name.includes('Google') || v.name.includes('Samantha') || v.lang.startsWith('en')
        );
        if (preferredVoice) {
            utterance.voice = preferredVoice;
        }

        this.speechSynth.speak(utterance);
    }

    // Voice output buffer and processing
    processVoiceOutput(text) {
        if (!this.voiceEnabled) return;

        // Initialize buffer if needed
        if (!this.voiceBuffer) {
            this.voiceBuffer = '';
            this.voiceTimeout = null;
            this.lastSpokenTime = 0;
        }

        // Clean ANSI codes
        const cleaned = text
            .replace(/\x1b\[[0-9;]*[a-zA-Z]/g, '') // Remove ANSI escape sequences
            .replace(/\x1b\[\?[0-9;]*[a-zA-Z]/g, '') // Remove cursor codes
            .replace(/\x07/g, '') // Remove bell
            .replace(/[\x00-\x1f]/g, (c) => c === '\n' || c === '\r' ? c : ''); // Keep newlines

        this.voiceBuffer += cleaned;

        // Debounce - wait for pause in output
        if (this.voiceTimeout) {
            clearTimeout(this.voiceTimeout);
        }

        this.voiceTimeout = setTimeout(() => {
            this.speakBuffer();
        }, 800); // Wait 800ms after last output
    }

    speakBuffer() {
        if (!this.voiceBuffer || !this.voiceEnabled) return;

        // Clean up the buffer
        let text = this.voiceBuffer
            .replace(/\r\n|\r|\n/g, ' ')
            .replace(/\s+/g, ' ')
            .trim();

        // Skip prompts and short outputs
        const skipPatterns = [
            /^\$\s*$/,           // Empty prompt
            /^[a-zA-Z0-9_-]+@[a-zA-Z0-9_-]+/,  // SSH-style prompts
            /^[~\/][^\s]*\$\s*$/, // Path prompts
            /^\s*$/,             // Whitespace only
        ];

        if (skipPatterns.some(p => p.test(text))) {
            this.voiceBuffer = '';
            return;
        }

        // Only speak substantial content
        if (text.length > 15 && text.length < 2000) {
            // Rate limit - don't speak too frequently
            const now = Date.now();
            if (now - this.lastSpokenTime > 2000) {
                // Truncate very long text
                if (text.length > 500) {
                    text = text.substring(0, 500) + '...';
                }
                this.speak(text);
                this.lastSpokenTime = now;
            }
        }

        this.voiceBuffer = '';
    }

    // Speak terminal output (direct call)
    speakTerminalOutput(text) {
        if (!this.voiceEnabled) return;

        // Extract meaningful content (skip escape codes, prompts)
        const cleaned = text
            .replace(/\x1b\[[0-9;]*m/g, '') // Remove ANSI codes
            .replace(/\r\n|\r|\n/g, ' ')
            .trim();

        if (cleaned.length > 10 && cleaned.length < 500) {
            this.speak(cleaned);
        }
    }

    // Layout Management
    initLayout() {
        // Load saved layout preference
        const savedLayout = localStorage.getItem('st-layout') || 'side';
        this.setLayout(savedLayout);

        document.getElementById('toggleLayout').addEventListener('click', () => {
            this.toggleLayout();
        });
    }

    setLayout(layout) {
        this.layout = layout;
        const dashboard = document.getElementById('dashboard');

        if (layout === 'bottom') {
            dashboard.classList.add('layout-bottom');
        } else {
            dashboard.classList.remove('layout-bottom');
        }

        localStorage.setItem('st-layout', layout);

        // Refit all terminals
        setTimeout(() => {
            this.terminals.forEach(t => t.fitAddon.fit());
        }, 100);
    }

    toggleLayout() {
        this.setLayout(this.layout === 'side' ? 'bottom' : 'side');
    }

    // Terminal Management
    createTerminal() {
        const id = ++this.terminalCounter;
        const name = `Terminal ${id}`;

        // Create terminal instance
        const terminal = new Terminal({
            cursorBlink: true,
            cursorStyle: 'block',
            fontSize: 14,
            fontFamily: "'IBM Plex Mono', 'JetBrains Mono', 'Fira Code', monospace",
            theme: {
                background: '#0f1815',
                foreground: '#e8f2ec',
                cursor: '#e8f2ec',
                cursorAccent: '#0f1815',
                selectionBackground: 'rgba(31, 122, 110, 0.35)',
                black: '#0f1815',
                red: '#d76c6c',
                green: '#4fb38c',
                yellow: '#e0a84b',
                blue: '#5f80bf',
                magenta: '#b06a2e',
                cyan: '#3bb3a0',
                white: '#f5efe6',
                brightBlack: '#475046',
                brightRed: '#e48f8f',
                brightGreen: '#6dd3a8',
                brightYellow: '#f0c075',
                brightBlue: '#7c9bd6',
                brightMagenta: '#c88a52',
                brightCyan: '#5fd6c2',
                brightWhite: '#ffffff'
            },
            allowTransparency: true,
            scrollback: 10000
        });

        const fitAddon = new FitAddon.FitAddon();
        terminal.loadAddon(fitAddon);

        // Create container
        const container = document.createElement('div');
        container.className = 'terminal-instance';
        container.id = `terminal-${id}`;
        document.getElementById('terminalsWrapper').appendChild(container);

        terminal.open(container);

        // Create tab
        const tab = document.createElement('div');
        tab.className = 'terminal-tab';
        tab.dataset.id = id;
        tab.innerHTML = `
            <span class="tab-title">${name}</span>
            <span class="tab-close" title="Close">&times;</span>
        `;
        document.getElementById('terminalTabs').appendChild(tab);

        // Tab click handlers
        tab.addEventListener('click', (e) => {
            if (!e.target.classList.contains('tab-close')) {
                this.activateTerminal(id);
            }
        });

        tab.querySelector('.tab-close').addEventListener('click', (e) => {
            e.stopPropagation();
            this.closeTerminal(id);
        });

        // WebSocket connection
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws/terminal`;
        const ws = new WebSocket(wsUrl);

        ws.onopen = () => {
            this.updateConnectionStatus(true);
            const { cols, rows } = terminal;
            ws.send(JSON.stringify({ type: 'resize', cols, rows }));
        };

        ws.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data);
                switch (msg.type) {
                    case 'output':
                        terminal.write(msg.data);
                        // Voice output for significant content
                        this.processVoiceOutput(msg.data);
                        break;
                    case 'system':
                        terminal.write(`\r\n\x1b[36m[System: ${msg.message}]\x1b[0m\r\n`);
                        if (this.voiceEnabled) {
                            this.speak(`System: ${msg.message}`);
                        }
                        break;
                    case 'exit':
                        terminal.write(`\r\n[Process exited with code ${msg.code}]\r\n`);
                        if (this.voiceEnabled) {
                            this.speak(`Process exited with code ${msg.code}`);
                        }
                        break;
                    case 'error':
                        terminal.write(`\r\n\x1b[31m[Error: ${msg.message}]\x1b[0m\r\n`);
                        if (this.voiceEnabled) {
                            this.speak(`Error: ${msg.message}`);
                        }
                        break;
                }
            } catch (e) {
                console.error('Failed to parse message:', e);
            }
        };

        ws.onclose = () => {
            this.updateConnectionStatus(false);
            terminal.write('\r\n\x1b[33m[Disconnected]\x1b[0m\r\n');
        };

        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
        };

        // Terminal input
        terminal.onData(data => {
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(JSON.stringify({ type: 'input', data }));
            }
        });

        // Terminal resize
        terminal.onResize(({ cols, rows }) => {
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(JSON.stringify({ type: 'resize', cols, rows }));
            }
        });

        // Store terminal info
        const terminalInfo = { id, name, terminal, fitAddon, ws, tab, container };
        this.terminals.push(terminalInfo);

        // Activate this terminal
        this.activateTerminal(id);

        // Fit after a short delay
        setTimeout(() => fitAddon.fit(), 50);

        return terminalInfo;
    }

    activateTerminal(id) {
        // Deactivate all
        this.terminals.forEach(t => {
            t.tab.classList.remove('active');
            t.container.classList.remove('active');
        });

        // Activate selected
        const terminalInfo = this.terminals.find(t => t.id === id);
        if (terminalInfo) {
            terminalInfo.tab.classList.add('active');
            terminalInfo.container.classList.add('active');
            this.activeTerminalId = id;
            terminalInfo.terminal.focus();
            terminalInfo.fitAddon.fit();
        }
    }

    closeTerminal(id) {
        const index = this.terminals.findIndex(t => t.id === id);
        if (index === -1) return;

        const terminalInfo = this.terminals[index];

        // Close WebSocket
        if (terminalInfo.ws) {
            terminalInfo.ws.close();
        }

        // Remove DOM elements
        terminalInfo.tab.remove();
        terminalInfo.container.remove();

        // Remove from array
        this.terminals.splice(index, 1);

        // If this was active, activate another
        if (this.activeTerminalId === id && this.terminals.length > 0) {
            this.activateTerminal(this.terminals[0].id);
        }

        // If no terminals left, create a new one
        if (this.terminals.length === 0) {
            this.createTerminal();
        }
    }

    getActiveTerminal() {
        return this.terminals.find(t => t.id === this.activeTerminalId);
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

        // File search/filter
        const searchInput = document.getElementById('fileSearchInput');
        searchInput.addEventListener('input', (e) => this.filterFiles(e.target.value));
        searchInput.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                searchInput.value = '';
                this.filterFiles('');
                const active = this.getActiveTerminal();
                if (active) active.terminal.focus();
            }
        });
    }

    filterFiles(query) {
        const items = document.querySelectorAll('.file-item');
        const lowerQuery = query.toLowerCase();

        items.forEach(item => {
            const name = item.querySelector('.file-name').textContent.toLowerCase();
            if (!query || name.includes(lowerQuery)) {
                item.classList.remove('hidden');
            } else {
                item.classList.add('hidden');
            }
        });
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

        item.addEventListener('click', (e) => this.handleFileClick(file, e));
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

    handleFileClick(file, e) {
        // Update selection
        document.querySelectorAll('.file-item.selected').forEach(el => el.classList.remove('selected'));
        e.currentTarget.classList.add('selected');
        this.selectedFile = file;

        if (!file.is_dir) {
            this.previewFile(file);
        }
    }

    handleFileDoubleClick(file) {
        if (file.is_dir) {
            this.loadFiles(file.path);
        } else {
            this.previewFile(file);
        }
    }

    async previewFile(file) {
        const container = document.getElementById('previewContainer');
        const content = document.getElementById('previewContent');
        const title = document.getElementById('previewTitle');

        title.textContent = file.name;
        container.classList.add('visible');
        this.showPreviewHandle(true);

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

        // Resize terminals to fit
        setTimeout(() => {
            this.terminals.forEach(t => t.fitAddon.fit());
        }, 100);
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }

    // Resizers
    initResizers() {
        // Sidebar resizer
        this.initSidebarResizer();

        // Terminal resizer (for bottom layout)
        this.initTerminalResizer();

        // Preview resizer
        this.initPreviewResizer();

        // Window resize
        window.addEventListener('resize', () => {
            this.terminals.forEach(t => t.fitAddon.fit());
        });
    }

    initSidebarResizer() {
        const handle = document.getElementById('resizeHandle');
        const sidebar = document.getElementById('sidebar');
        let isResizing = false;

        const startResize = () => {
            isResizing = true;
            document.body.style.cursor = 'col-resize';
            document.body.style.userSelect = 'none';
        };

        const doResize = (clientX) => {
            if (!isResizing) return;
            const newWidth = clientX;
            if (newWidth >= 150 && newWidth <= 500) {
                sidebar.style.width = newWidth + 'px';
                this.sidebarWidth = newWidth;
            }
        };

        const endResize = () => {
            if (isResizing) {
                isResizing = false;
                document.body.style.cursor = '';
                document.body.style.userSelect = '';
                this.terminals.forEach(t => t.fitAddon.fit());
                this.debouncedSaveLayout();
            }
        };

        // Mouse events
        handle.addEventListener('mousedown', startResize);
        document.addEventListener('mousemove', (e) => doResize(e.clientX));
        document.addEventListener('mouseup', endResize);

        // Touch events
        handle.addEventListener('touchstart', (e) => {
            e.preventDefault();
            startResize();
        });
        document.addEventListener('touchmove', (e) => {
            if (isResizing) doResize(e.touches[0].clientX);
        });
        document.addEventListener('touchend', endResize);
    }

    initTerminalResizer() {
        const handle = document.getElementById('terminalResizeHandle');
        const terminalContainer = document.getElementById('terminalContainer');
        let isResizing = false;
        let startY = 0;
        let startHeight = 0;

        const startResize = (clientY) => {
            isResizing = true;
            startY = clientY;
            startHeight = terminalContainer.offsetHeight;
            document.body.style.cursor = 'row-resize';
            document.body.style.userSelect = 'none';
        };

        const doResize = (clientY) => {
            if (!isResizing) return;
            // Terminal at bottom, drag up = taller
            const delta = startY - clientY;
            const newHeight = Math.max(100, Math.min(startHeight + delta, window.innerHeight * 0.8));
            terminalContainer.style.height = newHeight + 'px';
            document.documentElement.style.setProperty('--terminal-height', newHeight + 'px');
            this.terminalHeight = newHeight;
        };

        const endResize = () => {
            if (isResizing) {
                isResizing = false;
                document.body.style.cursor = '';
                document.body.style.userSelect = '';
                this.terminals.forEach(t => t.fitAddon.fit());
                this.debouncedSaveLayout();
            }
        };

        // Mouse events
        handle.addEventListener('mousedown', (e) => startResize(e.clientY));
        document.addEventListener('mousemove', (e) => doResize(e.clientY));
        document.addEventListener('mouseup', endResize);

        // Touch events
        handle.addEventListener('touchstart', (e) => {
            e.preventDefault();
            startResize(e.touches[0].clientY);
        });
        document.addEventListener('touchmove', (e) => {
            if (isResizing) doResize(e.touches[0].clientY);
        });
        document.addEventListener('touchend', endResize);
    }

    initPreviewResizer() {
        const handle = document.getElementById('previewResizeHandle');
        const preview = document.getElementById('previewContainer');
        let isResizing = false;
        let startX = 0;
        let startWidth = 0;

        const startResize = (clientX) => {
            isResizing = true;
            startX = clientX;
            startWidth = preview.offsetWidth;
            document.body.style.cursor = 'col-resize';
            document.body.style.userSelect = 'none';
        };

        const doResize = (clientX) => {
            if (!isResizing) return;
            // Handle is on LEFT of preview, preview is on RIGHT
            // Drag left (negative delta) = preview gets wider
            // Drag right (positive delta) = preview gets smaller
            const delta = clientX - startX;
            const newWidth = Math.max(200, Math.min(startWidth - delta, window.innerWidth * 0.6));
            preview.style.width = newWidth + 'px';
            this.previewWidth = newWidth;
        };

        const endResize = () => {
            if (isResizing) {
                isResizing = false;
                document.body.style.cursor = '';
                document.body.style.userSelect = '';
                this.terminals.forEach(t => t.fitAddon.fit());
                this.debouncedSaveLayout();
            }
        };

        // Mouse events
        handle.addEventListener('mousedown', (e) => startResize(e.clientX));
        document.addEventListener('mousemove', (e) => doResize(e.clientX));
        document.addEventListener('mouseup', endResize);

        // Touch events
        handle.addEventListener('touchstart', (e) => {
            e.preventDefault();
            startResize(e.touches[0].clientX);
        });
        document.addEventListener('touchmove', (e) => {
            if (isResizing) doResize(e.touches[0].clientX);
        });
        document.addEventListener('touchend', endResize);
    }

    // Show/hide preview resize handle
    showPreviewHandle(show) {
        const handle = document.getElementById('previewResizeHandle');
        if (show) {
            handle.classList.add('visible');
        } else {
            handle.classList.remove('visible');
        }
    }

    // Event Listeners
    initEventListeners() {
        document.getElementById('closePreview').addEventListener('click', () => {
            document.getElementById('previewContainer').classList.remove('visible');
            this.showPreviewHandle(false);
            setTimeout(() => {
                this.terminals.forEach(t => t.fitAddon.fit());
            }, 100);
        });

        document.getElementById('newTerminal').addEventListener('click', () => {
            this.createTerminal();
        });

        // Quick action buttons
        this.initQuickActions();
    }

    // Quick Action Buttons
    initQuickActions() {
        document.getElementById('btnClaudeContinue').addEventListener('click', () => {
            this.runCommand('claude --dangerously-skip-permissions -c');
        });

        document.getElementById('btnClaudeNew').addEventListener('click', () => {
            this.runCommand('claude --dangerously-skip-permissions');
        });

        document.getElementById('btnST').addEventListener('click', () => {
            this.runCommand('st -m ai .');
        });
    }

    // Send command to active terminal
    runCommand(command) {
        const active = this.getActiveTerminal();
        if (active && active.ws && active.ws.readyState === WebSocket.OPEN) {
            // Send the command with a newline
            active.ws.send(JSON.stringify({ type: 'input', data: command + '\n' }));
            active.terminal.focus();
        }
    }

    // Keyboard Shortcuts
    initKeyboardShortcuts() {
        document.addEventListener('keydown', (e) => {
            // Ctrl+B: Toggle sidebar
            if (e.ctrlKey && e.key === 'b') {
                e.preventDefault();
                this.toggleSidebar();
            }
            // Ctrl+J: Toggle layout
            if (e.ctrlKey && e.key === 'j') {
                e.preventDefault();
                this.toggleLayout();
            }
            // Ctrl+`: Focus active terminal
            if (e.ctrlKey && e.key === '`') {
                e.preventDefault();
                const active = this.getActiveTerminal();
                if (active) active.terminal.focus();
            }
            // Ctrl+Shift+`: New terminal
            if (e.ctrlKey && e.shiftKey && e.key === '`') {
                e.preventDefault();
                this.createTerminal();
            }
            // Escape: Close preview
            if (e.key === 'Escape') {
                const preview = document.getElementById('previewContainer');
                if (preview.classList.contains('visible')) {
                    preview.classList.remove('visible');
                    this.showPreviewHandle(false);
                    setTimeout(() => {
                        this.terminals.forEach(t => t.fitAddon.fit());
                    }, 100);
                }
            }
            // Ctrl+P: Quick file search
            if (e.ctrlKey && e.key === 'p') {
                e.preventDefault();
                this.focusFileSearch();
            }
            // Ctrl+W: Close terminal tab
            if (e.ctrlKey && e.key === 'w') {
                e.preventDefault();
                if (this.activeTerminalId) {
                    this.closeTerminal(this.activeTerminalId);
                }
            }
            // Ctrl+Tab: Next terminal
            if (e.ctrlKey && e.key === 'Tab') {
                e.preventDefault();
                this.nextTerminal(e.shiftKey ? -1 : 1);
            }
        });
    }

    nextTerminal(direction) {
        if (this.terminals.length <= 1) return;

        const currentIndex = this.terminals.findIndex(t => t.id === this.activeTerminalId);
        let nextIndex = currentIndex + direction;

        if (nextIndex < 0) nextIndex = this.terminals.length - 1;
        if (nextIndex >= this.terminals.length) nextIndex = 0;

        this.activateTerminal(this.terminals[nextIndex].id);
    }

    toggleSidebar() {
        const sidebar = document.getElementById('sidebar');
        const handle = document.getElementById('resizeHandle');

        if (sidebar.classList.contains('collapsed')) {
            sidebar.classList.remove('collapsed');
            sidebar.style.width = this.sidebarWidth + 'px';
            handle.style.display = '';
        } else {
            sidebar.classList.add('collapsed');
            sidebar.style.width = '0';
            handle.style.display = 'none';
        }
        setTimeout(() => {
            this.terminals.forEach(t => t.fitAddon.fit());
        }, 200);
    }

    focusFileSearch() {
        const sidebar = document.getElementById('sidebar');
        // Ensure sidebar is visible
        if (sidebar.classList.contains('collapsed')) {
            this.toggleSidebar();
        }
        // Focus the search input
        document.getElementById('fileSearchInput').focus();
    }

    // Health Check
    async loadHealth() {
        try {
            const response = await fetch('/api/health');
            const data = await response.json();
            document.getElementById('versionDisplay').textContent = data.version;
            document.getElementById('connectionCount').textContent = `${data.connections} connection${data.connections !== 1 ? 's' : ''}`;

            // Update git branch
            const gitBranch = document.getElementById('gitBranch');
            if (data.git_branch) {
                gitBranch.textContent = data.git_branch;
                gitBranch.title = `Git branch: ${data.git_branch}`;
            } else {
                gitBranch.textContent = '';
            }
        } catch (e) {
            console.error('Health check failed:', e);
        }
    }
}

// ============================================================================
// Wave Compass - Real-time MCP Activity Visualization
// ============================================================================

class WaveCompass {
    constructor(canvas, onHint) {
        this.canvas = canvas;
        this.ctx = canvas.getContext('2d');
        this.onHint = onHint;
        this.hotRegions = new Map(); // path -> {x, y, intensity, label}
        this.trail = []; // [{x, y, age}]
        this.animationId = null;

        // Setup canvas sizing
        this.resize();
        window.addEventListener('resize', () => this.resize());

        // Click handling
        canvas.addEventListener('click', (e) => this.handleClick(e));

        // Start animation loop
        this.animate();
    }

    resize() {
        const rect = this.canvas.parentElement.getBoundingClientRect();
        const dpr = window.devicePixelRatio || 1;
        this.canvas.width = rect.width * dpr;
        this.canvas.height = rect.height * dpr;
        this.ctx.scale(dpr, dpr);
        this.width = rect.width;
        this.height = rect.height;
    }

    // Update from state_update message
    update(data) {
        // Update hot regions from wave_compass data
        if (data.wave_compass) {
            // Clear old regions with decay
            for (const [path, region] of this.hotRegions) {
                region.intensity *= 0.95;
                if (region.intensity < 0.05) {
                    this.hotRegions.delete(path);
                }
            }

            // Add new hot regions
            for (const region of data.wave_compass.hot_regions || []) {
                const key = region.label;
                const existing = this.hotRegions.get(key);
                if (existing) {
                    existing.intensity = Math.min(1.0, existing.intensity + region.intensity * 0.5);
                } else {
                    this.hotRegions.set(key, {
                        x: region.x,
                        y: region.y,
                        intensity: region.intensity,
                        label: region.label
                    });
                }
            }

            // Update trail
            if (data.wave_compass.trail) {
                for (const point of data.wave_compass.trail) {
                    this.trail.push({ x: point[0], y: point[1], age: 0 });
                }
                // Keep trail bounded
                while (this.trail.length > 50) {
                    this.trail.shift();
                }
            }
        }

        // Age the trail
        for (const point of this.trail) {
            point.age += 16; // ~60fps
        }
        this.trail = this.trail.filter(p => p.age < 5000);
    }

    animate() {
        this.render();
        this.animationId = requestAnimationFrame(() => this.animate());
    }

    render() {
        const { ctx, width, height } = this;

        // Clear with dark background
        ctx.fillStyle = 'rgba(15, 24, 21, 0.95)';
        ctx.fillRect(0, 0, width, height);

        // Draw subtle grid
        this.drawGrid();

        // Draw exploration trail
        this.drawTrail();

        // Draw hot regions
        for (const [path, region] of this.hotRegions) {
            this.drawGlowingRegion(region);
        }

        // Draw labels for high-intensity regions
        this.drawLabels();

        // Draw title
        ctx.fillStyle = '#1f7a6e66';
        ctx.font = '10px "IBM Plex Mono", monospace';
        ctx.fillText('WAVE COMPASS', 8, 14);
    }

    drawGrid() {
        const { ctx, width, height } = this;
        ctx.strokeStyle = 'rgba(31, 122, 110, 0.15)';
        ctx.lineWidth = 0.5;

        // Vertical lines
        for (let x = 0; x <= width; x += 40) {
            ctx.beginPath();
            ctx.moveTo(x, 0);
            ctx.lineTo(x, height);
            ctx.stroke();
        }

        // Horizontal lines
        for (let y = 0; y <= height; y += 40) {
            ctx.beginPath();
            ctx.moveTo(0, y);
            ctx.lineTo(width, y);
            ctx.stroke();
        }

        // Quadrant labels
        ctx.fillStyle = 'rgba(176, 106, 46, 0.25)';
        ctx.font = '9px "IBM Plex Mono", monospace';
        ctx.fillText('src/', 10, height * 0.15);
        ctx.fillText('tests/', width * 0.55, height * 0.15);
        ctx.fillText('docs/', 10, height * 0.65);
        ctx.fillText('scripts/', width * 0.55, height * 0.65);
    }

    drawTrail() {
        const { ctx, width, height, trail } = this;
        if (trail.length < 2) return;

        ctx.beginPath();
        ctx.moveTo(trail[0].x * width, trail[0].y * height);

        for (let i = 1; i < trail.length; i++) {
            const point = trail[i];
            const alpha = Math.max(0, 1 - point.age / 5000);
            ctx.strokeStyle = `rgba(59, 179, 160, ${alpha * 0.5})`;
            ctx.lineWidth = 2 * alpha;
            ctx.lineTo(point.x * width, point.y * height);
        }
        ctx.stroke();
    }

    drawGlowingRegion(region) {
        const { ctx, width, height } = this;
        const x = region.x * width;
        const y = region.y * height;
        const intensity = region.intensity;

        // Pulsing effect
        const pulse = Math.sin(Date.now() / 300) * 0.2 + 0.8;
        const radius = 15 + intensity * 25 * pulse;

        // Outer glow
        const gradient = ctx.createRadialGradient(x, y, 0, x, y, radius);
        gradient.addColorStop(0, `rgba(59, 179, 160, ${intensity * 0.8})`);
        gradient.addColorStop(0.5, `rgba(31, 122, 110, ${intensity * 0.4})`);
        gradient.addColorStop(1, 'rgba(31, 122, 110, 0)');

        ctx.fillStyle = gradient;
        ctx.beginPath();
        ctx.arc(x, y, radius, 0, Math.PI * 2);
        ctx.fill();

        // Core
        ctx.fillStyle = `rgba(121, 219, 195, ${intensity})`;
        ctx.beginPath();
        ctx.arc(x, y, 5 + intensity * 5, 0, Math.PI * 2);
        ctx.fill();
    }

    drawLabels() {
        const { ctx, width, height } = this;
        ctx.font = '10px "IBM Plex Mono", monospace';
        ctx.textAlign = 'center';

        for (const [path, region] of this.hotRegions) {
            if (region.intensity > 0.3) {
                const x = region.x * width;
                const y = region.y * height + 25;

                ctx.fillStyle = `rgba(176, 106, 46, ${region.intensity})`;
                ctx.fillText(region.label, x, y);
            }
        }
        ctx.textAlign = 'left';
    }

    handleClick(event) {
        const rect = this.canvas.getBoundingClientRect();
        const x = (event.clientX - rect.left) / this.width;
        const y = (event.clientY - rect.top) / this.height;

        // Find closest hot region
        let closest = null;
        let minDist = Infinity;

        for (const [path, region] of this.hotRegions) {
            const dist = Math.hypot(x - region.x, y - region.y);
            if (dist < 0.1 && dist < minDist) {
                closest = path;
                minDist = dist;
            }
        }

        if (closest && this.onHint) {
            this.onHint({ type: 'click', target: closest });
        }
    }

    destroy() {
        if (this.animationId) {
            cancelAnimationFrame(this.animationId);
        }
    }
}

// ============================================================================
// State Sync - WebSocket connection for real-time MCP activity
// ============================================================================

class StateSync {
    constructor(onUpdate, onHint) {
        this.onUpdate = onUpdate;
        this.onHint = onHint;
        this.ws = null;
        this.reconnectDelay = 1000;
        this.maxReconnectDelay = 30000;
        this.connected = false;

        this.connect();
    }

    connect() {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws/state`;

        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
            console.log('[StateSync] Connected');
            this.connected = true;
            this.reconnectDelay = 1000; // Reset on successful connect
        };

        this.ws.onmessage = (event) => {
            try {
                const data = JSON.parse(event.data);
                if (data.type === 'state_update' && this.onUpdate) {
                    this.onUpdate(data);
                }
            } catch (e) {
                console.error('[StateSync] Failed to parse message:', e);
            }
        };

        this.ws.onclose = () => {
            console.log('[StateSync] Disconnected, reconnecting...');
            this.connected = false;
            setTimeout(() => this.connect(), this.reconnectDelay);
            this.reconnectDelay = Math.min(this.reconnectDelay * 2, this.maxReconnectDelay);
        };

        this.ws.onerror = (error) => {
            console.error('[StateSync] Error:', error);
        };
    }

    sendHint(hint) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            const message = {
                type: 'hint',
                hint_type: hint.type,
                target: hint.target || null,
                content: hint.content || null,
                transcript: hint.transcript || null,
                salience: hint.salience || null
            };
            this.ws.send(JSON.stringify(message));
            console.log('[StateSync] Sent hint:', message);
        }
    }

    destroy() {
        if (this.ws) {
            this.ws.close();
        }
    }
}

// ============================================================================
// Hint Input - Text input for sending hints to AI
// ============================================================================

class HintInput {
    constructor(container, onSend) {
        this.onSend = onSend;
        this.element = this.createUI(container);
    }

    createUI(container) {
        const wrapper = document.createElement('div');
        wrapper.className = 'hint-input-wrapper';
        wrapper.innerHTML = `
            <input type="text" class="hint-input" placeholder="Type a hint for the AI..." />
            <button class="hint-send-btn" title="Send hint">→</button>
        `;

        const input = wrapper.querySelector('.hint-input');
        const button = wrapper.querySelector('.hint-send-btn');

        input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && input.value.trim()) {
                this.send(input.value.trim());
                input.value = '';
            }
        });

        button.addEventListener('click', () => {
            if (input.value.trim()) {
                this.send(input.value.trim());
                input.value = '';
            }
        });

        container.appendChild(wrapper);
        return wrapper;
    }

    send(content) {
        if (this.onSend) {
            this.onSend({ type: 'text', content });
        }
    }
}

// ============================================================================
// Voice Input - Push-to-talk voice recording for hints
// ============================================================================

class VoiceInput {
    constructor(container, onHint) {
        this.onHint = onHint;
        this.mediaRecorder = null;
        this.audioChunks = [];
        this.isRecording = false;
        this.element = this.createUI(container);
        this.checkMicrophoneSupport();
    }

    createUI(container) {
        const wrapper = document.createElement('div');
        wrapper.className = 'voice-input-wrapper';
        wrapper.innerHTML = `
            <button class="voice-record-btn" title="Hold to record voice hint">
                <span class="voice-icon">&#x1F3A4;</span>
                <span class="voice-label">Hold to speak</span>
            </button>
            <div class="voice-status"></div>
        `;

        const btn = wrapper.querySelector('.voice-record-btn');
        const status = wrapper.querySelector('.voice-status');

        // Mouse events (desktop)
        btn.addEventListener('mousedown', (e) => {
            e.preventDefault();
            this.startRecording();
        });
        btn.addEventListener('mouseup', () => this.stopRecording());
        btn.addEventListener('mouseleave', () => {
            if (this.isRecording) this.stopRecording();
        });

        // Touch events (mobile)
        btn.addEventListener('touchstart', (e) => {
            e.preventDefault();
            this.startRecording();
        });
        btn.addEventListener('touchend', () => this.stopRecording());
        btn.addEventListener('touchcancel', () => {
            if (this.isRecording) this.stopRecording();
        });

        this.btn = btn;
        this.status = status;

        container.appendChild(wrapper);
        return wrapper;
    }

    async checkMicrophoneSupport() {
        if (!navigator.mediaDevices || !navigator.mediaDevices.getUserMedia) {
            this.btn.disabled = true;
            this.status.textContent = 'Microphone not supported';
            this.btn.title = 'Microphone not supported in this browser';
        }
    }

    async startRecording() {
        if (this.isRecording) return;

        try {
            const stream = await navigator.mediaDevices.getUserMedia({ audio: true });

            // Use webm-opus for good quality and size
            const mimeType = MediaRecorder.isTypeSupported('audio/webm;codecs=opus')
                ? 'audio/webm;codecs=opus'
                : 'audio/webm';

            this.mediaRecorder = new MediaRecorder(stream, { mimeType });
            this.audioChunks = [];

            this.mediaRecorder.ondataavailable = (e) => {
                if (e.data.size > 0) {
                    this.audioChunks.push(e.data);
                }
            };

            this.mediaRecorder.onstop = async () => {
                stream.getTracks().forEach(track => track.stop());
                if (this.audioChunks.length > 0) {
                    await this.processRecording();
                }
            };

            this.mediaRecorder.start(100); // Collect data every 100ms
            this.isRecording = true;
            this.btn.classList.add('recording');
            this.status.textContent = 'Recording...';
        } catch (err) {
            console.error('[VoiceInput] Failed to start recording:', err);
            this.status.textContent = 'Microphone access denied';
        }
    }

    stopRecording() {
        if (!this.isRecording || !this.mediaRecorder) return;

        this.mediaRecorder.stop();
        this.isRecording = false;
        this.btn.classList.remove('recording');
        this.status.textContent = 'Processing...';
    }

    async processRecording() {
        const audioBlob = new Blob(this.audioChunks, { type: 'audio/webm' });

        // Check minimum recording length (~0.5 seconds)
        if (audioBlob.size < 5000) {
            this.status.textContent = 'Recording too short';
            return;
        }

        try {
            const formData = new FormData();
            formData.append('audio', audioBlob, 'voice.webm');

            const response = await fetch('/api/voice/transcribe', {
                method: 'POST',
                body: formData
            });

            if (response.ok) {
                const result = await response.json();
                this.status.textContent = result.text || 'Transcribed';

                // Send as voice hint
                if (this.onHint && result.text) {
                    this.onHint({
                        type: 'voice',
                        transcript: result.text,
                        salience: result.salience || 0.5,
                        speaker: result.speaker || null
                    });
                }

                // Clear status after a delay
                setTimeout(() => {
                    this.status.textContent = '';
                }, 3000);
            } else {
                const error = await response.text();
                console.error('[VoiceInput] Transcription failed:', error);
                this.status.textContent = 'Voice not available';
            }
        } catch (err) {
            console.error('[VoiceInput] Request failed:', err);
            this.status.textContent = 'Connection error';
        }
    }
}

// ============================================================================
// MCP Activity Panel - Shows current tool and operation status
// ============================================================================

class McpActivityPanel {
    constructor(container) {
        this.container = container;
        this.element = this.createUI();
    }

    createUI() {
        const panel = document.createElement('div');
        panel.className = 'mcp-activity-panel';
        panel.innerHTML = `
            <div class="mcp-status">
                <span class="mcp-status-dot"></span>
                <span class="mcp-status-text">Ready</span>
            </div>
            <div class="mcp-operation"></div>
            <div class="mcp-stats">
                <span class="mcp-tools-count">0 tools</span>
                <span class="mcp-hints-count">0 hints</span>
            </div>
        `;
        this.container.appendChild(panel);
        return panel;
    }

    update(data) {
        const dot = this.element.querySelector('.mcp-status-dot');
        const text = this.element.querySelector('.mcp-status-text');
        const operation = this.element.querySelector('.mcp-operation');
        const toolsCount = this.element.querySelector('.mcp-tools-count');
        const hintsCount = this.element.querySelector('.mcp-hints-count');

        if (data.mcp) {
            if (data.mcp.active_tool) {
                dot.classList.add('active');
                text.textContent = data.mcp.active_tool;
            } else {
                dot.classList.remove('active');
                text.textContent = 'Ready';
            }
            operation.textContent = data.mcp.current_operation || '';
            toolsCount.textContent = `${data.mcp.tools_executed || 0} tools`;
        }

        hintsCount.textContent = `${data.hints_pending || 0} hints`;
    }
}

// ============================================================================
// Collaboration Panel - Real-time collaboration lounge
// ============================================================================

class CollabPanel {
    constructor() {
        this.section = document.getElementById('collabSection');
        if (!this.section) return;

        this.ws = null;
        this.connected = false;
        this.participantId = null;
        this.hotTub = false;
        this.presence = new Map();

        this.cacheElements();
        this.bindEvents();
        this.restoreInputs();
        this.updateVisibility(false);
    }

    cacheElements() {
        this.toggleBtn = document.getElementById('collabToggle');
        this.body = document.getElementById('collabBody');
        this.statusEl = document.getElementById('collabStatus');
        this.joinEl = document.getElementById('collabJoin');
        this.liveEl = document.getElementById('collabLive');
        this.onboarding = document.getElementById('collabOnboarding');
        this.stepShare = this.onboarding?.querySelector('[data-step="share"]');
        this.stepJoin = this.onboarding?.querySelector('[data-step="join"]');
        this.stepStatus = this.onboarding?.querySelector('[data-step="status"]');

        this.nameInput = document.getElementById('collabName');
        this.typeSelect = document.getElementById('collabType');
        this.joinBtn = document.getElementById('collabJoinBtn');

        this.copyBtn = document.getElementById('copyDashboardUrl');
        this.focusJoinBtn = document.getElementById('focusCollabJoin');
        this.focusStatusBtn = document.getElementById('focusCollabStatus');

        this.presenceList = document.getElementById('presenceList');
        this.statusInput = document.getElementById('collabStatusInput');
        this.workingInput = document.getElementById('collabWorkingInput');
        this.updateStatusBtn = document.getElementById('collabUpdateStatus');
        this.hotTubBtn = document.getElementById('collabHotTub');

        this.chatLog = document.getElementById('collabChatLog');
        this.chatInput = document.getElementById('collabChatInput');
        this.chatSendBtn = document.getElementById('collabSendBtn');
    }

    bindEvents() {
        this.toggleBtn?.addEventListener('click', () => this.toggleCollapse());
        this.copyBtn?.addEventListener('click', () => this.copyDashboardUrl());
        this.focusJoinBtn?.addEventListener('click', () => this.focusJoin());
        this.focusStatusBtn?.addEventListener('click', () => this.focusStatus());

        this.joinBtn?.addEventListener('click', () => this.join());
        this.chatSendBtn?.addEventListener('click', () => this.sendChat());
        this.chatInput?.addEventListener('keydown', (e) => {
            if (e.key === 'Enter') {
                e.preventDefault();
                this.sendChat();
            }
        });

        this.updateStatusBtn?.addEventListener('click', () => this.sendStatus());
        this.hotTubBtn?.addEventListener('click', () => this.toggleHotTub());
    }

    restoreInputs() {
        const savedName = localStorage.getItem('st-collab-name');
        const savedType = localStorage.getItem('st-collab-type');
        if (savedName && this.nameInput) this.nameInput.value = savedName;
        if (savedType && this.typeSelect) this.typeSelect.value = savedType;
    }

    toggleCollapse() {
        this.section.classList.toggle('collapsed');
        if (this.section.classList.contains('collapsed')) {
            this.toggleBtn.textContent = '▸';
        } else {
            this.toggleBtn.textContent = '▾';
        }
    }

    focusJoin() {
        if (this.section.classList.contains('collapsed')) this.toggleCollapse();
        this.nameInput?.focus();
    }

    focusStatus() {
        if (this.section.classList.contains('collapsed')) this.toggleCollapse();
        this.statusInput?.focus();
    }

    updateVisibility(isLive) {
        if (!this.joinEl || !this.liveEl) return;
        this.joinEl.classList.toggle('hidden', isLive);
        this.liveEl.classList.toggle('visible', isLive);
    }

    setStatus(message, state = 'idle') {
        if (!this.statusEl) return;
        this.statusEl.textContent = message;
        this.statusEl.dataset.state = state;
    }

    join() {
        const name = this.nameInput?.value.trim();
        const type = this.typeSelect?.value || 'human';

        if (!name) {
            this.setStatus('Pick a name to join.', 'error');
            this.nameInput?.focus();
            return;
        }

        localStorage.setItem('st-collab-name', name);
        localStorage.setItem('st-collab-type', type);
        this.connect(name, type);
    }

    connect(name, type) {
        if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
            return;
        }

        this.setStatus('Connecting...', 'busy');
        if (this.joinBtn) this.joinBtn.disabled = true;

        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const wsUrl = `${protocol}//${window.location.host}/ws/collab`;
        this.ws = new WebSocket(wsUrl);

        this.ws.onopen = () => {
            this.connected = true;
            this.send({ action: 'join', name, participant_type: type });
            this.setStatus('Joining the room...', 'busy');
        };

        this.ws.onmessage = (event) => {
            try {
                const msg = JSON.parse(event.data);
                this.handleServerMessage(msg);
            } catch (e) {
                console.error('[Collab] Failed to parse message:', e);
            }
        };

        this.ws.onclose = () => {
            this.connected = false;
            this.participantId = null;
            this.setStatus('Disconnected', 'error');
            this.updateVisibility(false);
            if (this.joinBtn) this.joinBtn.disabled = false;
        };

        this.ws.onerror = (error) => {
            console.error('[Collab] WebSocket error:', error);
            this.setStatus('Connection error', 'error');
            if (this.joinBtn) this.joinBtn.disabled = false;
        };
    }

    handleServerMessage(msg) {
        const collabTypes = new Set([
            'join',
            'leave',
            'chat',
            'status_update',
            'file_activity',
            'hot_tub_toggle',
            'system',
            'presence'
        ]);

        if (msg.type === 'welcome') {
            this.participantId = msg.participant_id;
            this.setStatus(`Connected as ${msg.name}`, 'online');
            this.updateVisibility(true);
            this.markStepComplete('join');
            this.addSystemMessage(`Welcome ${msg.name}.`);
            if (this.joinBtn) this.joinBtn.disabled = false;
            return;
        }

        if (msg.type === 'error') {
            this.setStatus(msg.message || 'Collab error', 'error');
            if (this.joinBtn) this.joinBtn.disabled = false;
            return;
        }

        if (msg.type === 'collab' && msg.collab && collabTypes.has(msg.collab.type)) {
            this.handleCollabMessage(msg.collab);
            return;
        }

        if (collabTypes.has(msg.type)) {
            this.handleCollabMessage(msg);
        }
    }

    handleCollabMessage(msg) {
        switch (msg.type) {
            case 'presence':
                this.syncPresence(msg.participants || [], msg.hot_tub_count || 0);
                break;
            case 'join':
                if (msg.participant) {
                    this.presence.set(msg.participant.id, this.toSummary(msg.participant));
                    this.renderPresence();
                    this.addSystemMessage(`${msg.participant.name} joined the room.`);
                }
                break;
            case 'leave':
                if (msg.participant_id) {
                    this.presence.delete(msg.participant_id);
                    this.renderPresence();
                }
                if (msg.name) {
                    this.addSystemMessage(`${msg.name} stepped out.`);
                }
                break;
            case 'chat':
                this.addChatMessage({
                    from: msg.from,
                    name: msg.from_name,
                    message: msg.message,
                    hot: msg.hot_tub
                });
                break;
            case 'status_update':
                this.updatePresenceStatus(msg.participant_id, msg.status);
                break;
            case 'hot_tub_toggle':
                this.addSystemMessage(msg.entering ? `${msg.name} entered Hot Tub mode.` : `${msg.name} left Hot Tub mode.`);
                if (msg.participant_id === this.participantId) {
                    this.setHotTubState(msg.entering);
                }
                break;
            case 'system':
                if (msg.message) this.addSystemMessage(msg.message);
                break;
            case 'file_activity':
                if (msg.path) {
                    this.addSystemMessage(`File activity: ${msg.action} ${msg.path}`);
                }
                break;
            default:
                break;
        }
    }

    syncPresence(participants, hotTubCount) {
        this.presence.clear();
        participants.forEach((p) => {
            this.presence.set(p.id, p);
        });
        this.renderPresence();
        if (typeof hotTubCount === 'number') {
            this.hotTubBtn?.setAttribute('data-count', hotTubCount.toString());
        }
    }

    updatePresenceStatus(id, status) {
        if (!id) return;
        const existing = this.presence.get(id);
        if (existing) {
            existing.status = status;
            this.presence.set(id, existing);
            this.renderPresence();
        }
    }

    renderPresence() {
        if (!this.presenceList) return;
        this.presenceList.innerHTML = '';
        const participants = Array.from(this.presence.values());
        participants.sort((a, b) => (a.name || '').localeCompare(b.name || ''));

        participants.forEach((p) => {
            const row = document.createElement('div');
            row.className = 'presence-item' + (p.id === this.participantId ? ' me' : '');
            row.innerHTML = `
                <span class="presence-emoji">${this.typeEmoji(p.participant_type)}</span>
                <span class="presence-name">${this.escapeHtml(p.name || 'Unknown')}</span>
                ${p.in_hot_tub ? '<span class="presence-badge">🛁</span>' : ''}
                ${p.status ? `<span class="presence-status">${this.escapeHtml(p.status)}</span>` : ''}
            `;
            this.presenceList.appendChild(row);
        });
    }

    toSummary(participant) {
        return {
            id: participant.id,
            name: participant.name,
            participant_type: participant.participant_type,
            status: participant.status,
            in_hot_tub: participant.in_hot_tub
        };
    }

    sendChat() {
        const message = this.chatInput?.value.trim();
        if (!message) return;
        this.send({ action: 'chat', message });
        this.chatInput.value = '';
    }

    sendStatus() {
        const status = this.statusInput?.value.trim() || null;
        const workingOn = this.workingInput?.value.trim() || null;
        this.send({ action: 'status', status, working_on: workingOn });
        this.markStepComplete('status');
    }

    toggleHotTub() {
        this.send({ action: 'hot_tub' });
    }

    setHotTubState(enabled) {
        this.hotTub = enabled;
        if (this.hotTubBtn) {
            this.hotTubBtn.classList.toggle('active', enabled);
            this.hotTubBtn.textContent = enabled ? 'Hot Tub Mode: On' : 'Hot Tub Mode';
        }
    }

    addChatMessage({ from, name, message, hot }) {
        if (!this.chatLog) return;
        const line = document.createElement('div');
        const isMe = from && this.participantId && from === this.participantId;
        line.className = `chat-message${isMe ? ' me' : ''}${hot ? ' hot' : ''}`;
        const time = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        line.innerHTML = `
            <div class="chat-meta">
                <span class="chat-name">${this.escapeHtml(name || 'Unknown')}</span>
                <span class="chat-time">${time}</span>
            </div>
            <div class="chat-text">${this.escapeHtml(message || '')}</div>
        `;
        this.chatLog.appendChild(line);
        this.chatLog.scrollTop = this.chatLog.scrollHeight;
    }

    addSystemMessage(message) {
        if (!this.chatLog) return;
        const line = document.createElement('div');
        line.className = 'chat-message system';
        line.textContent = message;
        this.chatLog.appendChild(line);
        this.chatLog.scrollTop = this.chatLog.scrollHeight;
    }

    send(payload) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(payload));
        }
    }

    copyDashboardUrl() {
        const url = window.location.href;
        if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard.writeText(url).then(() => {
                this.setStatus('Invite link copied.', 'online');
                this.markStepComplete('share');
            }).catch(() => {
                this.fallbackCopy(url);
            });
        } else {
            this.fallbackCopy(url);
        }
    }

    fallbackCopy(text) {
        const input = document.createElement('input');
        input.value = text;
        document.body.appendChild(input);
        input.select();
        document.execCommand('copy');
        input.remove();
        this.setStatus('Invite link copied.', 'online');
        this.markStepComplete('share');
    }

    markStepComplete(step) {
        const map = { share: this.stepShare, join: this.stepJoin, status: this.stepStatus };
        const el = map[step];
        if (el) el.classList.add('completed');
    }

    typeEmoji(type) {
        switch ((type || '').toLowerCase()) {
            case 'human': return '👤';
            case 'claude': return '🤖';
            case 'omni': return '🌀';
            case 'grok': return '⚡';
            case 'gemini': return '✨';
            case 'local_llm': return '🏠';
            case 'smart_tree': return '🌳';
            default: return '❓';
        }
    }

    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }
}

// Initialize on DOM ready
document.addEventListener('DOMContentLoaded', () => {
    window.dashboard = new Dashboard();

    // Initialize MCP activity visualization if Wave Compass container exists
    const compassContainer = document.getElementById('wave-compass-container');
    const compassCanvas = document.getElementById('wave-compass');
    const activityPanel = document.getElementById('mcp-activity-panel');

    if (compassCanvas && compassContainer) {
        // Create state sync connection
        const stateSync = new StateSync(
            (data) => {
                // Update Wave Compass
                if (window.waveCompass) {
                    window.waveCompass.update(data);
                }
                // Update activity panel
                if (window.mcpActivityPanel) {
                    window.mcpActivityPanel.update(data);
                }
            }
        );

        // Create Wave Compass
        window.waveCompass = new WaveCompass(compassCanvas, (hint) => {
            stateSync.sendHint(hint);
        });

        // Create hint input
        window.hintInput = new HintInput(compassContainer, (hint) => {
            stateSync.sendHint(hint);
        });

        // Create voice input
        window.voiceInput = new VoiceInput(compassContainer, (hint) => {
            stateSync.sendHint(hint);
        });

        // Create activity panel
        if (activityPanel) {
            window.mcpActivityPanel = new McpActivityPanel(activityPanel);
        }

        window.stateSync = stateSync;
    }

    window.collabPanel = new CollabPanel();
});
