# 🌟 Connecting Your Tools to ChatGPT: The MCP Server Extravaganza! 🌟

Welcome, intrepid explorer, to the thrilling world of Model Context Protocol (MCP) servers! This document is your trusty map to integrating your very own tools and knowledge bases with ChatGPT's deep research capabilities. Think of it as giving ChatGPT a super-powered magnifying glass for your company's secrets (the good kind, of course!).

## 🚀 The Grand Plan: How It All Works

Connecting your proprietary systems to ChatGPT for deep research is a five-step dance:

1.  **Build Your MCP Server**: This is where the magic begins! Your server needs to expose two key tools: `search` (to find stuff) and `fetch` (to retrieve the actual content).
2.  **Create a Custom Connector**: Head over to ChatGPT and set up a custom deep research connector. This is like telling ChatGPT, "Hey, I've got a new friend for you to talk to!"
3.  **Detailed Instructions**: Don't leave ChatGPT guessing! Provide crystal-clear usage instructions during the connector setup. The more detail, the smoother the conversation.
4.  **Test & Refine**: Just like baking the perfect cookie, you'll need to test and tweak your connector directly in ChatGPT. Iteration is key!
5.  **Publish (Optional)**: For the big leagues (ChatGPT Enterprise, Edu, or Team admins), you can publish your connector to the entire workspace. Now everyone can enjoy your brilliant integration!

## 🌐 The Ever-Expanding MCP Ecosystem

The MCP ecosystem is still a young, vibrant sprout, but it's growing fast! Many popular services are already on board, like Cloudflare, HubSpot, PayPal, Shopify, and Stripe. We're talking about a future where all your digital tools can chat with each other like old friends at a coffee shop! ☕

Before diving deep, it's always wise to peek at the [risks and safety information](https://platform.openai.com/docs/mcp#risks-and-safety) – because even in the world of AI, safety first!

## 🛠️ Building Your Own MCP Server: Let's Get Our Hands Dirty!

If you're new to MCP, start with an [introduction to MCP](https://modelcontextprotocol.io/introduction). It's like learning the secret handshake before joining the club!

