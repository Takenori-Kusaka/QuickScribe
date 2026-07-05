workspace "QuickScribe (sample)" "最小C4サンプル。連載記事が本物の構成図に差し替える土台。" {

    model {
        user = person "ジャーナル利用者" "音声で思考を吐き出し、整形された記録を読み返す人。"

        quickscribe = softwareSystem "QuickScribe" "思考整理・自己理解のためのローカル完結ボイスジャーナル。" {
            ui = container "Desktop UI" "録音・レビュー・設定のフロントエンド。" "Tauri / Svelte"
            stt = container "STT Engine" "音声をローカルで文字起こしする。" "whisper.cpp"
            formatter = container "Formatting Engine" "ニュアンスを残しつつ思考を整形する（コア価値）。" "Rust"
            store = container "Local Store" "ジャーナルと設定をローカル保存する。" "SQLite / FS"
        }

        user -> ui "話す / 記録を読み返す"
        ui -> stt "音声を渡す"
        stt -> formatter "文字起こしを渡す"
        formatter -> store "整形済みエントリを保存する"
        ui -> store "記録を読み出す"
    }

    views {
        systemContext quickscribe "context" {
            include *
            autolayout lr
        }

        container quickscribe "containers" {
            include *
            autolayout lr
        }

        theme default
    }
}
