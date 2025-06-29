#!/usr/bin/env node

/**
 * Smart Tree MCP Server Wrapper
 * Ensures the correct binary is installed before running
 */

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');
const { install, getPlatformInfo } = require('./install');

async function main() {
    try {
        const platformInfo = getPlatformInfo();
        const binaryPath = path.join(__dirname, platformInfo.binaryName);
        
        // Check if binary exists
        if (!fs.existsSync(binaryPath)) {
            console.error('Smart Tree binary not found. Installing...');
            await install();
        }
        
        // Spawn the actual MCP server
        const args = process.argv.slice(2);
        const child = spawn(binaryPath, ['--mcp', ...args], {
            stdio: 'inherit',
            env: process.env
        });
        
        // Forward signals
        process.on('SIGINT', () => child.kill('SIGINT'));
        process.on('SIGTERM', () => child.kill('SIGTERM'));
        
        // Exit with same code as child
        child.on('exit', (code) => {
            process.exit(code || 0);
        });
        
    } catch (error) {
        console.error('Failed to start Smart Tree MCP server:', error.message);
        process.exit(1);
    }
}

main(); 