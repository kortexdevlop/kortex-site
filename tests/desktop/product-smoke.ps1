$ErrorActionPreference='Stop'
$repoRoot=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$exe=(Resolve-Path (Join-Path $repoRoot 'target/debug/kortex-desktop.exe')).Path
$testRoot=Join-Path $repoRoot 'target/desktop-product-smoke'
if((Get-NetTCPConnection -LocalPort 9342 -State Listen -ErrorAction SilentlyContinue)){throw 'Port de test déjà utilisé.'}
$oldArgs=$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
$oldData=$env:KORTEX_DESKTOP_DATA_DIR
$oldTrust=$env:KORTEX_DESKTOP_TRUST
$oldOffline=$env:KORTEX_DESKTOP_OFFLINE
$oldWebView=$env:WEBVIEW2_USER_DATA_FOLDER
Push-Location $repoRoot
try{
 $release=Get-Content 'target/phase12/convertisseur-casse/release-report.json' -Raw -Encoding UTF8|ConvertFrom-Json
 $folder=Join-Path $testRoot 'installed/convertisseur-casse/1.0.0'
 New-Item -ItemType Directory -Path $folder -Force|Out-Null
 Copy-Item -LiteralPath $release.archive -Destination (Join-Path $folder 'package.tool') -Force
 Copy-Item -LiteralPath $release.descriptor -Destination (Join-Path $folder 'package.tool.json') -Force
 $env:KORTEX_DESKTOP_DATA_DIR=$testRoot
 $env:KORTEX_DESKTOP_TRUST=$release.host_trust
 $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9342 --remote-allow-origins=*'
 foreach($mode in @('0','1')){
  $app=$null
  try{
   $env:KORTEX_DESKTOP_OFFLINE=$mode
   $env:WEBVIEW2_USER_DATA_FOLDER=Join-Path $testRoot ('webview-'+$mode)
   $app=Start-Process -FilePath $exe -WorkingDirectory $repoRoot -WindowStyle Hidden -PassThru -RedirectStandardError (Join-Path $testRoot ('stderr-'+$mode+'.log'))
   node tests/desktop/product-smoke.cjs
   if($LASTEXITCODE -ne 0){throw 'Échec du test produit Windows.'}
  }finally{if($app -and -not $app.HasExited){Stop-Process -Id $app.Id -ErrorAction SilentlyContinue}}
 }
}finally{
 $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=$oldArgs
 $env:KORTEX_DESKTOP_DATA_DIR=$oldData
 $env:KORTEX_DESKTOP_TRUST=$oldTrust
 $env:KORTEX_DESKTOP_OFFLINE=$oldOffline
 $env:WEBVIEW2_USER_DATA_FOLDER=$oldWebView
 Pop-Location
}
