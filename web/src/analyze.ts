import { Chess } from 'chess.js';
import { matchOpening } from './openings';

export type Side = 'white' | 'black';
export type GameResult = 'win' | 'draw' | 'loss';

export type TerminationCategory =
  | 'checkmate'
  | 'resignation'
  | 'time'
  | 'abandoned'
  | 'stalemate'
  | 'repetition'
  | 'agreement'
  | 'fifty-move'
  | 'insufficient-material'
  | 'timeout-vs-insufficient'
  | 'unknown';

export interface TimeControlInfo {
  raw: string;
  type: 'standard' | 'daily';
  base: number; // seconds (or seconds-per-move for daily)
  increment: number;
  label: string;
}

export interface Ply {
  ply: number; // 1-indexed, ply 1 is White's first move
  moveNumber: number; // full move number
  color: Side;
  san: string;
  fenBefore: string;
  fenAfter: string;
  clockAfter: number | null; // seconds
  isCapture: boolean;
  captured?: string; // piece symbol captured (opponent-of-mover's piece type)
  captureSquare?: string;
  isCheck: boolean;
  isMate: boolean;
  castle: 'kingside' | 'queenside' | null;
  isPromotion: boolean;
  promotion?: string;
  piece: string;
}

export interface ClockSample {
  ply: number;
  clockBefore: number;
  clockAfter: number;
  spent: number;
}

export interface LoosePieceEvent {
  moveNumber: number;
  piece: string;
}

export interface GameAnalysis {
  headers: Record<string, string>;
  player: string;
  color: Side;
  result: GameResult;
  resultRaw: string;
  opponent: string;
  myElo: number | null;
  opponentElo: number | null;
  timeControl: TimeControlInfo;
  date: Date | null;
  termination: { category: TerminationCategory; raw: string };
  opening: { eco: string; name: string };
  ecoUrl?: string;
  link?: string;
  myAccuracy?: number;
  opponentAccuracy?: number;
  moves: string[];
  plies: Ply[];
  startFen: string;
  finalFen: string;
  pieceCounts: number[]; // non-pawn-non-king piece count after every ply, index 0 = start
  materialBalance: number[]; // from my side, index 0 = start
  castling: { side: 'kingside' | 'queenside'; moveNumber: number } | null;
  queenMovedEarly: boolean;
  checksGiven: number;
  promotions: number;
  myCaptureSquares: string[];
  piecesCaptured: Record<string, number>;
  piecesLost: Record<string, number>;
  matingPiece: { piece: string; by: 'me' | 'opponent' } | null;
  loosePieces: LoosePieceEvent[];
  maxMaterial: number;
  minMaterial: number;
  blownLead: boolean;
  comeback: boolean;
  lostWhileAhead: boolean;
  myClocks: ClockSample[];
  minClock: number | null;
  endClock: number | null;
  clockShareUsed: number | null;
  duration: { seconds: number; exact: boolean } | null;
}

const PIECE_VALUES: Record<string, number> = { p: 1, n: 3, b: 3, r: 5, q: 9, k: 0 };

/** Single-pass scan of a FEN's piece placement: non-pawn/king piece count and
 * material balance from `myColor`'s side. Called once per ply per game, so
 * this avoids redundant passes and regex/case-conversion overhead. */
function analyzeBoardFen(fen: string, myColor: Side): { pieceCount: number; materialBalance: number } {
  const end = fen.indexOf(' ');
  const placementLength = end === -1 ? fen.length : end;
  let pieceCount = 0;
  let mine = 0;
  let theirs = 0;
  for (let i = 0; i < placementLength; i++) {
    const code = fen.charCodeAt(i);
    let lower: string;
    let isWhite: boolean;
    if (code >= 65 && code <= 90) {
      isWhite = true;
      lower = String.fromCharCode(code + 32);
    } else if (code >= 97 && code <= 122) {
      isWhite = false;
      lower = String.fromCharCode(code);
    } else {
      continue; // digit or '/'
    }
    if (lower !== 'p' && lower !== 'k') pieceCount++;
    const value = PIECE_VALUES[lower] ?? 0;
    if ((myColor === 'white') === isWhite) mine += value;
    else theirs += value;
  }
  return { pieceCount, materialBalance: mine - theirs };
}

export function parseClockToSeconds(clock: string): number {
  const parts = clock.split(':').map(Number);
  let seconds = 0;
  for (const p of parts) seconds = seconds * 60 + p;
  return seconds;
}

