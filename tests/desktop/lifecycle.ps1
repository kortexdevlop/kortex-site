$ErrorActionPreference='Stop'
$root=(Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$info=Get-Content -LiteralPath $env:KORTEX_LIFECYCLE_INFO -Raw -Encoding UTF8|ConvertFrom-Json
$testRoot=[IO.Path]::GetFullPath($info.data_dir)
$expected=[IO.Path]::GetFullPath((Join-Path $root 'target/desktop-lifecycle'))
if(-not $testRoot.StartsWith($expected+[IO.Path]::DirectorySeparatorChar)){throw 'Dossier de test hors périmètre.'}
if([Net.NetworkInformation.IPGlobalProperties]::GetIPGlobalProperties().GetActiveTcpListeners().Port -contains 9343){throw 'Port de test déjà occupé.'}
$exe=(Resolve-Path (Join-Path $root 'target/debug/kortex-desktop.exe')).Path
New-Item -ItemType Directory -Path $testRoot -Force|Out-Null
$names=@('KORTEX_DESKTOP_DATA_DIR','KORTEX_DESKTOP_TRUST','KORTEX_DESKTOP_OFFLINE','KORTEX_DESKTOP_API_ORIGIN','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')
$old=@{};foreach($name in $names){$old[$name]=[Environment]::GetEnvironmentVariable($name)}
$app=$null
Push-Location $root
try{
 $env:KORTEX_DESKTOP_DATA_DIR=$testRoot
 $env:KORTEX_DESKTOP_TRUST=$info.trust_path
 $env:KORTEX_DESKTOP_OFFLINE='0'
 $env:KORTEX_DESKTOP_API_ORIGIN='https://kortex-api-staging.userpgl.workers.dev'
 $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9343 --remote-allow-origins=*'
 $env:WEBVIEW2_USER_DATA_FOLDER=Join-Path $testRoot 'webview'
 $app=Start-Process -FilePath $exe -WorkingDirectory $root -WindowStyle Hidden -PassThru -RedirectStandardError (Join-Path $testRoot 'stderr.log')
 node tests/desktop/lifecycle.cjs
 if($LASTEXITCODE -ne 0){throw 'Échec du parcours installation/mise à jour Windows.'}
}finally{
 if($app -and -not $app.HasExited){Stop-Process -Id $app.Id -ErrorAction SilentlyContinue}
 foreach($name in $names){[Environment]::SetEnvironmentVariable($name,$old[$name])}
 Pop-Location
}
