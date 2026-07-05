workspace "QuickScribe — プライバシーの可視化" "ローカルファースト既定と「オフラインにする」導線の C4 図。連載#4用。" {

    model {
        user = person "利用者" "音声で思考を吐き出し、整形された記録を読み返す人。"
        cloud = softwareSystem "クラウド AI" "整形 LLM / クラウド STT。鍵を入れて選んだときだけ送信先になる。"

        quickscribe = softwareSystem "QuickScribe（プライバシー可視化）" "現在の構成が完全オンデバイスか、クラウド送信を伴うかを正直に見せる。" {
            frontend = container "設定・プライバシー" "プロバイダ選択と、状態表示・オフライン切替。" "Svelte / TypeScript" {
                settings = component "設定" "整形プロバイダと STT プロバイダの選択。既定はローカル（ADR-0021）。" "provider / sttProvider"
                derive = component "isFullyLocal（導出）" "整形=ローカル かつ STT=ローカル のときだけ true。主張ではなく現在の構成から導く。" "$derived"
                indicator = component "プライバシー状態インジケータ" "「オンデバイス完結」か「クラウド送信あり」を設定先頭に常時表示。誇張しない。" "UI"
                offline = component "「オフラインにする」導線" "ワンクリックで provider=ollama / stt=local へ固定する。トグルは1つに留める。" "makeOffline / offlineMode"
            }
        }

        user -> settings "プロバイダを選ぶ"
        settings -> derive "現在の構成を渡す"
        derive -> indicator "完結か送信ありかを表示"
        settings -> cloud "クラウド選択時のみ送信"
        user -> offline "オフラインにする"
        offline -> settings "ローカルへ上書きする"
    }

    views {
        component frontend "components" {
            include *
            autolayout lr
        }

        theme default
    }
}
