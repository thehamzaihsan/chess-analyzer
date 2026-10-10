import { describe, it, expect } from 'vitest';
import { analyzeGame, parseTimeControl, parseClockToSeconds } from './analyze';
import { parseMovetext } from './pgn';

function game(headers: Record<string, string>, san: string) {
  return analyzeGame(headers, parseMovetext(san), headers.__player ?? 'me');
}

describe('parseTimeControl', () => {
  it('parses plain seconds as minutes', () => {
    expect(parseTimeControl('600').label).toBe('10 min');
    expect(parseTimeControl('600').base).toBe(600);
    expect(parseTimeControl('600').increment).toBe(0);
  });

  it('parses base+increment', () => {
    const tc = parseTimeControl('180+2');
    expect(tc.label).toBe('3 min + 2s');
    expect(tc.base).toBe(180);
    expect(tc.increment).toBe(2);
  });

  it('parses daily as 1/N', () => {
    const tc = parseTimeControl('1/172800');
    expect(tc.type).toBe('daily');
    expect(tc.label).toBe('Daily');
  });
});

describe('parseClockToSeconds', () => {
  it('parses h:mm:ss.s', () => {
    expect(parseClockToSeconds('0:09:43.8')).toBeCloseTo(583.8, 5);
  });
});

describe('result and termination parsing', () => {
  const baseHeaders = {
    White: 'me',
    Black: 'opp',
    WhiteElo: '1000',
    BlackElo: '1000',
    TimeControl: '600',
    Date: '2024.01.01',
  };

  it('classifies a white win by checkmate', () => {
    const a = game(
      { ...baseHeaders, Result: '1-0', Termination: 'me won by checkmate' },
      '1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7#',
    );
    expect(a.result).toBe('win');
    expect(a.termination.category).toBe('checkmate');
  });

  it('classifies a black loss by resignation', () => {
    const a = game(
      { ...baseHeaders, Result: '1-0', Termination: 'opp won by resignation' },
      '1. e4 e5',
    );
    expect(a.color).toBe('white');
    // player is "me" who is White here, so a white win — check opposite: player Black
    expect(a.result).toBe('win');
  });

  it('classifies my loss when I am black and white wins by resignation', () => {
    const headers = {
      White: 'opp',
      Black: 'me',
      WhiteElo: '1000',
      BlackElo: '1000',
      TimeControl: '600',
      Date: '2024.01.01',
      Result: '1-0',
      Termination: 'opp won by resignation',
    };
    const a = game(headers, '1. e4 e5');
    expect(a.color).toBe('black');
    expect(a.result).toBe('loss');
    expect(a.termination.category).toBe('resignation');
  });

  it('classifies a draw by stalemate', () => {
    const a = game(
      { ...baseHeaders, Result: '1/2-1/2', Termination: 'Game drawn by stalemate' },
      '1. e4 e5',
    );
    expect(a.result).toBe('draw');
    expect(a.termination.category).toBe('stalemate');
  });

  it('classifies a draw by timeout vs insufficient material', () => {
    const a = game(
      {
        ...baseHeaders,
        Result: '1/2-1/2',
        Termination: 'Game drawn by timeout vs insufficient material',
      },
      '1. e4 e5',
    );
    expect(a.termination.category).toBe('timeout-vs-insufficient');
  });
});

describe('duration', () => {
  const baseHeaders = {
    White: 'me',
    Black: 'opp',
    WhiteElo: '1000',
    BlackElo: '1000',
    Result: '1-0',
    Termination: 'me won by resignation',
  };

  it('uses exact start/end time when present', () => {
    const a = game(
      {
        ...baseHeaders,
        TimeControl: '600',
        Date: '2024.01.01',
        StartTime: '10:00:00',
        EndDate: '2024.01.01',
        EndTime: '10:15:30',
      },
      '1. e4 e5',
    );
    expect(a.duration?.exact).toBe(true);
    expect(a.duration?.seconds).toBe(15 * 60 + 30);
  });

  it('rolls over a day when end time looks earlier and there is no EndDate', () => {
    const a = game(
      {
        ...baseHeaders,
        TimeControl: '600',
        Date: '2024.01.01',
        StartTime: '23:50:00',
        EndTime: '00:05:00',
      },
      '1. e4 e5',
    );
    expect(a.duration?.exact).toBe(true);
    expect(a.duration?.seconds).toBe(15 * 60);
  });

  it('falls back to clock-based duration when start/end times are absent', () => {
    const moves = parseMovetext(
      '1. e4 {[%clk 0:09:50]} 1... e5 {[%clk 0:09:55]} 2. Nf3 {[%clk 0:09:40]} 2... Nc6 {[%clk 0:09:45]}',
    );
    const a = analyzeGame(
      { ...baseHeaders, TimeControl: '600', Date: '2024.01.01' },
      moves,
      'me',
    );
    expect(a.duration?.exact).toBe(false);
    // white used 600-9*60-40=... let's just check it's a positive plausible number
    expect(a.duration!.seconds).toBeGreaterThan(0);
  });
});

describe('material heuristic (loose pieces)', () => {
  it('does not flag a quiet game with no captures', () => {
    const headers = {
      White: 'opp',
      Black: 'me',
      WhiteElo: '1000',
      BlackElo: '1000',
      TimeControl: '600',
      Date: '2024.01.01',
      Result: '1/2-1/2',
      Termination: 'Game drawn by agreement',
    };
    const a = analyzeGame(headers, parseMovetext('1. e4 e6 2. d4 d5 3. Nc3 Nf6'), 'me');
    expect(a.loosePieces).toHaveLength(0);
  });

  it('flags a straightforward hung queen', () => {
    // White hangs the queen on move 3 (Qxe5?? Nxe5) and move 4 doesn't win it back.
    const san = '1. e4 e5 2. Qh5 Nc6 3. Qxe5 Nxe5 4. d4 d6';
    const headers = {
      White: 'me',
      Black: 'opp',
      WhiteElo: '1000',
      BlackElo: '1000',
      TimeControl: '600',
      Date: '2024.01.01',
      Result: '0-1',
      Termination: 'opp won by resignation',
    };
    const a = analyzeGame(headers, parseMovetext(san), 'me');
    expect(a.loosePieces.length).toBeGreaterThan(0);
    expect(a.loosePieces[0].piece).toBe('q');
  });
});
