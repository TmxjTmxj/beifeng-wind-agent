. "$PSScriptRoot\settings.ps1"

$settings = Get-BeiFengSettings
$projectRoot = Get-BeiFengProjectRoot
$rustRoot = Join-Path $projectRoot 'rust'
$modelName = Get-BeiFengSettingValue -Settings $settings -Key 'model.name'
$ragUrl = Get-BeiFengSettingValue -Settings $settings -Key 'rag.service_url'
$memoryPath = Resolve-BeiFengPath -Settings $settings -Key 'paths.memory'

Set-Location $rustRoot
Set-BeiFengEnvDefault -Name 'CLAW_RAG_SERVICE_URL' -Value $ragUrl
Set-BeiFengEnvDefault -Name 'CLAW_RAG_MEMORY_DIR' -Value $memoryPath

.\target\debug\claw.exe `
  --model $modelName `
  --allowedTools wind_knowledge_query,wind_fault_analysis,wind_report_generate