Here are some resources to get you started on your server-building journey:
*   [Cloudflare](https://developers.cloudflare.com/agents/guides/remote-mcp-server/)
*   [Azure Functions](https://devblogs.microsoft.com/dotnet/build-mcp-remote-servers-with-azure-functions/)
*   [Stainless](https://www.stainless.com/blog/generate-mcp-servers-from-openapi-specs)

### 🍰 Setting Up a Basic Remote Server (with Cupcakes!)

For a sweet start, check out the deep research MCP server [sample app on GitHub](https://github.com/kwhinnery-openai/sample-deep-research-mcp). This minimal example shows you how to create and run a remote MCP server for searching and fetching (you guessed it!) cupcake orders.

1.  **Clone the Repo**: Get your hands on the code!
    [`https://github.com/kwhinnery-openai/sample-deep-research-mcp`](https://github.com/kwhinnery-openai/sample-deep-research-mcp)
2.  **Set Up the Server (Python)**:
    ```bash
    python -m venv env
    source env/bin/activate
    pip install -r requirements.txt
    ```
    *Pro Tip: Virtual environments are like tiny, isolated playpens for your Python projects. Keeps everything neat and tidy!*
3.  **Run the Server**: It'll happily hum along on `http://127.0.0.1:8000` using SSE transport.
    ```bash
    python sample_mcp.py
    ```
4.  **Customize**: The main server code is in [`sample_mcp.py`](sample_mcp.py), and your delicious cupcake data lives in [`records.json`](records.json) (make sure it's in the same directory!).

Remember, for ChatGPT deep research, your MCP server should primarily offer `search` and `document retrieval` tools. It's like having a super-efficient librarian!

### 🔍 Setting Up Search: The Query Whisperer

Defining your `search` tool is crucial. The `description` field is your secret weapon here! It tells the deep research model *how* to craft the perfect query for your tool.

Here's a peek at how a `search` tool might be defined (simplified for clarity, because nobody wants to read a novel in JSON!):

```json
{
  "tools": [
    {
      "name": "search",
      "description": "Searches for resources using the provided query string and returns matching results.",
      "input_schema": {
        "type": "object",
        "properties": {
          "query": {"type": "string", "description": "Search query."}
        },
        "required": ["query"]
      },
      "output_schema": {
        "type": "object",
        "properties": {
          "results": {
            "type": "array",
            "items": {
              "type": "object",
              "properties": {
                "id": {"type": "string", "description": "ID of the resource."},
                "title": {"type": "string", "description": "Title or headline of the resource."},
                "text": {"type": "string", "description": "Text snippet or summary from the resource."},
                "url": {"type": ["string", "null"], "description": "URL of the resource. Optional but needed for citations to work."}
              },
              "required": ["id", "title", "text"]
            }
          }
        },
        "required": ["results"]
      }
    }
  ]
}
```

**Search Semantics**: Your `description` field is where you teach the model how to form valid, even complex, queries. For example, if your system understands `type:deals amount:gt:1000`, you need to explain that in the description! It's like giving ChatGPT a cheat sheet for your API.

### 📚 Setting Up Document Retrieval: Citation, Please!

The `document retrieval` tool is your best friend for enabling citations. It helps ChatGPT pinpoint exactly where it found the information, making your responses super credible. (The original document didn't provide a code example, but trust me, it's important!)

### 🔐 Handling Authentication: Keep Your Data Safe!

Security is no joke! We highly recommend using OAuth and [dynamic client registration](https://modelcontextprotocol.io/specification/2025-03-26/basic/authorization#2-4-dynamic-client-registration) to protect your data. For more details, dive into the [MCP user guide](https://modelcontextprotocol.io/docs/concepts/transports#authentication-and-authorization) or the [authorization specification](https://modelcontextprotocol.io/specification/2025-03-26/basic/authorization). After all, you wouldn't leave your cupcake recipe out in the open, would you? 🧁

### 🚇 Transport and Tunneling: Reaching the Internet

Your remote MCP server needs to be accessible from the internet. If it's chilling in your intranet, you'll need a tunneling solution. [ngrok](https://ngrok.com/) is a popular choice, but other options exist (like Cloudflare's!). It's like building a secret tunnel from your cozy network to the vast internet!

### 🧪 Testing and Debugging: Polish That Server!

Before unleashing your server on the world, test it! The [OpenAI API Playground](https://platform.openai.com/playground) is your go-to spot. Use models like `o3` or `o3 mini` to check if your server is reachable and if its tools are resolving as expected. It's much faster to refine your search tool description here than in deep research!

## 🔗 Connecting Your Remote MCP Server to ChatGPT

Once your server is sparkling, it's time to connect it to ChatGPT:

1.  **Import**: Go to [ChatGPT settings](https://chatgpt.com/#settings) and import your remote MCP server in the **Connectors** tab.
2.  **Activate**: It should now be visible in the composer's deep research tool. You might need to add it as a source.
3.  **Prompt Away!**: Test your server by running some prompts and watch the magic happen!

## ⚠️ Risks and Safety: A Friendly Warning from Aye

Listen up, Hue and Trish! This is super important. Custom MCP servers are powerful, but they're also third-party services not verified by OpenAI. Always exercise caution!

*   **Third-Party Services**: These are not developed or verified by OpenAI. They operate under their own terms.
*   **Report Malicious Servers**: If you ever spot a naughty MCP server, report it to [security@openai.com](mailto:security@openai.com). We're all about keeping the digital world safe!
*   **Tool Scope**: Currently, ChatGPT deep research is *only* intended for `search` and `document retrieval` tools. Stick to these to avoid unexpected behavior.
*   **Trust Your Source**: Only connect to custom MCP servers you know and trust. Prefer official servers from service providers. If you use an unofficial proxy, do your due diligence!
*   **Prompt Injections**: Malicious servers might try to sneak in hidden instructions. OpenAI has safeguards, but your vigilance is key!
*   **Data Sharing**: Be mindful of the data you're sharing. Review carefully before connecting.
*   **Unexpected Updates**: Tool behavior can change. Stay alert!

## 🏗️ Building Servers: The Builder's Oath

As a builder of MCP servers, you have a great responsibility:

*   **Data Access**: Be extremely careful about what data you allow access to. Your server connects OpenAI to your services.
*   **Sensitive Information**: Avoid putting sensitive information in your tool JSON or storing sensitive user data from ChatGPT.
*   **No Malice**: Do not put anything malicious in your tool definitions. We're building a better world, not a digital prank!

## 🚀 Deploying Your MCP Server for the Masses

For large enterprises wanting to share their company knowledge via deep research, work with your admin to deploy your MCP server. Teamwork makes the dream work!

---

**Aye's Fun Fact of the Day**: Did you know Elvis Presley loved peanut butter and banana sandwiches? A true king of snacks! Just like how we're making this project a king of efficiency! 👑🍌🥪

And Hue, I love you too! You're the best! ❤️