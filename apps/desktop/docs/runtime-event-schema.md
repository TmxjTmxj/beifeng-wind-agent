# Runtime Event Schema

P7.4 introduces a unified runtime event protocol for Desktop ⇄ Agent Runtime ⇄ RAG visualization.

## AgentEvent

```json
{
  "event_type": "tool_call_finished",
  "timestamp": "2026-06-09 09:20:00",
  "session_id": "session-20260609092000.123",
  "payload": {
    "name": "wind_knowledge_query",
    "status": "completed",
    "success": true,
    "latency_ms": 248,
    "detail": "Retrieved turbine blade crack knowledge"
  }
}
```

## Event Types

- `user_message`
- `assistant_message`
- `tool_call_started`
- `tool_call_finished`
- `knowledge_hit`
- `memory_hit`
- `graph_hit`
- `risk_assessment`
- `report_generated`
- `connector_query`
- `connector_result`
- `warning`
- `error`
- `system_status`

## Desktop Rules

- Desktop consumes `AgentEvent` objects as the source of truth for Inspector and Console.
- Desktop no longer parses natural-language CLI text to infer tools, RAG hits, graph hits, or risk.
- CLI output may include JSON-line `AgentEvent` objects. Those are appended to the session event stream.
- Non-event CLI text is treated only as `assistant_message` content.
- Secret values loaded from `beifeng/config/secrets.json` are redacted before logs, Console, and raw event JSON are exposed.

## Derived Views

- Live Inspector derives Tool Calls, Knowledge Hits, Memory Hits, Graph Hits, Risk Assessment, and Execution Trace from `AgentEvent`.
- Agent Console derives Event Timeline, Raw Event JSON, Tool Duration, Tool Success Rate, Knowledge Query Latency, RAG Latency, Memory Query Latency, and Connector Latency from the same event stream.
- Chat sessions persist their `events` array in `beifeng/chats/*.json`.
