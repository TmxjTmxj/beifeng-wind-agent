# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Detected stack
- Languages: Rust, TypeScript (desktop), PowerShell (scripts).
- Frameworks: Tauri 2 (desktop), React + Vite (frontend).

## Verification
- From this `rust/` directory, run: `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`; formatting with `cargo fmt --all --check`.
- Active Rust source and tests live under `rust/crates/*/src/` and `rust/crates/*/tests/`; update the matching crate surfaces together when behavior changes.

## Repository shape
- `rust/` contains the Rust workspace and the active agent CLI/runtime implementation.
- `beifeng/` contains the Wind O&M business assets: knowledge base, fault graph, skills, config, and evaluation harness.
- `apps/desktop/` contains the Tauri desktop shell (React frontend).
- `scripts/` contains PowerShell workflow helpers for build, ingest, RAG serve, and report generation.

## Working agreement
- Prefer small, reviewable changes.
- Never commit live secrets: API keys belong in local `.env` or `beifeng/config/secrets.json` (gitignored).
- Respect the safety boundaries in `beifeng/config/settings.json` — no unauthorized remote shutdown/reset recommendations.
