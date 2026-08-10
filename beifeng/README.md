# BeiFeng

This directory contains the project-specific Wind O&M layer for BeiFeng Wind O&M Agent.

Key areas:

- `config/`: agent and path configuration.
- `prompts/`: Wind O&M system prompt.
- `knowledge/`: Wind Knowledge Hub source documents and fault graph.
- `reports/`: generated Markdown reports and templates.
- `memory/`: reserved memory schemas and examples.
- `skills/`: project-level skill specifications.
- `workflows/`: documented Wind O&M workflows.
- `evals/`: manual evaluation cases and regression checklist.
- `connectors/`: reserved connector schemas.

Runtime defaults now point at this directory for knowledge, graph, database, and report output.

