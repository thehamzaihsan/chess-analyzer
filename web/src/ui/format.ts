export function pct(value: number, decimals = 1): string {
  return `${value.toFixed(decimals)}%`;
}

export function round(value: number, decimals = 0): string {
  return value.toFixed(decimals);
}

export function signedPawns(cp: number): string {
  const pawns = cp / 100;
  const sign = pawns >= 0 ? '+' : '';
  return `${sign}${pawns.toFixed(2)}`;
}

export function formatDate(d: Date | null): string {
  if (!d) return 'unknown date';
  return d.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
}

export function formatDateShort(d: Date | null): string {
  if (!d) return '?';
  return d.toLocaleDateString(undefined, { year: '2-digit', month: 'short', day: 'numeric' });
}

export function formatClock(seconds: number): string {
  const s = Math.max(0, Math.round(seconds));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  if (h > 0) return `${h}:${String(m).padStart(2, '0')}:${String(sec).padStart(2, '0')}`;
  return `${m}:${String(sec).padStart(2, '0')}`;
}

export function ordinal(n: number): string {
  const s = ['th', 'st', 'nd', 'rd'];
  const v = n % 100;
  return n + (s[(v - 20) % 10] || s[v] || s[0]);
}
