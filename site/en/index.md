---
layout: home
hero:
  name: QuickScribe
  text: Speak, and your thinking gets organized.
  tagline: A local-first, privacy-focused voice journal. Record → transcribe → AI refines while keeping your nuance → into your journal.
  actions:
    - theme: brand
      text: Download
      link: /en/download
    - theme: alt
      text: Guide
      link: /en/guide
    - theme: alt
      text: GitHub
      link: https://github.com/Takenori-Kusaka/QuickScribe
features:
  - icon: 🔒
    title: Local-first, privacy by default
    details: Recording and transcription (whisper.cpp) run on your device. Audio is never sent anywhere by default. Cloud services are used only when you explicitly opt in.
  - icon: ✨
    title: Refinement that keeps your nuance
    details: Not just transcription — the AI structures your thinking while preserving hesitations and second thoughts. Move between verbatim / summary / brainstorm, or build your own custom refinement.
  - icon: 🎙️
    title: Your voice plus system audio
    details: Besides the microphone, you can capture the other party's voice played by your PC (system audio) — review whole meetings and calls.
  - icon: ⏯️
    title: Physical-button workflow
    details: Works with global hotkeys, mouse buttons, foot switches, and Stream Deck. A momentary "record only while held" mode is included.
  - icon: 🏷️
    title: Tag it, then rediscover the past
    details: Tag your entries and search or filter your journal. The AI reads across multiple entries to surface recurring themes and your next step.
  - icon: 🧩
    title: On your terms
    details: Transcribe with local whisper / kotoba-whisper or cloud (Groq, OpenAI, Deepgram, Azure). Bring your own key for the refinement LLM. Keys are stored in your OS secure storage.
---

## What is QuickScribe

**QuickScribe is a local-first voice journal for organizing your thinking and understanding yourself.**
It is optimized for the experience of **speaking and reflecting** rather than writing, and invests in the **intelligence to organize thought while keeping nuance** rather than raw transcription accuracy.

- Press a physical button the moment an idea strikes, and start recording.
- Transcribe locally (cloud is optional if you want it).
- Check and replace misheard terms in the "term check" step, then let the AI refine in your own words.
- Accumulate entries as Markdown, bundle them with tags, and rediscover the past.

![QuickScribe main screen](https://raw.githubusercontent.com/Takenori-Kusaka/QuickScribe/main/docs/assets/screenshot-main-en.png)

> Your journal is **plain Markdown / text files** on your own device. Open them directly in tools like Obsidian.

## Why QuickScribe? (How it differs)

Many tools turn speech into text, but QuickScribe **narrows its purpose to "organizing your own thinking"** and invests in nuance-preserving refinement and local-first privacy.

| | Meeting notes (Otter/Granola) | Fast dictation (superwhisper, etc.) | Cloud diary (Day One, etc.) | **QuickScribe** |
|---|---|---|---|---|
| Main use | Meeting summaries / minutes | Quick text entry | Diary (cloud sync) | **Organizing thought / self-understanding** |
| Refinement | Summarize and **discard** | Clean up only | Keep as handwritten | **Keep nuance and grow it** |
| Privacy | Cloud-based | Mostly cloud | Cloud sync | **Local-first is an option** |
| Trigger | — | Hotkey-centric | — | Hotkey / physical button / foot switch |

**Not "summarize and discard," but "keep the nuance, refine it, and grow it by looking back later"** — that is QuickScribe.
