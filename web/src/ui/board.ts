import { el } from './dom';
import type { Side } from '../analyze';

const UNICODE_PIECES: Record<string, string> = {
  K: '♔',
  Q: '♕',
  R: '♖',
  B: '♗',
  N: '♘',
  P: '♙',
  k: '♚',
  q: '♛',
  r: '♜',
  b: '♝',
  n: '♞',
  p: '♟',
};

const FILES = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

function fenToBoard(fen: string): (string | null)[][] {
  const placement = fen.split(' ')[0];
  const rows = placement.split('/');
  return rows.map((row) => {
    const cells: (string | null)[] = [];
    for (const ch of row) {
      if (/\d/.test(ch)) {
        for (let i = 0; i < Number(ch); i++) cells.push(null);
      } else {
        cells.push(ch);
      }
    }
    return cells;
  });
}

/** Renders a FEN as a CSS-grid board with coordinates, oriented from `orientation`'s side. */
export function renderBoard(fen: string, orientation: Side): HTMLDivElement {
  const rows = fenToBoard(fen); // rows[0] = rank 8 .. rows[7] = rank 1
  const ranks = [8, 7, 6, 5, 4, 3, 2, 1];
  const displayRanks = orientation === 'white' ? ranks : [...ranks].reverse();
  const displayFiles = orientation === 'white' ? FILES : [...FILES].reverse();

  const grid = el('div', { class: 'board-grid' });
  for (const rank of displayRanks) {
    const rowCells = rows[8 - rank];
    for (const file of displayFiles) {
      const fileIdx = FILES.indexOf(file);
      const piece = rowCells[fileIdx];
      const isLight = (fileIdx + rank) % 2 === 1;
      const cell = el('div', { class: `board-cell ${isLight ? 'light' : 'dark'}` }, [
        piece ? UNICODE_PIECES[piece] ?? '' : '',
      ]);
      grid.append(cell);
    }
  }
  return grid;
}
