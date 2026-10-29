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
    $taskRuntimeKeys = @('HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}','HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}','HKCU:\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}')
    $taskRuntimeVersions = @($taskRuntimeKeys | ForEach-Object { $taskVersion = (Get-ItemProperty -LiteralPath $_ -ErrorAction SilentlyContinue).pv; if ($taskVersion -match '^\d+\.\d+\.\d+\.\d+$' -and $taskVersion -ne '0.0.0.0') { $taskVersion } })
    @{os_version=[Environment]::OSVersion.Version.ToString();runtime_versions=$taskRuntimeVersions;browser_override=$env:WEBVIEW2_BROWSER_EXECUTABLE_FOLDER} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskLogDir 'webview2.json')
    if ($taskRuntimeVersions.Count -eq 0) { throw 'Bundled WebView2 installer did not register the Evergreen Runtime' }
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
    if ($null -ne $taskFailure -and $env:GITHUB_ACTIONS -eq 'true') {
        foreach ($taskLog in @((Join-Path $env:ProgramData 'Microsoft/EdgeUpdate/Log/MicrosoftEdgeUpdate.log'),(Join-Path $env:TEMP 'msedge_installer.log'),(Join-Path $env:LOCALAPPDATA 'Temp/msedge_installer.log'))) {
            if (Test-Path -LiteralPath $taskLog -PathType Leaf) { Get-Content -LiteralPath $taskLog -Tail 100 | Set-Content -LiteralPath (Join-Path $taskLogDir ([IO.Path]::GetFileName($taskLog))) }
        }
    }
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
