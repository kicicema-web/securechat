#!/bin/bash
# Build script for SecureChat Android APK
# Run this on an x86_64 Linux machine with Android SDK installed

set -e

echo "🔐 SecureChat Android Build Script"
echo "==================================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[*]${NC} $1"
}

print_error() {
    echo -e "${RED}[!]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Check if running on x86_64
ARCH=$(uname -m)
if [ "$ARCH" != "x86_64" ]; then
    print_error "This build requires an x86_64 architecture. Current: $ARCH"
    print_error "Please run this script on an x86_64 machine or use a CI/CD service."
    exit 1
fi

# Check dependencies
check_deps() {
    print_status "Checking dependencies..."
    
    if ! command -v java &> /dev/null; then
        print_error "Java not found. Please install OpenJDK 17"
        exit 1
    fi
    
    JAVA_VERSION=$(java -version 2>&1 | head -1 | cut -d'"' -f2)
    print_status "Found Java: $JAVA_VERSION"
    
    if [ -z "$ANDROID_SDK_ROOT" ] && [ -z "$ANDROID_HOME" ]; then
        print_error "ANDROID_SDK_ROOT or ANDROID_HOME not set"
        print_error "Please set one of these environment variables"
        exit 1
    fi
    
    print_status "Android SDK found"
}

# Build the APK
build_apk() {
    print_status "Building Android APK..."
    
    cd android
    
    # Clean previous builds
    ./gradlew clean
    
    # Build debug APK
    ./gradlew assembleDebug
    
    # Check if APK was created
    APK_PATH="app/build/outputs/apk/debug/app-debug.apk"
    if [ -f "$APK_PATH" ]; then
        print_status "Debug APK built successfully!"
        print_status "APK location: $PWD/$APK_PATH"
        print_status "APK size: $(du -h $APK_PATH | cut -f1)"
        
        # Also build release
        print_status "Building release APK..."
        ./gradlew assembleRelease
        
        RELEASE_PATH="app/build/outputs/apk/release/app-release-unsigned.apk"
        if [ -f "$RELEASE_PATH" ]; then
            print_status "Release APK built successfully!"
            print_status "APK location: $PWD/$RELEASE_PATH"
            print_status "APK size: $(du -h $RELEASE_PATH | cut -f1)"
            print_warning "Release APK is unsigned. Sign it before distribution."
        fi
    else
        print_error "APK build failed!"
        exit 1
    fi
    
    cd ..
}

# Install APK to connected device (optional)
install_apk() {
    if command -v adb &> /dev/null; then
        if adb devices | grep -q "device$"; then
            print_status "Installing APK to connected device..."
            adb install -r android/app/build/outputs/apk/debug/app-debug.apk
            print_status "APK installed successfully!"
        else
            print_warning "No Android device connected. Skipping installation."
        fi
    else
        print_warning "ADB not found. Skipping installation."
    fi
}

# Main
main() {
    check_deps
    build_apk
    
    if [ "${1:-}" = "--install" ]; then
        install_apk
    fi
    
    print_status "Build complete!"
    print_status "Debug APK: android/app/build/outputs/apk/debug/app-debug.apk"
    print_status "Release APK: android/app/build/outputs/apk/release/app-release-unsigned.apk"
}

main "$@"
