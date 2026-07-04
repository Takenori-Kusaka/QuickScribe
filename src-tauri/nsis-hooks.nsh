; QuickScribe NSIS インストーラフック（#511）。
; アンインストール時に DL 済み whisper モデルを掃除する。モデルは非同梱・初回自動DLで
; dirs::data_dir()\QuickScribe\models（Windows = %APPDATA%\QuickScribe\models）に保存される
; （ADR-0021）。数百MB〜GB になり得るため、アンインストールで残置しない。
; ※ジャーナル本体はユーザー選択の保存先（既定 Documents\QuickScribe）にあり、本フックでは触れない。

!macro NSIS_HOOK_POSTUNINSTALL
  ; DL済みモデルを削除し、空になれば QuickScribe データフォルダも除去する。
  RMDir /r "$APPDATA\QuickScribe\models"
  RMDir "$APPDATA\QuickScribe"
!macroend
