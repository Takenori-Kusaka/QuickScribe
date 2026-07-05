workspace "QuickScribe — 物理ボタン統合体験" "独自デバイスAPIを作らず、公式手段でホットキー/CLIへ橋渡しする C4 図。連載#6用。" {

    model {
        user = person "利用者" "物理ボタンひとつで、話す→整形→残すまでを摩擦なく回したい人。"
        device = softwareSystem "物理デバイス" "Stream Deck / Logitech / Razer / HID フットスイッチ。各社の公式手段でホットキーを送るか、CLI を起動する。"

        quickscribe = softwareSystem "QuickScribe（録音の起動点）" "物理トリガーから録音を start/stop する。" {
            app = container "録音アプリ" "ホットキーと CLI を起動点にし、録音モードに応じて start/stop する。" "Rust / Tauri" {
                hotkey = component "グローバルホットキー" "F13–F24 推奨（通常入力と衝突しない）。押下/離しを拾う。" "tauri-plugin-global-shortcut"
                cli = component "CLI 起動点" "--toggle / --start / --stop-record。single-instance で実行中プロセスへ転送。" "CLI"
                mode = component "録音モード" "トグル / 押している間（momentary）。既定はトグル。" "設定"
                rec = component "録音 start/stop" "トグルも momentary も上位で構成できるよう録音 API を分離。停止に release delay（末尾切れ防止）。" "record"
            }
        }

        user -> device "ボタンを押す"
        device -> hotkey "公式手段でホットキー送出"
        device -> cli "または CLI を直接起動"
        hotkey -> mode "押下／離しを渡す"
        cli -> rec "start / stop / toggle"
        mode -> rec "モードに応じて start/stop"
    }

    views {
        component app "components" {
            include *
            autolayout lr
        }

        theme default
    }
}
