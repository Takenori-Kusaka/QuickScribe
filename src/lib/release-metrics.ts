// リリースDL数の集計（採用状況の計測 / #60 S9.6）。
// アプリ本体はプライバシー設計上テレメトリを持たない(ADR-0020)。採用状況は
// GitHub Releases のアセットDL数(サーバー側の公開統計・個人を追跡しない)で測る。

export interface ReleaseAsset {
  name: string;
  download_count: number;
}
export interface Release {
  tag_name: string;
  prerelease?: boolean;
  assets: ReleaseAsset[];
}

export interface ReleaseDownloadSummary {
  tag: string;
  total: number;
  perAsset: { name: string; count: number }[];
}

export interface DownloadAggregate {
  total: number;
  releases: ReleaseDownloadSummary[];
  /** アセット名(OS/形式)ごとの合計。 */
  perAsset: Record<string, number>;
}

/** リリース配列からDL数を集計する（純粋・テスト対象）。updater用の latest.json 等は除外。 */
export function aggregateDownloads(
  releases: Release[],
  opts: { includePrerelease?: boolean } = {},
): DownloadAggregate {
  const includePrerelease = opts.includePrerelease ?? true;
  const perAsset: Record<string, number> = {};
  const summaries: ReleaseDownloadSummary[] = [];
  let total = 0;
  for (const r of releases) {
    if (!includePrerelease && r.prerelease) continue;
    const assets = (r.assets ?? []).filter((a) => !isMetadataAsset(a.name));
    let relTotal = 0;
    for (const a of assets) {
      relTotal += a.download_count;
      perAsset[a.name] = (perAsset[a.name] ?? 0) + a.download_count;
    }
    total += relTotal;
    summaries.push({
      tag: r.tag_name,
      total: relTotal,
      perAsset: assets.map((a) => ({ name: a.name, count: a.download_count })),
    });
  }
  return { total, releases: summaries, perAsset };
}

/** 配布物でない付随ファイル(署名・updaterメタ等)はDL統計から除く。 */
export function isMetadataAsset(name: string): boolean {
  return /(^latest\.json$|\.sig$|\.sha256$|^SHA256SUMS)/i.test(name);
}
