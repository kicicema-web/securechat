#!/bin/bash
set -e

echo "🔐 SecureChat Build Script"
echo "=========================="

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    echo -e "${GREEN}[*]${NC} $1"
}

print_error() {
    echo -e "${RED}[!]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[!]${NC} $1"
}

# Check dependencies
check_deps() {
    print_status "Checking dependencies..."
    
    # Check Rust
    if ! command -v rustc &> /dev/null; then
        print_error "Rust not found. Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    # Check Node.js
    if ! command -v node &> /dev/null; then
        print_error "Node.js not found. Please install Node.js 20+"
        exit 1
    fi
    
    # Check Cargo
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo not found. Please install Rust"
        exit 1
    fi
    
    print_status "All dependencies found!"
}

# Build core library
build_core() {
    print_status "Building core library..."
    cd core
    cargo build --release
    cd ..
    print_status "Core library built successfully!"
}

# Build desktop app
build_desktop() {
    print_status "Building desktop app..."
    cd desktop
    
    # Install dependencies
    print_status "Installing Node dependencies..."
    cd src
    npm install
    cd ..
    
    # Build with Tauri
    print_status "Building with Tauri..."
    cargo tauri build
    
    cd ..
    print_status "Desktop app built successfully!"
}

# Build Android app
build_android() {
    print_status "Building Android app..."
    cd android
    
    # Check for Android SDK
    if [ -z "$ANDROID_SDK_ROOT" ] && [ -z "$ANDROID_HOME" ]; then
        print_warning "ANDROID_SDK_ROOT or ANDROID_HOME not set"
        print_warning "Please set one of these environment variables"
    fi
    
    # Build
    ./gradlew assembleRelease
    
    cd ..
    print_status "Android app built successfully!"
}

# Run tests
run_tests() {
    print_status "Running tests..."
    cd core
    cargo test
    cd ..
    print_status "Tests passed!"
}

# Main
main() {
    case "${1:-all}" in
        core)
            check_deps
            build_core
            ;;
        desktop)
            check_deps
            build_core
            build_desktop
            ;;
        android)
            check_deps
            build_core
            build_android
            ;;
        test)
            check_deps
            run_tests
            ;;
        all)
            check_deps
            run_tests
            build_core
            build_desktop
            # build_android  # Skip Android by default as it needs SDK
            print_status "Build complete!"
            print_status "Desktop app: desktop/src-tauri/target/release/bundle/"
            ;;
        *)
            echo "Usage: $0 [core|desktop|android|test|all]"
            exit 1
            ;;
    esac
}

main "$@"
