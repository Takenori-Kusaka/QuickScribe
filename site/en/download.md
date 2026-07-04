# Download

The latest version is available from GitHub Releases.

<div style="margin: 1.5rem 0;">
  <a href="https://github.com/Takenori-Kusaka/QuickScribe/releases/latest" style="display:inline-block;padding:0.7rem 1.4rem;background:#4f46e5;color:#fff;border-radius:8px;font-weight:700;text-decoration:none;">Open latest release (GitHub Releases)</a>
</div>

## By platform

| OS | File |
|---|---|
| Windows (x64) | `QuickScribe_<version>_x64-setup.exe` |
| Windows (ARM64) | `QuickScribe_<version>_arm64-setup.exe` |
| Linux (AppImage) | `QuickScribe_<version>_amd64.AppImage` |
| Linux (deb) | `QuickScribe_<version>_amd64.deb` |

After installation, the app updates itself to the latest version via the built-in auto-updater.

## System requirements & supported formats

| Item | Details |
|---|---|
| Supported OS | Windows 10/11 (x64 / ARM64), Linux (AppImage / deb, x64) |
| Supported audio formats | `mp3` / `wav` / `m4a` / `flac` / `ogg` / `opus` / `aac` |
| Max input file size | 500 MB |
| Local transcription model | A whisper model is downloaded automatically on your first transcription (default `base` ≈ 142 MB; the Japanese-specialized `kotoba-whisper` quantized model ≈ 538 MB and others are selectable). It is then stored locally and reused. |

> Recording and local transcription stay on your device. Data is sent to a provider only when you explicitly choose cloud STT or AI refinement ([Privacy Policy](/privacy)).

## Code signing (currently unsigned)

The Windows binaries of QuickScribe are **currently unsigned**. As a result, Microsoft Defender SmartScreen may show an "unknown publisher" warning on first launch. When it does, you can run it via **"More info" → "Run anyway"**.

We plan to adopt free code signing for open-source projects in the future (such as the [SignPath Foundation](https://signpath.org/) OSS signing program; application under review). Once signing is in place, this warning will no longer appear.

## Verifying integrity

Auto-update artifacts are protected by the Tauri updater signature. For manual downloads, always obtain files from the official GitHub Releases URL.
