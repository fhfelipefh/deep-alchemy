#!/usr/bin/env bash
set -e

MODE="${1:-release}"
TARGET="${2:-all}"
ABI="${3:-arm64-v8a}"

echo "========================================="
echo "   DEEP ALCHEMY - ANDROID BUILD SYSTEM   "
echo "========================================="

# 1. Auto-detect Android SDK
if [ -z "$ANDROID_HOME" ] && [ -z "$ANDROID_SDK_ROOT" ]; then
    if [ -d "$HOME/Android/Sdk" ]; then
        export ANDROID_HOME="$HOME/Android/Sdk"
        export ANDROID_SDK_ROOT="$HOME/Android/Sdk"
    elif [ -d "$HOME/Library/Android/sdk" ]; then
        export ANDROID_HOME="$HOME/Library/Android/sdk"
        export ANDROID_SDK_ROOT="$HOME/Library/Android/sdk"
    fi
elif [ -z "$ANDROID_HOME" ]; then
    export ANDROID_HOME="$ANDROID_SDK_ROOT"
fi

# 2. Auto-detect Android NDK
if [ -z "$ANDROID_NDK_HOME" ] && [ -z "$NDK_HOME" ]; then
    if [ -n "$ANDROID_HOME" ] && [ -d "$ANDROID_HOME/ndk" ]; then
        LATEST_NDK=$(ls -1d "$ANDROID_HOME/ndk"/* 2>/dev/null | sort -V | tail -n 1)
        if [ -n "$LATEST_NDK" ]; then
            export ANDROID_NDK_HOME="$LATEST_NDK"
            export NDK_HOME="$LATEST_NDK"
        fi
    fi
elif [ -z "$ANDROID_NDK_HOME" ]; then
    export ANDROID_NDK_HOME="$NDK_HOME"
fi

# 3. Create local.properties if missing
if [ ! -f "android/local.properties" ] && [ -n "$ANDROID_HOME" ]; then
    echo "sdk.dir=$ANDROID_HOME" > android/local.properties
    if [ -n "$ANDROID_NDK_HOME" ]; then
        echo "ndk.dir=$ANDROID_NDK_HOME" >> android/local.properties
    fi
fi

# 4. Install targets
echo "[1/3] Ensuring Rust Android target is installed..."
if [ "$ABI" = "all" ]; then
    rustup target add aarch64-linux-android x86_64-linux-android
else
    rustup target add aarch64-linux-android
fi

# 5. Compile native libraries via cargo-ndk
RELEASE_FLAG=""
if [ "$MODE" = "release" ]; then
    RELEASE_FLAG="--release"
fi

mkdir -p android/app/src/main/jniLibs
echo "[2/3] Compiling native game library via cargo-ndk ($MODE, ABI: $ABI)..."
if [ "$ABI" = "all" ]; then
    cargo ndk -t arm64-v8a -t x86_64 -o android/app/src/main/jniLibs build $RELEASE_FLAG
else
    cargo ndk -t arm64-v8a -o android/app/src/main/jniLibs build $RELEASE_FLAG
fi

# 6. Package AAB and/or APK via Gradle
echo "[3/3] Packaging Android artifacts via Gradle..."
cd android
chmod +x gradlew 2>/dev/null || true

TASK_MODE="Release"
if [ "$MODE" = "debug" ]; then
    TASK_MODE="Debug"
fi

if [ "$TARGET" = "bundle" ] || [ "$TARGET" = "all" ]; then
    ./gradlew "bundle$TASK_MODE"
fi

if [ "$TARGET" = "apk" ] || [ "$TARGET" = "all" ]; then
    ./gradlew "assemble$TASK_MODE"
fi

cd ..

echo "========================================="
echo "   BUILD SUCCESSFUL! ARTIFACT LOCATIONS  "
echo "========================================="
MODE_LOWER=$(echo "$MODE" | tr '[:upper:]' '[:lower:]')
AAB_PATH="android/app/build/outputs/bundle/${MODE_LOWER}/app-${MODE_LOWER}.aab"
APK_PATH="android/app/build/outputs/apk/${MODE_LOWER}/app-${MODE_LOWER}.apk"

if [ -f "$AAB_PATH" ]; then
    echo ">> Play Store App Bundle (AAB): $AAB_PATH"
fi
if [ -f "$APK_PATH" ]; then
    echo ">> Direct Installable APK: $APK_PATH"
fi
echo "========================================="
