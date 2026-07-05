workspace "QuickScribe — 残して育てるデータ設計" "音声・生の文字起こし・整形を捨てず、プレーンファイルで非破壊保存する C4 図。連載#5用。" {

    model {
        user = person "利用者" "音声で思考を吐き出し、整形された記録を読み返す人。"
        obsidian = softwareSystem "外部ツール（Obsidian 等）" "同じ Markdown ファイルをタグ／全文で横断・グラフ化して「育てる」。"

        quickscribe = softwareSystem "QuickScribe（保管庫）" "記録を捨てず、可搬なプレーンファイルで残す。" {
            backend = container "保存バックエンド" "本文を組み立て、上書きせずファイルに残す。" "Rust" {
                audio = component "音声保存（任意）" "WAV / Ogg Opus。保存しなければ完全メモリ完結。" "audio_save.rs"
                build = component "本文組み立て" "種別・スタイル・タグ・スキーマ版から決定的に本文を作る純粋関数。" "entry.rs: build_document"
                unique = component "非破壊保存" "一時名→rename。同名衝突は一意名に退避し、既存を上書きしない。" "next_unique_name"
                vault = component "保管庫（プレーンファイル）" "生の文字起こし / 整形 / メモを md・txt で保存。DB にしない。" "md + frontmatter"
            }
        }

        user -> build "話す→文字起こし→整形→保存"
        audio -> vault "音声を残す（任意）"
        build -> unique "本文を渡す"
        unique -> vault "上書きせず保存する"
        vault -> obsidian "同じ md を外部ツールで開ける"
        obsidian -> user "タグ／全文で横断して育てる"
    }

    views {
        component backend "components" {
            include *
            autolayout lr
        }

        theme default
    }
}