export function parseTimeControl(raw: string): TimeControlInfo {
  if (raw.includes('/')) {
    const [, secondsPerMoveStr] = raw.split('/');
    const secondsPerMove = Number(secondsPerMoveStr) || 0;
    return { raw, type: 'daily', base: secondsPerMove, increment: 0, label: 'Daily' };
  }
  const [baseStr, incStr] = raw.split('+');
  const base = Number(baseStr) || 0;
  const increment = Number(incStr) || 0;
  return { raw, type: 'standard', base, increment, label: formatStandardLabel(base, increment) };
}

function formatStandardLabel(base: number, increment: number): string {
  let baseLabel: string;
  if (base % 60 === 0) {
    baseLabel = `${base / 60} min`;
  } else if (base < 60) {
    baseLabel = `${base} sec`;
  } else {
    baseLabel = `${(base / 60).toFixed(1).replace(/\.0$/, '')} min`;
  }
  return increment > 0 ? `${baseLabel} + ${increment}s` : baseLabel;
}

function classifyTermination(text: string, result: GameResult): TerminationCategory {
  const t = text.toLowerCase();
  if (result === 'draw') {
    if (t.includes('timeout') && t.includes('insufficient')) return 'timeout-vs-insufficient';
    if (t.includes('insufficient')) return 'insufficient-material';
    if (t.includes('50-move') || t.includes('fifty')) return 'fifty-move';
    if (t.includes('repetition')) return 'repetition';
    if (t.includes('stalemate')) return 'stalemate';
    if (t.includes('agree')) return 'agreement';
    return 'unknown';
  }
  if (t.includes('checkmate')) return 'checkmate';
  if (t.includes('resign')) return 'resignation';
  if (t.includes('abandon')) return 'abandoned';
  if (t.includes('time')) return 'time';
  return 'unknown';
}

function parseResult(resultRaw: string, color: Side): GameResult {
  if (resultRaw === '1/2-1/2') return 'draw';
  if (resultRaw === '1-0') return color === 'white' ? 'win' : 'loss';
  if (resultRaw === '0-1') return color === 'black' ? 'win' : 'loss';
  return 'draw';
}

function parseDateHeader(dateStr: string): { y: number; m: number; d: number } | null {
  const m = /^(\d{4})\.(\d{2})\.(\d{2})$/.exec(dateStr);
  if (!m) return null;
  return { y: Number(m[1]), m: Number(m[2]), d: Number(m[3]) };
}

function parseTimeHeader(timeStr: string): { h: number; m: number; s: number } | null {
  const m = /^(\d{2}):(\d{2}):(\d{2})/.exec(timeStr);
  if (!m) return null;
  return { h: Number(m[1]), m: Number(m[2]), s: Number(m[3]) };
}

function parseGameDate(headers: Record<string, string>): Date | null {
  const dateStr = headers.UTCDate ?? headers.Date;
  const timeStr = headers.UTCTime ?? headers.EndTime;
  if (!dateStr) return null;
  const d = parseDateHeader(dateStr);
  if (!d) return null;
  const t = timeStr ? parseTimeHeader(timeStr) : null;
  return new Date(
    Date.UTC(d.y, d.m - 1, d.d, t?.h ?? 0, t?.m ?? 0, t?.s ?? 0),
  );
}

function parseHeaderDateTime(dateStr: string, timeStr: string): Date | null {
  const d = parseDateHeader(dateStr);
  const t = parseTimeHeader(timeStr);
  if (!d || !t) return null;
  return new Date(Date.UTC(d.y, d.m - 1, d.d, t.h, t.m, t.s));
}

function computeDuration(
  headers: Record<string, string>,
  timeControl: TimeControlInfo,
  plies: Ply[],
): { seconds: number; exact: boolean } | null {
  const startDateStr = headers.UTCDate ?? headers.Date;
  const startTimeStr = headers.StartTime;
  if (startDateStr && startTimeStr) {
    const start = parseHeaderDateTime(startDateStr, startTimeStr);
    const endDateStr = headers.EndDate ?? headers.UTCDate;
    const endTimeStr = headers.EndTime;
    if (start && endTimeStr) {
      let end = parseHeaderDateTime(endDateStr ?? startDateStr, endTimeStr);
      if (end) {
        if (!headers.EndDate && end.getTime() < start.getTime()) {
          end = new Date(end.getTime() + 24 * 3600 * 1000);
        }
        return { seconds: (end.getTime() - start.getTime()) / 1000, exact: true };
      }
    }
  }

  if (timeControl.type === 'daily') return null;
  const { base, increment } = timeControl;
  const whitePlies = plies.filter((p) => p.color === 'white');
  const blackPlies = plies.filter((p) => p.color === 'black');
  const lastClock = (arr: Ply[]): number | null => {
    for (let i = arr.length - 1; i >= 0; i--) {
      if (arr[i].clockAfter != null) return arr[i].clockAfter;
    }
    return null;
  };
  const whiteLast = lastClock(whitePlies) ?? base;
  const blackLast = lastClock(blackPlies) ?? base;
  const whiteUsed = base + increment * whitePlies.length - whiteLast;
  const blackUsed = base + increment * blackPlies.length - blackLast;
  return { seconds: whiteUsed + blackUsed, exact: false };
}

