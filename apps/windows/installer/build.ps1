<#
.SYNOPSIS
    Builds the Bornopata installer on Windows: a release build of the binaries, then bornopata.iss with Inno Setup.
.DESCRIPTION
    Runs on a build machine with the MSVC toolchain and Inno Setup. Steps:
      1) cargo build --release for the Server, the DLL and the settings program, and a 32 bit DLL on its own;
      2) the version is read from apps\windows\server\Cargo.toml (a -dev version gets the short git hash);
      3) ISCC.exe is looked for (BORNOPATA_ISCC, Program Files, PATH);
      4) iscc /DAppVersion=<version>; the result is target\installer\bornopata-<version>-windows-x86_64-setup.exe.
    The data (words.bnd, lm.bnd, gloss-en.bnd) is taken from target\release-inputs, where the release workflow and
    tools\windows-fetch-data.sh put it; without it the installer still builds and the Server answers from the small
    built-in word list only.
.PARAMETER SkipBuild
    Do not run cargo (the data or the script changed, the binaries did not).
#>
[CmdletBinding()]
param([switch]$SkipBuild)

$ErrorActionPreference = 'Stop'

# The repository root: this script is in apps\windows\installer, three levels below it.
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..\..')).Path
$Iss  = Join-Path $PSScriptRoot 'bornopata.iss'

# 1) The binaries: the Server, the 64 bit DLL and the settings program, then a 32 bit DLL for 32 bit applications.
if (-not $SkipBuild) {
    Write-Host 'Building the release binaries...' -ForegroundColor Cyan
    Push-Location $Repo
    try {
        cargo build --release --locked -p borno-windows-server -p borno-windows-tsf -p borno-settings
        if ($LASTEXITCODE -ne 0) { throw "cargo build failed (exit code $LASTEXITCODE)" }
        cargo build --release --locked -p borno-windows-tsf --target i686-pc-windows-msvc
        if ($LASTEXITCODE -ne 0) { throw "the 32 bit DLL build failed (exit code $LASTEXITCODE)" }
    } finally { Pop-Location }
}

foreach ($t in @('release\bornopata_tsf.dll', 'i686-pc-windows-msvc\release\bornopata_tsf.dll', 'release\bornopata-server.exe')) {
    $p = Join-Path $Repo "target\$t"
    if (-not (Test-Path $p)) { throw "missing $p, run once without -SkipBuild" }
}

# 2) The version from the Server's Cargo.toml (each shell has its own version, not the workspace's).
$cargoToml = Get-Content (Join-Path $Repo 'apps\windows\server\Cargo.toml')
$verLine = $cargoToml | Where-Object { $_ -match '^\s*version\s*=\s*"(.+)"' } | Select-Object -First 1
if (-not ($verLine -match '"(.+)"')) { throw 'no version in apps\windows\server\Cargo.toml' }
$Version = $Matches[1]
# A development version gets the short git hash (0.1.0-dev-1a2b3c4, a + when the tree is dirty), so a bug can be pinned
# to a change; a release commit has no -dev and gets nothing.
if ($Version.EndsWith('-dev')) {
    Push-Location $Repo
    try {
        $rev = (git rev-parse --short HEAD 2>$null)
        if ($LASTEXITCODE -eq 0 -and $rev) {
            if (git status --porcelain 2>$null) { $rev = "$rev+" }
            $Version = "$Version-$rev"
        }
    } finally { Pop-Location }
}
# Inno's VersionInfoVersion takes numbers only: drop a suffix such as -alpha.1.
$VersionNumeric = $Version -replace '-.*$', ''
Write-Host "Version $Version" -ForegroundColor Cyan

# 3) ISCC.exe: Inno Setup 7 in Program Files first, then the PATH, then 6. BORNOPATA_ISCC names it directly.
$iscc = $env:BORNOPATA_ISCC
if (-not $iscc) {
    $candidates = @(
        "${env:ProgramFiles}\Inno Setup 7\ISCC.exe",
        "${env:ProgramFiles(x86)}\Inno Setup 7\ISCC.exe",
        "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
        "${env:ProgramFiles}\Inno Setup 6\ISCC.exe"
    )
    $iscc = $candidates | Where-Object { Test-Path $_ } | Select-Object -First 1
}
if (-not $iscc) { $iscc = (Get-Command iscc.exe -ErrorAction SilentlyContinue).Source }
if (-not $iscc) { throw 'ISCC.exe not found: install Inno Setup or set BORNOPATA_ISCC' }
Write-Host "Using $iscc" -ForegroundColor Cyan

# 4) The installer.
& $iscc "/DAppVersion=$Version" "/DAppVersionNumeric=$VersionNumeric" $Iss
if ($LASTEXITCODE -ne 0) { throw "iscc failed (exit code $LASTEXITCODE)" }

$out = Join-Path $Repo "target\installer\bornopata-$Version-windows-x86_64-setup.exe"
Write-Host "Done: $out" -ForegroundColor Green
