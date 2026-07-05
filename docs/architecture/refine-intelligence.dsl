workspace "QuickScribe — 整形の知性" "要約せずニュアンスを残す整形の C4 図。連載記事#3用。" {

    model {
        user = person "利用者" "音声で思考を吐き出し、整形された記録を読み返す人。"
        llm = softwareSystem "整形 LLM" "ローカル Ollama / クラウド各社（前章の抽象境界越しに差し替え）。"

        quickscribe = softwareSystem "QuickScribe（整形サブシステム）" "話した内容を、要約せずニュアンスを残して整える。" {
            backend = container "整形バックエンド" "プロンプトを組み立て、LLM 出力から本文を取り出す。" "Rust: refine.rs" {
                request = component "RefineRequest" "スタイル・本文・カスタム指示・出力言語を束ねた入力。" "struct"
                sysprompt = component "システム指示（不変）" "捏造禁止・ニュアンス保持・<journal> タグ境界。スタイルやカスタム指示に関わらず常に付く。" "system_prompt"
                style = component "スタイル指示" "構造化 / 逐語 / 要約 / ブレストの指示ブロック。カスタム指示で差し替え可。" "RefineStyle"
                builder = component "プロンプト構築" "不変の共通条件＋スタイル/カスタム指示＋本文を合成する純粋関数。" "build_user_prompt"
                engine = component "FormattingEngine" "LLM を呼ぶ（第2章の抽象境界）。" "trait"
                extract = component "本文抽出" "<journal>…</journal> のタグ内だけを決定的に取り出す。" ""
            }
        }

        user -> request "話す→文字起こし→整形を指示"
        request -> sysprompt "共通の不変条件を付与"
        request -> style "スタイル／カスタム指示を選ぶ"
        sysprompt -> builder "不変条件を渡す"
        style -> builder "指示ブロックを渡す"
        builder -> engine "プロンプトを渡す"
        engine -> llm "整形を依頼"
        llm -> extract "自由生成（<journal> で囲む）"
        extract -> user "本文だけを返す"
    }

    views {
        component backend "components" {
            include *
            autolayout lr
        }

        theme default
    }
}
