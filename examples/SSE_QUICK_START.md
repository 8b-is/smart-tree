# Quick Start: Testing SSE with Curl

## 1. Install Dependencies (for test server)

```bash
cd /home/hue/source/smart-tree/examples
pip install -r sse_requirements.txt
```

## 2. Start the Test Server

```bash
python3 sse_test_server.py
```

This starts a server on http://localhost:8420 with:
- Browser test page: http://localhost:8420
- SSE endpoint: http://localhost:8420/sse

## 3. Run Curl Tests

### Option A: Run all tests automatically
```bash
./test_sse_curl.sh
```

### Option B: Run individual curl commands

#### Basic SSE stream:
```bash
curl -N -H "Accept: text/event-stream" http://localhost:8420/sse
```

#### Pretty-printed JSON events:
```bash
curl -N -H "Accept: text/event-stream" http://localhost:8420/sse 2>/dev/null | \
while IFS= read -r line; do
    [[ $line =~ ^data: ]] && echo "$line" | cut -d' ' -f2- | jq .
done
```

#### Filter specific events:
```bash
curl -N -H "Accept: text/event-stream" http://localhost:8420/sse 2>/dev/null | \
grep -E '^event: (created|modified|deleted)' -A2
```

#### With timeout (10 seconds):
```bash
timeout 10 curl -N -H "Accept: text/event-stream" http://localhost:8420/sse
```

## 4. Understanding the Output

SSE events follow this format:
```
event: created
id: 1234567890
data: {"type":"created","path":"/path/to/file.txt","node":{...}}

event: heartbeat
id: 1234567891
data: {"type":"heartbeat","count":1}
```

## 5. Test in Browser

Open http://localhost:8420 in a browser to see a visual representation of the SSE stream.

## 6. Advanced Examples

See `sse_curl_examples.sh` for more advanced usage patterns including:
- Event counting
- Real-time monitoring display
- Event filtering with jq
- Performance monitoring
- Event recording and replay

## Common Curl Flags for SSE

- `-N` / `--no-buffer`: Disable buffering for real-time output
- `-H "Accept: text/event-stream"`: Required SSE header
- `--compressed`: Enable compression
- `-m <seconds>`: Set maximum time
- `-v`: Verbose mode (shows headers)

## Troubleshooting

1. **No output**: Check server is running on port 8420
2. **Buffered output**: Make sure to use `-N` flag
3. **Connection drops**: Normal for SSE, client should reconnect
4. **JSON parse errors**: Use `2>/dev/null` to suppress curl progress