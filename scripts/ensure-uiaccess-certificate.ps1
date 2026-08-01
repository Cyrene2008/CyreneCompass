param(
  [string]$Subject = 'CN=CyreneCompass UIAccess'
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$certificateDirectory = Join-Path $projectRoot 'src-tauri\certificates'
$publicCertificate = Join-Path $certificateDirectory 'CyreneCompassUIAccess.cer'
$now = Get-Date

$certificate = Get-ChildItem Cert:\CurrentUser\My |
  Where-Object {
    $_.Subject -eq $Subject -and
    $_.HasPrivateKey -and
    $_.NotBefore -le $now -and
    $_.NotAfter -gt $now.AddMonths(3) -and
    ($_.EnhancedKeyUsageList.ObjectId -contains '1.3.6.1.5.5.7.3.3')
  } |
  Sort-Object NotAfter -Descending |
  Select-Object -First 1

if (-not $certificate) {
  $certificate = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject $Subject `
    -FriendlyName 'CyreneCompass UIAccess Code Signing' `
    -CertStoreLocation 'Cert:\CurrentUser\My' `
    -KeyAlgorithm RSA `
    -KeyLength 3072 `
    -HashAlgorithm SHA256 `
    -KeyExportPolicy ExportableEncrypted `
    -NotAfter $now.AddYears(10)
}

New-Item -ItemType Directory -Path $certificateDirectory -Force | Out-Null
Export-Certificate -Cert $certificate -FilePath $publicCertificate -Force | Out-Null

foreach ($store in @('Cert:\CurrentUser\Root', 'Cert:\CurrentUser\TrustedPublisher')) {
  if (-not (Test-Path (Join-Path $store $certificate.Thumbprint))) {
    Import-Certificate -FilePath $publicCertificate -CertStoreLocation $store | Out-Null
  }
}

$certificate.Thumbprint
