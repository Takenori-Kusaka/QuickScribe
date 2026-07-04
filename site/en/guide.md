# Guide

## 1. Install

Download the installer for your OS from the [Download page](/en/download) and run it (Windows x64 / ARM64, Linux AppImage / deb).
The model for local transcription is downloaded automatically the first time you transcribe.

## 2. Record

- Start/stop recording with the **"Start recording"** button in the window, or with the **global hotkey** (default `Ctrl/Cmd + Shift + R`).
- QuickScribe lives in the tray, so it keeps waiting in the background even when you close the window. You can also enable **launch at login** in settings.

### Recording mode and source
- **Recording mode**: "Toggle (press once to start/stop)" or **"record only while held" (momentary)**.
- **Recording source**: microphone, **system audio (the other party's voice/sound played by your PC)**, or **microphone + system audio at once** (Windows).

### Physical-button integration
Just assign QuickScribe's hotkey to a mouse side button, foot switch, Stream Deck, and so on. See `docs/guide/physical-triggers.md` in the repository for details.

## 3. Transcribe → term check → refine

1. When you stop recording, **transcription** runs (local whisper or cloud STT).
2. Optionally run **"✓ Term check"**: the AI detects likely mis-transcriptions (e.g. "L L M" misheard) and suggests replacements. You choose which to apply.
3. **"✨ Refine"**: organize your thinking while keeping nuance, using **structured / verbatim / summary / brainstorm**, or your **own custom pattern**.

## 4. Look back in the journal

- Entries are saved to your **output folder** (default: Documents/QuickScribe). Raw transcripts are named `transcript-…` and refined ones `refined-….md`.
- From **Journal** in the header, browse, full-text search, and filter past entries by tag.
- **"✨ Cross-entry discovery"**: the AI reads the entries you filtered and extracts recurring themes, emotional trends, unresolved questions, and your next step.
- Use **"Open output folder"** in the refined result's action column (or in settings) to open the save folder directly.

## 5. Settings

- **Transcription engine**: local whisper (`base` / `kotoba-whisper` [Japanese-specialized], etc.) or cloud (Groq, OpenAI, Deepgram, Azure).
- **Refinement provider**: Gemini / Anthropic / OpenAI / local Ollama / AWS Bedrock / Claude Platform on AWS. **Keys are stored in your OS secure storage.**
- Output format (txt / Markdown), hotkeys, launch at login, and more.

> Data is sent to a provider only if you use cloud services. If privacy is your priority, use "local" ([Privacy Policy](/privacy)).

## System requirements & supported formats

| Item | Details |
|---|---|
| Supported OS | Windows 10/11 (x64 / ARM64), Linux (AppImage / deb, x64) |
| Supported audio formats | `mp3` / `wav` / `m4a` / `flac` / `ogg` / `opus` / `aac` |
| Max input file size | 500 MB |
| Local transcription model | A whisper model is downloaded automatically on your first transcription (default `base` ≈ 142 MB; the Japanese-specialized `kotoba-whisper` quantized model ≈ 538 MB and others are selectable). It is then stored locally and reused. |

> Recording and local transcription stay on your device. Data is sent to a provider only when you explicitly choose cloud STT or AI refinement ([Privacy Policy](/privacy)).
