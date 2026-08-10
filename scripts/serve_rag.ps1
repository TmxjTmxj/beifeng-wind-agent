. "$PSScriptRoot\settings.ps1"

$settings = Get-BeiFengSettings
$projectRoot = Get-BeiFengProjectRoot
$dbPath = Resolve-BeiFengPath -Settings $settings -Key 'rag.db_path'
$memoryPath = Resolve-BeiFengPath -Settings $settings -Key 'paths.memory'

Set-Location $projectRoot
Set-BeiFengEnvDefault -Name 'CLAW_RAG_MOCK_PROVIDERS' -Value '1'
Set-BeiFengEnvDefault -Name 'CLAW_RAG_MEMORY_DIR' -Value $memoryPath

cargo run --manifest-path .\rust\Cargo.toml -p claw-rag-service -- serve `
  --db $dbPath
