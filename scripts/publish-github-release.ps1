param(
    [string]$Repo = "Evil-Bane/MyVal",
    [string]$NotesFile = ""
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
Push-Location $Root
try {
    $GradleFile = Get-Content (Join-Path $Root "android\app\build.gradle.kts") -Raw
    $VerMatch = [regex]::Match($GradleFile, 'versionName\s*=\s*"([^"]+)"')
    if (-not $VerMatch.Success) { throw "Could not determine versionName from build.gradle.kts" }
    $Version = $VerMatch.Groups[1].Value
    $Tag = "v$Version"

    $DistDir = Join-Path $Root "dist"
    $ApkPath = Join-Path $DistDir "MyVal-$Tag-arm64-v8a.apk"
    $SumsPath = Join-Path $DistDir "SHA256SUMS.txt"

    if (-not (Test-Path $ApkPath)) {
        throw "Release APK not found at $ApkPath. Run scripts/build-release-apk.ps1 first."
    }
    if (-not (Test-Path $SumsPath)) {
        throw "SHA256SUMS.txt not found at $SumsPath. Run scripts/build-release-apk.ps1 first."
    }

    $ghArgs = @(
        "release", "create", $Tag,
        $ApkPath, $SumsPath,
        "--repo", $Repo,
        "--title", "MyVal $Tag",
        "--latest"
    )
    if ($NotesFile -and (Test-Path $NotesFile)) {
        $ghArgs += @("--notes-file", $NotesFile)
    } else {
        $ghArgs += @("--generate-notes")
    }

    Write-Host "Publishing $Tag to $Repo..."
    & gh @ghArgs
    if ($LASTEXITCODE -ne 0) { throw "gh release create failed" }
    Write-Host "OK: Published MyVal $Tag to https://github.com/$Repo/releases/tag/$Tag"
}
finally {
    Pop-Location
}
