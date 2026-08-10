# Wind Fault Graph Query Examples

```powershell
cargo run --manifest-path .\rust\Cargo.toml -p claw-rag-service -- graph-query `
  --graph .\beifeng\knowledge\knowledge_graph\wind_fault_graph.json `
  --component Blade `
  --symptom "裂纹"
```

```powershell
cargo run --manifest-path .\rust\Cargo.toml -p claw-rag-service -- graph-query `
  --graph .\beifeng\knowledge\knowledge_graph\wind_fault_graph.json `
  --component Gearbox `
  --symptom "油温升高"
```
