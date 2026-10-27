$ErrorActionPreference='Stop'
$repoRoot=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $repoRoot
$app=$null
$oldArguments=$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
try {
    cargo build -p kortex-runtime-desktop --locked
    if ($LASTEXITCODE -ne 0) { throw 'Desktop build failed' }
    node tests/fixtures/prepare-calculatrice.mjs
    if ($LASTEXITCODE -ne 0) { throw 'Calculator fixture failed' }
    $target=if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $repoRoot 'target' }
    $exe=(Resolve-Path (Join-Path $target 'debug/kortex-runtime-desktop.exe')).Path
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9332 --remote-allow-origins=*'
    $app=Start-Process -FilePath $exe -WorkingDirectory $repoRoot -ArgumentList @('target/calculatrice-fixture/calculatrice.tool','target/calculatrice-fixture/trust.json') -WindowStyle Hidden -PassThru
    node tests/e2e/calculatrice-desktop.cjs
    if ($LASTEXITCODE -ne 0) { throw 'Calculator Windows UI failed' }
} finally {
    if ($app -and -not $app.HasExited) { Stop-Process -Id $app.Id -ErrorAction SilentlyContinue }
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=$oldArguments
    Pop-Location
}
