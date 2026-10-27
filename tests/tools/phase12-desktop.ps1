$ErrorActionPreference='Stop'
$repoRoot=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$releases=Get-Content -LiteralPath (Join-Path $repoRoot 'target/phase12/releases.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$exe=(Resolve-Path (Join-Path $repoRoot 'target/debug/kortex-runtime-desktop.exe')).Path
$previousArguments=$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
$previousTool=$env:KORTEX_PHASE12_TOOL
Push-Location $repoRoot
try {
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9332 --remote-allow-origins=*'
    foreach($release in $releases) {
        $app=$null
        try {
            $env:KORTEX_PHASE12_TOOL=$release.tool_id
            $app=Start-Process -FilePath $exe -WorkingDirectory $repoRoot -ArgumentList @($release.archive,$release.host_trust) -WindowStyle Hidden -PassThru
            node tests/tools/phase12-desktop.cjs
            if($LASTEXITCODE -ne 0){throw "Windows tool gate failed: $($release.tool_id)"}
        } finally {if($app -and -not $app.HasExited){Stop-Process -Id $app.Id -ErrorAction SilentlyContinue}}
    }
} finally {
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=$previousArguments
    $env:KORTEX_PHASE12_TOOL=$previousTool
    Pop-Location
}
