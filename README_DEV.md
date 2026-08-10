# BeiFeng Developer Guide

## Rust Crates

- `rusty-claude-cli`: main Agent CLI and session/runtime entrypoint.
- `api`: provider routing and provider clients, including DeepSeek.
- `runtime`: configuration, prompt loading, sessions, hooks, and execution support.
- `tools`: built-in Agent tools, including Wind O&M tools.
- `claw-rag-service`: Wind Knowledge Hub ingestion, search, graph, advice, risk, and report CLI/API.
- `plugins`, `commands`, `telemetry`: supporting Claw runtime crates.

## DeepSeek Provider

DeepSeek uses:

- `DEEPSEEK_BASE_URL`
- `DEEPSEEK_API_KEY`

The default project model is recorded in `beifeng/config/agent_config.json`.

## Tool Registry

Built-in tools are registered in `rust/crates/tools/src/lib.rs` through:

- `mvp_tool_specs()`
- `execute_tool_with_enforcer()`
- input structs and runner functions

Wind tools:

- `wind_knowledge_query`
- `wind_fault_analysis`
- `wind_report_generate`

After adding a tool, run:

```powershell
cargo check -p tools
cargo check -p rusty-claude-cli
cargo test -p tools <tool_name>
```

## RAG Service

`claw-rag-service` owns:

- document ingestion
- SQLite chunk/embedding storage
- hybrid search
- fault graph query
- advice generation
- risk assessment
- report generation

Defaults now point to `beifeng/config/paths.json` locations.

## Wind Module

Project-specific Wind O&M assets live under `beifeng/`:

- prompt: `beifeng/prompts/CLAUDE.md`
- knowledge base: `beifeng/knowledge/knowledge_base`
- fault graph: `beifeng/knowledge/knowledge_graph/wind_fault_graph.json`
- report output: `beifeng/reports/generated`

## Add A Tool

1. Add the input struct.
2. Add a `ToolSpec` in `mvp_tool_specs()`.
3. Add dispatch in `execute_tool_with_enforcer()`.
4. Add runner and unit tests.
5. Verify `--allowedTools <tool_name>` works after rebuilding `rusty-claude-cli`.

## Add A Skill

Add a folder under `beifeng/skills/<skill_name>/` with:

- `SKILL.md`
- `examples.md`

Skills are currently documentation-first unless runtime loading is explicitly wired.

## Add Memory

Memory schemas live under `beifeng/memory/`. Memory remains disabled by default.

Do not enable writes without explicit authorization and tests.

## Testing

Core checks:

```powershell
cargo build -p rusty-claude-cli
cargo check -p claw-rag-service
cargo check -p tools
cargo test -p claw-rag-service
cargo test -p tools wind_report_generate
```

