. "$PSScriptRoot\settings.ps1"

$projectRoot = Get-BeiFengProjectRoot
Set-Location $projectRoot

cargo build --manifest-path .\rust\Cargo.toml -p rusty-claude-cli