export function analyzeGame(
  headers: Record<string, string>,
  moves: { san: string; clock: string | null }[],
  playerNameLower: string,
): GameAnalysis {
  const whiteIsPlayer = headers.White.toLowerCase() === playerNameLower;
  const color: Side = whiteIsPlayer ? 'white' : 'black';
  const opponent = whiteIsPlayer ? headers.Black : headers.White;
  const resultRaw = headers.Result ?? '*';
  const result = parseResult(resultRaw, color);

  const myEloRaw = whiteIsPlayer ? headers.WhiteElo : headers.BlackElo;
  const oppEloRaw = whiteIsPlayer ? headers.BlackElo : headers.WhiteElo;
  const myElo = myEloRaw ? Number(myEloRaw) : null;
  const opponentElo = oppEloRaw ? Number(oppEloRaw) : null;

  const timeControl = parseTimeControl(headers.TimeControl ?? '');
  const date = parseGameDate(headers);
  const termination = {
    category: classifyTermination(headers.Termination ?? '', result),
    raw: headers.Termination ?? '',
  };

  const chess = new Chess();
  const startFen = chess.fen();
  const plies: Ply[] = [];
  const fens: string[] = [startFen];
  const sanList: string[] = [];
  let currentFen = startFen;

  for (let i = 0; i < moves.length; i++) {
    const { san, clock } = moves[i];
    const fenBefore = currentFen;
    let moveObj;
    try {
      moveObj = chess.move(san);
    } catch {
      break; // stop replay on any illegal/unparseable move
    }
    if (!moveObj) break;
    const fenAfter = chess.fen();
    currentFen = fenAfter;
    const ply = i + 1;
    const moveColor: Side = moveObj.color === 'w' ? 'white' : 'black';
    const castle = moveObj.isKingsideCastle()
      ? 'kingside'
      : moveObj.isQueensideCastle()
        ? 'queenside'
        : null;
    let captureSquare: string | undefined;
    if (moveObj.isCapture()) {
      captureSquare = moveObj.isEnPassant()
        ? moveObj.to.charAt(0) + moveObj.from.charAt(1)
        : moveObj.to;
    }
    plies.push({
      ply,
      moveNumber: Math.ceil(ply / 2),
      color: moveColor,
      san: moveObj.san,
      fenBefore,
      fenAfter,
      clockAfter: clock ? parseClockToSeconds(clock) : null,
      isCapture: moveObj.isCapture(),
      captured: moveObj.captured,
      captureSquare,
      isCheck: moveObj.san.includes('+') || moveObj.san.includes('#'),
      isMate: moveObj.san.includes('#'),
      castle,
      isPromotion: moveObj.isPromotion(),
      promotion: moveObj.promotion,
      piece: moveObj.piece,
    });
    fens.push(fenAfter);
    sanList.push(moveObj.san);
  }

  const finalFen = fens[fens.length - 1];
  const boardStats = fens.map((f) => analyzeBoardFen(f, color));
  const pieceCounts = boardStats.map((b) => b.pieceCount);
  const materialBalances = boardStats.map((b) => b.materialBalance);

  const opening = headers.ECO
    ? { eco: headers.ECO, name: headers.ECOUrl ? openingNameFromUrl(headers.ECOUrl) : matchOpening(sanList).name }
    : matchOpening(sanList);

  // Castling
  let castling: GameAnalysis['castling'] = null;
  for (const p of plies) {
    if (p.color === color && p.castle) {
      castling = { side: p.castle, moveNumber: p.moveNumber };
      break;
    }
  }

  // Queen moved early (within my first 5 moves)
  const myPlies = plies.filter((p) => p.color === color);
  const queenMovedEarly = myPlies.slice(0, 5).some((p) => p.piece === 'q');

  // Checks given, promotions (by me)
  const checksGiven = myPlies.filter((p) => p.isCheck).length;
  const promotions = myPlies.filter((p) => p.isPromotion).length;

  // Captures: mine captured, mine lost, and squares where my pieces were taken
  const piecesCaptured: Record<string, number> = {};
  const piecesLost: Record<string, number> = {};
  const myCaptureSquares: string[] = [];
  for (const p of plies) {
    if (!p.isCapture || !p.captured) continue;
    if (p.color === color) {
      piecesCaptured[p.captured] = (piecesCaptured[p.captured] ?? 0) + 1;
    } else {
      piecesLost[p.captured] = (piecesLost[p.captured] ?? 0) + 1;
      if (p.captureSquare) myCaptureSquares.push(p.captureSquare);
    }
  }

  // Mating piece
  let matingPiece: GameAnalysis['matingPiece'] = null;
  const lastPly = plies[plies.length - 1];
  if (lastPly && lastPly.isMate) {
    matingPiece = { piece: lastPly.piece, by: lastPly.color === color ? 'me' : 'opponent' };
  }

  // Loose pieces heuristic
  const loosePieces: LoosePieceEvent[] = [];
  const myMoveIndices = plies
    .map((p, idx) => (p.color === color ? idx : -1))
    .filter((idx) => idx >= 0);
  // balanceBefore[j] / balanceAfter[j] for my j-th move, indices into `fens`/`materialBalances`
  const before: number[] = myMoveIndices.map((idx) => materialBalances[idx]); // ply idx in plies == index-1 in fens (fens[0]=start)
  const after: number[] = myMoveIndices.map((idx) => materialBalances[idx + 1]);
  for (let j = 0; j < myMoveIndices.length - 1; j++) {
    const drop = before[j + 1] - before[j];
    if (drop <= -3 && after[j + 1] <= before[j] - 3) {
      // Identify the opponent ply between my move j and my move j+1
      const myPlyIdx = myMoveIndices[j];
      const opponentPlyIdx = myPlyIdx + 1;
      const opponentPly = plies[opponentPlyIdx];
      const piece = opponentPly?.captured ?? 'unknown';
      const myReplyPly = plies[myMoveIndices[j + 1]];
      loosePieces.push({ moveNumber: myReplyPly.moveNumber, piece });
    }
  }

  const maxMaterial = Math.max(...materialBalances);
  const minMaterial = Math.min(...materialBalances);
  const finalBalance = materialBalances[materialBalances.length - 1];
  const blownLead = result === 'loss' && maxMaterial >= 5;
  const comeback = result === 'win' && minMaterial <= -5;
  const lostWhileAhead = result === 'loss' && finalBalance >= 3;

  // Clocks (mine)
  const myClocks: ClockSample[] = [];
  let prevClock = timeControl.type === 'standard' ? timeControl.base : null;
  for (const p of myPlies) {
    if (p.clockAfter == null) {
      prevClock = null;
      continue;
    }
    if (prevClock != null) {
      const spent = prevClock + timeControl.increment - p.clockAfter;
      myClocks.push({ ply: p.ply, clockBefore: prevClock, clockAfter: p.clockAfter, spent });
    }
    prevClock = p.clockAfter;
  }
  const minClock = myClocks.length ? Math.min(...myClocks.map((c) => c.clockAfter)) : null;
  const endClock = myClocks.length ? myClocks[myClocks.length - 1].clockAfter : null;
  const clockShareUsed =
    endClock != null && timeControl.type === 'standard' && timeControl.base > 0
      ? (timeControl.base - endClock) / timeControl.base
      : null;

  const duration = computeDuration(headers, timeControl, plies);

  return {
    headers,
    player: whiteIsPlayer ? headers.White : headers.Black,
    color,
    result,
    resultRaw,
    opponent,
    myElo,
    opponentElo,
    timeControl,
    date,
    termination,
    opening,
    ecoUrl: headers.ECOUrl,
    link: headers.Link,
    myAccuracy: parseAccuracy(whiteIsPlayer ? headers.WhiteAccuracy : headers.BlackAccuracy),
    opponentAccuracy: parseAccuracy(whiteIsPlayer ? headers.BlackAccuracy : headers.WhiteAccuracy),
    moves: sanList,
    plies,
    startFen,
    finalFen,
    pieceCounts,
    materialBalance: materialBalances,
    castling,
    queenMovedEarly,
    checksGiven,
    promotions,
    myCaptureSquares,
    piecesCaptured,
    piecesLost,
    matingPiece,
    loosePieces,
    maxMaterial,
    minMaterial,
    blownLead,
    comeback,
    lostWhileAhead,
    myClocks,
    minClock,
    endClock,
    clockShareUsed,
    duration,
  };
}

function parseAccuracy(v: string | undefined): number | undefined {
  if (v == null) return undefined;
  const n = Number(v);
  return Number.isFinite(n) ? n : undefined;
}

function openingNameFromUrl(url: string): string {
  const last = url.split('/').pop() ?? '';
  return last.replace(/-/g, ' ').replace(/\d+\.\.\..*/, '').trim() || 'Unknown opening';
}
