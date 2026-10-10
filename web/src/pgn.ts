export interface ParsedGame {
  headers: Record<string, string>;
  moves: ParsedMove[];
}

export interface ParsedMove {
  san: string;
  clock: string | null;
}

const HEADER_RE = /^\s*\[([A-Za-z0-9_]+)\s+"((?:[^"\\]|\\.)*)"\]\s*$/;

/**
 * Splits raw PGN text (possibly many games concatenated) into games.
 * A header line that appears after movetext has started closes the
 * previous game and opens a new one.
 */
export function splitGames(pgnText: string): ParsedGame[] {
  const lines = pgnText.split(/\r\n|\r|\n/);
  const games: ParsedGame[] = [];

  let headers: Record<string, string> = {};
  let movetextLines: string[] = [];
  let inMovetext = false;
  let hasAnyHeader = false;

  const flush = () => {
    if (hasAnyHeader || movetextLines.length > 0) {
      const moves = parseMovetext(movetextLines.join(' '));
      if (headers.White && headers.Black) {
        games.push({ headers, moves });
      }
    }
    headers = {};
    movetextLines = [];
    inMovetext = false;
    hasAnyHeader = false;
  };

  for (const rawLine of lines) {
    const line = rawLine.trim();
    if (line.length === 0) continue;
    const m = HEADER_RE.exec(line);
    if (m) {
      if (inMovetext) {
        // A header after movetext started means a new game begins here.
        flush();
      }
      headers[m[1]] = unescapeHeaderValue(m[2]);
      hasAnyHeader = true;
    } else {
      inMovetext = true;
      movetextLines.push(line);
    }
  }
  flush();

  return games;
}

function unescapeHeaderValue(value: string): string {
  return value.replace(/\\(.)/g, '$1');
}

const RESULT_TOKENS = new Set(['1-0', '0-1', '1/2-1/2', '*']);

/**
 * Parses movetext into an ordered list of SAN moves with attached clock
 * times, stripping variations, comments (other than %clk), move numbers,
 * NAGs, annotation glyphs, and the trailing result token.
 */
export function parseMovetext(text: string): ParsedMove[] {
  const withoutVariations = stripVariations(text);

  const moves: ParsedMove[] = [];
  // Tokenize: either a {...} comment, or a run of non-space/non-brace chars.
  const tokenRe = /\{[^}]*\}|\S+/g;
  let tokens: string[] = withoutVariations.match(tokenRe) ?? [];

  let pendingClock: string | null = null;
  let lastMoveIndex = -1;

  for (const token of tokens) {
    if (token.startsWith('{')) {
      const clkMatch = /\[%clk\s+([0-9:.]+)\]/.exec(token);
      if (clkMatch) {
        if (lastMoveIndex >= 0) {
          moves[lastMoveIndex].clock = clkMatch[1];
        } else {
          pendingClock = clkMatch[1];
        }
      }
      continue;
    }
    if (RESULT_TOKENS.has(token)) continue;
    if (/^\$\d+$/.test(token)) continue; // NAG

    // Strip leading move numbers like "12." or "12..."
    let san = token.replace(/^\d+\.(\.\.)?/, '');
    if (san.length === 0) continue;
    // Strip trailing annotation glyphs like !?, !!, ??, !, ?
    san = san.replace(/[!?]+$/, '');
    if (san.length === 0) continue;
    if (RESULT_TOKENS.has(san)) continue;

    moves.push({ san, clock: pendingClock });
    pendingClock = null;
    lastMoveIndex = moves.length - 1;
  }

  return moves;
}

function stripVariations(text: string): string {
  let result = '';
  let depth = 0;
  for (let i = 0; i < text.length; i++) {
    const ch = text[i];
    if (ch === '(') {
      depth++;
      continue;
    }
    if (ch === ')') {
      if (depth > 0) depth--;
      continue;
    }
    if (depth === 0) result += ch;
  }
  return result;
}
