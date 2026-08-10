. "$PSScriptRoot\settings.ps1"

$settings = Get-BeiFengSettings
$projectRoot = Get-BeiFengProjectRoot
$dbPath = Resolve-BeiFengPath -Settings $settings -Key 'rag.db_path'
$graphPath = Resolve-BeiFengPath -Settings $settings -Key 'paths.knowledge_graph'
$reportsPath = Resolve-BeiFengPath -Settings $settings -Key 'paths.reports'

Set-Location $projectRoot

cargo run --manifest-path .\rust\Cargo.toml -p claw-rag-service -- report-generate `
  --db $dbPath `
  --graph $graphPath `
  --reports-dir $reportsPath `
  --problem "叶片裂纹" `
  --component Blade `
  --symptom "裂纹" `
  --report-type inspection_report
