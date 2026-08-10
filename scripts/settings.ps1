function Get-BeiFengProjectRoot {
  Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')
}

function Get-BeiFengSettings {
  $projectRoot = Get-BeiFengProjectRoot
  $settingsPath = Join-Path $projectRoot 'beifeng/config/settings.json'
  if (-not (Test-Path -LiteralPath $settingsPath)) {
    throw "Missing settings file: $settingsPath"
  }
  Get-Content -LiteralPath $settingsPath -Raw | ConvertFrom-Json
}

function Get-BeiFengSettingValue {
  param(
    [Parameter(Mandatory = $true)] $Settings,
    [Parameter(Mandatory = $true)] [string] $Key
  )
  $property = $Settings.PSObject.Properties[$Key]
  if ($null -eq $property) {
    throw "Missing settings key: $Key"
  }
  $property.Value
}

function Resolve-BeiFengPath {
  param(
    [Parameter(Mandatory = $true)] $Settings,
    [Parameter(Mandatory = $true)] [string] $Key
  )
  $value = Get-BeiFengSettingValue -Settings $Settings -Key $Key
  if ([System.IO.Path]::IsPathRooted($value)) {
    return $value
  }
  $root = Get-BeiFengProjectRoot
  Join-Path $root $value
}

function Set-BeiFengEnvDefault {
  param(
    [Parameter(Mandatory = $true)] [string] $Name,
    [Parameter(Mandatory = $true)] [string] $Value
  )
  if (-not [System.Environment]::GetEnvironmentVariable($Name, 'Process')) {
    [System.Environment]::SetEnvironmentVariable($Name, $Value, 'Process')
  }
}
