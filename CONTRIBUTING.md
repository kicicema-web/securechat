# Contributing to SecureChat

Thank you for your interest in contributing to SecureChat! We welcome contributions from everyone.

## Code of Conduct

This project and everyone participating in it is governed by our commitment to providing a harassment-free experience for everyone.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check the existing issues to see if the problem has already been reported. When you are creating a bug report, please include as many details as possible:

- Use a clear and descriptive title
- Describe the exact steps to reproduce the problem
- Describe the behavior you observed and what behavior you expected
- Include screenshots if applicable
- Include your OS and app version

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues. When creating an enhancement suggestion, please include:

- A clear and descriptive title
- A detailed description of the proposed enhancement
- Explain why this enhancement would be useful

### Pull Requests

1. Fork the repository
2. Create a new branch from `develop` for your feature or fix
3. Make your changes
4. Run tests to ensure nothing is broken
5. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.75+ with Cargo
- Node.js 20+
- Android Studio (for Android development)
- JDK 17

### Building

```bash
# Clone the repository
git clone https://github.com/kicicema-web/securechat.git
cd securechat

# Build the core library
cd core
cargo build --release

# Build desktop app
cd ../desktop
cargo tauri build

# Build Android app
cd ../android
./gradlew assembleRelease
```

## Style Guidelines

### Rust Code

- Follow the Rust Style Guide
- Run `cargo fmt` before committing
- Run `cargo clippy` to check for common mistakes
- Add documentation comments for public APIs

### Kotlin Code

- Follow the official Kotlin Coding Conventions
- Use meaningful variable names
- Add KDoc comments for public functions

## Security

If you discover a security vulnerability, please do NOT open an issue. Email security@securechat.dev instead.

## Questions?

Feel free to open an issue for any questions about contributing.
