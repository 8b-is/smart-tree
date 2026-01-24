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

        this.init();
    }

    async init() {
        this.initLayout();
        this.initMobile();
        this.initVoice();
        this.createTerminal(); // Create first terminal
        this.initFileBrowser();
        this.initResizers();
        this.initEventListeners();
        this.initKeyboardShortcuts();
        await this.loadHealth();

        // Refresh health periodically
        setInterval(() => this.loadHealth(), 30000);
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
            if (newWidth >= 150 && newWidth <= 400) {
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

// Initialize on DOM ready
document.addEventListener('DOMContentLoaded', () => {
    window.dashboard = new Dashboard();
});
