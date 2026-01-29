# Security Policy

## Supported Versions

We release patches for security vulnerabilities for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability within SecureChat, please send an email to security@securechat.dev. **Do not open a public issue.**

Please include the following information in your report:

- Type of vulnerability
- Full paths of source file(s) related to the vulnerability
- Location of affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the vulnerability

We will acknowledge receipt of your report within 48 hours and will strive to keep you informed of our progress toward a fix.

## Security Measures

### Encryption

- All messages are encrypted with AES-256-GCM
- Keys are generated and stored locally on your device
- End-to-end encryption ensures only you and your recipient can read messages

### Key Management

- Private keys never leave your device
- Keys are encrypted at rest using your password
- Argon2id is used for password hashing

### Network Security

- P2P communication with no central servers
- TLS/Noise protocol for transport security
- mDNS for local network discovery

## Security Best Practices for Users

1. **Use a strong password** - Choose a unique, complex password for your account
2. **Keep your app updated** - Install updates promptly to receive security patches
3. **Verify contacts** - Verify the identity of your contacts using fingerprints
4. **Backup your keys** - Keep a secure backup of your account recovery key
5. **Be cautious with links** - Don't click suspicious links in messages
