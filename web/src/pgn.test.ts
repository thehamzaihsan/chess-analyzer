import { describe, it, expect } from 'vitest';
import { splitGames, parseMovetext } from './pgn';

describe('splitGames', () => {
  it('parses a single game with headers and movetext', () => {
    const pgn = `[Event "Live Chess"]
[White "alice"]
[Black "bob"]
[Result "1-0"]

1. e4 {[%clk 0:09:59]} 1... e5 {[%clk 0:09:58]} 2. Nf3 1-0
`;
    const games = splitGames(pgn);
    expect(games).toHaveLength(1);
    expect(games[0].headers.White).toBe('alice');
    expect(games[0].headers.Black).toBe('bob');
    expect(games[0].moves.map((m) => m.san)).toEqual(['e4', 'e5', 'Nf3']);
    expect(games[0].moves[0].clock).toBe('0:09:59');
  });

  it('splits two concatenated games where a header follows movetext', () => {
    const pgn = `[White "a"]
[Black "b"]
[Result "1-0"]

1. e4 e5 1-0

[White "c"]
[Black "d"]
[Result "0-1"]

1. d4 d5 0-1
`;
    const games = splitGames(pgn);
    expect(games).toHaveLength(2);
    expect(games[0].headers.White).toBe('a');
    expect(games[1].headers.White).toBe('c');
  });

  it('drops games missing White or Black headers', () => {
    const pgn = `[White "a"]
[Result "1-0"]

1. e4 1-0
`;
    expect(splitGames(pgn)).toHaveLength(0);
  });
});

describe('parseMovetext', () => {
  it('strips nested variations', () => {
    const moves = parseMovetext('1. e4 (1. d4 d5) 1... e5 2. Nf3 (2. Bc4 Nc6) Nc6');
    expect(moves.map((m) => m.san)).toEqual(['e4', 'e5', 'Nf3', 'Nc6']);
  });

  it('extracts clocks and attaches to preceding move', () => {
    const moves = parseMovetext('1. e4 {[%clk 0:09:59.9]} 1... e5 {[%clk 0:09:58.1]}');
    expect(moves[0].clock).toBe('0:09:59.9');
    expect(moves[1].clock).toBe('0:09:58.1');
  });

  it('strips move numbers including ellipsis form', () => {
    const moves = parseMovetext('1. e4 e5 2. Nf3 2... Nc6');
    expect(moves.map((m) => m.san)).toEqual(['e4', 'e5', 'Nf3', 'Nc6']);
  });

  it('strips NAGs and annotation glyphs', () => {
    const moves = parseMovetext('1. e4?! e5!! 2. Qh5?? Nc6! $1 $16');
    expect(moves.map((m) => m.san)).toEqual(['e4', 'e5', 'Qh5', 'Nc6']);
  });

  it('strips trailing result token', () => {
    const moves = parseMovetext('1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7# 1-0');
    expect(moves.map((m) => m.san)).toEqual(['e4', 'e5', 'Qh5', 'Nc6', 'Bc4', 'Nf6', 'Qxf7#']);
  });

  it('keeps check and mate symbols', () => {
    const moves = parseMovetext('1. e4 e5 2. Qh5 Nc6 3. Bc4 Nf6 4. Qxf7#');
    expect(moves[moves.length - 1].san).toBe('Qxf7#');
  });
});
