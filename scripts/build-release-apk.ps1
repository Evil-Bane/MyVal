$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
Push-Location $Root
try {
    $CandidateJdks = @(
        $env:JAVA_HOME,
        "D:\Android\jdk-21",
        (Join-Path $env:ProgramFiles "Android\Android Studio\jbr"),
        (Join-Path $env:ProgramFiles "Java\jdk-21")
    ) | Where-Object { $_ -and (Test-Path (Join-Path $_ "bin\java.exe")) }

    $SelectedJdk = $CandidateJdks | Select-Object -First 1
    if ($SelectedJdk) {
        $env:JAVA_HOME = $SelectedJdk
        $JdkBin = Join-Path $SelectedJdk "bin"
        if (-not (($env:PATH -split ';') -contains $JdkBin)) {
            $env:PATH = "$JdkBin;$env:PATH"
        }
        Write-Host "Using JAVA_HOME=$($env:JAVA_HOME)"
    } elseif (-not (Get-Command java -ErrorAction SilentlyContinue)) {
        throw "No Java 17/21 JDK found. Set JAVA_HOME or install JDK 21 / Android Studio."
    }

    Write-Host "=== 1. Running Rust unit tests ==="
    cargo test --workspace
    if ($LASTEXITCODE -ne 0) { throw "cargo test failed" }

    Write-Host "=== 2. Building stripped LTO arm64-v8a Rust core & UniFFI bindings ==="
    & (Join-Path $Root "scripts\build-android-rust.ps1")
    if ($LASTEXITCODE -ne 0) { throw "build-android-rust.ps1 failed" }

    Write-Host "=== 3. Running Android JUnit tests & signed R8 Release APK ==="
    Push-Location (Join-Path $Root "android")
    try {
        .\gradlew.bat testDebugUnitTest assembleRelease --no-daemon
        if ($LASTEXITCODE -ne 0) { throw "Gradle release build failed" }
    }
    finally {
        Pop-Location
    }

    $GradleFile = Get-Content (Join-Path $Root "android\app\build.gradle.kts") -Raw
    $VerMatch = [regex]::Match($GradleFile, 'versionName\s*=\s*"([^"]+)"')
    $Version = if ($VerMatch.Success) { $VerMatch.Groups[1].Value } else { "1.0.0" }

    $DistDir = Join-Path $Root "dist"
    New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

    $ApkSrc = Join-Path $Root "android\app\build\outputs\apk\release\app-release.apk"
    $ApkName = "MyVal-v$Version-arm64-v8a.apk"
    $ApkDst = Join-Path $DistDir $ApkName
    Copy-Item -Force $ApkSrc $ApkDst

    $Hash = (Get-FileHash -Algorithm SHA256 -Path $ApkDst).Hash.ToLowerInvariant()
    "$Hash  $ApkName" | Set-Content -Path (Join-Path $DistDir "SHA256SUMS.txt") -Encoding ASCII

    Write-Host ""
    Write-Host "OK: Built signed release APK:"
    Write-Host "  File:   $ApkDst ($((Get-Item $ApkDst).Length) bytes)"
    Write-Host "  SHA256: $Hash"
}
finally {
    Pop-Location
}
