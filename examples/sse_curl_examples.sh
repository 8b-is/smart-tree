#!/bin/bash
# Advanced SSE curl examples for Smart Tree

echo "🌳 Smart Tree SSE Curl Examples"
echo "==============================="
echo

# Example 1: Basic SSE stream
echo "1️⃣ Basic SSE Stream (press Ctrl+C to stop):"
echo "curl -N -H 'Accept: text/event-stream' http://localhost:8420/sse"
echo

# Example 2: SSE with custom headers
echo "2️⃣ SSE with Custom Headers:"
echo "curl -N \\"
echo "  -H 'Accept: text/event-stream' \\"
echo "  -H 'Cache-Control: no-cache' \\"
echo "  -H 'X-Client-ID: curl-test' \\"
echo "  http://localhost:8420/sse"
echo

# Example 3: Parse specific event types
echo "3️⃣ Parse Only File Events:"
cat << 'EOF'
curl -N -H 'Accept: text/event-stream' http://localhost:8420/sse 2>/dev/null | \
while IFS= read -r line; do
    case "$line" in
        "event: created"|"event: modified"|"event: deleted")
            event_type="${line#event: }"
            read -r id_line
            read -r data_line
            echo "[$event_type] $(echo "$data_line" | cut -d' ' -f2- | jq -r .path)"
            ;;
    esac
done
EOF
echo

# Example 4: Convert SSE to JSON Lines
echo "4️⃣ Convert SSE to JSON Lines Format:"
cat << 'EOF'
curl -N -H 'Accept: text/event-stream' http://localhost:8420/sse 2>/dev/null | \
awk '
    /^event:/ { event = $2 }
    /^id:/ { id = $2 }
    /^data:/ { 
        data = substr($0, 7)
        print "{\"event\":\"" event "\",\"id\":\"" id "\",\"data\":" data "}"
    }
'
EOF
echo

# Example 5: Monitor with timestamp
echo "5️⃣ Monitor Events with Timestamps:"
cat << 'EOF'
curl -N -H 'Accept: text/event-stream' http://localhost:8420/sse 2>/dev/null | \
while IFS= read -r line; do
    if [[ $line =~ ^data: ]]; then
        timestamp=$(date '+%Y-%m-%d %H:%M:%S')
        echo "[$timestamp] $line"
    fi
done
EOF
echo

# Example 6: Count events by type
echo "6️⃣ Count Events by Type:"
cat << 'EOF'
curl -N -H 'Accept: text/event-stream' http://localhost:8420/sse 2>/dev/null | \
awk '
    /^event:/ { events[$2]++ }
    END { for (e in events) print e ": " events[e] }
' &
sleep 30
kill $!
EOF
echo

# Example 7: Real-time directory monitoring simulation
echo "7️⃣ Real-time Directory Monitor Display:"
cat << 'EOF'
curl -N -H 'Accept: text/event-stream' http://localhost:8420/sse 2>/dev/null | \
while IFS= read -r line; do
    if [[ $line =~ ^event: ]]; then
        event_type="${line#event: }"
    elif [[ $line =~ ^data: ]]; then
        data=$(echo "$line" | cut -d' ' -f2-)
        case "$event_type" in
            "created")
                echo "➕ $(echo "$data" | jq -r .path) [NEW]"
                ;;
            "modified")
                echo "📝 $(echo "$data" | jq -r .path) [MODIFIED]"
                ;;
            "deleted")
                echo "❌ $(echo "$data" | jq -r .path) [DELETED]"
                ;;
            "stats")
                files=$(echo "$data" | jq -r .stats.total_files)
                size=$(echo "$data" | jq -r .stats.total_size)
                echo "📊 Stats Update: $files files, $((size / 1024 / 1024))MB"
                ;;
        esac
    fi
done
EOF
echo

# Example 8: Save and replay events
echo "8️⃣ Save Events for Replay:"
echo "# Save events"
echo "curl -N -H 'Accept: text/event-stream' http://localhost:8420/sse > events.sse"
echo
echo "# Replay events"
echo "cat events.sse | grep -E '^(event:|data:|id:)' | while read -r line; do echo \"\$line\"; sleep 0.1; done"
echo

# Example 9: Using with jq for complex filtering
echo "9️⃣ Complex Event Filtering with jq:"
cat << 'EOF'
curl -N -H 'Accept: text/event-stream' http://localhost:8420/sse 2>/dev/null | \
grep '^data:' | cut -d' ' -f2- | \
jq -r 'select(.type == "stats") | "\(.path): \(.stats.total_files) files, \(.stats.total_size) bytes"'
EOF
echo

# Example 10: Performance monitoring
echo "🔟 Monitor Event Rate:"
cat << 'EOF'
curl -N -H 'Accept: text/event-stream' http://localhost:8420/sse 2>/dev/null | \
awk '
    BEGIN { start = systime(); count = 0 }
    /^event:/ { 
        count++
        elapsed = systime() - start
        if (elapsed > 0) {
            rate = count / elapsed
            printf "\rEvents: %d, Rate: %.2f/sec", count, rate
        }
    }
'
EOF
echo
echo

echo "📝 Notes:"
echo "- Use -N flag to disable buffering for real-time updates"
echo "- Add 2>/dev/null to suppress curl progress output"
echo "- Use timeout command to limit execution time"
echo "- Pipe to jq for pretty JSON formatting"
echo "- Use grep/awk for event filtering and processing"