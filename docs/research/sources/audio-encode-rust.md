# Research Sources — Rust Audio Encoding (任意保存・モダン圧縮)

調査日: 2026-06-18 / 一次情報優先 (crates.io JSON API, GitHub raw, IETF, xiph)

## A. モダン圧縮形式 (Opus 推奨の根拠)

- xiph wiki Opus Recommended Settings — https://wiki.xiph.org/Opus_Recommended_Settings (取得2026-06-18)
  - mono speech: VoIP 10–24 kb/s, audiobook/podcast 24 kb/s。既定フレーム20ms。
  - narrowband ~12kb/s, wideband(8kHz) ~16–20kb/s。
- RFC 7845 (Ogg Encapsulation for Opus) — https://datatracker.ietf.org/doc/html/rfc7845 (取得2026-06-18)
  - Ogg Opus は先頭2パケットに OpusHead(識別) + OpusTags(コメント, Vorbis comment形式) が必須。
  - 単一Opusストリームのファイル拡張子は **.opus** を RECOMMENDED。
- 形式比較 (二次情報, 傾向確認用):
  - https://cloudinary.com/guides/video-formats/aac-vs-opus
  - https://news.ycombinator.com/item?id=37729791
  - 要点: 低ビットレート音声で Opus が AAC/MP3 に明確に優位 (32kbps Opus ≈ より高bitrateのAAC)。Opus は royalty-free (RFC6716, SILK+CELTハイブリッド)。MP3/AAC は特許/ライセンス経緯あり。

## B. Rust エンコードクレート (crates.io JSON API + GitHub)

### Opus (本命)
- crate `opus` (SpaceManiac/opus-rs) — newest 0.3.1, 更新 2026-01-03, DL 1.15M, recent 251k。License MIT/Apache-2.0。
  - **公開版 0.3.1 の依存**: `audiopus_sys ^0.2.0` (crates.io /opus/0.3.1/dependencies で確認)。
  - GitHub master (未公開) は `opusic-sys = 0.7.3` へ移行済 (Cargo.toml)。→ 公開版とmasterで差異あり【要確認: 次のリリースで反映されるか】。
- `audiopus_sys` 0.2.2 — 更新2021-04-22, License **ISC**。features: default/dynamic/static/generate_binding。build-deps: cmake, pkg-config, log, (bindgen optional)。
  - README: 既定は pkg-config 探索 → 無ければ **ソースからビルド**。Windows は pkg-config 無いので自動でソースビルド、**static既定**。Linux(非musl)はdynamic既定 (LIBOPUS_STATIC/static featureで上書き可)。clangはgenerate_binding時のみ。
- `opusic-sys` 0.7.3 (DoumanAsh) — 更新2026-05-02, License **BSD-3-Clause**, recent DL 382k。
  - features: `bundled` が **default** (`dep:cmake`)。bundled = vendoredソースをCMakeでビルド→`static=opus`。他: build-bindgen, dred, osce, fixed-point, no-simd 等。
  - build.rs: bundled時 cmake::Config で opus/ をビルド (static)。非bundled時は OPUS_LIB_DIR/OPUS_LIB_STATIC でシステムlib。bindgenは build-bindgen時のみ。→ **既定でcmake+Cコンパイラのみ、clang不要、システムlibopus不要**。
- `opusenc` 0.3.1 + `opusenc-sys` 0.3.0 (d-k-bo) — License BSD-3-Clause, 更新2026-01。libopusenc高水準バインドで直接.opus生成。
  - **不採用理由**: opusenc-sys/build.rs は `pkg-config` で **システムの libopusenc を要求** (vendoringなし)。Windows配布で外部依存になるため制約違反。
- `ogg` 0.9.2 (RustAudio) — 純Rust, License BSD-3-Clause, DL 7.8M。Oggコンテナのpack/unpackのみ (コーデック非依存)。Opusパケットを.oggに詰めるには OpusHead/OpusTags を自前生成しpacketとして書く必要 (RFC7845)。

### MP3
- `mp3lame-encoder` 0.2.4 + `mp3lame-sys` 0.1.11 (DoumanAsh) — License **LGPL-3.0**。mp3lame-sysはLAMEをビルド。
  - **懸念**: LGPL。静的同梱する場合 LGPL の再リンク要件 (オブジェクト提供 or 動的リンク) が生じGPL回避方針と相性が悪い。MIT/Apache/BSD希望に外れる。

### AAC
- `fdk-aac` 0.8.0 (haileys/fdk-aac-rs) — 更新2025-09。libfdk-aac バインド。
  - **懸念**: fdk-aac は「FDK AAC License」(BSD系だが特許条項あり、純粋なFOSSライセンスでない/Fedora等が配布除外)。純Rust AACエンコーダは実用なし。→ voiceジャーナル用途で選ぶ理由が薄い。

### 純Rust / 低依存
- `hound` 3.5.1 — WAV読み書き, **純Rust**, License Apache-2.0。DL 13.3M。依存ゼロ級。無圧縮(大)。
- `flac-bound` 0.5.0 (nabijaczleweli) — License MIT。libFLAC FFI。`flac`(システムlib) / `libflac`(libflac-sysでソースビルド, cmake+Cツールチェーン) / `libflac-nobuild`。
  - **Windows懸念**: README曰く libflac-sys は「GNU/NT環境であまり(全く)うまく動かない」。Windows配布リスク。
  - 純Rust FLACエンコーダは実用品が乏しい。可逆だがvoiceでは過大サイズ。
- `vorbis_rs` 0.5.5 — Ogg Vorbis (C lib高水準バインド)。Opusに対し新規性なし、voice低bitrateで劣後。

## C/D/E は最終回答本文に統合。

## whisper-rs 同居懸念 (横断確認)
- whisper-rs は既に clang/cmake で whisper.cpp をビルド済。opusic-sys(bundled)/audiopus_sys も **cmake + Cコンパイラ** を使うため追加ツール要件は実質増えない (clangは whisper 側で既に整備済、opus側はbindgen不要なら不使用)。リンクは別々のstaticライブラリで衝突しない見込み【要確認: 実ビルドで重複シンボル/SIMDフラグ衝突がないか CIで検証】。
