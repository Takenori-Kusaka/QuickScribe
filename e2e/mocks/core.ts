// スクショ用 Tauri core モック。実バックエンド無しでフロントを描画するため、
// invoke を決め打ちのダミーデータで応答する（ブラウザ/ヘッドレスで動作）。
// すべてダミー（実ジャーナル・実APIキー不使用）。

const REFINED = `### 発話の要点（サマリー）
- 生成AIを使った開発スタイル「AIDLC（AI Software Development Life Cycle）」への関心が高まっている。
- 既存の開発フローにどう馴染ませるかを整理したい、という動機がある。

### 思考の流れ（ニュアンス保持）
最近「AIDLC」という言葉を意識するようになった。1年ほど生成AIを使った開発を続けてきたが、
既存の開発スタイルに"うまく落とし込む"には、まだ理解を深める必要があると感じている。
検証用の専用リポジトリを作り、小さく試しながら確かめていきたい——という気持ちが芯にある。

### 次の一歩
- 検証用リポジトリの目的（何を確かめたいか）を1行で言語化する。
- 既存フローのどの工程をAIに任せるか、棚卸しする。`;

const TRANSCRIPT = `えーと、最近AIDLCっていう、AIを使った開発のライフサイクルっていう考え方が気になっていて、
生成AIを使った開発を1年くらい続けてきたんですけど、既存の開発スタイルにうまく落とし込むには
いろいろ理解を深めないといけないなと思っていて、検証用の専用リポジトリを作りたいなと考えています。`;

const ENTRIES = [
  {
    path: "refined-2026-06-27T154339.md",
    name: "refined-2026-06-27T154339.md",
    created: "2026-06-27T15:43:39",
    kind: "refined",
    tags: ["アイデア", "開発", "AIDLC"],
    preview:
      "生成AIを使った開発スタイル「AIDLC」について思考を整理し、検証用の専用リポジトリを作りたいと考えている。まず最初のステップとして目的を言語化する…",
  },
  {
    path: "refined-2026-06-27T154251.md",
    name: "refined-2026-06-27T154251.md",
    created: "2026-06-27T15:42:51",
    kind: "refined",
    tags: ["開発", "課題"],
    preview:
      "AIDLCを考える背景と現在の日本の開発現場の課題。生成AIを使った開発を1年くらい続けてきたが、既存の開発スタイルに上手く落とし込むには理解が要る…",
  },
  {
    path: "note-2026-06-27T141903.md",
    name: "note-2026-06-27T141903.md",
    created: "2026-06-27T14:19:03",
    kind: "note",
    tags: ["メモ", "業務プロセス"],
    preview:
      "生成AIを活用した開発ライフサイクル「AIDLC」の会社業務プロセスへの落とし込み。まずは小さな実証から始め、効果と摩擦の両方を観察したい…",
  },
  {
    path: "transcript-2026-06-26T220110.txt",
    name: "transcript-2026-06-26T220110.txt",
    created: "2026-06-26T22:01:10",
    kind: "transcript",
    tags: ["不安", "ふりかえり"],
    preview:
      "今日はやることが多くて、何から手をつければいいか少し迷っていた。明日の朝に優先順位をもう一度整理したい…",
  },
];

export async function invoke<T = unknown>(cmd: string, _args?: unknown): Promise<T> {
  switch (cmd) {
    case "list_entries":
      return ENTRIES as unknown as T;
    case "read_text_file":
      return (REFINED + "\n\n（※スクショ用ダミー本文）") as unknown as T;
    case "refine_text":
      return REFINED as unknown as T;
    case "transcribe_file":
      return TRANSCRIPT as unknown as T;
    case "resolve_model":
      return { id: "base", label: "base (147MB)", ready: true, path: "" } as unknown as T;
    case "list_audio_sources":
      return [] as unknown as T;
    case "get_secret":
      return null as unknown as T;
    default:
      // 状態変更系（set_*, start/stop_recording, open_vault 等）は副作用なしで無視。
      return null as unknown as T;
  }
}

export function convertFileSrc(filePath: string): string {
  return filePath;
}

// テスト/スクショから「文字起こし完了」などのイベントを発火するためのダミー本文を共有。
export const __mockData = { REFINED, TRANSCRIPT, ENTRIES };
