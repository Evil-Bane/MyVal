$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
Push-Location $Root
try {
    $ValidNdk = if ($env:ANDROID_NDK_HOME -and (Test-Path (Join-Path $env:ANDROID_NDK_HOME "toolchains"))) {
        $env:ANDROID_NDK_HOME
    } else {
        $null
    }

    $CandidateSdks = @(
        $env:ANDROID_HOME,
        $env:ANDROID_SDK_ROOT,
        "D:\Android",
        (Join-Path $env:LOCALAPPDATA "Android\Sdk"),
        "C:\Android"
    ) | Where-Object { $_ -and (Test-Path $_) }

    if (-not $ValidNdk) {
        foreach ($Sdk in $CandidateSdks) {
            $NdkRoot = Join-Path $Sdk "ndk"
            if (Test-Path $NdkRoot) {
                $LatestNdk = Get-ChildItem -Path $NdkRoot -Directory | Sort-Object Name -Descending | Select-Object -First 1
                if ($LatestNdk -and (Test-Path (Join-Path $LatestNdk.FullName "toolchains"))) {
                    $env:ANDROID_HOME = $Sdk
                    $ValidNdk = $LatestNdk.FullName
                    break
                }
            }
        }
    } elseif (-not $env:ANDROID_HOME -or -not (Test-Path (Join-Path $env:ANDROID_HOME "platforms"))) {
        $PreferredSdk = $CandidateSdks | Where-Object { Test-Path (Join-Path $_ "platforms") } | Select-Object -First 1
        if ($PreferredSdk) { $env:ANDROID_HOME = $PreferredSdk }
    }

    if (-not $ValidNdk) {
        throw "Could not locate an Android NDK installation. Install NDK (e.g. r27) via Android Studio SDK Manager or set ANDROID_NDK_HOME."
    }
    $env:ANDROID_NDK_HOME = $ValidNdk
    Write-Host "Using ANDROID_HOME=$($env:ANDROID_HOME)"
    Write-Host "Using ANDROID_NDK_HOME=$($env:ANDROID_NDK_HOME)"

    $JniOut = Join-Path $Root "android\app\src\main\jniLibs"
    $KtOut = Join-Path $Root "android\app\src\main\java"
    New-Item -ItemType Directory -Force -Path $JniOut | Out-Null
    New-Item -ItemType Directory -Force -Path $KtOut | Out-Null

    Write-Host "=== 1. Cross-compiling myval-ffi for arm64-v8a (release, LTO, stripped) ==="
    cargo ndk -t arm64-v8a -o $JniOut build -p myval-ffi --release
    if ($LASTEXITCODE -ne 0) { throw "cargo ndk build failed" }

    Write-Host "=== 2. Building host myval_ffi library for UniFFI metadata extraction ==="
    cargo build -p myval-ffi
    if ($LASTEXITCODE -ne 0) { throw "host cargo build -p myval-ffi failed" }

    $HostDll = Join-Path $Root "target\debug\myval_ffi.dll"
    if (-not (Test-Path $HostDll)) {
        throw "Expected host library not found at $HostDll"
    }

    Write-Host "=== 3. Generating Kotlin UniFFI bindings ==="
    cargo run -p myval-ffi --features bindgen-cli --bin uniffi-bindgen -- generate --library $HostDll --language kotlin --out-dir $KtOut
    if ($LASTEXITCODE -ne 0) { throw "uniffi-bindgen failed" }

    $SoPath = Join-Path $JniOut "arm64-v8a\libmyval_ffi.so"
    $KtPath = Join-Path $KtOut "uniffi\myval_ffi\myval_ffi.kt"
    if (-not (Test-Path $SoPath)) { throw "Missing $SoPath" }
    if (-not (Test-Path $KtPath)) { throw "Missing $KtPath" }

    Write-Host "OK: Built $SoPath ($((Get-Item $SoPath).Length) bytes)"
    Write-Host "OK: Generated $KtPath ($((Get-Item $KtPath).Length) bytes)"
}
finally {
    Pop-Location
}
