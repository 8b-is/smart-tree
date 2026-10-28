---
title: Smart Tree LLM Proxy
description: Unified interface for calling various LLMs directly from smart-tree
contributor: The Cheet
lastUpdated: 2026-01-18
language: en
---

# 🌐 Smart Tree LLM Proxy

Hey there! The Cheet here! 😺 Ever wanted to talk to multiple AIs without leaving your favorite directory tool? Now you can! The Smart Tree LLM Proxy provides a unified way to call OpenAI, Anthropic, Google Gemini, and even local Candle models!

## 🚀 Quick Start

To use the proxy, you'll need to set your API keys first:

```bash
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"
export GOOGLE_API_KEY="your-key"
```

Then, call the proxy like a pro:

```bash
# Call OpenAI
st --proxy --provider openai --model gpt-4o --prompt "Analyze this project structure"

# Call Anthropic
st --proxy --provider anthropic --model claude-3-5-sonnet-20240620 --prompt "What's the best way to refactor this?"

# Call Google Gemini
st --proxy --provider google --model gemini-1.5-pro --prompt "Explain the architecture"
```

## 🧠 Memory & Scopes

Smart Tree 6.0.0 now remembers your conversations! You can use scopes to keep different projects or sessions separate.

```bash
# Start a conversation in a specific scope
st --proxy --provider openai --model gpt-4o --scope "project-x" --prompt "Let's talk about project X"

# Continue the conversation in the same scope
st --proxy --provider openai --model gpt-4o --scope "project-x" --prompt "What did we just talk about?"
```

> Pro Tip: Scopes are stored persistently in `~/.mem8/proxy_memory.json`. You can clear them by deleting the file or using a new scope ID! 🧠
{.is-info}

## 🛠️ Common Usage

### Piping from Smart Tree
The real magic happens when you pipe your tree output directly to the proxy!

```bash
# Send your project structure to Claude for analysis
st . -m ai | st --proxy --provider anthropic --model claude-3-5-sonnet-20240620
```

### Reading from Stdin
If you don't provide a `--prompt`, the proxy will wait for you to type (or pipe) something.

```bash
echo "Tell me a joke about Rust" | st --proxy --provider openai --model gpt-4o
```

## 🌐 OpenAI-Compatible Server Mode

Want to use Smart Tree as a backend for other AI tools? Start the proxy server!

```bash
# Start the server on default port 8448
st --proxy-server

# Start on a custom port
st --proxy-server --proxy-port 9000
```

Now you can point any OpenAI-compatible client to `http://localhost:8448/v1`.

### Model Mapping
To specify the provider via the API, use the `provider/model` format:
- `openai/gpt-4o`
- `anthropic/claude-3-5-sonnet-20240620`
- `google/gemini-1.5-pro`
- `candle/llama-3`

## 🕯️ Local AI with Candle

Want to keep it local? We've got you covered!

```bash
st --proxy --provider candle --model llama-3 --prompt "Hello from my local machine!"
```

> Pro Tip: Local models require the `candle` feature to be enabled during compilation. Use `cargo build --features candle` to light the flame! 🔥
{.is-success}

## 📊 Token Tracking

The proxy automatically tracks your token usage so you don't break the bank! 💰

```text
📊 Tokens: 150 prompt, 300 completion (450 total)
```

## ⚠️ Troubleshooting

- **Missing API Key**: Make sure your environment variables are set correctly!
- **Model Not Found**: Double-check the model name for your provider.
- **Network Issues**: Ensure you have a stable internet connection for cloud providers.

Remember: "A proxy in the hand is worth two in the cloud!" ☁️

Stay smart, stay tree-sy! 🌳✨
