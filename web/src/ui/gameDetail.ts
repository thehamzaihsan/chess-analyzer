import { el, escapeHtml } from './dom';
import { formatDate } from './format';
import { renderBoard } from './board';
import { lineChart, COLORS } from './svg';
import { winPctForSide } from '../engine/metrics';
import type { AnalyzedGame } from '../aggregate/ratings';

function annotationFor(drop: number): { mark: string; cls: string } | null {
  if (drop >= 15) return { mark: '??', cls: 'annotation-blunder' };
  if (drop >= 10) return { mark: '?', cls: 'annotation-mistake' };
  if (drop >= 5) return { mark: '?!', cls: 'annotation-inaccuracy' };
  return null;
}

function buildMoveList(game: AnalyzedGame): HTMLElement {
  const evalByPly = new Map(game.engine?.moveEvals.map((e) => [e.ply, e]) ?? []);
  const container = el('div', { class: 'move-list' });
  let line = '';
  const parts: (string | HTMLElement)[] = [];
  for (const ply of game.plies) {
    const prefix = ply.color === 'white' ? `${ply.moveNumber}. ` : ply.moveNumber === 1 ? '1... ' : '';
    const evalInfo = evalByPly.get(ply.ply);
    const annotation = evalInfo ? annotationFor(evalInfo.drop) : null;
    const span = el('span', { class: ply.color === game.color ? 'mine' : '' }, [prefix + ply.san]);
    parts.push(span, ' ');
    if (annotation) {
      parts.push(el('span', { class: annotation.cls }, [annotation.mark]), ' ');
    }
  }
  void line;
  container.append(...parts);
  return container;
}

function buildWinPctGraph(game: AnalyzedGame): SVGSVGElement | null {
  if (!game.engine) return null;
  const points = game.engine.evalsByPly.map((cp, ply) => {
    const myWinPct = game.color === 'white' ? winPctForSide(cp, 'white') : winPctForSide(cp, 'black');
    return { ply, myWinPct };
  });
  const moveEvalByPly = new Map(game.engine.moveEvals.map((e) => [e.ply, e]));
  const svg = lineChart(
    points.map((p) => {
      const moveEval = moveEvalByPly.get(p.ply);
      const isMine = moveEval?.color === game.color;
      let color: string | undefined;
      if (isMine && moveEval) {
        if (moveEval.isBlunder) color = COLORS.loss;
        else if (moveEval.isMistake) color = COLORS.warning;
      }
      return { x: p.ply, y: p.myWinPct, color, title: `Ply ${p.ply}: ${p.myWinPct.toFixed(0)}% win chance` };
    }),
    { yMin: 0, yMax: 100, height: 160 },
  );
  return svg;
}

export interface GameDetailOptions {
  onClose: () => void;
}

export function renderGameDetail(game: AnalyzedGame, options: GameDetailOptions): HTMLElement {
  const backdrop = el('div', { class: 'modal-backdrop', role: 'dialog', 'aria-modal': 'true' });
  const modal = el('div', { class: 'modal' });

  const closeButton = el('button', { 'aria-label': 'Close', onClick: () => options.onClose() }, ['Close']);

  const header = el('div', {}, [
    el('h3', {}, [`${escapeHtml(game.headers.White ?? '')} vs ${escapeHtml(game.headers.Black ?? '')}`]),
    el('p', { class: 'muted' }, [
      `${formatDate(game.date)} · ${game.timeControl.label} · ${game.opening.name} (${game.opening.eco})`,
    ]),
    el('p', { class: 'muted' }, [
      `Result: ${game.result} · Termination: ${game.termination.raw || game.termination.category} · ${Math.ceil(game.moves.length / 2)} moves`,
      game.castling ? ` · Castled ${game.castling.side} on move ${game.castling.moveNumber}` : ' · Did not castle',
    ]),
  ]);

  const accuracyLine: string[] = [];
  if (game.engine) {
    accuracyLine.push(
      `Engine accuracy: you ${game.engine.myAccuracy?.toFixed(1) ?? '?'} · opponent ${game.engine.opponentAccuracy?.toFixed(1) ?? '?'}`,
    );
    accuracyLine.push(
      `Your errors: ${game.engine.myErrors.inaccuracies} inaccuracies, ${game.engine.myErrors.mistakes} mistakes, ${game.engine.myErrors.blunders} blunders`,
    );
  }
  if (game.myAccuracy != null) {
    accuracyLine.push(`Chess.com accuracy: you ${game.myAccuracy} · opponent ${game.opponentAccuracy ?? '?'}`);
  }

  const body = el('div', { class: 'grid-2' }, [
    el('div', {}, [renderBoard(game.finalFen, game.color)]),
    el('div', {}, [
      ...accuracyLine.map((line) => el('p', { class: 'muted' }, [line])),
      buildWinPctGraph(game) ?? '',
      buildMoveList(game),
      game.link ? el('p', {}, [el('a', { href: game.link, target: '_blank', rel: 'noreferrer' }, ['Open on Chess.com →'])]) : '',
    ]),
  ]);

  modal.append(closeButton, header, body);
  backdrop.append(modal);
  backdrop.addEventListener('click', (e) => {
    if (e.target === backdrop) options.onClose();
  });
  return backdrop;
}
