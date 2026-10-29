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
$taskSetup = Start-Process -FilePath $taskInstaller -ArgumentList @('/S',"/D=$taskInstallPath") -WindowStyle Hidden -PassThru -Wait
if ($taskSetup.ExitCode -ne 0) { throw "Installer exit $($taskSetup.ExitCode)" }
$taskExe = Join-Path $taskInstallPath 'kortex-desktop.exe'
if (-not (Test-Path -LiteralPath $taskExe -PathType Leaf)) { throw 'Installed application missing' }
$env:KORTEX_DESKTOP_DATA_DIR = Join-Path $taskInstallPath 'smoke-profile'
$taskApp = Start-Process -FilePath $taskExe -WindowStyle Hidden -PassThru
Start-Sleep -Seconds 4
if ($taskApp.HasExited) { throw "Application exited: $($taskApp.ExitCode)" }
Stop-Process -Id $taskApp.Id -Force
$taskUninstaller = Get-ChildItem -LiteralPath $taskInstallPath -Filter '*uninstall*.exe' -File | Select-Object -First 1
if (-not $taskUninstaller) { throw 'Uninstaller missing' }
$taskUninstall = Start-Process -FilePath $taskUninstaller.FullName -ArgumentList @('/S',"_?=$taskInstallPath") -WindowStyle Hidden -PassThru -Wait
if ($taskUninstall.ExitCode -ne 0) { throw 'Uninstall failed' }
if (Test-Path -LiteralPath $taskExe) { throw 'Application remains after uninstall' }
@{passed=$true;platform='windows-x64';installed=$true;process_start=$true;uninstalled=$true;visual_interactions=$false;production_signed=$false} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $taskWorkspace 'target/desktop-package/installer-smoke.json')
