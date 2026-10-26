param(
    [string]$Target = "all",
    [string]$Mode = "release",
    [string]$Abi = "arm64-v8a",
    [string]$Tool = "ndk"
)

$ErrorActionPreference = "Stop"

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "   DEEP ALCHEMY - ANDROID BUILD SYSTEM   " -ForegroundColor Yellow
Write-Host "=========================================" -ForegroundColor Cyan

# 1. Auto-detect Android SDK
if (-not $env:ANDROID_HOME -and -not $env:ANDROID_SDK_ROOT) {
    $defaultSdk = Join-Path $env:LOCALAPPDATA "Android\Sdk"
    if (Test-Path $defaultSdk) {
        $env:ANDROID_HOME = $defaultSdk
        $env:ANDROID_SDK_ROOT = $defaultSdk
        Write-Host "Auto-detected Android SDK: $defaultSdk" -ForegroundColor Green
    } else {
        Write-Warning "ANDROID_HOME or ANDROID_SDK_ROOT not found. Please install Android Studio or SDK commandline tools."
    }
} elseif (-not $env:ANDROID_HOME) {
    $env:ANDROID_HOME = $env:ANDROID_SDK_ROOT
}

# 2. Auto-detect Android NDK
if (-not $env:ANDROID_NDK_HOME -and -not $env:NDK_HOME) {
    if ($env:ANDROID_HOME) {
        $ndkDir = Join-Path $env:ANDROID_HOME "ndk"
        if (Test-Path $ndkDir) {
            $latestNdk = Get-ChildItem $ndkDir | Sort-Object Name -Descending | Select-Object -First 1
            if ($latestNdk) {
                $env:ANDROID_NDK_HOME = $latestNdk.FullName
                $env:NDK_HOME = $latestNdk.FullName
                Write-Host "Auto-detected Android NDK: $($latestNdk.FullName)" -ForegroundColor Green
            }
        }
    }
} elseif (-not $env:ANDROID_NDK_HOME) {
    $env:ANDROID_NDK_HOME = $env:NDK_HOME
}

# 3. Ensure local.properties exists for Gradle
$localProps = "android/local.properties"
if (-not (Test-Path $localProps)) {
    $escapedSdk = $env:ANDROID_HOME.Replace("\", "\\")
    $propsContent = "sdk.dir=$escapedSdk`n"
    if ($env:ANDROID_NDK_HOME) {
        $escapedNdk = $env:ANDROID_NDK_HOME.Replace("\", "\\")
        $propsContent += "ndk.dir=$escapedNdk`n"
    }
    Set-Content -Path $localProps -Value $propsContent
    Write-Host "Created android/local.properties" -ForegroundColor Green
}

# 4. Check / Install cargo-ndk if needed
$cargoNdk = Get-Command cargo-ndk -ErrorAction SilentlyContinue
if (-not $cargoNdk) {
    Write-Host "cargo-ndk not found. Installing cargo-ndk..." -ForegroundColor Yellow
    cargo install cargo-ndk
}

# 5. Add Rust compilation targets
Write-Host "[1/3] Ensuring Rust Android target is installed..." -ForegroundColor Green
if ($Abi -eq "all") {
    rustup target add aarch64-linux-android x86_64-linux-android
} else {
    rustup target add aarch64-linux-android
}

# 6. Native compilation via cargo-ndk
$releaseFlag = if ($Mode -eq "release") { "--release" } else { "" }
$jniPath = "android/app/src/main/jniLibs"
if (-not (Test-Path $jniPath)) {
    New-Item -ItemType Directory -Path $jniPath -Force | Out-Null
}

Write-Host "[2/3] Compiling native game library via cargo-ndk ($Mode, ABI: $Abi)..." -ForegroundColor Green
if ($Abi -eq "all") {
    if ($releaseFlag) {
        cargo ndk -t arm64-v8a -t x86_64 -o $jniPath build --release
    } else {
        cargo ndk -t arm64-v8a -t x86_64 -o $jniPath build
    }
} else {
    if ($releaseFlag) {
        cargo ndk -t arm64-v8a -o $jniPath build --release
    } else {
        cargo ndk -t arm64-v8a -o $jniPath build
    }
}

# 7. Package AAB and/or APK via Gradle
Write-Host "[3/3] Packaging Android artifacts via Gradle..." -ForegroundColor Green
Push-Location android

$gradleCmd = ".\gradlew.bat"
if (-not (Test-Path $gradleCmd)) {
    $gradleCmd = "gradle"
}

$taskMode = if ($Mode -eq "release") { "Release" } else { "Debug" }

if ($Target -eq "bundle" -or $Target -eq "all") {
    Write-Host "Building App Bundle (bundle$taskMode)..." -ForegroundColor Yellow
    & $gradleCmd "bundle$taskMode"
}

if ($Target -eq "apk" -or $Target -eq "all") {
    Write-Host "Building APK (assemble$taskMode)..." -ForegroundColor Yellow
    & $gradleCmd "assemble$taskMode"
}

Pop-Location

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "   BUILD SUCCESSFUL! ARTIFACT LOCATIONS  " -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Cyan

$aabPath = "android/app/build/outputs/bundle/$($Mode.ToLower())/app-$($Mode.ToLower()).aab"
$apkPath = "android/app/build/outputs/apk/$($Mode.ToLower())/app-$($Mode.ToLower()).apk"

if (Test-Path $aabPath) {
    $aabItem = Get-Item $aabPath
    Write-Host ">> Play Store App Bundle (AAB):" -ForegroundColor Green
    Write-Host "   Path: $($aabItem.FullName)" -ForegroundColor White
    Write-Host "   Size: $([math]::Round($aabItem.Length / 1MB, 2)) MB" -ForegroundColor White
}

if (Test-Path $apkPath) {
    $apkItem = Get-Item $apkPath
    Write-Host ">> Direct Installable APK:" -ForegroundColor Green
    Write-Host "   Path: $($apkItem.FullName)" -ForegroundColor White
    Write-Host "   Size: $([math]::Round($apkItem.Length / 1MB, 2)) MB" -ForegroundColor White
}
Write-Host "=========================================" -ForegroundColor Cyan
