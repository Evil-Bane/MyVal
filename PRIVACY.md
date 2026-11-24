# Privacy Policy, Terms of Use & Fair-Play Pledge — MyVal

**Effective Date:** November 24, 2026  
**Project Repository:** https://github.com/Evil-Bane/MyVal

## 1. Zero Server Storage & No Data Collection
MyVal is a strictly **client-side, open-source (`GPL-3.0`) Android companion app** for VALORANT.

- **We store nothing.** The developer operates no backend database, analytics pipeline, telemetry collector, ad SDK, or login relay server.
- **Direct Riot Authentication:** Nobody forces anyone to sign in. If you choose to authenticate, you log in voluntarily and directly on Riot Games' official `auth.riotgames.com` page inside an on-device WebView.
- **Local Keystore Encryption:** Session tokens stay strictly on your own phone encrypted at rest via **Android Keystore (`AES-256-GCM`)**, with Android cloud backups disabled (`android:allowBackup="false"`).
- **Direct Network Calls Only:** Match history, daily shop, loadout, and live hub requests travel directly from your device's Rust HTTP client to Riot's endpoints and public cosmetic APIs (`valorant-api.com` / GitHub Releases update checks).

## 2. Fair-Play Commitment (Zero Unfair Advantage)
MyVal is built strictly around convenience and post-match visualization — **never competitive cheating or automation**:

- **Anti-Instalock Guard (`15s` Delay):** MyVal strictly disables instant agent hovering or locking (`instapicking` / `instalocking`). Both **Hover** and **Lock Agent** enforce an explicit **15-second wait** in Agent Select before either action is permitted.
- **No In-Game Overlays or PC Hooks:** MyVal runs solely on Android and never injects overlays, scripts, macros, automation, or memory readers into your PC or Riot Vanguard.

## 3. Voluntary Terms of Use & Disclaimer
- **Use at Your Own Discretion:** MyVal connects to standard Riot companion endpoints rather than an official Riot Games Production API key. By choosing to authenticate with your account, you acknowledge that you provide tokens voluntarily on your device at your own responsibility.
- **No 100% Ban Guarantee:** No account bans have happened to anyone we know, and following discussions with a Riot developer, Riot's stance is that companion utilities are generally fine as long as no automation, script, or tool creates an unfair competitive advantage. Still, because MyVal is unofficial software provided **"AS IS"** under the GNU GPL v3.0 license without warranty, we do not claim a 100% ban-safe guarantee.
- **Riot Games Notice:** MyVal isn't endorsed by Riot Games and doesn't reflect the views or opinions of Riot Games or anyone officially involved in producing or managing Riot Games properties. Riot Games and VALORANT are trademarks or registered trademarks of Riot Games, Inc.
