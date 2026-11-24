# Security Policy — MyVal

## Zero-Middleman Architecture

MyVal is designed from the ground up so your Riot account credentials never leave your device:

- **Direct Riot Authentication:** Sign-in loads Riot Games' official `auth.riotgames.com` page directly on your phone.
- **Hardware-Backed On-Device Encryption:** Session tokens and optional SSO refresh cookies are encrypted at rest with **AES-256-GCM** backed by **Android Keystore** (`android:allowBackup="false"` disables Android cloud backup of app storage).
- **No Proprietary Relay Server:** All live match, store, career, and loadout requests go directly from your device's native Rust HTTP stack (`rustls`) to Riot's official endpoints (`pd.<shard>.a.pvp.net`, `glz-<region>-1.<shard>.a.pvp.net`).
- **No Game-File Modification:** MyVal runs solely on Android and never touches PC game memory, files, or Vanguard.

## Verifying Official Releases

Every official APK published in [GitHub Releases](https://github.com/Evil-Bane/MyVal/releases) is:
1. Signed with the official MyVal release signing certificate (consistent across all versions):
   - **Signer DN:** `CN=MyVal, OU=Mobile, O=Evil-Bane, C=IN`
   - **Certificate SHA-256:** `c855b36e0120095bc5ae81f8c7d65724e35775b7b8f4e0ce7c17cc6e959dd16d`
2. Accompanied by a `SHA256SUMS.txt` digest file.

Verify before installing:
```bash
# Linux / macOS
sha256sum -c SHA256SUMS.txt

# Windows PowerShell
Get-FileHash MyVal-v1.0.5-arm64-v8a.apk -Algorithm SHA256
```

## Reporting a Vulnerability

If you discover a security or privacy issue in MyVal, please **do not** open a public issue with exploit details. Instead, use GitHub's private Security Advisory tab on `Evil-Bane/MyVal` or contact the maintainer directly via GitHub profile `@Evil-Bane`.
