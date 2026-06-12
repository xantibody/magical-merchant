/**
 * Scores how well a query fuzzy-matches a target (case-insensitive).
 * Returns null when the query is not a subsequence of the target.
 * Higher is better: substring beats subsequence, earlier beats later,
 * shorter targets beat longer ones at equal quality.
 */
export function fuzzyScore(query: string, target: string): number | null {
  if (!target) return null;
  if (!query) return 0;

  const q = query.toLowerCase();
  const t = target.toLowerCase();

  const substringIndex = t.indexOf(q);
  if (substringIndex >= 0) {
    return 1000 - substringIndex * 10 - (t.length - q.length);
  }

  // Subsequence scan: every gap between matched chars costs points.
  let score = 500;
  let pos = 0;
  for (const ch of q) {
    const found = t.indexOf(ch, pos);
    if (found < 0) return null;
    score -= (found - pos) * 2;
    pos = found + 1;
  }
  return score - (t.length - q.length);
}
