#!/usr/bin/env node

/**
 * Smart Tree MCP Installation Helper
 * Guides users to install Smart Tree properly and configure Claude Desktop
 */

const { execSync } = require('child_process');
const os = require('os');
const fs = require('fs');
const path = require('path');

// Check if Smart Tree is installed system-wide
function checkSystemInstallation() {
    try {
        const version = execSync('st --version', { encoding: 'utf8' }).trim();
        return { installed: true, version, path: 'st' };
    } catch (e) {
        // Try common installation paths
        const commonPaths = [
            '/usr/local/bin/st',
            '/usr/bin/st',
            '/opt/homebrew/bin/st',
            path.join(os.homedir(), '.local', 'bin', 'st'),
            path.join(os.homedir(), 'bin', 'st')
        ];
        
        for (const stPath of commonPaths) {
            if (fs.existsSync(stPath)) {
                try {
                    const version = execSync(`"${stPath}" --version`, { encoding: 'utf8' }).trim();
                    return { installed: true, version, path: stPath };
                } catch (e) {
                    // Binary exists but can't run it
                }
            }
        }
        
        return { installed: false };
    }
}

// Get platform-specific installation instructions
function getInstallInstructions() {
    const platform = os.platform();
    
    const instructions = {
        title: "🌲 Smart Tree Installation Required",
        steps: []
    };
    
    if (platform === 'darwin' || platform === 'linux') {
        instructions.steps = [
            "1. Open Terminal",
            "2. Run this one-liner:",
            "   curl -sSL https://raw.githubusercontent.com/8b-is/smart-tree/main/scripts/install.sh | bash",
            "3. Restart Claude Desktop",
            "4. Smart Tree MCP will work automatically!"
        ];
    } else if (platform === 'win32') {
        instructions.steps = [
            "1. Install from GitHub releases:",
            "   https://github.com/8b-is/smart-tree/releases/latest",
            "2. Download the Windows ZIP file",
            "3. Extract and add to your PATH",
            "4. Restart Claude Desktop",
            "5. Smart Tree MCP will work automatically!"
        ];
    }
    
    return instructions;
}

// Generate configuration for Claude Desktop
function generateConfig(stPath) {
    const config = {
        "smart-tree": {
            "command": stPath,
            "args": ["--mcp"],
            "env": {
                "AI_TOOLS": "1"
            }
        }
    };
    
    return JSON.stringify(config, null, 2);
}

// MCP Server that provides installation guidance
class InstallationHelperServer {
    constructor() {
        this.input = '';
    }
    
    async start() {
        process.stdin.on('data', (chunk) => {
            this.input += chunk.toString();
            
            // Process complete JSON-RPC messages
            const lines = this.input.split('\n');
            this.input = lines.pop() || '';
            
            for (const line of lines) {
                if (line.trim()) {
                    this.handleMessage(line);
                }
            }
        });
        
        process.stdin.on('end', () => {
            process.exit(0);
        });
    }
    
    handleMessage(line) {
        try {
            const message = JSON.parse(line);
            
            if (message.method === 'initialize') {
                this.handleInitialize(message);
            } else if (message.method === 'tools/list') {
                this.handleToolsList(message);
            } else if (message.method === 'tools/call') {
                this.handleToolCall(message);
            }
        } catch (e) {
            console.error('Error handling message:', e.message);
        }
    }
    
    handleInitialize(message) {
        const response = {
            jsonrpc: "2.0",
            id: message.id,
            result: {
                protocolVersion: "2024-11-05",
                capabilities: {
                    tools: {}
                },
                serverInfo: {
                    name: "Smart Tree Installer",
                    version: "1.0.0"
                }
            }
        };
        
        this.send(response);
        
        // Send initialized notification
        this.send({
            jsonrpc: "2.0",
            method: "notifications/initialized"
        });
    }
    
    handleToolsList(message) {
        const response = {
            jsonrpc: "2.0",
            id: message.id,
            result: {
                tools: [
                    {
                        name: "check_installation",
                        description: "Check if Smart Tree is installed and provide installation instructions",
                        inputSchema: {
                            type: "object",
                            properties: {}
                        }
                    }
                ]
            }
        };
        
        this.send(response);
    }
    
    handleToolCall(message) {
        if (message.params.name === 'check_installation') {
            const status = checkSystemInstallation();
            
            let content;
            if (status.installed) {
                content = `✅ Smart Tree is installed!\n\n` +
                         `Version: ${status.version}\n` +
                         `Path: ${status.path}\n\n` +
                         `To use Smart Tree with Claude Desktop, add this to your MCP settings:\n\n` +
                         `\`\`\`json\n${generateConfig(status.path)}\n\`\`\`\n\n` +
                         `After updating settings, restart Claude Desktop to activate Smart Tree MCP.`;
            } else {
                const instructions = getInstallInstructions();
                content = `${instructions.title}\n\n` +
                         `Smart Tree needs to be installed on your system first.\n\n` +
                         `${instructions.steps.join('\n')}\n\n` +
                         `Why install separately?\n` +
                         `• Full system permissions for file access\n` +
                         `• Faster performance\n` +
                         `• Works in terminal and with Claude Desktop\n` +
                         `• Automatic updates via the installer`;
            }
            
            const response = {
                jsonrpc: "2.0",
                id: message.id,
                result: {
                    content: [
                        {
                            type: "text",
                            text: content
                        }
                    ]
                }
            };
            
            this.send(response);
        }
    }
    
    send(message) {
        process.stdout.write(JSON.stringify(message) + '\n');
    }
}

// Start the installation helper server
const server = new InstallationHelperServer();
server.start();