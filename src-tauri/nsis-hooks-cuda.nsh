; QuickScribe CUDA変種 専用 NSIS フック（ADR-0027 Phase2 / 研究: gpu-driver-prerequisite-ux.md）。
; 基本フック(モデル掃除 #511)に加え、インストール前に NVIDIA ドライバの有無を検出し、
; 未検出なら「①入手へ誘導 ②なぜ必要か ③スキップ可(CPUで動作)」を1画面で案内する。
; ドライバは再配布不可のため同梱・自動導入はしない(NVIDIA公式remedyも検出+案内)。
; CUDAランタイム(cudart/cublas)はインストーラ同梱済み＝別途インストール不要。

!include "x64.nsh"

!macro NSIS_HOOK_PREINSTALL
  ; 64bitドライバ nvcuda.dll は System32(ネイティブ)にある。32bitインストーラのFSリダイレクトを
  ; 一時無効化して実体を見る。存在すればドライバあり(版が古い場合は起動時にCPUへ自動フォールバック)。
  ${DisableX64FSRedirection}
  IfFileExists "$SYSDIR\nvcuda.dll" qs_driver_ok qs_driver_missing
  qs_driver_missing:
    ${EnableX64FSRedirection}
    ; ③スキップ可(続行=CPUで動作)/①入手/中止 の3択。②理由=速度差を本文で明示。
    ; /SD IDNO: サイレント/無人インストール(/S・MDM等)では「続行(CPUで動作)」を自動選択し、
    ; ダイアログでハング・中止しない(レビュー指摘)。
    ; NSIS 構文: MessageBox mode text [/SD return] [return_check label ...]（/SD は本文の後）。
    MessageBox MB_YESNOCANCEL|MB_ICONINFORMATION|MB_DEFBUTTON2 \
      "これは QuickScribe の GPU版(CUDA)です。$\r$\n$\r$\nNVIDIA GPU のドライバが見つかりませんでした。$\r$\nGPUがあると文字起こしが大幅に高速になります(実測で約36倍)。$\r$\nドライバが無くても、この版は自動的にCPUで動作します(遅くなります)。$\r$\n$\r$\n[はい] NVIDIAドライバの入手ページを開く$\r$\n[いいえ] このまま続行する(CPUで動作)$\r$\n[キャンセル] インストールを中止する" \
      /SD IDNO \
      IDYES qs_open_driver IDNO qs_continue
    ; キャンセル → 中止
    Abort
  qs_open_driver:
    ExecShell "open" "https://www.nvidia.com/Download/index.aspx"
    Goto qs_continue
  qs_driver_ok:
    ${EnableX64FSRedirection}
  qs_continue:
!macroend

; モデル掃除(#511): アンインストール時に DL 済み whisper モデルを削除する(CPU版と同一)。
!macro NSIS_HOOK_POSTUNINSTALL
  RMDir /r "$APPDATA\QuickScribe\models"
  RMDir "$APPDATA\QuickScribe"
!macroend
