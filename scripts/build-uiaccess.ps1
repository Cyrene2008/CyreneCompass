$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$thumbprint = & (Join-Path $PSScriptRoot 'ensure-uiaccess-certificate.ps1')
if ([string]::IsNullOrWhiteSpace($thumbprint)) {
  throw 'Could not prepare the UIAccess signing certificate.'
}

$env:CYRENE_UIACCESS = '1'
$env:CYRENE_UIACCESS_CERT_THUMBPRINT = $thumbprint.Trim()
$env:VITE_CYRENE_BUILD_VARIANT = 'uiaccess'

Push-Location $projectRoot
try {
  Remove-Item (Join-Path $projectRoot 'src-tauri\target\release\uiaccess-signed') -Recurse -Force -ErrorAction SilentlyContinue
  & (Join-Path $projectRoot 'node_modules\.bin\tauri.cmd') build --config src-tauri\tauri.uiaccess.conf.json
  if ($LASTEXITCODE -ne 0) { throw "Tauri UIAccess build failed with exit code $LASTEXITCODE." }
  & (Get-Command node).Source scripts\rename-installer.mjs uiaccess
  if ($LASTEXITCODE -ne 0) { throw "Installer rename failed with exit code $LASTEXITCODE." }

  $application = Join-Path $projectRoot 'src-tauri\target\release\uiaccess-signed\cyrene-compass.exe'
  $installer = Join-Path $projectRoot 'src-tauri\target\release\bundle\nsis\CyreneCompass_26.0.0_x64-setup.exe'
  foreach ($file in @($application, $installer)) {
    $signature = Get-AuthenticodeSignature $file
    if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Thumbprint -ne $env:CYRENE_UIACCESS_CERT_THUMBPRINT) {
      throw "UIAccess artifact signature verification failed: $file"
    }
  }

  $binaryText = [Text.Encoding]::UTF8.GetString([IO.File]::ReadAllBytes($application))
  if (-not $binaryText.Contains('uiAccess="true"')) {
    throw 'The release executable does not contain the uiAccess=true manifest.'
  }
} finally {
  Pop-Location
  Remove-Item Env:CYRENE_UIACCESS -ErrorAction SilentlyContinue
  Remove-Item Env:CYRENE_UIACCESS_CERT_THUMBPRINT -ErrorAction SilentlyContinue
  Remove-Item Env:VITE_CYRENE_BUILD_VARIANT -ErrorAction SilentlyContinue
}
