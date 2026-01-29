# SecureChat

A privacy-first, end-to-end encrypted messaging application inspired by Briar, Signal, and Element X.

## Features

- 🔐 **AES-256 End-to-End Encryption** - All messages encrypted before leaving your device
- 💻 **Cross-Platform** - Linux, Windows, and Android support
- 📱 **Local-First Storage** - All data stored encrypted on your device
- 🎨 **Simple, Clean UI** - Intuitive interface inspired by the best
- 🌐 **P2P Messaging** - Direct peer-to-peer communication, no central servers
- 👻 **No Phone Number Required** - Use public keys for identity

## Architecture

```
securechat/
├── core/          # Rust core library (encryption, protocol)
├── desktop/       # Tauri desktop app (Windows/Linux)
└── android/       # Android native app
```

## Security

- AES-256-GCM for message encryption
- X25519 for key exchange
- Ed25519 for message signing
- Argon2id for password hashing
- All keys generated and stored locally

## Building

### Desktop (Windows/Linux)
```bash
cd desktop
cargo tauri build
```

### Android
```bash
cd android
./gradlew assembleRelease
```

## License

GPL-3.0
