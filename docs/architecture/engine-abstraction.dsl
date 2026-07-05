workspace "QuickScribe — エンジン差し替えの抽象境界" "文字起こし(STT)と整形(LLM)を trait とファクトリで差し替え可能にした C4 図。連載記事#2用。" {

    model {
        user = person "利用者" "音声で思考を吐き出し、整形された記録を読み返す人。"

        localWhisper = softwareSystem "ローカル whisper.cpp" "オフラインで動く既定の文字起こし。鍵不要・送信なし。"
        cloudStt = softwareSystem "クラウドSTT" "Groq / OpenAI / Deepgram / Azure（鍵設定時のみ）。"
        localOllama = softwareSystem "ローカル Ollama" "オフラインで動く既定の整形。鍵不要・送信なし。"
        cloudLlm = softwareSystem "クラウドLLM" "Gemini / Anthropic / OpenAI / AWS（選択時のみ）。"

        quickscribe = softwareSystem "QuickScribe" "ローカル完結ボイスジャーナル。" {
            backend = container "バックエンド" "Tauri コマンド層と各エンジンの抽象境界。" "Rust" {
                cmd = component "コマンド層" "録音→文字起こし→整形→保存のオーケストレーション。具体プロバイダを知らない。" "lib.rs"
                sttFactory = component "STT ファクトリ" "設定文字列からエンジンを解決する。未知はローカルへフォールバック。" "stt.rs: engine_for"
                sttTrait = component "TranscriptionEngine" "文字起こしの抽象（Strategy / DIP 境界）。" "trait"
                refineFactory = component "整形ファクトリ" "RefineProvider に集約して解決する（別名・既定モデル・種別）。" "refine.rs"
                refineTrait = component "FormattingEngine" "整形の抽象（Strategy / DIP 境界）。" "trait"
            }
        }

        user -> cmd "録音・整形を指示する"
        cmd -> sttFactory "設定から STT を解決する"
        cmd -> refineFactory "設定から整形を解決する"
        sttFactory -> sttTrait "Box<dyn> を返す"
        refineFactory -> refineTrait "Box<dyn> を返す"
        sttTrait -> localWhisper "既定（ローカル）"
        sttTrait -> cloudStt "鍵設定時のみ"
        refineTrait -> localOllama "既定（ローカル）"
        refineTrait -> cloudLlm "選択時のみ"
    }

    views {
        component backend "components" {
            include *
            autolayout lr
        }

        systemContext quickscribe "context" {
            include *
            autolayout lr
        }

        theme default
    }
}
