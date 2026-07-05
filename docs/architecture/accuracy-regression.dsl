workspace "QuickScribe — 精度を競争軸にしない" "文字起こし精度をコモディティ扱いし、CI の回帰監視に落とす C4 図。連載記事用。" {

    model {
        dev = person "開発者" "モデルや設定を変えたとき、精度が劣化していないかを知りたい人。"
        catalog = softwareSystem "モデルカタログ" "base→kotoba-q5→…→tiny。削らず用途で並べ替え、ラベルで正しい選択肢へ導く（model.rs）。"

        ci = softwareSystem "CI（perf.yml の日本語精度ジョブ）" "精度を売りにせず、回帰していないかを継続監視する。" {
            bench = container "日本語CERベンチ" "固定音源で文字起こしし、CER を算出して基準値と比べる。" "Python / CI" {
                fixtures = component "固定音源" "本人音読のパブリックドメイン作品3点（N=3・相対/回帰指標）。" "tests/fixtures/ja-accuracy"
                recog = component "文字起こし" "各モデルで認識する（QS_LANG=ja）。" ""
                cer = component "CER 算出" "NFKC・約物空白除去・文字単位 Levenshtein / 参照長。" "cer_ja.py"
                gate = component "回帰ゲート" "基準値と比べ、margin を超える悪化で失敗させる。" "ja-cer-baseline.json"
            }
        }

        dev -> fixtures "PR を出す"
        catalog -> recog "評価対象のモデル"
        fixtures -> recog "音源を渡す"
        recog -> cer "認識結果を渡す"
        cer -> gate "CER を渡す"
        gate -> dev "回帰なら赤で知らせる"
    }

    views {
        component bench "components" {
            include *
            autolayout lr
        }

        theme default
    }
}
