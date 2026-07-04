// 習慣ナッジ用の「寛容ストリーク」計算（#58 S9.4 / サーバー無し・ローカル完結）。
// 毎日完璧でなくても続く「forgiving streak」: 連続する記録日の間隔が graceDays+1 日
// 以内なら途切れない(=1日サボっても継続)。最新の記録が今日から grace 内にあるときのみ「継続中」。

/** "YYYY-MM-DD"(またはISO日時)を UTC 日付の通し番号(日)に変換する。 */
function toDayNumber(iso: string): number | null {
  const m = iso.match(/^(\d{4})-(\d{2})-(\d{2})/);
  if (!m) return null;
  const [, y, mo, d] = m;
  // Date.UTC は決定的(タイムゾーン非依存)。1日=86400000ms。
  return Math.floor(Date.UTC(Number(y), Number(mo) - 1, Number(d)) / 86400000);
}

/**
 * 習慣ナッジ（S9.4 #58）を発火すべきかを返す純粋関数。
 * 「継続中のストリークがあり、かつ今日まだ記録していない」ときだけ true。
 * forgiving 方針: すでに途切れている（grace超）ユーザーや新規ユーザーは急かさない
 * （罪悪感を煽らず、保全に値する習慣だけを守る）。今日記録済みなら当然 false。
 * @param dates 記録日(重複可, ISO日付/日時)。
 * @param today 基準日("YYYY-MM-DD"等)。
 * @param graceDays 何日サボりを許すか(computeStreak と同義, 既定1)。
 */
export function shouldNudge(dates: string[], today: string, graceDays = 1): boolean {
  const todayNum = toDayNumber(today);
  if (todayNum === null) return false;
  const recordedToday = dates.some((d) => toDayNumber(d) === todayNum);
  if (recordedToday) return false;
  return computeStreak(dates, today, graceDays) > 0;
}

/**
 * 寛容ストリーク(継続中の連続記録日数)を返す。
 * @param dates 記録日(重複可, ISO日付/日時)。
 * @param today 基準日("YYYY-MM-DD"等)。
 * @param graceDays 何日サボりを許すか(既定1=1日まで飛ばしてOK)。
 * @returns 継続中のストリーク日数。最新記録が今日から grace を超えて古ければ 0。
 */
export function computeStreak(dates: string[], today: string, graceDays = 1): number {
  const todayNum = toDayNumber(today);
  if (todayNum === null) return 0;
  // ユニークな記録日を新しい順に。
  const days = Array.from(
    new Set(dates.map(toDayNumber).filter((n): n is number => n !== null)),
  ).sort((a, b) => b - a);
  if (days.length === 0) return 0;
  // 最新記録が今日から grace を超えて古ければ継続していない。
  if (todayNum - days[0] > graceDays + 1) return 0;
  let streak = 1;
  for (let i = 1; i < days.length; i++) {
    const gap = days[i - 1] - days[i];
    if (gap <= 0) continue; // 同日(重複)は無視。
    if (gap <= graceDays + 1) streak++;
    else break;
  }
  return streak;
}
