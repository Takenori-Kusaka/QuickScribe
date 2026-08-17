# 変更履歴 / Changelog

本ファイルは [Keep a Changelog](https://keepachangelog.com/ja/1.1.0/) に概ね従い、
バージョンは [Semantic Versioning](https://semver.org/lang/ja/) に従います。
v0.6.4 以降は [release-please](https://github.com/googleapis/release-please) が
Conventional Commits から自動生成します（#400）。以下は導入前の主な履歴の要約です。

## [1.5.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.11.0...v1.5.0) (2026-08-17)


### ✨ 新機能 / Features

* **a11y:** モーダルのdialog化・フォーカストラップ・Esc閉じ＋コントラスト是正 ([#395](https://github.com/Takenori-Kusaka/QuickScribe/issues/395)) ([#413](https://github.com/Takenori-Kusaka/QuickScribe/issues/413)) ([4ea8ade](https://github.com/Takenori-Kusaka/QuickScribe/commit/4ea8adee45f86e81951ae27444eafca4642345c6))
* AI整形(コア価値)+設定+スクロールUI+手動更新確認 ([#308](https://github.com/Takenori-Kusaka/QuickScribe/issues/308)) ([3982c6d](https://github.com/Takenori-Kusaka/QuickScribe/commit/3982c6d1c7273933ad46b06bd01990b3b71eab77))
* **app:** OSログイン時の自動起動オプション (S6.3 [#41](https://github.com/Takenori-Kusaka/QuickScribe/issues/41)) ([#370](https://github.com/Takenori-Kusaka/QuickScribe/issues/370)) ([31ac83e](https://github.com/Takenori-Kusaka/QuickScribe/commit/31ac83e2eda2be44b1b57e0425e93e5238cfeb01))
* **brand:** ロゴの余白を詰め背景を透過(白ボックス解消) ([#337](https://github.com/Takenori-Kusaka/QuickScribe/issues/337)) ([5392532](https://github.com/Takenori-Kusaka/QuickScribe/commit/539253261c099c0cc112f8328af4c479c57295de))
* **brand:** ロゴを「二重の発光クレッセント」に刷新（暗indigo squircle） ([#340](https://github.com/Takenori-Kusaka/QuickScribe/issues/340)) ([1221b12](https://github.com/Takenori-Kusaka/QuickScribe/commit/1221b12af6b2eb019837057c9983faf54dd9dbfe))
* **design:** ブランドカラーのトークン化とstylelintによる生hex禁止 ([#567](https://github.com/Takenori-Kusaka/QuickScribe/issues/567)) ([8579473](https://github.com/Takenori-Kusaka/QuickScribe/commit/8579473f5869a3a3145029db8c71853e55617783))
* **eval:** Common Voice ja(CC0)の日本語CERベンチをCIに追加（ADR-0024第二スライス / [#578](https://github.com/Takenori-Kusaka/QuickScribe/issues/578)） ([#602](https://github.com/Takenori-Kusaka/QuickScribe/issues/602)) ([c4e5dd7](https://github.com/Takenori-Kusaka/QuickScribe/commit/c4e5dd72773f191c7b27bb37e60352ec8af0405e))
* **eval:** FLEURS ja(CC-BY)を2つ目の定点コーパスに追加＋fetch汎用化（ADR-0024第二スライス / [#578](https://github.com/Takenori-Kusaka/QuickScribe/issues/578)） ([#609](https://github.com/Takenori-Kusaka/QuickScribe/issues/609)) ([2d29f99](https://github.com/Takenori-Kusaka/QuickScribe/commit/2d29f99c8d5290b1ba8786ce7c5b3645c20154b6))
* **eval:** 日本語ASR評価コア（正規化・CER・ブートストラップCI）＋ADR-0024 Accepted（[#578](https://github.com/Takenori-Kusaka/QuickScribe/issues/578)） ([#601](https://github.com/Takenori-Kusaka/QuickScribe/issues/601)) ([1a7be52](https://github.com/Takenori-Kusaka/QuickScribe/commit/1a7be52ee03c0f9f02289625404432e1dba05c19))
* **gpu:** CUDA変種のドライバ前提UX（NSIS案内＋アプリ内リンク / ADR-0027 Phase2） ([#628](https://github.com/Takenori-Kusaka/QuickScribe/issues/628)) ([b5abf7e](https://github.com/Takenori-Kusaka/QuickScribe/commit/b5abf7e6bbfa819ae2e5e8c3fc0a7c9f523c7d6e))
* **i18n:** i18n基盤(svelte-i18n)導入＋メイン画面スライスをキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#434](https://github.com/Takenori-Kusaka/QuickScribe/issues/434)) ([717a928](https://github.com/Takenori-Kusaka/QuickScribe/commit/717a928b9cc92ba3a10501d56f97d913c8502274))
* **i18n:** refine.rs のユーザー向けエラーも安定コード化 ([#462](https://github.com/Takenori-Kusaka/QuickScribe/issues/462)) ([#538](https://github.com/Takenori-Kusaka/QuickScribe/issues/538)) ([67c4e11](https://github.com/Takenori-Kusaka/QuickScribe/commit/67c4e112eb312cb0cc020b7eb806d9dfa3dda74f))
* **i18n:** Rustバックエンドのエラー文字列を安定コード化 ([#462](https://github.com/Takenori-Kusaka/QuickScribe/issues/462)) ([#537](https://github.com/Takenori-Kusaka/QuickScribe/issues/537)) ([4b2cb7e](https://github.com/Takenori-Kusaka/QuickScribe/commit/4b2cb7e1597067202537bba0333ac173776be22b))
* **i18n:** validateRefineConfig(cfgErr)をエラーコード化([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase2) ([#463](https://github.com/Takenori-Kusaka/QuickScribe/issues/463)) ([aac2a12](https://github.com/Takenori-Kusaka/QuickScribe/commit/aac2a12b35d155d033083e92906414bc9b8a4444))
* **i18n:** zh/es カタログ追加（4言語対応）([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)) ([#454](https://github.com/Takenori-Kusaka/QuickScribe/issues/454)) ([a842a9f](https://github.com/Takenori-Kusaka/QuickScribe/commit/a842a9ff95e3405c2fcf1ee59e9563bd3b394d42))
* **i18n:** ジャーナルパネルをキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#436](https://github.com/Takenori-Kusaka/QuickScribe/issues/436)) ([7646ebd](https://github.com/Takenori-Kusaka/QuickScribe/commit/7646ebdefe325bf825aa7a4e715881b8a4707e3e))
* **i18n:** メインのホットキーヒント＋設定の選択肢をキー化(ja/en) ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)) ([#445](https://github.com/Takenori-Kusaka/QuickScribe/issues/445)) ([54e9073](https://github.com/Takenori-Kusaka/QuickScribe/commit/54e90731e86df4484b96c6995a03334b4658e845))
* **i18n:** 結果・アクション領域＋ヘッダ補助ラベルをキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#435](https://github.com/Takenori-Kusaka/QuickScribe/issues/435)) ([ee60470](https://github.com/Takenori-Kusaka/QuickScribe/commit/ee60470f6717cee61242247471ef96e633e57092))
* **i18n:** 結果カード/用語補正パネル等の残りUI文言をキー化(ja/en) ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)) ([#451](https://github.com/Takenori-Kusaka/QuickScribe/issues/451)) ([fe48713](https://github.com/Takenori-Kusaka/QuickScribe/commit/fe487131955cc455779aa766a934785b460f3e82))
* **i18n:** 英語カタログ＋ロケール切替＋起動時OS言語デフォルト ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase3) ([#443](https://github.com/Takenori-Kusaka/QuickScribe/issues/443)) ([df6c97a](https://github.com/Takenori-Kusaka/QuickScribe/commit/df6c97a4d72fca1dcd096aa1f5cb47c9c1520812))
* **i18n:** 設定のAWS/STTラベル・主要tipをキー化(ja/en) ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)) ([#447](https://github.com/Takenori-Kusaka/QuickScribe/issues/447)) ([ecbfd10](https://github.com/Takenori-Kusaka/QuickScribe/commit/ecbfd109853b1e8e12fe565d722d8407db2d50ec))
* **i18n:** 設定の詳細ラベル15項目をキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#442](https://github.com/Takenori-Kusaka/QuickScribe/issues/442)) ([f2a1cd9](https://github.com/Takenori-Kusaka/QuickScribe/commit/f2a1cd9c95ae7d9132169a166a42529746a77783))
* **i18n:** 設定パネル残りの文言・複雑inline-HTML tipをキー化(ja/en) ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)) ([#450](https://github.com/Takenori-Kusaka/QuickScribe/issues/450)) ([fbd8d96](https://github.com/Takenori-Kusaka/QuickScribe/commit/fbd8d96234039e3bb57af20f15be8528676460a1))
* **i18n:** 設定画面の見出し・主要ラベルをキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#438](https://github.com/Takenori-Kusaka/QuickScribe/issues/438)) ([e68d36a](https://github.com/Takenori-Kusaka/QuickScribe/commit/e68d36ae48cf92487ff562a38df0916138847b1a))
* **installer:** アンインストール時に DL済み whisper モデルを掃除（NSIS フック） ([#511](https://github.com/Takenori-Kusaka/QuickScribe/issues/511)) ([#549](https://github.com/Takenori-Kusaka/QuickScribe/issues/549)) ([d162f90](https://github.com/Takenori-Kusaka/QuickScribe/commit/d162f9092a087adb3b527cf5630e923433c7467c))
* **jobs:** ジョブ一覧を直近3件表示＋展開/スクロールで表示エリアの無限拡大を防ぐ ([#644](https://github.com/Takenori-Kusaka/QuickScribe/issues/644)) ([c34098f](https://github.com/Takenori-Kusaka/QuickScribe/commit/c34098fca0155d3010c2583bda82523fe4976874))
* **journal:** 習慣ストリーク(寛容)をジャーナルに表示([#58](https://github.com/Takenori-Kusaka/QuickScribe/issues/58) S9.4 第1弾) ([#502](https://github.com/Takenori-Kusaka/QuickScribe/issues/502)) ([aa2ebe9](https://github.com/Takenori-Kusaka/QuickScribe/commit/aa2ebe9ce2bf427a15754413b1575a4acbc974e0))
* **legal:** v0.1.0法的MUST — ライセンス明記＋第三者NOTICE同梱＋deny.toml ([#341](https://github.com/Takenori-Kusaka/QuickScribe/issues/341)) ([2132199](https://github.com/Takenori-Kusaka/QuickScribe/commit/2132199d7f990313384af21fa385eab8a05fb1fc))
* **metrics:** リリースDL数の計測とテレメトリ方針の明文化([#60](https://github.com/Takenori-Kusaka/QuickScribe/issues/60) S9.6) ([#501](https://github.com/Takenori-Kusaka/QuickScribe/issues/501)) ([5772be8](https://github.com/Takenori-Kusaka/QuickScribe/commit/5772be816d77358fa06cac1cd4cf9c80c3cea5b7))
* **models:** モデル選択に相対処理速度クラスを表示（[#598](https://github.com/Takenori-Kusaka/QuickScribe/issues/598) 第一スライス） ([#608](https://github.com/Takenori-Kusaka/QuickScribe/issues/608)) ([a3ddd94](https://github.com/Takenori-Kusaka/QuickScribe/commit/a3ddd947db1195932f861433b426cbc756001932))
* **nudge:** 習慣ナッジ＝起動アンカーのopt-inローカル通知 ([#58](https://github.com/Takenori-Kusaka/QuickScribe/issues/58)) ([#563](https://github.com/Takenori-Kusaka/QuickScribe/issues/563)) ([7dae8b5](https://github.com/Takenori-Kusaka/QuickScribe/commit/7dae8b5b4499f16d5d2d21626e533c96a1f35c74))
* **onboarding:** カードを操作ボタンより上部へ配置([#510](https://github.com/Takenori-Kusaka/QuickScribe/issues/510)) ([#518](https://github.com/Takenori-Kusaka/QuickScribe/issues/518)) ([b3da691](https://github.com/Takenori-Kusaka/QuickScribe/commit/b3da691d3412ac5bd0c03b6a8cb1a342e847c44c))
* **onboarding:** 初回オンボーディングと空状態の導線([#397](https://github.com/Takenori-Kusaka/QuickScribe/issues/397)) ([#461](https://github.com/Takenori-Kusaka/QuickScribe/issues/461)) ([4251291](https://github.com/Takenori-Kusaka/QuickScribe/commit/4251291fcde8787ad172cd8147304741c84f039f))
* **onboarding:** 初回のaha体験「サンプルで試す」([#57](https://github.com/Takenori-Kusaka/QuickScribe/issues/57) S9.3) ([#505](https://github.com/Takenori-Kusaka/QuickScribe/issues/505)) ([92b53fa](https://github.com/Takenori-Kusaka/QuickScribe/commit/92b53fafdbac8f4db1dd6d113fa2f98848a0fbf5))
* **perf:** アイドルCPU使用率を計測指標として追加 (Linux CI + Windows実機手順) ([#669](https://github.com/Takenori-Kusaka/QuickScribe/issues/669)) ([c5f2109](https://github.com/Takenori-Kusaka/QuickScribe/commit/c5f21092d55f4243ce23b440dc7e27f66a099501))
* **perf:** 日本語精度CERベンチ([#26](https://github.com/Takenori-Kusaka/QuickScribe/issues/26))＋モデルカタログ精選(ADR-0022) ([#561](https://github.com/Takenori-Kusaka/QuickScribe/issues/561)) ([2249087](https://github.com/Takenori-Kusaka/QuickScribe/commit/22490874ae7a8f2db1516ec9c6794eb959c1c8b3))
* **perf:** 起動時間ベンチを追加（run()→UI ready をアプリ計装＋xvfbでCI計測） ([#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403)) ([#554](https://github.com/Takenori-Kusaka/QuickScribe/issues/554)) ([1a6159e](https://github.com/Takenori-Kusaka/QuickScribe/commit/1a6159e20146872d92c63989009fe64c822f3b59))
* **perf:** 非機能ベンチに英語CER(精度指標)を追加 ([#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403)) ([#550](https://github.com/Takenori-Kusaka/QuickScribe/issues/550)) ([37ab293](https://github.com/Takenori-Kusaka/QuickScribe/commit/37ab2931392cdb175fea5283b4e0922d3c57a563))
* **privacy:** オフライン固定モードの実体化([#465](https://github.com/Takenori-Kusaka/QuickScribe/issues/465)) ([#508](https://github.com/Takenori-Kusaka/QuickScribe/issues/508)) ([113d512](https://github.com/Takenori-Kusaka/QuickScribe/commit/113d512c46ac8240aafacb55bcf5ab5972b24a62))
* **privacy:** オンデバイス/クラウドのインジケータと「オフラインにする」([#465](https://github.com/Takenori-Kusaka/QuickScribe/issues/465)) ([#480](https://github.com/Takenori-Kusaka/QuickScribe/issues/480)) ([d5a0d68](https://github.com/Takenori-Kusaka/QuickScribe/commit/d5a0d6836c83068b4f630d6f3c12c0187feef47d))
* **privacy:** クラウド整形プロバイダ選択時に送信同意の警告を表示 ([#465](https://github.com/Takenori-Kusaka/QuickScribe/issues/465)) ([#547](https://github.com/Takenori-Kusaka/QuickScribe/issues/547)) ([196de28](https://github.com/Takenori-Kusaka/QuickScribe/commit/196de284de434969c89d4cc9025f92d52a54a39e))
* **privacy:** ローカルファースト既定へ変更（整形=Ollama / 日本語STT=kotoba）+ ADR-0021 ([#465](https://github.com/Takenori-Kusaka/QuickScribe/issues/465) [#511](https://github.com/Takenori-Kusaka/QuickScribe/issues/511)) ([#546](https://github.com/Takenori-Kusaka/QuickScribe/issues/546)) ([d030601](https://github.com/Takenori-Kusaka/QuickScribe/commit/d030601e94e4182e97c2e9cedb99e136dec6a6b3))
* **readme:** コアループのデモGIFを追加([#55](https://github.com/Takenori-Kusaka/QuickScribe/issues/55) S9.1) ([#503](https://github.com/Takenori-Kusaka/QuickScribe/issues/503)) ([9444725](https://github.com/Takenori-Kusaka/QuickScribe/commit/944472505497203989d21fa704eb9977f29d321a))
* **record:** システム音ループバックを録音ソースに統合 (S1.3 Phase0 [#19](https://github.com/Takenori-Kusaka/QuickScribe/issues/19)) ([#359](https://github.com/Takenori-Kusaka/QuickScribe/issues/359)) ([85b4b2f](https://github.com/Takenori-Kusaka/QuickScribe/commit/85b4b2f2509dd21155ce019496eacaf8c1c17578))
* **record:** マイク＋システム音の同時取得 (S1.3 Phase1 [#19](https://github.com/Takenori-Kusaka/QuickScribe/issues/19)) ([#368](https://github.com/Takenori-Kusaka/QuickScribe/issues/368)) ([a0373fc](https://github.com/Takenori-Kusaka/QuickScribe/commit/a0373fc10714e4a52d8c38173c5cdd1c7ee9987d))
* **record:** モーメンタリ録音＋CLI start/stop分離 (S1.5 Phase1 [#21](https://github.com/Takenori-Kusaka/QuickScribe/issues/21)) ([#369](https://github.com/Takenori-Kusaka/QuickScribe/issues/369)) ([0000c08](https://github.com/Takenori-Kusaka/QuickScribe/commit/0000c083d114078dcf7342b66d7b84a747cd8b98))
* **record:** 入力デバイス選択UI＋実行時切替 (S1.2 [#18](https://github.com/Takenori-Kusaka/QuickScribe/issues/18)) ([#358](https://github.com/Takenori-Kusaka/QuickScribe/issues/358)) ([bac1b24](https://github.com/Takenori-Kusaka/QuickScribe/commit/bac1b2438b230ec726eb82b0402a6e8a8141384f))
* **refine:** AWS Bedrock / Claude Platform on AWS プロバイダ追加 (ADR-0011) ([#349](https://github.com/Takenori-Kusaka/QuickScribe/issues/349)) ([88cf954](https://github.com/Takenori-Kusaka/QuickScribe/commit/88cf954df8dc42dc7827968d070a5d71177866fa))
* **refine:** OpenAI互換base_url設定可能化＋プライバシー判定のURLホスト評価化（[#593](https://github.com/Takenori-Kusaka/QuickScribe/issues/593)） ([#597](https://github.com/Takenori-Kusaka/QuickScribe/issues/597)) ([40f4a94](https://github.com/Takenori-Kusaka/QuickScribe/commit/40f4a945a01e11334a7e540b46783d0af8cb4a0f))
* **refine:** カスタム整形パターン (S3.3拡張) ([#371](https://github.com/Takenori-Kusaka/QuickScribe/issues/371)) ([f218487](https://github.com/Takenori-Kusaka/QuickScribe/commit/f2184878a9c9c57f304e2a9b7a5c83e3382f90a6))
* **refine:** ローカルLLM(Ollama)対応で完全ローカル整形 (S3.4) ([#344](https://github.com/Takenori-Kusaka/QuickScribe/issues/344)) ([152c464](https://github.com/Takenori-Kusaka/QuickScribe/commit/152c4646351d28271271a9cb24ff925382ea3fab))
* **refine:** 整形スタイルに解説tips＋処理画面は表示のみに ([#347](https://github.com/Takenori-Kusaka/QuickScribe/issues/347)) ([00ba0aa](https://github.com/Takenori-Kusaka/QuickScribe/commit/00ba0aab57b39a6e5f969ca8ef2f47dab427d90d))
* **refine:** 整形をFormattingEngine抽象化＋整形スタイル(構造化/逐語/要約/ブレスト) ([#343](https://github.com/Takenori-Kusaka/QuickScribe/issues/343)) ([499b5d3](https://github.com/Takenori-Kusaka/QuickScribe/commit/499b5d3cbc66939aa19ddbb44732b2354c492f2c))
* **refine:** 整形出力言語(翻訳)オプション([#453](https://github.com/Takenori-Kusaka/QuickScribe/issues/453)) ([#479](https://github.com/Takenori-Kusaka/QuickScribe/issues/479)) ([ea7c0af](https://github.com/Takenori-Kusaka/QuickScribe/commit/ea7c0af90aec1e39acc672d4cd68063c964c7e7e))
* **refine:** 結果から別スタイルで整形し直す＋コピー（S3.5 [#31](https://github.com/Takenori-Kusaka/QuickScribe/issues/31)） ([#356](https://github.com/Takenori-Kusaka/QuickScribe/issues/356)) ([fba6e8e](https://github.com/Takenori-Kusaka/QuickScribe/commit/fba6e8e8bc0e5647581655f4a0a235147b6bb084))
* **release:** nightlyチャネルとupdater鍵分離Runbook ([#52](https://github.com/Takenori-Kusaka/QuickScribe/issues/52)) ([#564](https://github.com/Takenori-Kusaka/QuickScribe/issues/564)) ([61b88d8](https://github.com/Takenori-Kusaka/QuickScribe/commit/61b88d8887f6221d2689f6bc51fd62f4fb385d3f))
* **release:** Windows x64＋ARM64 の2アーキ配布（Phase1 / ADR-0012） ([#354](https://github.com/Takenori-Kusaka/QuickScribe/issues/354)) ([e5fbb57](https://github.com/Takenori-Kusaka/QuickScribe/commit/e5fbb5716eb2ebd2714fb2e49c2e42854625cb71))
* **robustness:** 入力ファイルのサイズ上限ガード＋対応形式/上限のUI通知 ([#397](https://github.com/Takenori-Kusaka/QuickScribe/issues/397)) ([#414](https://github.com/Takenori-Kusaka/QuickScribe/issues/414)) ([6cd301f](https://github.com/Takenori-Kusaka/QuickScribe/commit/6cd301f38e98aade10fbf0d0bd173892903b89cc))
* **robustness:** 設定・エントリのスキーマ版管理＋検証 (S5.3 [#38](https://github.com/Takenori-Kusaka/QuickScribe/issues/38) / S4.4 [#35](https://github.com/Takenori-Kusaka/QuickScribe/issues/35)) ([#382](https://github.com/Takenori-Kusaka/QuickScribe/issues/382)) ([168d24a](https://github.com/Takenori-Kusaka/QuickScribe/commit/168d24a7904a2ad84cd44af1c0b3661bcc63bcab))
* **secrets:** API鍵/AWS秘密をOSセキュアストレージ(keyring)に保管 (S3.2 [#28](https://github.com/Takenori-Kusaka/QuickScribe/issues/28)) ([#355](https://github.com/Takenori-Kusaka/QuickScribe/issues/355)) ([8c865bb](https://github.com/Takenori-Kusaka/QuickScribe/commit/8c865bb1a5e86fac3e31de8c1540ec0a895c428e))
* **security:** 制限的CSPを設定（既定の無効化を解消）([#391](https://github.com/Takenori-Kusaka/QuickScribe/issues/391)) ([#455](https://github.com/Takenori-Kusaka/QuickScribe/issues/455)) ([2156944](https://github.com/Takenori-Kusaka/QuickScribe/commit/21569443a1255a18525e58d1cf7e0d97e179bc60))
* **settings:** 「整形」セクションを折りたたみグループ化([#404](https://github.com/Takenori-Kusaka/QuickScribe/issues/404)) ([#488](https://github.com/Takenori-Kusaka/QuickScribe/issues/488)) ([d97f735](https://github.com/Takenori-Kusaka/QuickScribe/commit/d97f73514534f5b42141b75070e335de3bf10c38))
* **settings:** カテゴリを頻度メジャー順に再編([#404](https://github.com/Takenori-Kusaka/QuickScribe/issues/404)) ([#460](https://github.com/Takenori-Kusaka/QuickScribe/issues/460)) ([851f387](https://github.com/Takenori-Kusaka/QuickScribe/commit/851f387d419a1b382ae3b110ab105cb539d04ce0))
* **settings:** 未設定時の動線改善(不足明示・フォーカス・保存ガード)([#516](https://github.com/Takenori-Kusaka/QuickScribe/issues/516)) ([#519](https://github.com/Takenori-Kusaka/QuickScribe/issues/519)) ([72aaaf1](https://github.com/Takenori-Kusaka/QuickScribe/commit/72aaaf16965a754817f7e86ca57869fb57785af3))
* **settings:** 設定を5タブに再編し到達性を改善([#512](https://github.com/Takenori-Kusaka/QuickScribe/issues/512)) ([#521](https://github.com/Takenori-Kusaka/QuickScribe/issues/521)) ([e006e34](https://github.com/Takenori-Kusaka/QuickScribe/commit/e006e340eaddb9ee5b12a594d9a623c70b1002b4))
* **site:** GitHub Pages ドキュメントサイト(VitePress) (S9.2 [#56](https://github.com/Takenori-Kusaka/QuickScribe/issues/56)) ([#385](https://github.com/Takenori-Kusaka/QuickScribe/issues/385)) ([0210ced](https://github.com/Takenori-Kusaka/QuickScribe/commit/0210ced0ca9888f6865c4258e489260eff15d593))
* **spike:** Windowsタスクバーのサムネイルツールバーに録音トグルボタン ([#322](https://github.com/Takenori-Kusaka/QuickScribe/issues/322)) ([bde5767](https://github.com/Takenori-Kusaka/QuickScribe/commit/bde5767eaf54941605497f2ccd97d4a6a19eb920))
* **storage:** 保管庫の非破壊保存＋保管庫を開く (S4.1 [#32](https://github.com/Takenori-Kusaka/QuickScribe/issues/32)) ([#367](https://github.com/Takenori-Kusaka/QuickScribe/issues/367)) ([c9bddc5](https://github.com/Takenori-Kusaka/QuickScribe/commit/c9bddc5a911ab6bc4c7760a527d0178de6ed62e1))
* **storage:** 内省タグの付与(md frontmatter/txt行) (S4.3 Phase0 [#34](https://github.com/Takenori-Kusaka/QuickScribe/issues/34)) ([#373](https://github.com/Takenori-Kusaka/QuickScribe/issues/373)) ([026ff14](https://github.com/Takenori-Kusaka/QuickScribe/commit/026ff14bd0d9ef73d936ca155293f1b9a0044d83))
* **storage:** 出力形式(txt/Markdown)＋メタデータ付与 (S4.2 [#33](https://github.com/Takenori-Kusaka/QuickScribe/issues/33)) ([#372](https://github.com/Takenori-Kusaka/QuickScribe/issues/372)) ([a8f4325](https://github.com/Takenori-Kusaka/QuickScribe/commit/a8f4325d7ea8b3c1cf3b1ce93a2ab181ed63c895))
* **stt:** GPU配布を単一Vulkanビルドへ一本化＋起動時デバイス検出（ADR-0028） ([#629](https://github.com/Takenori-Kusaka/QuickScribe/issues/629)) ([cf89c3d](https://github.com/Takenori-Kusaka/QuickScribe/commit/cf89c3dd925f19ca6d1f39a8f4507ec1d0e530d4))
* **stt:** whisper.cpp統合の土台+音声ファイル変換 [WIP S2.1/S1.6] ([#301](https://github.com/Takenori-Kusaka/QuickScribe/issues/301)) ([66f69d4](https://github.com/Takenori-Kusaka/QuickScribe/commit/66f69d4927d703d3050f32d9111c7a6575a4a998))
* **stt:** whisperモデル選択＋kotoba-whisper対応 (S2.2 [#23](https://github.com/Takenori-Kusaka/QuickScribe/issues/23)) ([#379](https://github.com/Takenori-Kusaka/QuickScribe/issues/379)) ([683d358](https://github.com/Takenori-Kusaka/QuickScribe/commit/683d358faed93089e69690556384dfe0d3590f5b))
* **stt:** クラウドSTT Phase B (Deepgram/Azure) (S2.4 [#25](https://github.com/Takenori-Kusaka/QuickScribe/issues/25)) ([#378](https://github.com/Takenori-Kusaka/QuickScribe/issues/378)) ([cbd1964](https://github.com/Takenori-Kusaka/QuickScribe/commit/cbd1964be04486a54b75c2d6d2a7a8e54c21ceb8))
* **stt:** クラウドSTT(Groq/OpenAI) Phase A (S2.4 [#25](https://github.com/Takenori-Kusaka/QuickScribe/issues/25)) ([#375](https://github.com/Takenori-Kusaka/QuickScribe/issues/375)) ([c857aa7](https://github.com/Takenori-Kusaka/QuickScribe/commit/c857aa7ae5bff58e10abceaf70b26efc5565b86d))
* **stt:** マルチジョブ逐次キューのバックエンド（ADR-0026 [#621](https://github.com/Takenori-Kusaka/QuickScribe/issues/621) Phase1） ([#622](https://github.com/Takenori-Kusaka/QuickScribe/issues/622)) ([e50dc3c](https://github.com/Takenori-Kusaka/QuickScribe/commit/e50dc3c101dd7ece3a786cc81af3e19c7d1194fb))
* **stt:** モデルのDL方式明示と日本語ロケール既定をkotobaに([#511](https://github.com/Takenori-Kusaka/QuickScribe/issues/511) 一部) ([#520](https://github.com/Takenori-Kusaka/QuickScribe/issues/520)) ([646ad0a](https://github.com/Takenori-Kusaka/QuickScribe/commit/646ad0a38e0eb6115ceed5f9bf1c47673d43c837))
* **stt:** 日本語の既定STTを kotoba-q5 → large-v3-turbo へ（実測に基づく / ADR-0025） ([#610](https://github.com/Takenori-Kusaka/QuickScribe/issues/610)) ([1d23c3f](https://github.com/Takenori-Kusaka/QuickScribe/commit/1d23c3f12392e70836bdd4ffb073380aec25d462))
* **stt:** 話者特定オプション(default-OFF)を追加 ― sherpa-onnx・オンデマンドDL・遅延ロード(ADR-0031) ([#633](https://github.com/Takenori-Kusaka/QuickScribe/issues/633)) ([8d432c5](https://github.com/Takenori-Kusaka/QuickScribe/commit/8d432c506360a91d75f9ddc99cf52b5f869b993f))
* **taskbar:** ウィジェットにツールチップ追加+ウィンドウ風グリフで停止と区別 ([#335](https://github.com/Takenori-Kusaka/QuickScribe/issues/335)) ([89a379d](https://github.com/Takenori-Kusaka/QuickScribe/commit/89a379d8add197be22465a065c2e8a75cdc71068))
* **taskbar:** ウィジェットをper-pixel alpha化し右ボタンの赤縁を根本除去 ([#339](https://github.com/Takenori-Kusaka/QuickScribe/issues/339)) ([c20642d](https://github.com/Takenori-Kusaka/QuickScribe/commit/c20642df6affd2e5ae2de767562eb136fae86a5e))
* **taskbar:** ウィジェット背景をクロマキー透過にしテーマ非依存に ([#333](https://github.com/Takenori-Kusaka/QuickScribe/issues/333)) ([1b32c64](https://github.com/Takenori-Kusaka/QuickScribe/commit/1b32c640f0f8669f3f715152f1a1ed1acbab1b40))
* **taskbar:** タスクバーウィジェットの表示ON/OFFを設定に追加 ([#352](https://github.com/Takenori-Kusaka/QuickScribe/issues/352)) ([a5b2133](https://github.com/Takenori-Kusaka/QuickScribe/commit/a5b2133bb302f4bc66e80a95b1b27ba681a8dbff))
* **taskbar:** 右ボタンをアプリアイコン化＋描画をダブルバッファでちらつき低減 ([#336](https://github.com/Takenori-Kusaka/QuickScribe/issues/336)) ([7dd1605](https://github.com/Takenori-Kusaka/QuickScribe/commit/7dd160556fb6ddf986b626b6199dc552c46a3a3e))
* **transcribe:** 用語補正フェーズ(誤変換の確認・置換) を整形前に追加 ([#384](https://github.com/Takenori-Kusaka/QuickScribe/issues/384)) ([f0f6de8](https://github.com/Takenori-Kusaka/QuickScribe/commit/f0f6de84b30ca2912aa7db70f53751155a6c9c6f))
* **ui:** アプリのバージョンを表示（実行結果の共有用） ([#625](https://github.com/Takenori-Kusaka/QuickScribe/issues/625)) ([d4309cd](https://github.com/Takenori-Kusaka/QuickScribe/commit/d4309cd94153d2fb09036379a42b32daf655384c))
* **ui:** ヘッダUX改善(保管庫→ジャーナル/SVGアイコン/IA整理) ([#388](https://github.com/Takenori-Kusaka/QuickScribe/issues/388)) ([f994674](https://github.com/Takenori-Kusaka/QuickScribe/commit/f9946748919e586c238fd6bd25cf7abcf34708f5))
* **ui:** マルチジョブ・ジョブ一覧UI（ADR-0026 [#621](https://github.com/Takenori-Kusaka/QuickScribe/issues/621) Phase2） ([#624](https://github.com/Takenori-Kusaka/QuickScribe/issues/624)) ([41540e7](https://github.com/Takenori-Kusaka/QuickScribe/commit/41540e71b4b774c00e76b211b95f42b4dd74bcc6))
* **ui:** 文字起こしの進捗%を明確化し残り時間(ETA)を表示 ([#303](https://github.com/Takenori-Kusaka/QuickScribe/issues/303)) ([9657a93](https://github.com/Takenori-Kusaka/QuickScribe/commit/9657a93cc818c7283a79dd714a8af754c8056fb4))
* **updater:** 自動アップデート(起動時チェック→背景DL→確認して再起動) ([#304](https://github.com/Takenori-Kusaka/QuickScribe/issues/304)) ([ccb6021](https://github.com/Takenori-Kusaka/QuickScribe/commit/ccb60215390ec34302942d618c2243523ddd7e83))
* **ux:** ヘッダー重なり修正＋フォルダボタン＋命名則(生/整形)＋整形は常にmd ([#381](https://github.com/Takenori-Kusaka/QuickScribe/issues/381)) ([d801503](https://github.com/Takenori-Kusaka/QuickScribe/commit/d8015030b3883dd55a4e042484fae1fa9b4e7d09))
* **ux:** 文字起こし完結ユーザー向けUX（出力フォルダボタン＋未設定でも保存可・警告のみ）（[#603](https://github.com/Takenori-Kusaka/QuickScribe/issues/603)） ([#605](https://github.com/Takenori-Kusaka/QuickScribe/issues/605)) ([920a592](https://github.com/Takenori-Kusaka/QuickScribe/commit/920a59264a6d6971531be3bb83ae99b2e0fee3a7))
* **ux:** 既定幅拡大・ブレスト削除・「文字起こしから整形」に改称([#513](https://github.com/Takenori-Kusaka/QuickScribe/issues/513)/[#514](https://github.com/Takenori-Kusaka/QuickScribe/issues/514)/[#515](https://github.com/Takenori-Kusaka/QuickScribe/issues/515)) ([#517](https://github.com/Takenori-Kusaka/QuickScribe/issues/517)) ([32b2257](https://github.com/Takenori-Kusaka/QuickScribe/commit/32b22576da5bef3d3825ef587e3d1a2c69bce955))
* **ux:** 空テキスト整形ガードとオンボーディング再表示([#397](https://github.com/Takenori-Kusaka/QuickScribe/issues/397)/[#398](https://github.com/Takenori-Kusaka/QuickScribe/issues/398)) ([#484](https://github.com/Takenori-Kusaka/QuickScribe/issues/484)) ([912b414](https://github.com/Takenori-Kusaka/QuickScribe/commit/912b414b334c0590a2735f5d74dd35e1efd50510))
* **vault:** エントリ名を日付+内容由来ラベルに変更 (ADR-0032) ([#657](https://github.com/Takenori-Kusaka/QuickScribe/issues/657)) ([918c443](https://github.com/Takenori-Kusaka/QuickScribe/commit/918c443b254224919a145529b3c7409ca7938d82))
* **vault:** ファイル名を日付先頭 {yyyymmdd}-{種別}-{ラベル} に変更 (ADR-0032追記) ([#659](https://github.com/Takenori-Kusaka/QuickScribe/issues/659)) ([553cfd5](https://github.com/Takenori-Kusaka/QuickScribe/commit/553cfd5cf3df576ce9c1e78a0917b60230ccbe11))
* **vault:** 保管庫エントリの一覧・タグ/全文絞り込み・閲覧 (S4.3 Phase1 [#34](https://github.com/Takenori-Kusaka/QuickScribe/issues/34)) ([#376](https://github.com/Takenori-Kusaka/QuickScribe/issues/376)) ([140133c](https://github.com/Takenori-Kusaka/QuickScribe/commit/140133ce49c0ae31983b2ff7a931016e7baffc39))
* **vault:** 横断発見(複数エントリのAI読み解き) (S4.3 Phase2 [#34](https://github.com/Takenori-Kusaka/QuickScribe/issues/34)) ([#377](https://github.com/Takenori-Kusaka/QuickScribe/issues/377)) ([5a8070c](https://github.com/Takenori-Kusaka/QuickScribe/commit/5a8070cbffafcdda2a80d91de88e843d1a42a3ad))
* Walking Skeleton (Phase 1) — Tauri 2 + Svelte 5 scaffold + CI/Release ([#4](https://github.com/Takenori-Kusaka/QuickScribe/issues/4)) ([991fb7d](https://github.com/Takenori-Kusaka/QuickScribe/commit/991fb7dc02bb0f4bd7c4c9195a1fd821b0e54914))
* タスクバーボタンに録音中バッジ(オーバーレイ) + トレイ操作の撤回 ([#328](https://github.com/Takenori-Kusaka/QuickScribe/issues/328)) ([62ffedd](https://github.com/Takenori-Kusaka/QuickScribe/commit/62ffeddda4f0aa1fe633732a26e7cf4aabe09569))
* タスクバー埋め込みウィジェット(録音/停止・ウィンドウ表示) + 録音バッジ赤丸化 ([#329](https://github.com/Takenori-Kusaka/QuickScribe/issues/329)) ([060aff3](https://github.com/Takenori-Kusaka/QuickScribe/commit/060aff37a2ebf728dbbca3e30122580d7a891b02))
* トレイ左クリックで録音トグル + サムネイルボタンの不具合修正 ([#327](https://github.com/Takenori-Kusaka/QuickScribe/issues/327)) ([2e4ccbd](https://github.com/Takenori-Kusaka/QuickScribe/commit/2e4ccbded742629d857f1545691a38f064b4675f))
* ホットキー設定UX改善(変更ボタン→キー押下で登録 / Ctrl-Cmd表記) ([#334](https://github.com/Takenori-Kusaka/QuickScribe/issues/334)) ([26b1ab3](https://github.com/Takenori-Kusaka/QuickScribe/commit/26b1ab37a4429441658d2ec66fda348a1d7bf723))
* マイク録音→文字起こし(S1.1) + 整形プロバイダ3種対応 ([#319](https://github.com/Takenori-Kusaka/QuickScribe/issues/319)) ([cc19350](https://github.com/Takenori-Kusaka/QuickScribe/commit/cc19350dab538259ca0bcfd04ac88c13042ffa31))
* メモ/テキスト入力で整形のみ + 録音ホットキーを設定で変更可能に ([#320](https://github.com/Takenori-Kusaka/QuickScribe/issues/320)) ([84ba048](https://github.com/Takenori-Kusaka/QuickScribe/commit/84ba048fffafb072a17c9625a397a1d48252e36b))
* 単一インスタンス化し `--toggle-record` で常駐インスタンスへ録音トグル転送 ([#321](https://github.com/Takenori-Kusaka/QuickScribe/issues/321)) ([e006fa6](https://github.com/Takenori-Kusaka/QuickScribe/commit/e006fa6ee5e7b2374624b7432d5edcd7e6e62fdc))
* 文字起こし対象が無い録音は通知し音声を保存しない ([#330](https://github.com/Takenori-Kusaka/QuickScribe/issues/330)) ([bed59d1](https://github.com/Takenori-Kusaka/QuickScribe/commit/bed59d14e58ff4bad9d68ad12db481ed02f6bcb2))
* 録音の非同期化 + 停止後の自動一気通貫整形 + トレイ録音状態表示 ([#332](https://github.com/Takenori-Kusaka/QuickScribe/issues/332)) ([ef8b328](https://github.com/Takenori-Kusaka/QuickScribe/commit/ef8b328f680f2cee5af77f47f01342a763c42eef))
* 録音音声のモダン圧縮保存 Opus(.opus) ([#325](https://github.com/Takenori-Kusaka/QuickScribe/issues/325)) ([058201a](https://github.com/Takenori-Kusaka/QuickScribe/commit/058201a087327786885d7e8bc5808510f669d23f))
* 録音音声の保存(WAV)・保存先フォルダ・文字起こしテキスト保持を設定可能に ([#324](https://github.com/Takenori-Kusaka/QuickScribe/issues/324)) ([68d656c](https://github.com/Takenori-Kusaka/QuickScribe/commit/68d656c39449dbbeec6c273a95392bdde82e006b))


### 🐛 修正 / Bug Fixes

* **a11y:** WCAG2.0 AA / JIS X 8341-3 のAA違反を是正([#395](https://github.com/Takenori-Kusaka/QuickScribe/issues/395)) ([#482](https://github.com/Takenori-Kusaka/QuickScribe/issues/482)) ([69eb9ee](https://github.com/Takenori-Kusaka/QuickScribe/commit/69eb9ee19c7af406e97b406b19d7661aa2d86d25))
* **backend:** 入力検証の堅牢化と用語是正([#398](https://github.com/Takenori-Kusaka/QuickScribe/issues/398)/[#18](https://github.com/Takenori-Kusaka/QuickScribe/issues/18)) ([#486](https://github.com/Takenori-Kusaka/QuickScribe/issues/486)) ([24da526](https://github.com/Takenori-Kusaka/QuickScribe/commit/24da526dbc95516c3a5608ac8c0916c755b0a395))
* **build:** whisper(ggml)を移植性ベースラインでビルド（SIGILL対策） ([#353](https://github.com/Takenori-Kusaka/QuickScribe/issues/353)) ([25f3ba8](https://github.com/Takenori-Kusaka/QuickScribe/commit/25f3ba837344d2713145e5ef3243f0910dbbb01c))
* **ci:** AVX2 CFLAGSを撤去(Windowsリリースビルド破壊の修正) ([#307](https://github.com/Takenori-Kusaka/QuickScribe/issues/307)) ([bb43c38](https://github.com/Takenori-Kusaka/QuickScribe/commit/bb43c389a5b83c82ce96d13bdb242cd2bbf3b0e4))
* **ci:** PlantUMLの太字日本語(豆腐)対策にfonts-noto-cjk導入 ([#586](https://github.com/Takenori-Kusaka/QuickScribe/issues/586)) ([b74a69b](https://github.com/Takenori-Kusaka/QuickScribe/commit/b74a69bb1b8451ae251ca8df6a90675204d3a6f7))
* **ci:** カバレッジ監査是正 — rust-coverageジョブ修復とFE branches 80%ゲート化 ([#566](https://github.com/Takenori-Kusaka/QuickScribe/issues/566)) ([0c4adcf](https://github.com/Takenori-Kusaka/QuickScribe/commit/0c4adcfedc3d824bda1ab06c535366beeef3762b))
* **ci:** テストビルドに updater 署名鍵を渡しビルドを完走させる ([27cdb0d](https://github.com/Takenori-Kusaka/QuickScribe/commit/27cdb0d022bc0c817191eec3f689838446f96294))
* **corrections:** 用語補正の置換を補正版として別ファイル保存（非破壊 / [#599](https://github.com/Takenori-Kusaka/QuickScribe/issues/599)） ([#607](https://github.com/Takenori-Kusaka/QuickScribe/issues/607)) ([66b4d7f](https://github.com/Takenori-Kusaka/QuickScribe/commit/66b4d7f92f9b40fdd34a276961311408894670ab))
* **e2e:** トグル検証を非同期状態待ちに変更しフレークを根治([#412](https://github.com/Takenori-Kusaka/QuickScribe/issues/412)) ([#506](https://github.com/Takenori-Kusaka/QuickScribe/issues/506)) ([cb9df80](https://github.com/Takenori-Kusaka/QuickScribe/commit/cb9df8085cdf852e124465fb175910c589e1c240))
* **i18n:** TS/Rust文言のSSOT是正とカタログ一本化（監査項目8/11/17/22） ([#570](https://github.com/Takenori-Kusaka/QuickScribe/issues/570)) ([58da8ad](https://github.com/Takenori-Kusaka/QuickScribe/commit/58da8ad7b1d5d9961c1b61bb94ef4b1beed751ce))
* **logging:** タスクバー診断ログを出力先(ドキュメント)からOSのLocalデータ領域へ移す ([#552](https://github.com/Takenori-Kusaka/QuickScribe/issues/552)) ([e821add](https://github.com/Takenori-Kusaka/QuickScribe/commit/e821add43f82321eb3836c36d57d85db9ee5e885))
* **onboarding:** プライバシー文言の過剰主張を是正([#397](https://github.com/Takenori-Kusaka/QuickScribe/issues/397)) ([#464](https://github.com/Takenori-Kusaka/QuickScribe/issues/464)) ([22a2387](https://github.com/Takenori-Kusaka/QuickScribe/commit/22a2387a523d110dcfd57f9545daf8934bb61d47))
* **perf:** アイドルCPUの観測窓を10秒1点から10秒×6区間に分割する ([#678](https://github.com/Takenori-Kusaka/QuickScribe/issues/678)) ([#679](https://github.com/Takenori-Kusaka/QuickScribe/issues/679)) ([2662430](https://github.com/Takenori-Kusaka/QuickScribe/commit/2662430b600372d796da339cefb8db82c86d9ec4))
* **perf:** 日本語精度ジョブにアイコン生成を追加 ([#562](https://github.com/Takenori-Kusaka/QuickScribe/issues/562)) ([1964bd4](https://github.com/Takenori-Kusaka/QuickScribe/commit/1964bd459e1c14f57687764970d97846d8dc8660))
* **refine:** 整形出力を本文だけに（XMLタグ境界抽出）＋スタイル選択を設定画面に ([#346](https://github.com/Takenori-Kusaka/QuickScribe/issues/346)) ([4000d72](https://github.com/Takenori-Kusaka/QuickScribe/commit/4000d72c7edb103e565c0d17397d0e29d218b39f))
* **release:** release-pleaseのノート上書きを解消（releaseBody除去 / [#572](https://github.com/Takenori-Kusaka/QuickScribe/issues/572)） ([#596](https://github.com/Takenori-Kusaka/QuickScribe/issues/596)) ([c6c42d7](https://github.com/Takenori-Kusaka/QuickScribe/commit/c6c42d7a5fcb4755fafb6bb629305d26c0f218f9))
* **release:** リリースノートを自動復旧するジョブを追加（[#572](https://github.com/Takenori-Kusaka/QuickScribe/issues/572) 恒久修正） ([#612](https://github.com/Takenori-Kusaka/QuickScribe/issues/612)) ([db9ce3d](https://github.com/Takenori-Kusaka/QuickScribe/commit/db9ce3d0a305b30dea5c9a8ea1e9be568e85ad52))
* **secrets:** keyring移行はwrite成功時のみ平文削除（データ損失防止 / S3.2） ([#357](https://github.com/Takenori-Kusaka/QuickScribe/issues/357)) ([7988509](https://github.com/Takenori-Kusaka/QuickScribe/commit/7988509f457cb40bf7e687f2f04989f86af8302d))
* **security:** quick-xml DoS勧告を根拠付きignoreで監査を正常化([#526](https://github.com/Takenori-Kusaka/QuickScribe/issues/526)) ([#527](https://github.com/Takenori-Kusaka/QuickScribe/issues/527)) ([58fc410](https://github.com/Takenori-Kusaka/QuickScribe/commit/58fc410bb9d8049f28711fee711a9f958e77cd8a))
* **settings:** 設定項目の誤分類を修正(タスクバー/自動起動→アプリ全般) ([#404](https://github.com/Takenori-Kusaka/QuickScribe/issues/404)) ([#422](https://github.com/Takenori-Kusaka/QuickScribe/issues/422)) ([66d54ad](https://github.com/Takenori-Kusaka/QuickScribe/commit/66d54adf0c16ab546c4313d801595e068d380ccb))
* **stt:** 自分で保存した .opus 録音を再文字起こし可能にする ([#560](https://github.com/Takenori-Kusaka/QuickScribe/issues/560)) ([3f5da61](https://github.com/Takenori-Kusaka/QuickScribe/commit/3f5da61b06c648e7ac47cd5b367d3303507dd84f))
* **taskbar:** タスクバーが隠れている時はウィジェットも隠す(最大化+auto-hide対応) ([#345](https://github.com/Takenori-Kusaka/QuickScribe/issues/345)) ([bfc3524](https://github.com/Takenori-Kusaka/QuickScribe/commit/bfc352443ed8a2cbb194fa17474c5a024156f4a2))
* **taskbar:** 最大化+auto-hideでウィジェットが前面に残る問題を堅牢化 ([#348](https://github.com/Takenori-Kusaka/QuickScribe/issues/348)) ([e9006f9](https://github.com/Takenori-Kusaka/QuickScribe/commit/e9006f915ce031de802032d14e528337fb160674))
* **tray:** トレイ常駐挙動を是正 (S6.1) — 閉じても終了せず常駐、トレイから操作 ([#300](https://github.com/Takenori-Kusaka/QuickScribe/issues/300)) ([618b7f7](https://github.com/Takenori-Kusaka/QuickScribe/commit/618b7f79250a495f84ba67d9fe4a0764d44c6aa9))
* **ui:** ヘッダのジャーナルをアイコンのみ＋ツールチップ化(重なり解消) ([#440](https://github.com/Takenori-Kusaka/QuickScribe/issues/440)) ([0c1754f](https://github.com/Takenori-Kusaka/QuickScribe/commit/0c1754f2327707ffd0d845d507541d1459136778))
* **ui:** 内部ID"S2.2"露出を除去＋エラー文言をユーザー向けに整備 ([#398](https://github.com/Takenori-Kusaka/QuickScribe/issues/398)) ([#411](https://github.com/Takenori-Kusaka/QuickScribe/issues/411)) ([1fa8011](https://github.com/Takenori-Kusaka/QuickScribe/commit/1fa8011714a482b7ec36196fb5b83e2c72e867ec))
* **ux:** レビュー指摘の是正(幅コンパクト回帰/ブレスト復活/whisper既定)([#513](https://github.com/Takenori-Kusaka/QuickScribe/issues/513)/[#514](https://github.com/Takenori-Kusaka/QuickScribe/issues/514)/[#511](https://github.com/Takenori-Kusaka/QuickScribe/issues/511)) ([#523](https://github.com/Takenori-Kusaka/QuickScribe/issues/523)) ([b8af470](https://github.com/Takenori-Kusaka/QuickScribe/commit/b8af470b72ea49f969374956f7921e52cb01357e))
* 文字起こしの非同期化+進捗/逐次表示+高速化+UI刷新 ([#302](https://github.com/Takenori-Kusaka/QuickScribe/issues/302)) ([b0f70aa](https://github.com/Takenori-Kusaka/QuickScribe/commit/b0f70aaec6e7327aab3ac3d85d6fb7a11a194b02))
* 設定をフルスクリーンのモーダルダイアログに変更 ([#326](https://github.com/Takenori-Kusaka/QuickScribe/issues/326)) ([d1b4596](https://github.com/Takenori-Kusaka/QuickScribe/commit/d1b4596db008a9bd02a8ba8fe3fc6b35a3f0b977))


### ⚡ パフォーマンス / Performance

* **record:** 音声保存OFF時に未使用の原音をRAMへ抱えない ([#663](https://github.com/Takenori-Kusaka/QuickScribe/issues/663)) ([#668](https://github.com/Takenori-Kusaka/QuickScribe/issues/668)) ([77bd6ed](https://github.com/Takenori-Kusaka/QuickScribe/commit/77bd6ed01ccd9d624b477c1eb544abde1b76b25a))
* **stt:** whisper.cppをAVX2/FMA有効でビルド+スレッドを物理コア数に ([#305](https://github.com/Takenori-Kusaka/QuickScribe/issues/305)) ([a40f1d4](https://github.com/Takenori-Kusaka/QuickScribe/commit/a40f1d46f27852831da3eac6d3d6699cd444c0f0))
* **vault:** list_entries でエントリ全文を読まない ([#665](https://github.com/Takenori-Kusaka/QuickScribe/issues/665)) ([#670](https://github.com/Takenori-Kusaka/QuickScribe/issues/670)) ([23b2e8a](https://github.com/Takenori-Kusaka/QuickScribe/commit/23b2e8a2c2f8a37f688645bc22f610ed437fdddb))
* **vault:** エントリ一覧を段階表示にし検索をデバウンスする ([#673](https://github.com/Takenori-Kusaka/QuickScribe/issues/673)) ([eeeb85b](https://github.com/Takenori-Kusaka/QuickScribe/commit/eeeb85be3a5913e0b933a047997cd8aad19416bc))


### ♻️ リファクタ / Refactor

* **app:** App.svelte を &lt;800 行に分割（shortcut/device/custom-styles/privacy 抽出） ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#544](https://github.com/Takenori-Kusaka/QuickScribe/issues/544)) ([325db7a](https://github.com/Takenori-Kusaka/QuickScribe/commit/325db7abeaaead3ff1e1cf5837bed91da41caeab))
* **app:** 保管庫閲覧を vault-view.svelte.ts へ抽出 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#539](https://github.com/Takenori-Kusaka/QuickScribe/issues/539)) ([cd527d3](https://github.com/Takenori-Kusaka/QuickScribe/commit/cd527d31eae3f56eead7ffb75ae5e55613935311))
* **app:** 秘密情報ブリッジを lib/secrets.ts へ抽出 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#540](https://github.com/Takenori-Kusaka/QuickScribe/issues/540)) ([6843671](https://github.com/Takenori-Kusaka/QuickScribe/commit/6843671594eb763c1fefc0a51545fcf1f6333df5))
* **app:** 自動アップデートを lib/update.svelte.ts へ抽出 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#542](https://github.com/Takenori-Kusaka/QuickScribe/issues/542)) ([519e366](https://github.com/Takenori-Kusaka/QuickScribe/commit/519e366d4a8e0e494c4ba114e34d8e31bf2f0471))
* **app:** 設定永続化を lib/settings-persist.ts へ抽出 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#541](https://github.com/Takenori-Kusaka/QuickScribe/issues/541)) ([8ecf365](https://github.com/Takenori-Kusaka/QuickScribe/commit/8ecf3652cb3dae9f9843f243267eb4ebf330db05))
* **brand:** ロゴ生成を再現可能なスクリプト化＋色付きグロウ除去 ([#338](https://github.com/Takenori-Kusaka/QuickScribe/issues/338)) ([5d3d968](https://github.com/Takenori-Kusaka/QuickScribe/commit/5d3d968b522d7df0cb4e7d0a61dbc826562e2498))
* **entry:** 保管庫ドキュメントドメインを entry.rs へ抽出 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#535](https://github.com/Takenori-Kusaka/QuickScribe/issues/535)) ([79081f7](https://github.com/Takenori-Kusaka/QuickScribe/commit/79081f75c39c644e25766182b2e1a934cff9c8a1))
* **front:** App.svelteの純粋関数をlib抽出＋テスト ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#425](https://github.com/Takenori-Kusaka/QuickScribe/issues/425)) ([c81794d](https://github.com/Takenori-Kusaka/QuickScribe/commit/c81794d29e033651d884cdd7bf2cd4f466b6ee76))
* **front:** refine引数組み立てをlib抽出＋カバレッジゲート有効化 ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#428](https://github.com/Takenori-Kusaka/QuickScribe/issues/428)) ([8442de2](https://github.com/Takenori-Kusaka/QuickScribe/commit/8442de27c5adba72cdc762d818e290d058b99797))
* **front:** プロバイダ定義・定数を constants.ts に集約(SSOT) ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase0) ([#410](https://github.com/Takenori-Kusaka/QuickScribe/issues/410)) ([752af0c](https://github.com/Takenori-Kusaka/QuickScribe/commit/752af0c4be1a585bae9b89cee5866bf677a9f6ef))
* **front:** プロバイダ鍵バリデーションをlib抽出＋テスト ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#426](https://github.com/Takenori-Kusaka/QuickScribe/issues/426)) ([e742ec0](https://github.com/Takenori-Kusaka/QuickScribe/commit/e742ec0df420db521f916c16dfad527de7976f04))
* **front:** モデルキャッシュ鮮度・横断発見ロジックをlib抽出＋テスト ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#430](https://github.com/Takenori-Kusaka/QuickScribe/issues/430)) ([31736b1](https://github.com/Takenori-Kusaka/QuickScribe/commit/31736b1c7eaea809b3581f0bf5e3e06039693cbc))
* **refine:** refine_text の16引数を RefineTextParams 構造体へ集約 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#545](https://github.com/Takenori-Kusaka/QuickScribe/issues/545)) ([ada4d6f](https://github.com/Takenori-Kusaka/QuickScribe/commit/ada4d6fcf424333af8666dfe37b7a0a6ecd9bd56))
* **refine:** RefineProvider enum で provider マッチを単一ソース化 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#536](https://github.com/Takenori-Kusaka/QuickScribe/issues/536)) ([9e55d4b](https://github.com/Takenori-Kusaka/QuickScribe/commit/9e55d4bd4abce6c55630f0d02cb2521e16171afe))
* **refine:** 整形エンジンのHTTP重複を共通ヘルパへ集約 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#533](https://github.com/Takenori-Kusaka/QuickScribe/issues/533)) ([2588fb6](https://github.com/Takenori-Kusaka/QuickScribe/commit/2588fb6f4e3a3e0f814d6c7a53ebb7e12c1e6708))
* **solid:** エントリ絞り込みロジックをlib抽出しテスト追加([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#524](https://github.com/Takenori-Kusaka/QuickScribe/issues/524)) ([35f81c7](https://github.com/Takenori-Kusaka/QuickScribe/commit/35f81c7594f5c39ad9ff46c84542ddbc4cea23ed))
* **solid:** 録音ソース値パースをlib抽出＋潜在バグ修正([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#525](https://github.com/Takenori-Kusaka/QuickScribe/issues/525)) ([2d3a4bf](https://github.com/Takenori-Kusaka/QuickScribe/commit/2d3a4bfd976f29166ee82aa4025da258b15d13a4))
* **stt:** TranscriptionEngine抽象(Strategy)を導入 (S2.3 [#24](https://github.com/Takenori-Kusaka/QuickScribe/issues/24)) ([#374](https://github.com/Takenori-Kusaka/QuickScribe/issues/374)) ([a52d0bb](https://github.com/Takenori-Kusaka/QuickScribe/commit/a52d0bb870ee55ff89ccf6d892d7ff6a1e884f34))
* **stt:** プロバイダ抽象をSttProvider enumに集約（[#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)の横展開 / [#581](https://github.com/Takenori-Kusaka/QuickScribe/issues/581)） ([#595](https://github.com/Takenori-Kusaka/QuickScribe/issues/595)) ([8d37517](https://github.com/Takenori-Kusaka/QuickScribe/commit/8d37517ac560e8ee37a3b4c20030cb77c943df13))


### 📝 ドキュメント / Docs

* 「なぜQuickScribe？」競合比較を読み手向けに公開([#396](https://github.com/Takenori-Kusaka/QuickScribe/issues/396)/[#399](https://github.com/Takenori-Kusaka/QuickScribe/issues/399)) ([#490](https://github.com/Takenori-Kusaka/QuickScribe/issues/490)) ([1d803ea](https://github.com/Takenori-Kusaka/QuickScribe/commit/1d803ea1206ef33c8766ce6a23f0b0272a0531a5))
* ADR-0005 技術スタック選定（Tauri 2） ([#1](https://github.com/Takenori-Kusaka/QuickScribe/issues/1)) ([054df11](https://github.com/Takenori-Kusaka/QuickScribe/commit/054df113621e3ecbbc7f4830e727aa356c5f58b8))
* **adr-0012:** Phase0/1完了・Phase2(AVX512)はVOI不成立で見送りを記録 ([12d699c](https://github.com/Takenori-Kusaka/QuickScribe/commit/12d699c6e8b447a074c8db9a2155380714fe54d6))
* **adr:** ADR-0009 リリースのバージョニングと v1.0.0 スコープ(現状=v0.1.0) ([cc48f08](https://github.com/Takenori-Kusaka/QuickScribe/commit/cc48f086435d53c8a44d85d8773efa68a946dcc7))
* **adr:** ADR-0013 システム音声ループバック＋録音ソース抽象の統一(S1.3 deep research) ([4116e27](https://github.com/Takenori-Kusaka/QuickScribe/commit/4116e27a950951c1410b7d6e1579d363c06b0a5c))
* **adr:** ADR-0024 評価基盤の再設計(CER刷新+ニュアンス計測) Proposed（[#578](https://github.com/Takenori-Kusaka/QuickScribe/issues/578) [#577](https://github.com/Takenori-Kusaka/QuickScribe/issues/577)） ([ab7cc13](https://github.com/Takenori-Kusaka/QuickScribe/commit/ab7cc13f8b13eb2c1dcfd0533812ad08e4f27b5e))
* **adr:** ADR-0026 複数バックグラウンド文字起こしジョブの逐次キュー化とジョブ一覧UI ([3998bcf](https://github.com/Takenori-Kusaka/QuickScribe/commit/3998bcf42e9d3b21ff68a50acb19fd2fbddb59a3))
* **adr:** ADR-0033 トレイ常駐時に WebView2 を保持する決定を明文化 ([#675](https://github.com/Takenori-Kusaka/QuickScribe/issues/675)) ([21b12c8](https://github.com/Takenori-Kusaka/QuickScribe/commit/21b12c8b76242868b82b35f71b79c911a603aa78))
* **adr:** ADR-0034 に「写してよい数字と写してはいけない数字」を足す ([#526](https://github.com/Takenori-Kusaka/QuickScribe/issues/526)) ([cb32e8b](https://github.com/Takenori-Kusaka/QuickScribe/commit/cb32e8b6a26f67e233987a00d88353b6bebc0dc8))
* **adr:** ADR-0034 依存脆弱性のリリース判定基準を起草する ([#526](https://github.com/Takenori-Kusaka/QuickScribe/issues/526)) ([3bda351](https://github.com/Takenori-Kusaka/QuickScribe/commit/3bda351499fc3ab1165e34320117a5cc350783e7))
* **adr:** ADR-0034 実装項目 a に npm override の実例を足す ([#683](https://github.com/Takenori-Kusaka/QuickScribe/issues/683)) ([f1119a2](https://github.com/Takenori-Kusaka/QuickScribe/commit/f1119a2178d397c427397a00897c545b343018ed)), closes [#526](https://github.com/Takenori-Kusaka/QuickScribe/issues/526)
* **adr:** ADR-0035 エージェント並行セッションの排他を起草する ([#680](https://github.com/Takenori-Kusaka/QuickScribe/issues/680)) ([#681](https://github.com/Takenori-Kusaka/QuickScribe/issues/681)) ([5705b06](https://github.com/Takenori-Kusaka/QuickScribe/commit/5705b06b76bc7ec5abb2dfd82e971e048dbf7db8))
* **articles:** Zenn/Qiita技術記事ドラフト＋公開CI基盤（[#59](https://github.com/Takenori-Kusaka/QuickScribe/issues/59)/[#16](https://github.com/Takenori-Kusaka/QuickScribe/issues/16)） ([#575](https://github.com/Takenori-Kusaka/QuickScribe/issues/575)) ([b84307d](https://github.com/Takenori-Kusaka/QuickScribe/commit/b84307d0858260f5e534f3d0bfaebd4f2e8ea445))
* **article:** 技術記事を「競合空白の非在理由」アングルで全面リライト ([#576](https://github.com/Takenori-Kusaka/QuickScribe/issues/576)) ([f61c3a3](https://github.com/Takenori-Kusaka/QuickScribe/commit/f61c3a3a5b52302dfb62f83999475d2db2e3e9a6))
* **book:** Zenn Bookのカバー画像を追加（ロゴ＋タイトル・500x700） ([33512c8](https://github.com/Takenori-Kusaka/QuickScribe/commit/33512c8f67ff74b8618d367e731fe815f31edb5d))
* **book:** Zenn Bookを公開（published: true） ([02a0334](https://github.com/Takenori-Kusaka/QuickScribe/commit/02a0334e2fe42e65d262486b7d9218c1f0e75cb1))
* **book:** 最終章「精度で殴らない」＋章順整理(全7章完結) ([#594](https://github.com/Takenori-Kusaka/QuickScribe/issues/594)) ([be49313](https://github.com/Takenori-Kusaka/QuickScribe/commit/be493131131fbc2c01ae87c53351aeba5edddf31))
* **book:** 第3章「整形の知性 ― 要約せずニュアンスを残す」 ([#588](https://github.com/Takenori-Kusaka/QuickScribe/issues/588)) ([313e8f3](https://github.com/Takenori-Kusaka/QuickScribe/commit/313e8f38b996d8903c6af73fef235616ca8a8814))
* **book:** 第4章「プライバシーは既定と表示の正直さで差がつく」 ([#589](https://github.com/Takenori-Kusaka/QuickScribe/issues/589)) ([b925b52](https://github.com/Takenori-Kusaka/QuickScribe/commit/b925b520bcb24161215dd10ee9c01a338dca022a))
* **book:** 第5章「捨てずに残して育てる ― データ設計」 ([#591](https://github.com/Takenori-Kusaka/QuickScribe/issues/591)) ([be4c4af](https://github.com/Takenori-Kusaka/QuickScribe/commit/be4c4af426aa201e0a5b8f60e7f4bf1608f16e4e))
* **book:** 第6章「物理ボタンひとつで話しはじめる」＋連載まとめ（最終章） ([#592](https://github.com/Takenori-Kusaka/QuickScribe/issues/592)) ([6054595](https://github.com/Takenori-Kusaka/QuickScribe/commit/6054595024466e47656b71ea861bd9269db36507))
* **book:** 追章「精度で殴らない、しかし床は抜かせない ― 既定モデルを実測で覆した日」 ([#614](https://github.com/Takenori-Kusaka/QuickScribe/issues/614)) ([fb17056](https://github.com/Takenori-Kusaka/QuickScribe/commit/fb17056ff2c9003547f2d287e9b606139aa5457e))
* **book:** 連載をZenn Book(books/quickscribe-design)に再編 ([#587](https://github.com/Takenori-Kusaka/QuickScribe/issues/587)) ([576a357](https://github.com/Takenori-Kusaka/QuickScribe/commit/576a3570f2ad3b85baebf6b2be4893c692aafc92))
* **community:** 行動規範(Contributor Covenant 2.1)＋Issueテンプレ config.yml (E8) ([3bc6f6f](https://github.com/Takenori-Kusaka/QuickScribe/commit/3bc6f6f5c535b8cc1b3797fd5f3a40ddf11eea1c))
* **github:** CODEOWNERS＋SUPPORT.md 追加 (OSSガバナンス / [#406](https://github.com/Takenori-Kusaka/QuickScribe/issues/406)) ([5865f52](https://github.com/Takenori-Kusaka/QuickScribe/commit/5865f5296f2e385cfa4786a5a38f77ecdbd7e41e))
* HANDOFF引継ぎ資料を最新化(v1.0.0プログラム進行状況) ([#423](https://github.com/Takenori-Kusaka/QuickScribe/issues/423)) ([05045c7](https://github.com/Takenori-Kusaka/QuickScribe/commit/05045c7b82d376cdc7ea8ce08daeacc7a1a8b834))
* HANDOFF更新(v0.6.4配信・[#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402)ゲート・[#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403) perf実測・[#427](https://github.com/Takenori-Kusaka/QuickScribe/issues/427)) ([#433](https://github.com/Takenori-Kusaka/QuickScribe/issues/433)) ([496c491](https://github.com/Takenori-Kusaka/QuickScribe/commit/496c491e7d5a32800e7ed3877b4f69d7801e9f93))
* **lib:** TSDoc(@param/[@returns](https://github.com/returns))整備＋typedoc検証をCIに追加 ([#393](https://github.com/Takenori-Kusaka/QuickScribe/issues/393)) ([#543](https://github.com/Takenori-Kusaka/QuickScribe/issues/543)) ([eee0e2e](https://github.com/Takenori-Kusaka/QuickScribe/commit/eee0e2e094ce1e02f0344fd409e054c2b83a4fd9))
* **license:** npm依存の第三者ライセンス帰属を生成・公開 ([#394](https://github.com/Takenori-Kusaka/QuickScribe/issues/394)) ([#421](https://github.com/Takenori-Kusaka/QuickScribe/issues/421)) ([555c8b8](https://github.com/Takenori-Kusaka/QuickScribe/commit/555c8b8aeb2026cdd992538f9adb87e571e945ef))
* **marketing:** ローンチキット(HN/Reddit/PH/LinkedIn文面+チェックリスト)([#59](https://github.com/Takenori-Kusaka/QuickScribe/issues/59) S9.5) ([#504](https://github.com/Takenori-Kusaka/QuickScribe/issues/504)) ([3b3b430](https://github.com/Takenori-Kusaka/QuickScribe/commit/3b3b4307a47cb978b62563fa660c1bd5c798e9e7))
* **marketing:** 英語版README/VitePress /en/ ロケール整備＋署名記述是正 ([#573](https://github.com/Takenori-Kusaka/QuickScribe/issues/573)) ([665ad81](https://github.com/Takenori-Kusaka/QuickScribe/commit/665ad8168a62c4d017c3c1d4f93b0ebf0a835d73))
* **migration:** migrate Zenn content to zenn-content submodule ([#691](https://github.com/Takenori-Kusaka/QuickScribe/issues/691)) ([892a4d7](https://github.com/Takenori-Kusaka/QuickScribe/commit/892a4d7371249fccea8a1999dfcbc9142ca2d6fb))
* NFRの誤ステータス是正とADR-0018/0019追加([#390](https://github.com/Takenori-Kusaka/QuickScribe/issues/390)/[#393](https://github.com/Takenori-Kusaka/QuickScribe/issues/393)) ([#485](https://github.com/Takenori-Kusaka/QuickScribe/issues/485)) ([3b2b747](https://github.com/Takenori-Kusaka/QuickScribe/commit/3b2b747e87f252b153c789febad19f14e4759b26))
* **perf:** 初回ベースライン実測を記録(RTF 0.857) ([#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403)) ([#432](https://github.com/Takenori-Kusaka/QuickScribe/issues/432)) ([478fc46](https://github.com/Takenori-Kusaka/QuickScribe/commit/478fc468fd9c0e962a946c3c93f5157327cf5a1e))
* **perf:** 日本語CERベースラインをCI実測値に確定（[#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403)） ([3925ee1](https://github.com/Takenori-Kusaka/QuickScribe/commit/3925ee13cf2df9d4e9a8e4ad49abdf28aa1bcf48))
* **planning:** 4. プロダクトバックログ(Epic→Story→Task)定義 ([#7](https://github.com/Takenori-Kusaka/QuickScribe/issues/7)) ([cfeb252](https://github.com/Takenori-Kusaka/QuickScribe/commit/cfeb252a57b58894b2fc566ea65fc1864be9be4b))
* **planning:** v1.0.0 リリース・レディネス監査(18観点の統合) (E9) ([df6e7fc](https://github.com/Takenori-Kusaka/QuickScribe/commit/df6e7fc63f313907df4d31603b7f78f796ea2d24))
* **planning:** ステップ3 企画計画一式 (3.1-3.7) ([#6](https://github.com/Takenori-Kusaka/QuickScribe/issues/6)) ([af63bcf](https://github.com/Takenori-Kusaka/QuickScribe/commit/af63bcf54d53eb492981f0c218c0c077c1fd4aed))
* **process:** デモGIFの実機キャプチャ手順 (S9.1 [#55](https://github.com/Takenori-Kusaka/QuickScribe/issues/55)) ([d29de27](https://github.com/Takenori-Kusaka/QuickScribe/commit/d29de27beb05f5b9a2a35f701a6bdd5d24e275ce))
* **process:** バージョニング/チャネル/鍵分離の方針を明文化([#52](https://github.com/Takenori-Kusaka/QuickScribe/issues/52) S8.5) ([#528](https://github.com/Takenori-Kusaka/QuickScribe/issues/528)) ([663e695](https://github.com/Takenori-Kusaka/QuickScribe/commit/663e695d1acd4d8ba2589a13d212f730471a800d))
* **process:** 配布・署名の手順とコストゼロ方針(SignPath OSS無料署名) (E8) ([c467127](https://github.com/Takenori-Kusaka/QuickScribe/commit/c4671270d73ad99fe09326c8ce0400d635c377e0))
* **readme:** バッジ/特徴/Quick start/Privacy節に刷新 (S9.1 [#55](https://github.com/Takenori-Kusaka/QuickScribe/issues/55)) ([925cce3](https://github.com/Takenori-Kusaka/QuickScribe/commit/925cce3b1aa58827b7bad96e07043d631744de59))
* **readme:** 実スクリーンショット差替とCI自動化の修正([#396](https://github.com/Takenori-Kusaka/QuickScribe/issues/396)/[#387](https://github.com/Takenori-Kusaka/QuickScribe/issues/387)/S9.1) ([#487](https://github.com/Takenori-Kusaka/QuickScribe/issues/487)) ([adf1528](https://github.com/Takenori-Kusaka/QuickScribe/commit/adf1528f782699f4d4bfe57036ce8c7dfd4e8361))
* **readme:** 視覚素材をGIFからスクリーンショットに変更 (S9.1 [#55](https://github.com/Takenori-Kusaka/QuickScribe/issues/55)) ([31ab6bd](https://github.com/Takenori-Kusaka/QuickScribe/commit/31ab6bd0b1ed15b967af2406c6cdfe754170532e))
* **research:** GPU/実行オプション拡充の調査（全PC環境対応・CPU並列・配布両立） ([6fe53f4](https://github.com/Takenori-Kusaka/QuickScribe/commit/6fe53f4c0cca2714f04a8263d125b734792d5678))
* **research:** GPU変種のドライバ前提UX調査（案内/自動導入の可否・最低ドライバ版・Vulkan代替） ([c64e1e9](https://github.com/Takenori-Kusaka/QuickScribe/commit/c64e1e93ccb9b83c00a28a108175349724cfdbb1))
* **research:** RTX 4060 実測を追記 — GPU(CUDA)で 196分→5.5分(36倍・RTF0.33) ([fc2ff1c](https://github.com/Takenori-Kusaka/QuickScribe/commit/fc2ff1c57fece8761230ab1dfbd2531c68609d60))
* **research:** turbo文字起こし高速化の調査（180分=反復ループ暴走の主因確定） ([ee037f8](https://github.com/Takenori-Kusaka/QuickScribe/commit/ee037f8a935be2c09cf029e2704b14a50daca479))
* **research:** 整形出力言語・翻訳オプションの設計調査([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)関連) ([#452](https://github.com/Takenori-Kusaka/QuickScribe/issues/452)) ([4a992cb](https://github.com/Takenori-Kusaka/QuickScribe/commit/4a992cb5d68b2abac769a2f76a52c4d1f42cc00a))
* **research:** 文字起こし効率化の限界調査（streaming vs チャンク化・日本語特性・性能限界） ([c68b7df](https://github.com/Takenori-Kusaka/QuickScribe/commit/c68b7df4a894b70f78767fff74d4f0ecc2475ebd))
* **research:** 競合ランドスケープ分析を作成(一次情報) ([#399](https://github.com/Takenori-Kusaka/QuickScribe/issues/399)) ([#424](https://github.com/Takenori-Kusaka/QuickScribe/issues/424)) ([b88d7f8](https://github.com/Takenori-Kusaka/QuickScribe/commit/b88d7f83aa6213cf1671dce29571b470f7a53d60))
* **security:** SECURITY.md(最新版のみ/PVR/プライバシー方針) (S8.6 [#53](https://github.com/Takenori-Kusaka/QuickScribe/issues/53)) ([20fc80f](https://github.com/Takenori-Kusaka/QuickScribe/commit/20fc80f18dca2a989b4765da2e07fd6ea1538ff8))
* **site:** 「保管庫」表記をアプリに合わせ「ジャーナル」へ統一 ([#388](https://github.com/Takenori-Kusaka/QuickScribe/issues/388)追従) ([bc54195](https://github.com/Takenori-Kusaka/QuickScribe/commit/bc54195179b3068a387c41546850aa9b040eea19))
* v1.0.0最終監査の是正（チャネル文書の矛盾解消・NFR実測同期・docs目次新設） ([b089f99](https://github.com/Takenori-Kusaka/QuickScribe/commit/b089f99e520042d71b7bf1638811b6cbb120466d)), closes [#481](https://github.com/Takenori-Kusaka/QuickScribe/issues/481)
* アーキテクチャ設計(design.md)＋非機能要件(NFR)集約 ([#390](https://github.com/Takenori-Kusaka/QuickScribe/issues/390)) ([#420](https://github.com/Takenori-Kusaka/QuickScribe/issues/420)) ([e0fad8b](https://github.com/Takenori-Kusaka/QuickScribe/commit/e0fad8b204757eda6cd43630bb915bdb2bee042d))
* スコープ完全性ポリシー(ADR-0006)と駆動開発リサーチを追加 ([e41c153](https://github.com/Takenori-Kusaka/QuickScribe/commit/e41c153993f9555185977e03bf6af4796a8ddff5))
* ドキュメントの矛盾・陳腐化を解消(ADR索引/署名方針/vision) ([#390](https://github.com/Takenori-Kusaka/QuickScribe/issues/390)) ([#419](https://github.com/Takenori-Kusaka/QuickScribe/issues/419)) ([3554622](https://github.com/Takenori-Kusaka/QuickScribe/commit/3554622193b449f0ebabfbe3458e495c2377e329))
* ライセンス・収益・配布方針を確定 (ADR-0008) ([#61](https://github.com/Takenori-Kusaka/QuickScribe/issues/61)) ([210b4d4](https://github.com/Takenori-Kusaka/QuickScribe/commit/210b4d43530cd8cb56580dd05fc9f1aadad56911))
* 仕様駆動開発の深掘り再調査（原典スナップショット付き） ([#3](https://github.com/Takenori-Kusaka/QuickScribe/issues/3)) ([5d81106](https://github.com/Takenori-Kusaka/QuickScribe/commit/5d811062746ea6ce297e6fd996a7c88203065d0b))
* 企画基盤（ビジョン＋ADR-0001〜0004）を確立 ([c063375](https://github.com/Takenori-Kusaka/QuickScribe/commit/c0633750e28ebe0d3cafffb21d56e94b57b1702d))
* 未署名の事実を反映しSignPath「利用/整備中」表記を是正([#50](https://github.com/Takenori-Kusaka/QuickScribe/issues/50)) ([#522](https://github.com/Takenori-Kusaka/QuickScribe/issues/522)) ([c549d3e](https://github.com/Takenori-Kusaka/QuickScribe/commit/c549d3e097e543d254baed5e7fd696e3d29ce611))
* 本質的な問いの立て方メソッドを標準化(ADR-0007) ([#2](https://github.com/Takenori-Kusaka/QuickScribe/issues/2)) ([a0eab7e](https://github.com/Takenori-Kusaka/QuickScribe/commit/a0eab7e780f767a371792a44c04a6c5eec477dfb))
* 音声ファイル変換/中間ファイル保存を追加(S1.6/S1.7) ([#298](https://github.com/Takenori-Kusaka/QuickScribe/issues/298)) ([3495832](https://github.com/Takenori-Kusaka/QuickScribe/commit/349583214bc0d5eaf064260c902b0bf41db88dfe))


### 🔧 雑務 / Chore

* release 1.5.0 ([e6b5e6c](https://github.com/Takenori-Kusaka/QuickScribe/commit/e6b5e6c56a2b8b4547b158ab31181d0b91fbbff7))
* v1.0.0 リリース（最終監査完了・Readiness達成） ([1682c28](https://github.com/Takenori-Kusaka/QuickScribe/commit/1682c28698167dccf30a84c953375d78deb22b37))

## [1.11.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.9.0...v1.11.0) (2026-07-30)


### ✨ 新機能 / Features

* **cli:** 音声文字起こしやLLM整形などの各機能をCLIから直接実行可能にする ([#672](https://github.com/Takenori-Kusaka/QuickScribe/issues/672))

## [1.9.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.8.0...v1.9.0) (2026-07-24)


### ✨ 新機能 / Features

* **vault:** ファイル名を日付先頭 {yyyymmdd}-{種別}-{ラベル} に変更 (ADR-0032追記) ([#659](https://github.com/Takenori-Kusaka/QuickScribe/issues/659)) ([553cfd5](https://github.com/Takenori-Kusaka/QuickScribe/commit/553cfd5cf3df576ce9c1e78a0917b60230ccbe11))

## [1.8.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.7.0...v1.8.0) (2026-07-24)


### ✨ 新機能 / Features

* **vault:** エントリ名を日付+内容由来ラベルに変更 (ADR-0032) ([#657](https://github.com/Takenori-Kusaka/QuickScribe/issues/657)) ([918c443](https://github.com/Takenori-Kusaka/QuickScribe/commit/918c443b254224919a145529b3c7409ca7938d82))

## [1.7.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.6.0...v1.7.0) (2026-07-15)


### ✨ 新機能 / Features

* **jobs:** ジョブ一覧を直近3件表示＋展開/スクロールで表示エリアの無限拡大を防ぐ ([#644](https://github.com/Takenori-Kusaka/QuickScribe/issues/644)) ([c34098f](https://github.com/Takenori-Kusaka/QuickScribe/commit/c34098fca0155d3010c2583bda82523fe4976874))

## [1.6.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.5.0...v1.6.0) (2026-07-10)


### ✨ 新機能 / Features

* **stt:** 話者特定オプション(default-OFF)を追加 ― sherpa-onnx・オンデマンドDL・遅延ロード(ADR-0031) ([#633](https://github.com/Takenori-Kusaka/QuickScribe/issues/633)) ([8d432c5](https://github.com/Takenori-Kusaka/QuickScribe/commit/8d432c506360a91d75f9ddc99cf52b5f869b993f))

## [1.5.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.4.0...v1.5.0) (2026-07-09)

配布とモデルカタログを簡素化しました（[ADR-0029](https://github.com/Takenori-Kusaka/QuickScribe/blob/main/docs/adr/0029-simplify-offering-drop-cuda-and-kotoba.md)）。GPU文字起こしは v1.4.0 の単一 Vulkan ビルドで CPU〜あらゆる GPU を1つのインストーラでカバーできるため、価値の薄い選択肢を整理しました。

### ♻️ 変更 / Changes

* **CUDA 変種を廃止**しました。Vulkan は NVIDIA GPU でも CUDA とほぼ同速（実測で差 約11%）でありながら、専用ドライバや DLL 同梱を必要とせず単一インストーラで完結します。2つ目の別インストーラを保守する意味が無くなったため撤去しました（[#631](https://github.com/Takenori-Kusaka/QuickScribe/pull/631)）。
* **kotoba-whisper モデルをモデル一覧から撤去**しました。本アプリのコア用途である自発的な発話（会話寄り）では精度が崩れやすく（会話 CER 実測で large-v3-turbo の約2.7倍）、長尺で末尾が欠落する弱点があり、開発元の重み更新も停止しているためです。**kotoba を選択していた場合は、自動的に安全な既定モデル（base）で動作します**。日本語の推奨は引き続き large-v3-turbo です（[#631](https://github.com/Takenori-Kusaka/QuickScribe/pull/631)）。

### 🔧 雑務 / Chore

* release 1.5.0 ([e6b5e6c](https://github.com/Takenori-Kusaka/QuickScribe/commit/e6b5e6c56a2b8b4547b158ab31181d0b91fbbff7))

## [1.4.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.3.0...v1.4.0) (2026-07-09)


### ✨ 新機能 / Features

* **gpu:** CUDA変種のドライバ前提UX（NSIS案内＋アプリ内リンク / ADR-0027 Phase2） ([#628](https://github.com/Takenori-Kusaka/QuickScribe/issues/628)) ([b5abf7e](https://github.com/Takenori-Kusaka/QuickScribe/commit/b5abf7e6bbfa819ae2e5e8c3fc0a7c9f523c7d6e))
* **stt:** GPU配布を単一Vulkanビルドへ一本化＋起動時デバイス検出（ADR-0028） ([#629](https://github.com/Takenori-Kusaka/QuickScribe/issues/629)) ([cf89c3d](https://github.com/Takenori-Kusaka/QuickScribe/commit/cf89c3dd925f19ca6d1f39a8f4507ec1d0e530d4))


### 📝 ドキュメント / Docs

* **research:** GPU変種のドライバ前提UX調査（案内/自動導入の可否・最低ドライバ版・Vulkan代替） ([c64e1e9](https://github.com/Takenori-Kusaka/QuickScribe/commit/c64e1e93ccb9b83c00a28a108175349724cfdbb1))

## [1.3.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.2.1...v1.3.0) (2026-07-09)


### ✨ 新機能 / Features

* **stt:** マルチジョブ逐次キューのバックエンド（ADR-0026 [#621](https://github.com/Takenori-Kusaka/QuickScribe/issues/621) Phase1） ([#622](https://github.com/Takenori-Kusaka/QuickScribe/issues/622)) ([e50dc3c](https://github.com/Takenori-Kusaka/QuickScribe/commit/e50dc3c101dd7ece3a786cc81af3e19c7d1194fb))
* **ui:** アプリのバージョンを表示（実行結果の共有用） ([#625](https://github.com/Takenori-Kusaka/QuickScribe/issues/625)) ([d4309cd](https://github.com/Takenori-Kusaka/QuickScribe/commit/d4309cd94153d2fb09036379a42b32daf655384c))
* **ui:** マルチジョブ・ジョブ一覧UI（ADR-0026 [#621](https://github.com/Takenori-Kusaka/QuickScribe/issues/621) Phase2） ([#624](https://github.com/Takenori-Kusaka/QuickScribe/issues/624)) ([41540e7](https://github.com/Takenori-Kusaka/QuickScribe/commit/41540e71b4b774c00e76b211b95f42b4dd74bcc6))


### 📝 ドキュメント / Docs

* **adr:** ADR-0026 複数バックグラウンド文字起こしジョブの逐次キュー化とジョブ一覧UI ([3998bcf](https://github.com/Takenori-Kusaka/QuickScribe/commit/3998bcf42e9d3b21ff68a50acb19fd2fbddb59a3))
* **research:** GPU/実行オプション拡充の調査（全PC環境対応・CPU並列・配布両立） ([6fe53f4](https://github.com/Takenori-Kusaka/QuickScribe/commit/6fe53f4c0cca2714f04a8263d125b734792d5678))
* **research:** RTX 4060 実測を追記 — GPU(CUDA)で 196分→5.5分(36倍・RTF0.33) ([fc2ff1c](https://github.com/Takenori-Kusaka/QuickScribe/commit/fc2ff1c57fece8761230ab1dfbd2531c68609d60))
* **research:** turbo文字起こし高速化の調査（180分=反復ループ暴走の主因確定） ([ee037f8](https://github.com/Takenori-Kusaka/QuickScribe/commit/ee037f8a935be2c09cf029e2704b14a50daca479))
* **research:** 文字起こし効率化の限界調査（streaming vs チャンク化・日本語特性・性能限界） ([c68b7df](https://github.com/Takenori-Kusaka/QuickScribe/commit/c68b7df4a894b70f78767fff74d4f0ecc2475ebd))

## [1.2.1](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.2.0...v1.2.1) (2026-07-07)


### 🐛 修正 / Bug Fixes

* **release:** リリースノートを自動復旧するジョブを追加（[#572](https://github.com/Takenori-Kusaka/QuickScribe/issues/572) 恒久修正） ([#612](https://github.com/Takenori-Kusaka/QuickScribe/issues/612)) ([db9ce3d](https://github.com/Takenori-Kusaka/QuickScribe/commit/db9ce3d0a305b30dea5c9a8ea1e9be568e85ad52))


### 📝 ドキュメント / Docs

* **book:** 追章「精度で殴らない、しかし床は抜かせない ― 既定モデルを実測で覆した日」 ([#614](https://github.com/Takenori-Kusaka/QuickScribe/issues/614)) ([fb17056](https://github.com/Takenori-Kusaka/QuickScribe/commit/fb17056ff2c9003547f2d287e9b606139aa5457e))

## [1.2.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.1.0...v1.2.0) (2026-07-06)


### ✨ 新機能 / Features

* **stt:** 日本語の既定STTを kotoba-q5 → large-v3-turbo へ（実測に基づく / ADR-0025） ([#610](https://github.com/Takenori-Kusaka/QuickScribe/issues/610)) ([1d23c3f](https://github.com/Takenori-Kusaka/QuickScribe/commit/1d23c3f12392e70836bdd4ffb073380aec25d462))

## [1.1.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v1.0.0...v1.1.0) (2026-07-06)


### ✨ 新機能 / Features

* **eval:** Common Voice ja(CC0)の日本語CERベンチをCIに追加（ADR-0024第二スライス / [#578](https://github.com/Takenori-Kusaka/QuickScribe/issues/578)） ([#602](https://github.com/Takenori-Kusaka/QuickScribe/issues/602)) ([c4e5dd7](https://github.com/Takenori-Kusaka/QuickScribe/commit/c4e5dd72773f191c7b27bb37e60352ec8af0405e))
* **eval:** FLEURS ja(CC-BY)を2つ目の定点コーパスに追加＋fetch汎用化（ADR-0024第二スライス / [#578](https://github.com/Takenori-Kusaka/QuickScribe/issues/578)） ([#609](https://github.com/Takenori-Kusaka/QuickScribe/issues/609)) ([2d29f99](https://github.com/Takenori-Kusaka/QuickScribe/commit/2d29f99c8d5290b1ba8786ce7c5b3645c20154b6))
* **eval:** 日本語ASR評価コア（正規化・CER・ブートストラップCI）＋ADR-0024 Accepted（[#578](https://github.com/Takenori-Kusaka/QuickScribe/issues/578)） ([#601](https://github.com/Takenori-Kusaka/QuickScribe/issues/601)) ([1a7be52](https://github.com/Takenori-Kusaka/QuickScribe/commit/1a7be52ee03c0f9f02289625404432e1dba05c19))
* **models:** モデル選択に相対処理速度クラスを表示（[#598](https://github.com/Takenori-Kusaka/QuickScribe/issues/598) 第一スライス） ([#608](https://github.com/Takenori-Kusaka/QuickScribe/issues/608)) ([a3ddd94](https://github.com/Takenori-Kusaka/QuickScribe/commit/a3ddd947db1195932f861433b426cbc756001932))
* **refine:** OpenAI互換base_url設定可能化＋プライバシー判定のURLホスト評価化（[#593](https://github.com/Takenori-Kusaka/QuickScribe/issues/593)） ([#597](https://github.com/Takenori-Kusaka/QuickScribe/issues/597)) ([40f4a94](https://github.com/Takenori-Kusaka/QuickScribe/commit/40f4a945a01e11334a7e540b46783d0af8cb4a0f))
* **ux:** 文字起こし完結ユーザー向けUX（出力フォルダボタン＋未設定でも保存可・警告のみ）（[#603](https://github.com/Takenori-Kusaka/QuickScribe/issues/603)） ([#605](https://github.com/Takenori-Kusaka/QuickScribe/issues/605)) ([920a592](https://github.com/Takenori-Kusaka/QuickScribe/commit/920a59264a6d6971531be3bb83ae99b2e0fee3a7))


### 🐛 修正 / Bug Fixes

* **ci:** PlantUMLの太字日本語(豆腐)対策にfonts-noto-cjk導入 ([#586](https://github.com/Takenori-Kusaka/QuickScribe/issues/586)) ([b74a69b](https://github.com/Takenori-Kusaka/QuickScribe/commit/b74a69bb1b8451ae251ca8df6a90675204d3a6f7))
* **corrections:** 用語補正の置換を補正版として別ファイル保存（非破壊 / [#599](https://github.com/Takenori-Kusaka/QuickScribe/issues/599)） ([#607](https://github.com/Takenori-Kusaka/QuickScribe/issues/607)) ([66b4d7f](https://github.com/Takenori-Kusaka/QuickScribe/commit/66b4d7f92f9b40fdd34a276961311408894670ab))
* **release:** release-pleaseのノート上書きを解消（releaseBody除去 / [#572](https://github.com/Takenori-Kusaka/QuickScribe/issues/572)） ([#596](https://github.com/Takenori-Kusaka/QuickScribe/issues/596)) ([c6c42d7](https://github.com/Takenori-Kusaka/QuickScribe/commit/c6c42d7a5fcb4755fafb6bb629305d26c0f218f9))


### ♻️ リファクタ / Refactor

* **stt:** プロバイダ抽象をSttProvider enumに集約（[#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)の横展開 / [#581](https://github.com/Takenori-Kusaka/QuickScribe/issues/581)） ([#595](https://github.com/Takenori-Kusaka/QuickScribe/issues/595)) ([8d37517](https://github.com/Takenori-Kusaka/QuickScribe/commit/8d37517ac560e8ee37a3b4c20030cb77c943df13))


### 📝 ドキュメント / Docs

* **adr:** ADR-0024 評価基盤の再設計(CER刷新+ニュアンス計測) Proposed（[#578](https://github.com/Takenori-Kusaka/QuickScribe/issues/578) [#577](https://github.com/Takenori-Kusaka/QuickScribe/issues/577)） ([ab7cc13](https://github.com/Takenori-Kusaka/QuickScribe/commit/ab7cc13f8b13eb2c1dcfd0533812ad08e4f27b5e))
* **articles:** Zenn/Qiita技術記事ドラフト＋公開CI基盤（[#59](https://github.com/Takenori-Kusaka/QuickScribe/issues/59)/[#16](https://github.com/Takenori-Kusaka/QuickScribe/issues/16)） ([#575](https://github.com/Takenori-Kusaka/QuickScribe/issues/575)) ([b84307d](https://github.com/Takenori-Kusaka/QuickScribe/commit/b84307d0858260f5e534f3d0bfaebd4f2e8ea445))
* **article:** 技術記事を「競合空白の非在理由」アングルで全面リライト ([#576](https://github.com/Takenori-Kusaka/QuickScribe/issues/576)) ([f61c3a3](https://github.com/Takenori-Kusaka/QuickScribe/commit/f61c3a3a5b52302dfb62f83999475d2db2e3e9a6))
* **book:** Zenn Bookのカバー画像を追加（ロゴ＋タイトル・500x700） ([33512c8](https://github.com/Takenori-Kusaka/QuickScribe/commit/33512c8f67ff74b8618d367e731fe815f31edb5d))
* **book:** Zenn Bookを公開（published: true） ([02a0334](https://github.com/Takenori-Kusaka/QuickScribe/commit/02a0334e2fe42e65d262486b7d9218c1f0e75cb1))
* **book:** 最終章「精度で殴らない」＋章順整理(全7章完結) ([#594](https://github.com/Takenori-Kusaka/QuickScribe/issues/594)) ([be49313](https://github.com/Takenori-Kusaka/QuickScribe/commit/be493131131fbc2c01ae87c53351aeba5edddf31))
* **book:** 第3章「整形の知性 ― 要約せずニュアンスを残す」 ([#588](https://github.com/Takenori-Kusaka/QuickScribe/issues/588)) ([313e8f3](https://github.com/Takenori-Kusaka/QuickScribe/commit/313e8f38b996d8903c6af73fef235616ca8a8814))
* **book:** 第4章「プライバシーは既定と表示の正直さで差がつく」 ([#589](https://github.com/Takenori-Kusaka/QuickScribe/issues/589)) ([b925b52](https://github.com/Takenori-Kusaka/QuickScribe/commit/b925b520bcb24161215dd10ee9c01a338dca022a))
* **book:** 第5章「捨てずに残して育てる ― データ設計」 ([#591](https://github.com/Takenori-Kusaka/QuickScribe/issues/591)) ([be4c4af](https://github.com/Takenori-Kusaka/QuickScribe/commit/be4c4af426aa201e0a5b8f60e7f4bf1608f16e4e))
* **book:** 第6章「物理ボタンひとつで話しはじめる」＋連載まとめ（最終章） ([#592](https://github.com/Takenori-Kusaka/QuickScribe/issues/592)) ([6054595](https://github.com/Takenori-Kusaka/QuickScribe/commit/6054595024466e47656b71ea861bd9269db36507))
* **book:** 連載をZenn Book(books/quickscribe-design)に再編 ([#587](https://github.com/Takenori-Kusaka/QuickScribe/issues/587)) ([576a357](https://github.com/Takenori-Kusaka/QuickScribe/commit/576a3570f2ad3b85baebf6b2be4893c692aafc92))
* **marketing:** 英語版README/VitePress /en/ ロケール整備＋署名記述是正 ([#573](https://github.com/Takenori-Kusaka/QuickScribe/issues/573)) ([665ad81](https://github.com/Takenori-Kusaka/QuickScribe/commit/665ad8168a62c4d017c3c1d4f93b0ebf0a835d73))

## [1.0.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v0.10.0...v1.0.0) (2026-07-04)


### ✨ 新機能 / Features

* **design:** ブランドカラーのトークン化とstylelintによる生hex禁止 ([#567](https://github.com/Takenori-Kusaka/QuickScribe/issues/567)) ([8579473](https://github.com/Takenori-Kusaka/QuickScribe/commit/8579473f5869a3a3145029db8c71853e55617783))
* **i18n:** refine.rs のユーザー向けエラーも安定コード化 ([#462](https://github.com/Takenori-Kusaka/QuickScribe/issues/462)) ([#538](https://github.com/Takenori-Kusaka/QuickScribe/issues/538)) ([67c4e11](https://github.com/Takenori-Kusaka/QuickScribe/commit/67c4e112eb312cb0cc020b7eb806d9dfa3dda74f))
* **i18n:** Rustバックエンドのエラー文字列を安定コード化 ([#462](https://github.com/Takenori-Kusaka/QuickScribe/issues/462)) ([#537](https://github.com/Takenori-Kusaka/QuickScribe/issues/537)) ([4b2cb7e](https://github.com/Takenori-Kusaka/QuickScribe/commit/4b2cb7e1597067202537bba0333ac173776be22b))
* **installer:** アンインストール時に DL済み whisper モデルを掃除（NSIS フック） ([#511](https://github.com/Takenori-Kusaka/QuickScribe/issues/511)) ([#549](https://github.com/Takenori-Kusaka/QuickScribe/issues/549)) ([d162f90](https://github.com/Takenori-Kusaka/QuickScribe/commit/d162f9092a087adb3b527cf5630e923433c7467c))
* **nudge:** 習慣ナッジ＝起動アンカーのopt-inローカル通知 ([#58](https://github.com/Takenori-Kusaka/QuickScribe/issues/58)) ([#563](https://github.com/Takenori-Kusaka/QuickScribe/issues/563)) ([7dae8b5](https://github.com/Takenori-Kusaka/QuickScribe/commit/7dae8b5b4499f16d5d2d21626e533c96a1f35c74))
* **perf:** 日本語精度CERベンチ([#26](https://github.com/Takenori-Kusaka/QuickScribe/issues/26))＋モデルカタログ精選(ADR-0022) ([#561](https://github.com/Takenori-Kusaka/QuickScribe/issues/561)) ([2249087](https://github.com/Takenori-Kusaka/QuickScribe/commit/22490874ae7a8f2db1516ec9c6794eb959c1c8b3))
* **perf:** 起動時間ベンチを追加（run()→UI ready をアプリ計装＋xvfbでCI計測） ([#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403)) ([#554](https://github.com/Takenori-Kusaka/QuickScribe/issues/554)) ([1a6159e](https://github.com/Takenori-Kusaka/QuickScribe/commit/1a6159e20146872d92c63989009fe64c822f3b59))
* **perf:** 非機能ベンチに英語CER(精度指標)を追加 ([#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403)) ([#550](https://github.com/Takenori-Kusaka/QuickScribe/issues/550)) ([37ab293](https://github.com/Takenori-Kusaka/QuickScribe/commit/37ab2931392cdb175fea5283b4e0922d3c57a563))
* **privacy:** クラウド整形プロバイダ選択時に送信同意の警告を表示 ([#465](https://github.com/Takenori-Kusaka/QuickScribe/issues/465)) ([#547](https://github.com/Takenori-Kusaka/QuickScribe/issues/547)) ([196de28](https://github.com/Takenori-Kusaka/QuickScribe/commit/196de284de434969c89d4cc9025f92d52a54a39e))
* **privacy:** ローカルファースト既定へ変更（整形=Ollama / 日本語STT=kotoba）+ ADR-0021 ([#465](https://github.com/Takenori-Kusaka/QuickScribe/issues/465) [#511](https://github.com/Takenori-Kusaka/QuickScribe/issues/511)) ([#546](https://github.com/Takenori-Kusaka/QuickScribe/issues/546)) ([d030601](https://github.com/Takenori-Kusaka/QuickScribe/commit/d030601e94e4182e97c2e9cedb99e136dec6a6b3))
* **release:** nightlyチャネルとupdater鍵分離Runbook ([#52](https://github.com/Takenori-Kusaka/QuickScribe/issues/52)) ([#564](https://github.com/Takenori-Kusaka/QuickScribe/issues/564)) ([61b88d8](https://github.com/Takenori-Kusaka/QuickScribe/commit/61b88d8887f6221d2689f6bc51fd62f4fb385d3f))


### 🐛 修正 / Bug Fixes

* **ci:** カバレッジ監査是正 — rust-coverageジョブ修復とFE branches 80%ゲート化 ([#566](https://github.com/Takenori-Kusaka/QuickScribe/issues/566)) ([0c4adcf](https://github.com/Takenori-Kusaka/QuickScribe/commit/0c4adcfedc3d824bda1ab06c535366beeef3762b))
* **i18n:** TS/Rust文言のSSOT是正とカタログ一本化（監査項目8/11/17/22） ([#570](https://github.com/Takenori-Kusaka/QuickScribe/issues/570)) ([58da8ad](https://github.com/Takenori-Kusaka/QuickScribe/commit/58da8ad7b1d5d9961c1b61bb94ef4b1beed751ce))
* **logging:** タスクバー診断ログを出力先(ドキュメント)からOSのLocalデータ領域へ移す ([#552](https://github.com/Takenori-Kusaka/QuickScribe/issues/552)) ([e821add](https://github.com/Takenori-Kusaka/QuickScribe/commit/e821add43f82321eb3836c36d57d85db9ee5e885))
* **perf:** 日本語精度ジョブにアイコン生成を追加 ([#562](https://github.com/Takenori-Kusaka/QuickScribe/issues/562)) ([1964bd4](https://github.com/Takenori-Kusaka/QuickScribe/commit/1964bd459e1c14f57687764970d97846d8dc8660))
* **stt:** 自分で保存した .opus 録音を再文字起こし可能にする ([#560](https://github.com/Takenori-Kusaka/QuickScribe/issues/560)) ([3f5da61](https://github.com/Takenori-Kusaka/QuickScribe/commit/3f5da61b06c648e7ac47cd5b367d3303507dd84f))


### ♻️ リファクタ / Refactor

* **app:** App.svelte を &lt;800 行に分割（shortcut/device/custom-styles/privacy 抽出） ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#544](https://github.com/Takenori-Kusaka/QuickScribe/issues/544)) ([325db7a](https://github.com/Takenori-Kusaka/QuickScribe/commit/325db7abeaaead3ff1e1cf5837bed91da41caeab))
* **app:** 保管庫閲覧を vault-view.svelte.ts へ抽出 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#539](https://github.com/Takenori-Kusaka/QuickScribe/issues/539)) ([cd527d3](https://github.com/Takenori-Kusaka/QuickScribe/commit/cd527d31eae3f56eead7ffb75ae5e55613935311))
* **app:** 秘密情報ブリッジを lib/secrets.ts へ抽出 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#540](https://github.com/Takenori-Kusaka/QuickScribe/issues/540)) ([6843671](https://github.com/Takenori-Kusaka/QuickScribe/commit/6843671594eb763c1fefc0a51545fcf1f6333df5))
* **app:** 自動アップデートを lib/update.svelte.ts へ抽出 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#542](https://github.com/Takenori-Kusaka/QuickScribe/issues/542)) ([519e366](https://github.com/Takenori-Kusaka/QuickScribe/commit/519e366d4a8e0e494c4ba114e34d8e31bf2f0471))
* **app:** 設定永続化を lib/settings-persist.ts へ抽出 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#541](https://github.com/Takenori-Kusaka/QuickScribe/issues/541)) ([8ecf365](https://github.com/Takenori-Kusaka/QuickScribe/commit/8ecf3652cb3dae9f9843f243267eb4ebf330db05))
* **entry:** 保管庫ドキュメントドメインを entry.rs へ抽出 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#535](https://github.com/Takenori-Kusaka/QuickScribe/issues/535)) ([79081f7](https://github.com/Takenori-Kusaka/QuickScribe/commit/79081f75c39c644e25766182b2e1a934cff9c8a1))
* **refine:** refine_text の16引数を RefineTextParams 構造体へ集約 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#545](https://github.com/Takenori-Kusaka/QuickScribe/issues/545)) ([ada4d6f](https://github.com/Takenori-Kusaka/QuickScribe/commit/ada4d6fcf424333af8666dfe37b7a0a6ecd9bd56))
* **refine:** RefineProvider enum で provider マッチを単一ソース化 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#536](https://github.com/Takenori-Kusaka/QuickScribe/issues/536)) ([9e55d4b](https://github.com/Takenori-Kusaka/QuickScribe/commit/9e55d4bd4abce6c55630f0d02cb2521e16171afe))
* **refine:** 整形エンジンのHTTP重複を共通ヘルパへ集約 ([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#533](https://github.com/Takenori-Kusaka/QuickScribe/issues/533)) ([2588fb6](https://github.com/Takenori-Kusaka/QuickScribe/commit/2588fb6f4e3a3e0f814d6c7a53ebb7e12c1e6708))


### 📝 ドキュメント / Docs

* **lib:** TSDoc(@param/[@returns](https://github.com/returns))整備＋typedoc検証をCIに追加 ([#393](https://github.com/Takenori-Kusaka/QuickScribe/issues/393)) ([#543](https://github.com/Takenori-Kusaka/QuickScribe/issues/543)) ([eee0e2e](https://github.com/Takenori-Kusaka/QuickScribe/commit/eee0e2e094ce1e02f0344fd409e054c2b83a4fd9))
* **perf:** 日本語CERベースラインをCI実測値に確定（[#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403)） ([3925ee1](https://github.com/Takenori-Kusaka/QuickScribe/commit/3925ee13cf2df9d4e9a8e4ad49abdf28aa1bcf48))
* v1.0.0最終監査の是正（チャネル文書の矛盾解消・NFR実測同期・docs目次新設） ([b089f99](https://github.com/Takenori-Kusaka/QuickScribe/commit/b089f99e520042d71b7bf1638811b6cbb120466d)), closes [#481](https://github.com/Takenori-Kusaka/QuickScribe/issues/481)


### 🔧 雑務 / Chore

* v1.0.0 リリース（最終監査完了・Readiness達成） ([1682c28](https://github.com/Takenori-Kusaka/QuickScribe/commit/1682c28698167dccf30a84c953375d78deb22b37))

## [0.10.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v0.9.0...v0.10.0) (2026-07-02)


### ✨ 新機能 / Features

* **journal:** 習慣ストリーク(寛容)をジャーナルに表示([#58](https://github.com/Takenori-Kusaka/QuickScribe/issues/58) S9.4 第1弾) ([#502](https://github.com/Takenori-Kusaka/QuickScribe/issues/502)) ([aa2ebe9](https://github.com/Takenori-Kusaka/QuickScribe/commit/aa2ebe9ce2bf427a15754413b1575a4acbc974e0))
* **metrics:** リリースDL数の計測とテレメトリ方針の明文化([#60](https://github.com/Takenori-Kusaka/QuickScribe/issues/60) S9.6) ([#501](https://github.com/Takenori-Kusaka/QuickScribe/issues/501)) ([5772be8](https://github.com/Takenori-Kusaka/QuickScribe/commit/5772be816d77358fa06cac1cd4cf9c80c3cea5b7))
* **onboarding:** カードを操作ボタンより上部へ配置([#510](https://github.com/Takenori-Kusaka/QuickScribe/issues/510)) ([#518](https://github.com/Takenori-Kusaka/QuickScribe/issues/518)) ([b3da691](https://github.com/Takenori-Kusaka/QuickScribe/commit/b3da691d3412ac5bd0c03b6a8cb1a342e847c44c))
* **onboarding:** 初回のaha体験「サンプルで試す」([#57](https://github.com/Takenori-Kusaka/QuickScribe/issues/57) S9.3) ([#505](https://github.com/Takenori-Kusaka/QuickScribe/issues/505)) ([92b53fa](https://github.com/Takenori-Kusaka/QuickScribe/commit/92b53fafdbac8f4db1dd6d113fa2f98848a0fbf5))
* **privacy:** オフライン固定モードの実体化([#465](https://github.com/Takenori-Kusaka/QuickScribe/issues/465)) ([#508](https://github.com/Takenori-Kusaka/QuickScribe/issues/508)) ([113d512](https://github.com/Takenori-Kusaka/QuickScribe/commit/113d512c46ac8240aafacb55bcf5ab5972b24a62))
* **readme:** コアループのデモGIFを追加([#55](https://github.com/Takenori-Kusaka/QuickScribe/issues/55) S9.1) ([#503](https://github.com/Takenori-Kusaka/QuickScribe/issues/503)) ([9444725](https://github.com/Takenori-Kusaka/QuickScribe/commit/944472505497203989d21fa704eb9977f29d321a))
* **settings:** 「整形」セクションを折りたたみグループ化([#404](https://github.com/Takenori-Kusaka/QuickScribe/issues/404)) ([#488](https://github.com/Takenori-Kusaka/QuickScribe/issues/488)) ([d97f735](https://github.com/Takenori-Kusaka/QuickScribe/commit/d97f73514534f5b42141b75070e335de3bf10c38))
* **settings:** 未設定時の動線改善(不足明示・フォーカス・保存ガード)([#516](https://github.com/Takenori-Kusaka/QuickScribe/issues/516)) ([#519](https://github.com/Takenori-Kusaka/QuickScribe/issues/519)) ([72aaaf1](https://github.com/Takenori-Kusaka/QuickScribe/commit/72aaaf16965a754817f7e86ca57869fb57785af3))
* **settings:** 設定を5タブに再編し到達性を改善([#512](https://github.com/Takenori-Kusaka/QuickScribe/issues/512)) ([#521](https://github.com/Takenori-Kusaka/QuickScribe/issues/521)) ([e006e34](https://github.com/Takenori-Kusaka/QuickScribe/commit/e006e340eaddb9ee5b12a594d9a623c70b1002b4))
* **stt:** モデルのDL方式明示と日本語ロケール既定をkotobaに([#511](https://github.com/Takenori-Kusaka/QuickScribe/issues/511) 一部) ([#520](https://github.com/Takenori-Kusaka/QuickScribe/issues/520)) ([646ad0a](https://github.com/Takenori-Kusaka/QuickScribe/commit/646ad0a38e0eb6115ceed5f9bf1c47673d43c837))
* **ux:** 既定幅拡大・ブレスト削除・「文字起こしから整形」に改称([#513](https://github.com/Takenori-Kusaka/QuickScribe/issues/513)/[#514](https://github.com/Takenori-Kusaka/QuickScribe/issues/514)/[#515](https://github.com/Takenori-Kusaka/QuickScribe/issues/515)) ([#517](https://github.com/Takenori-Kusaka/QuickScribe/issues/517)) ([32b2257](https://github.com/Takenori-Kusaka/QuickScribe/commit/32b22576da5bef3d3825ef587e3d1a2c69bce955))
* **ux:** 空テキスト整形ガードとオンボーディング再表示([#397](https://github.com/Takenori-Kusaka/QuickScribe/issues/397)/[#398](https://github.com/Takenori-Kusaka/QuickScribe/issues/398)) ([#484](https://github.com/Takenori-Kusaka/QuickScribe/issues/484)) ([912b414](https://github.com/Takenori-Kusaka/QuickScribe/commit/912b414b334c0590a2735f5d74dd35e1efd50510))


### 🐛 修正 / Bug Fixes

* **a11y:** WCAG2.0 AA / JIS X 8341-3 のAA違反を是正([#395](https://github.com/Takenori-Kusaka/QuickScribe/issues/395)) ([#482](https://github.com/Takenori-Kusaka/QuickScribe/issues/482)) ([69eb9ee](https://github.com/Takenori-Kusaka/QuickScribe/commit/69eb9ee19c7af406e97b406b19d7661aa2d86d25))
* **backend:** 入力検証の堅牢化と用語是正([#398](https://github.com/Takenori-Kusaka/QuickScribe/issues/398)/[#18](https://github.com/Takenori-Kusaka/QuickScribe/issues/18)) ([#486](https://github.com/Takenori-Kusaka/QuickScribe/issues/486)) ([24da526](https://github.com/Takenori-Kusaka/QuickScribe/commit/24da526dbc95516c3a5608ac8c0916c755b0a395))
* **e2e:** トグル検証を非同期状態待ちに変更しフレークを根治([#412](https://github.com/Takenori-Kusaka/QuickScribe/issues/412)) ([#506](https://github.com/Takenori-Kusaka/QuickScribe/issues/506)) ([cb9df80](https://github.com/Takenori-Kusaka/QuickScribe/commit/cb9df8085cdf852e124465fb175910c589e1c240))
* **security:** quick-xml DoS勧告を根拠付きignoreで監査を正常化([#526](https://github.com/Takenori-Kusaka/QuickScribe/issues/526)) ([#527](https://github.com/Takenori-Kusaka/QuickScribe/issues/527)) ([58fc410](https://github.com/Takenori-Kusaka/QuickScribe/commit/58fc410bb9d8049f28711fee711a9f958e77cd8a))
* **ux:** レビュー指摘の是正(幅コンパクト回帰/ブレスト復活/whisper既定)([#513](https://github.com/Takenori-Kusaka/QuickScribe/issues/513)/[#514](https://github.com/Takenori-Kusaka/QuickScribe/issues/514)/[#511](https://github.com/Takenori-Kusaka/QuickScribe/issues/511)) ([#523](https://github.com/Takenori-Kusaka/QuickScribe/issues/523)) ([b8af470](https://github.com/Takenori-Kusaka/QuickScribe/commit/b8af470b72ea49f969374956f7921e52cb01357e))


### ♻️ リファクタ / Refactor

* **solid:** エントリ絞り込みロジックをlib抽出しテスト追加([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#524](https://github.com/Takenori-Kusaka/QuickScribe/issues/524)) ([35f81c7](https://github.com/Takenori-Kusaka/QuickScribe/commit/35f81c7594f5c39ad9ff46c84542ddbc4cea23ed))
* **solid:** 録音ソース値パースをlib抽出＋潜在バグ修正([#392](https://github.com/Takenori-Kusaka/QuickScribe/issues/392)) ([#525](https://github.com/Takenori-Kusaka/QuickScribe/issues/525)) ([2d3a4bf](https://github.com/Takenori-Kusaka/QuickScribe/commit/2d3a4bfd976f29166ee82aa4025da258b15d13a4))


### 📝 ドキュメント / Docs

* 「なぜQuickScribe？」競合比較を読み手向けに公開([#396](https://github.com/Takenori-Kusaka/QuickScribe/issues/396)/[#399](https://github.com/Takenori-Kusaka/QuickScribe/issues/399)) ([#490](https://github.com/Takenori-Kusaka/QuickScribe/issues/490)) ([1d803ea](https://github.com/Takenori-Kusaka/QuickScribe/commit/1d803ea1206ef33c8766ce6a23f0b0272a0531a5))
* **marketing:** ローンチキット(HN/Reddit/PH/LinkedIn文面+チェックリスト)([#59](https://github.com/Takenori-Kusaka/QuickScribe/issues/59) S9.5) ([#504](https://github.com/Takenori-Kusaka/QuickScribe/issues/504)) ([3b3b430](https://github.com/Takenori-Kusaka/QuickScribe/commit/3b3b4307a47cb978b62563fa660c1bd5c798e9e7))
* NFRの誤ステータス是正とADR-0018/0019追加([#390](https://github.com/Takenori-Kusaka/QuickScribe/issues/390)/[#393](https://github.com/Takenori-Kusaka/QuickScribe/issues/393)) ([#485](https://github.com/Takenori-Kusaka/QuickScribe/issues/485)) ([3b2b747](https://github.com/Takenori-Kusaka/QuickScribe/commit/3b2b747e87f252b153c789febad19f14e4759b26))
* **process:** バージョニング/チャネル/鍵分離の方針を明文化([#52](https://github.com/Takenori-Kusaka/QuickScribe/issues/52) S8.5) ([#528](https://github.com/Takenori-Kusaka/QuickScribe/issues/528)) ([663e695](https://github.com/Takenori-Kusaka/QuickScribe/commit/663e695d1acd4d8ba2589a13d212f730471a800d))
* **readme:** 実スクリーンショット差替とCI自動化の修正([#396](https://github.com/Takenori-Kusaka/QuickScribe/issues/396)/[#387](https://github.com/Takenori-Kusaka/QuickScribe/issues/387)/S9.1) ([#487](https://github.com/Takenori-Kusaka/QuickScribe/issues/487)) ([adf1528](https://github.com/Takenori-Kusaka/QuickScribe/commit/adf1528f782699f4d4bfe57036ce8c7dfd4e8361))
* 未署名の事実を反映しSignPath「利用/整備中」表記を是正([#50](https://github.com/Takenori-Kusaka/QuickScribe/issues/50)) ([#522](https://github.com/Takenori-Kusaka/QuickScribe/issues/522)) ([c549d3e](https://github.com/Takenori-Kusaka/QuickScribe/commit/c549d3e097e543d254baed5e7fd696e3d29ce611))

## [0.9.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v0.8.0...v0.9.0) (2026-07-01)


### ✨ 新機能 / Features

* **i18n:** validateRefineConfig(cfgErr)をエラーコード化([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase2) ([#463](https://github.com/Takenori-Kusaka/QuickScribe/issues/463)) ([aac2a12](https://github.com/Takenori-Kusaka/QuickScribe/commit/aac2a12b35d155d033083e92906414bc9b8a4444))
* **i18n:** zh/es カタログ追加（4言語対応）([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)) ([#454](https://github.com/Takenori-Kusaka/QuickScribe/issues/454)) ([a842a9f](https://github.com/Takenori-Kusaka/QuickScribe/commit/a842a9ff95e3405c2fcf1ee59e9563bd3b394d42))
* **i18n:** メインのホットキーヒント＋設定の選択肢をキー化(ja/en) ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)) ([#445](https://github.com/Takenori-Kusaka/QuickScribe/issues/445)) ([54e9073](https://github.com/Takenori-Kusaka/QuickScribe/commit/54e90731e86df4484b96c6995a03334b4658e845))
* **i18n:** 結果カード/用語補正パネル等の残りUI文言をキー化(ja/en) ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)) ([#451](https://github.com/Takenori-Kusaka/QuickScribe/issues/451)) ([fe48713](https://github.com/Takenori-Kusaka/QuickScribe/commit/fe487131955cc455779aa766a934785b460f3e82))
* **i18n:** 英語カタログ＋ロケール切替＋起動時OS言語デフォルト ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase3) ([#443](https://github.com/Takenori-Kusaka/QuickScribe/issues/443)) ([df6c97a](https://github.com/Takenori-Kusaka/QuickScribe/commit/df6c97a4d72fca1dcd096aa1f5cb47c9c1520812))
* **i18n:** 設定のAWS/STTラベル・主要tipをキー化(ja/en) ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)) ([#447](https://github.com/Takenori-Kusaka/QuickScribe/issues/447)) ([ecbfd10](https://github.com/Takenori-Kusaka/QuickScribe/commit/ecbfd109853b1e8e12fe565d722d8407db2d50ec))
* **i18n:** 設定パネル残りの文言・複雑inline-HTML tipをキー化(ja/en) ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)) ([#450](https://github.com/Takenori-Kusaka/QuickScribe/issues/450)) ([fbd8d96](https://github.com/Takenori-Kusaka/QuickScribe/commit/fbd8d96234039e3bb57af20f15be8528676460a1))
* **onboarding:** 初回オンボーディングと空状態の導線([#397](https://github.com/Takenori-Kusaka/QuickScribe/issues/397)) ([#461](https://github.com/Takenori-Kusaka/QuickScribe/issues/461)) ([4251291](https://github.com/Takenori-Kusaka/QuickScribe/commit/4251291fcde8787ad172cd8147304741c84f039f))
* **privacy:** オンデバイス/クラウドのインジケータと「オフラインにする」([#465](https://github.com/Takenori-Kusaka/QuickScribe/issues/465)) ([#480](https://github.com/Takenori-Kusaka/QuickScribe/issues/480)) ([d5a0d68](https://github.com/Takenori-Kusaka/QuickScribe/commit/d5a0d6836c83068b4f630d6f3c12c0187feef47d))
* **refine:** 整形出力言語(翻訳)オプション([#453](https://github.com/Takenori-Kusaka/QuickScribe/issues/453)) ([#479](https://github.com/Takenori-Kusaka/QuickScribe/issues/479)) ([ea7c0af](https://github.com/Takenori-Kusaka/QuickScribe/commit/ea7c0af90aec1e39acc672d4cd68063c964c7e7e))
* **security:** 制限的CSPを設定（既定の無効化を解消）([#391](https://github.com/Takenori-Kusaka/QuickScribe/issues/391)) ([#455](https://github.com/Takenori-Kusaka/QuickScribe/issues/455)) ([2156944](https://github.com/Takenori-Kusaka/QuickScribe/commit/21569443a1255a18525e58d1cf7e0d97e179bc60))
* **settings:** カテゴリを頻度メジャー順に再編([#404](https://github.com/Takenori-Kusaka/QuickScribe/issues/404)) ([#460](https://github.com/Takenori-Kusaka/QuickScribe/issues/460)) ([851f387](https://github.com/Takenori-Kusaka/QuickScribe/commit/851f387d419a1b382ae3b110ab105cb539d04ce0))


### 🐛 修正 / Bug Fixes

* **onboarding:** プライバシー文言の過剰主張を是正([#397](https://github.com/Takenori-Kusaka/QuickScribe/issues/397)) ([#464](https://github.com/Takenori-Kusaka/QuickScribe/issues/464)) ([22a2387](https://github.com/Takenori-Kusaka/QuickScribe/commit/22a2387a523d110dcfd57f9545daf8934bb61d47))


### 📝 ドキュメント / Docs

* **research:** 整形出力言語・翻訳オプションの設計調査([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401)関連) ([#452](https://github.com/Takenori-Kusaka/QuickScribe/issues/452)) ([4a992cb](https://github.com/Takenori-Kusaka/QuickScribe/commit/4a992cb5d68b2abac769a2f76a52c4d1f42cc00a))

## [0.8.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v0.7.0...v0.8.0) (2026-06-29)


### ✨ 新機能 / Features

* **i18n:** 設定の詳細ラベル15項目をキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#442](https://github.com/Takenori-Kusaka/QuickScribe/issues/442)) ([f2a1cd9](https://github.com/Takenori-Kusaka/QuickScribe/commit/f2a1cd9c95ae7d9132169a166a42529746a77783))
* **i18n:** 設定画面の見出し・主要ラベルをキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#438](https://github.com/Takenori-Kusaka/QuickScribe/issues/438)) ([e68d36a](https://github.com/Takenori-Kusaka/QuickScribe/commit/e68d36ae48cf92487ff562a38df0916138847b1a))


### 🐛 修正 / Bug Fixes

* **ui:** ヘッダのジャーナルをアイコンのみ＋ツールチップ化(重なり解消) ([#440](https://github.com/Takenori-Kusaka/QuickScribe/issues/440)) ([0c1754f](https://github.com/Takenori-Kusaka/QuickScribe/commit/0c1754f2327707ffd0d845d507541d1459136778))

## [0.7.0](https://github.com/Takenori-Kusaka/QuickScribe/compare/v0.6.4...v0.7.0) (2026-06-29)


### ✨ 新機能 / Features

* **i18n:** i18n基盤(svelte-i18n)導入＋メイン画面スライスをキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#434](https://github.com/Takenori-Kusaka/QuickScribe/issues/434)) ([717a928](https://github.com/Takenori-Kusaka/QuickScribe/commit/717a928b9cc92ba3a10501d56f97d913c8502274))
* **i18n:** ジャーナルパネルをキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#436](https://github.com/Takenori-Kusaka/QuickScribe/issues/436)) ([7646ebd](https://github.com/Takenori-Kusaka/QuickScribe/commit/7646ebdefe325bf825aa7a4e715881b8a4707e3e))
* **i18n:** 結果・アクション領域＋ヘッダ補助ラベルをキー化 ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase1) ([#435](https://github.com/Takenori-Kusaka/QuickScribe/issues/435)) ([ee60470](https://github.com/Takenori-Kusaka/QuickScribe/commit/ee60470f6717cee61242247471ef96e633e57092))


### ♻️ リファクタ / Refactor

* **front:** refine引数組み立てをlib抽出＋カバレッジゲート有効化 ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#428](https://github.com/Takenori-Kusaka/QuickScribe/issues/428)) ([8442de2](https://github.com/Takenori-Kusaka/QuickScribe/commit/8442de27c5adba72cdc762d818e290d058b99797))
* **front:** モデルキャッシュ鮮度・横断発見ロジックをlib抽出＋テスト ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#430](https://github.com/Takenori-Kusaka/QuickScribe/issues/430)) ([31736b1](https://github.com/Takenori-Kusaka/QuickScribe/commit/31736b1c7eaea809b3581f0bf5e3e06039693cbc))


### 📝 ドキュメント / Docs

* HANDOFF更新(v0.6.4配信・[#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402)ゲート・[#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403) perf実測・[#427](https://github.com/Takenori-Kusaka/QuickScribe/issues/427)) ([#433](https://github.com/Takenori-Kusaka/QuickScribe/issues/433)) ([496c491](https://github.com/Takenori-Kusaka/QuickScribe/commit/496c491e7d5a32800e7ed3877b4f69d7801e9f93))
* **perf:** 初回ベースライン実測を記録(RTF 0.857) ([#403](https://github.com/Takenori-Kusaka/QuickScribe/issues/403)) ([#432](https://github.com/Takenori-Kusaka/QuickScribe/issues/432)) ([478fc46](https://github.com/Takenori-Kusaka/QuickScribe/commit/478fc468fd9c0e962a946c3c93f5157327cf5a1e))

## [0.6.4](https://github.com/Takenori-Kusaka/QuickScribe/compare/v0.6.3...v0.6.4) (2026-06-28)


### ✨ 新機能 / Features

* **a11y:** モーダルのdialog化・フォーカストラップ・Esc閉じ＋コントラスト是正 ([#395](https://github.com/Takenori-Kusaka/QuickScribe/issues/395)) ([#413](https://github.com/Takenori-Kusaka/QuickScribe/issues/413)) ([4ea8ade](https://github.com/Takenori-Kusaka/QuickScribe/commit/4ea8adee45f86e81951ae27444eafca4642345c6))
* **robustness:** 入力ファイルのサイズ上限ガード＋対応形式/上限のUI通知 ([#397](https://github.com/Takenori-Kusaka/QuickScribe/issues/397)) ([#414](https://github.com/Takenori-Kusaka/QuickScribe/issues/414)) ([6cd301f](https://github.com/Takenori-Kusaka/QuickScribe/commit/6cd301f38e98aade10fbf0d0bd173892903b89cc))
* **ui:** ヘッダUX改善(保管庫→ジャーナル/SVGアイコン/IA整理) ([#388](https://github.com/Takenori-Kusaka/QuickScribe/issues/388)) ([f994674](https://github.com/Takenori-Kusaka/QuickScribe/commit/f9946748919e586c238fd6bd25cf7abcf34708f5))


### 🐛 修正 / Bug Fixes

* **settings:** 設定項目の誤分類を修正(タスクバー/自動起動→アプリ全般) ([#404](https://github.com/Takenori-Kusaka/QuickScribe/issues/404)) ([#422](https://github.com/Takenori-Kusaka/QuickScribe/issues/422)) ([66d54ad](https://github.com/Takenori-Kusaka/QuickScribe/commit/66d54adf0c16ab546c4313d801595e068d380ccb))
* **ui:** 内部ID"S2.2"露出を除去＋エラー文言をユーザー向けに整備 ([#398](https://github.com/Takenori-Kusaka/QuickScribe/issues/398)) ([#411](https://github.com/Takenori-Kusaka/QuickScribe/issues/411)) ([1fa8011](https://github.com/Takenori-Kusaka/QuickScribe/commit/1fa8011714a482b7ec36196fb5b83e2c72e867ec))


### ♻️ リファクタ / Refactor

* **front:** App.svelteの純粋関数をlib抽出＋テスト ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#425](https://github.com/Takenori-Kusaka/QuickScribe/issues/425)) ([c81794d](https://github.com/Takenori-Kusaka/QuickScribe/commit/c81794d29e033651d884cdd7bf2cd4f466b6ee76))
* **front:** プロバイダ定義・定数を constants.ts に集約(SSOT) ([#401](https://github.com/Takenori-Kusaka/QuickScribe/issues/401) Phase0) ([#410](https://github.com/Takenori-Kusaka/QuickScribe/issues/410)) ([752af0c](https://github.com/Takenori-Kusaka/QuickScribe/commit/752af0c4be1a585bae9b89cee5866bf677a9f6ef))
* **front:** プロバイダ鍵バリデーションをlib抽出＋テスト ([#402](https://github.com/Takenori-Kusaka/QuickScribe/issues/402) Phase2) ([#426](https://github.com/Takenori-Kusaka/QuickScribe/issues/426)) ([e742ec0](https://github.com/Takenori-Kusaka/QuickScribe/commit/e742ec0df420db521f916c16dfad527de7976f04))


### 📝 ドキュメント / Docs

* **community:** 行動規範(Contributor Covenant 2.1)＋Issueテンプレ config.yml (E8) ([3bc6f6f](https://github.com/Takenori-Kusaka/QuickScribe/commit/3bc6f6f5c535b8cc1b3797fd5f3a40ddf11eea1c))
* **github:** CODEOWNERS＋SUPPORT.md 追加 (OSSガバナンス / [#406](https://github.com/Takenori-Kusaka/QuickScribe/issues/406)) ([5865f52](https://github.com/Takenori-Kusaka/QuickScribe/commit/5865f5296f2e385cfa4786a5a38f77ecdbd7e41e))
* HANDOFF引継ぎ資料を最新化(v1.0.0プログラム進行状況) ([#423](https://github.com/Takenori-Kusaka/QuickScribe/issues/423)) ([05045c7](https://github.com/Takenori-Kusaka/QuickScribe/commit/05045c7b82d376cdc7ea8ce08daeacc7a1a8b834))
* **license:** npm依存の第三者ライセンス帰属を生成・公開 ([#394](https://github.com/Takenori-Kusaka/QuickScribe/issues/394)) ([#421](https://github.com/Takenori-Kusaka/QuickScribe/issues/421)) ([555c8b8](https://github.com/Takenori-Kusaka/QuickScribe/commit/555c8b8aeb2026cdd992538f9adb87e571e945ef))
* **planning:** v1.0.0 リリース・レディネス監査(18観点の統合) (E9) ([df6e7fc](https://github.com/Takenori-Kusaka/QuickScribe/commit/df6e7fc63f313907df4d31603b7f78f796ea2d24))
* **process:** デモGIFの実機キャプチャ手順 (S9.1 [#55](https://github.com/Takenori-Kusaka/QuickScribe/issues/55)) ([d29de27](https://github.com/Takenori-Kusaka/QuickScribe/commit/d29de27beb05f5b9a2a35f701a6bdd5d24e275ce))
* **readme:** バッジ/特徴/Quick start/Privacy節に刷新 (S9.1 [#55](https://github.com/Takenori-Kusaka/QuickScribe/issues/55)) ([925cce3](https://github.com/Takenori-Kusaka/QuickScribe/commit/925cce3b1aa58827b7bad96e07043d631744de59))
* **readme:** 視覚素材をGIFからスクリーンショットに変更 (S9.1 [#55](https://github.com/Takenori-Kusaka/QuickScribe/issues/55)) ([31ab6bd](https://github.com/Takenori-Kusaka/QuickScribe/commit/31ab6bd0b1ed15b967af2406c6cdfe754170532e))
* **research:** 競合ランドスケープ分析を作成(一次情報) ([#399](https://github.com/Takenori-Kusaka/QuickScribe/issues/399)) ([#424](https://github.com/Takenori-Kusaka/QuickScribe/issues/424)) ([b88d7f8](https://github.com/Takenori-Kusaka/QuickScribe/commit/b88d7f83aa6213cf1671dce29571b470f7a53d60))
* **site:** 「保管庫」表記をアプリに合わせ「ジャーナル」へ統一 ([#388](https://github.com/Takenori-Kusaka/QuickScribe/issues/388)追従) ([bc54195](https://github.com/Takenori-Kusaka/QuickScribe/commit/bc54195179b3068a387c41546850aa9b040eea19))
* アーキテクチャ設計(design.md)＋非機能要件(NFR)集約 ([#390](https://github.com/Takenori-Kusaka/QuickScribe/issues/390)) ([#420](https://github.com/Takenori-Kusaka/QuickScribe/issues/420)) ([e0fad8b](https://github.com/Takenori-Kusaka/QuickScribe/commit/e0fad8b204757eda6cd43630bb915bdb2bee042d))
* ドキュメントの矛盾・陳腐化を解消(ADR索引/署名方針/vision) ([#390](https://github.com/Takenori-Kusaka/QuickScribe/issues/390)) ([#419](https://github.com/Takenori-Kusaka/QuickScribe/issues/419)) ([3554622](https://github.com/Takenori-Kusaka/QuickScribe/commit/3554622193b449f0ebabfbe3458e495c2377e329))

## [0.6.3] - 2026-06-27

### ✨ 新機能 / Features

- 用語補正フェーズ（誤変換の確認・置換）を整形前に追加（#384）
- 設定・エントリのスキーマ版管理＋検証（#382）
- GitHub Pages ドキュメントサイト（VitePress）公開（#385）

### 📝 ドキュメント / Docs

- SECURITY.md（最新版のみ / 非公開報告 / プライバシー方針）
- 配布・署名の手順とコストゼロ方針（SignPath OSS 無料署名）

### 🔧 その他 / Internal

- CI: crates.io ダウンロードの HTTP/2 フレーク対策（retry + HTTP/1.1）（#383）

## [0.6.0] - 2026-06-27

### ✨ 新機能 / Features

- クラウド文字起こしエンジン（Groq / OpenAI / Deepgram / Azure）の選択に対応
- 整形プロバイダの拡充（Gemini / Anthropic / OpenAI / Ollama / AWS Bedrock / Claude Platform on AWS）

## [0.2.7] - 2026-06

### ✨ 新機能 / Features

- システム音ループバック録音（Windows / WASAPI）
- 物理トリガー（グローバルホットキー・モーメンタリ録音）

## [0.1.0] - 2026-06

### ✨ 新機能 / Features

- Walking Skeleton: トレイ常駐・録音トグル・ローカル whisper 文字起こし・整形・保管庫保存の縦断実装

[0.6.3]: https://github.com/Takenori-Kusaka/QuickScribe/releases/tag/v0.6.3
[0.6.0]: https://github.com/Takenori-Kusaka/QuickScribe/releases/tag/v0.6.0
[0.2.7]: https://github.com/Takenori-Kusaka/QuickScribe/releases/tag/v0.2.7
[0.1.0]: https://github.com/Takenori-Kusaka/QuickScribe/releases/tag/v0.1.0
