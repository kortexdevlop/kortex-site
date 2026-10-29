param([Parameter(Mandatory=$true)][string]$Installer,[Parameter(Mandatory=$true)][string]$Destination)
$ErrorActionPreference = 'Stop'
$taskWorkspace = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$taskInstallPath = [IO.Path]::GetFullPath($Destination)
if (-not $taskInstallPath.StartsWith((Join-Path $taskWorkspace 'target') + [IO.Path]::DirectorySeparatorChar,[StringComparison]::OrdinalIgnoreCase)) { throw 'Installer smoke must stay under workspace target' }
if (Test-Path -LiteralPath $taskInstallPath) { throw 'Use a fresh smoke destination' }
foreach ($taskExisting in @((Join-Path $env:APPDATA 'Microsoft/Windows/Start Menu/Programs/KortexDesktop'),(Join-Path ([Environment]::GetFolderPath('Desktop')) 'KortexDesktop.lnk'),'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\site.kortex.desktop')) {
    if (Test-Path -LiteralPath $taskExisting) { throw 'An existing Kortex installation or shortcut must be preserved; use the fresh CI runner for smoke' }
}
$taskInstaller = (Resolve-Path -LiteralPath $Installer).Path
$taskLogDir = Join-Path $taskWorkspace 'target/desktop-package/smoke-logs'
New-Item -ItemType Directory -Path $taskLogDir -Force | Out-Null
$taskOldEnv = @{}
foreach ($taskName in @('KORTEX_DESKTOP_DATA_DIR','WEBVIEW2_USER_DATA_FOLDER','RUST_BACKTRACE')) { $taskOldEnv[$taskName] = [Environment]::GetEnvironmentVariable($taskName) }
$taskApp = $null
$taskFailure = $null
$taskInstalled = $false
$taskStarted = $false
$taskUninstalled = $false
try {
    $taskSetup = Start-Process -FilePath $taskInstaller -ArgumentList @('/S',"/D=$taskInstallPath") -WindowStyle Hidden -PassThru -Wait
    if ($taskSetup.ExitCode -ne 0) { throw "Installer exit $($taskSetup.ExitCode)" }
    $taskInstalled = $true
    $taskExe = Join-Path $taskInstallPath 'kortex-desktop.exe'
    if (-not (Test-Path -LiteralPath $taskExe -PathType Leaf)) { throw 'Installed application missing' }
    $env:KORTEX_DESKTOP_DATA_DIR = Join-Path $taskInstallPath 'smoke-profile'
    $env:WEBVIEW2_USER_DATA_FOLDER = Join-Path $taskInstallPath 'smoke-webview'
    $env:RUST_BACKTRACE = '1'
    $taskApp = Start-Process -FilePath $taskExe -WindowStyle Hidden -PassThru -RedirectStandardOutput (Join-Path $taskLogDir 'stdout.log') -RedirectStandardError (Join-Path $taskLogDir 'stderr.log')
    Start-Sleep -Seconds 4
    if ($taskApp.HasExited) { throw "Application exited: $($taskApp.ExitCode)" }
    $taskStarted = $true
} catch { $taskFailure = $_ }
finally {
    try {
        if ($null -ne $taskApp -and -not $taskApp.HasExited) { Stop-Process -Id $taskApp.Id -Force }
        if ($taskInstalled) {
            $taskUninstaller = Get-ChildItem -LiteralPath $taskInstallPath -Filter '*uninstall*.exe' -File | Select-Object -First 1
            if (-not $taskUninstaller) { throw 'Uninstaller missing' }
            $taskUninstall = Start-Process -FilePath $taskUninstaller.FullName -ArgumentList @('/S',"_?=$taskInstallPath") -WindowStyle Hidden -PassThru -Wait
            if ($taskUninstall.ExitCode -ne 0) { throw 'Uninstall failed' }
            if (Test-Path -LiteralPath (Join-Path $taskInstallPath 'kortex-desktop.exe')) { throw 'Application remains after uninstall' }
            $taskUninstalled = $true
        }
    } catch { if ($null -eq $taskFailure) { $taskFailure = $_ } else { Write-Warning $_.Exception.Message } }
    foreach ($taskName in $taskOldEnv.Keys) { [Environment]::SetEnvironmentVariable($taskName,$taskOldEnv[$taskName]) }
    @{passed=($null -eq $taskFailure -and $taskStarted -and $taskUninstalled);platform='windows-x64';installed=$taskInstalled;process_start=$taskStarted;uninstalled=$taskUninstalled;visual_interactions=$false;production_signed=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskWorkspace 'target/desktop-package/installer-smoke.json')
}
if ($null -ne $taskFailure) {
    Get-Content -LiteralPath (Join-Path $taskLogDir 'stderr.log') -Tail 45 -ErrorAction SilentlyContinue
    throw $taskFailure
}
