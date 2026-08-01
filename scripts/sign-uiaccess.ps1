param(
  [Parameter(Mandatory = $true)]
  [string]$Path
)

$ErrorActionPreference = 'Stop'
$thumbprint = $env:CYRENE_UIACCESS_CERT_THUMBPRINT
if ([string]::IsNullOrWhiteSpace($thumbprint)) {
  throw 'CYRENE_UIACCESS_CERT_THUMBPRINT is not set.'
}

$certificate = Get-Item "Cert:\CurrentUser\My\$thumbprint" -ErrorAction Stop
$signature = Set-AuthenticodeSignature -FilePath $Path -Certificate $certificate -HashAlgorithm SHA256
if ($signature.Status -ne 'Valid') {
  throw "Signing failed for $Path with status $($signature.Status): $($signature.StatusMessage)"
}

$resolvedPath = (Resolve-Path $Path).Path
if ([IO.Path]::GetFileName($resolvedPath) -eq 'cyrene-compass.exe') {
  $projectRoot = Split-Path -Parent $PSScriptRoot
  $verificationDirectory = Join-Path $projectRoot 'src-tauri\target\release\uiaccess-signed'
  New-Item -ItemType Directory -Path $verificationDirectory -Force | Out-Null
  Copy-Item -LiteralPath $resolvedPath -Destination (Join-Path $verificationDirectory 'cyrene-compass.exe') -Force
}

Write-Output "Signed: $Path"
