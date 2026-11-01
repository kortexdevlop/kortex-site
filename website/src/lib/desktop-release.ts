export const desktopRelease={version:'1.0.0',filename:'KortexDesktop_1.0.0_x64-setup.exe',url:'https://downloads.kortex.site/KortexDesktop_1.0.0_x64-setup.exe',sha256:'758b60b87178efc1c52c2af2b4adf5d5efab56f779f20eaf299232766a04f0f5'};
export const installCommand='irm https://kortex.site/install.ps1 | iex';
export const installCommandCmd='powershell -c "irm https://kortex.site/install.ps1 | iex"';
// Runs inside a script block: iex would otherwise leak preferences into the caller's session.
// Cleanup uses .NET: Remove-Item fails on the 8.3 TEMP paths of non-ASCII Windows profiles and would hide the real error.
export const installScript=({version,filename,url,sha256}=desktopRelease)=>`# Kortex Desktop ${version} pour Windows - https://kortex.site/telechargement
& {
  $ErrorActionPreference = 'Stop'
  $ProgressPreference = 'SilentlyContinue'
  if (-not [Environment]::Is64BitOperatingSystem) { throw 'Kortex Desktop nécessite Windows 64 bits.' }
  $work = Join-Path $env:TEMP ('kortex-' + [guid]::NewGuid())
  New-Item -ItemType Directory -Path $work | Out-Null
  try {
    $installer = Join-Path $work '${filename}'
    Write-Host 'Téléchargement de Kortex Desktop ${version} (environ 215 Mio)...'
    Invoke-WebRequest -UseBasicParsing -Uri '${url}' -OutFile $installer
    if ((Get-FileHash $installer -Algorithm SHA256).Hash -ne '${sha256}') { throw 'Empreinte SHA-256 incorrecte : installation annulée.' }
    Write-Host 'Empreinte vérifiée. Installation...'
    $setup = Start-Process -FilePath $installer -ArgumentList '/S' -Wait -PassThru
    if ($setup.ExitCode -ne 0) { throw "L'installation a échoué (code $($setup.ExitCode))." }
  } finally {
    try { [IO.Directory]::Delete($work, $true) } catch {}
  }
  $app = Join-Path $env:LOCALAPPDATA 'Programs\\KortexDesktop\\kortex-desktop.exe'
  if (-not (Test-Path -LiteralPath $app)) { throw 'Kortex Desktop est introuvable après installation.' }
  Write-Host 'Kortex Desktop est installé.'
  Start-Process -FilePath $app
}
`;
