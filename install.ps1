#!/usr/bin/env pwsh

$ErrorActionPreference = 'Stop'

$DuneRoot = $env:DUNE_ROOT
$BinDir = if ($DuneInstall) {
    "$DuneRoot\bin"
}
else {
    "$Home\.dune\bin"
}

$DuneZip = "$BinDir\dune.zip"
$DuneExe = "$BinDir\dune.exe"
$Target = 'x86_64-pc-windows-msvc'

# GitHub requires TLS 1.2
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$DuneUri = "https://github.com/aalykiot/dune/releases/latest/download/dune-${Target}.zip"

if (!(Test-Path $BinDir)) {
    New-Item $BinDir -ItemType Directory | Out-Null
}

curl.exe -Lo $DuneZip $DuneUri

tar.exe xf $DuneZip -C $BinDir

Remove-Item $DuneZip

$User = [EnvironmentVariableTarget]::User
$Path = [Environment]::GetEnvironmentVariable('Path', $User)
if (!(";$Path;".ToLower() -like "*;$BinDir;*".ToLower())) {
    [Environment]::SetEnvironmentVariable('Path', "$Path;$BinDir", $User)
    $Env:Path += ";$BinDir"
}

Write-Output "Dune was installed successfully to $DuneExe"
Write-Output "Run 'dune --help' to get started"