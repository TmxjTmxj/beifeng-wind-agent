. "$PSScriptRoot\settings.ps1"

$settings = Get-BeiFengSettings
$projectRoot = Get-BeiFengProjectRoot
$knowledgeBase = Resolve-BeiFengPath -Settings $settings -Key 'paths.knowledge_base'
$dbPath = Resolve-BeiFengPath -Settings $settings -Key 'rag.db_path'

Set-Location $projectRoot
Set-BeiFengEnvDefault -Name 'CLAW_RAG_MOCK_PROVIDERS' -Value '1'

cargo run --manifest-path .\rust\Cargo.toml -p claw-rag-service -- ingest `
  --knowledge-base $knowledgeBase `
  --db $dbPath
