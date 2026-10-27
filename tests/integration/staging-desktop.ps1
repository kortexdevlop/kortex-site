$ErrorActionPreference='Stop'
$repoRoot=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$reportPath=Join-Path $repoRoot 'target/staging-cloud/report.json'
$gate=Get-Content -LiteralPath $reportPath -Raw -Encoding UTF8 | ConvertFrom-Json
if (-not $gate.cloud_roundtrip_passed) { throw 'Real cloud roundtrip required before the Windows host gate' }
$cacheRoot=if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $repoRoot 'target' }
$exe=(Resolve-Path (Join-Path $cacheRoot 'debug/kortex-runtime-desktop.exe')).Path
$previousArgs=$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
$previousReport=$env:KORTEX_GATE_REPORT
$previousScreenshot=$env:KORTEX_GATE_SCREENSHOT
$app=$null
Push-Location $repoRoot
try {
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9332 --remote-allow-origins=*'
    $env:KORTEX_GATE_REPORT=$reportPath
    $env:KORTEX_GATE_SCREENSHOT='docs/evidence/staging9-windows.png'
    $app=Start-Process -FilePath $exe -WorkingDirectory $repoRoot -ArgumentList @($gate.archive,$gate.host_trust) -WindowStyle Hidden -PassThru
    node tests/e2e/calculatrice-desktop.cjs
    if ($LASTEXITCODE -ne 0) { throw 'Cloud artifact Windows host gate failed' }
} finally {
    if ($app -and -not $app.HasExited) { Stop-Process -Id $app.Id -ErrorAction SilentlyContinue }
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=$previousArgs
    $env:KORTEX_GATE_REPORT=$previousReport
    $env:KORTEX_GATE_SCREENSHOT=$previousScreenshot
    Pop-Location
}
