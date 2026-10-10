import { el } from './shared';
import { wdlBar, boardHeatmap } from '../svg';
import type { SectionContext } from './shared';

export function renderBoardHabitsSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'board-habits' }, [el('h2', {}, ['Board habits'])]);
  const r = ctx.report.boardHabits;

  section.append(
    el('div', { class: 'grid-2' }, [
      el('div', {}, [
        el('h4', {}, ['Loose pieces']),
        el('p', {}, [
          `${r.loosePieces.perGame.toFixed(2)} per game, in ${r.loosePieces.gamesWithAtLeastOnePct.toFixed(1)}% of games.`,
        ]),
        r.loosePieces.usualMoveNumber != null
          ? el('p', { class: 'muted' }, [`Usually around move ${r.loosePieces.usualMoveNumber}.`])
          : '',
        el('p', { class: 'muted' }, [`Queen lost by move 15 in ${r.loosePieces.queenLostByMove15} games.`]),
        el(
          'ul',
          {},
          Object.entries(r.loosePieces.byPiece).map(([piece, count]) => el('li', {}, [`${piece}: ${count}`])),
        ),
      ]),
      el('div', {}, [
        el('h4', {}, ['Castling']),
        el('p', {}, [
          `${r.castling.castledPct.toFixed(1)}% of games of 15+ moves (${r.castling.castledCount} of ${r.castling.eligibleGames}).`,
        ]),
        el('p', { class: 'muted' }, [`Kingside ${r.castling.kingsideCount}, queenside ${r.castling.queensideCount}.`]),
        r.castling.usualCastlingMove != null ? el('p', { class: 'muted' }, [`Usually on move ${r.castling.usualCastlingMove}.`]) : '',
        el('div', { class: 'grid-2' }, [
          el('div', {}, [wdlBar(r.castling.wdlCastled), el('p', { class: 'muted' }, ['Castled'])]),
          el('div', {}, [wdlBar(r.castling.wdlNotCastled), el('p', { class: 'muted' }, ['Not castled'])]),
        ]),
      ]),
    ]),
  );

  section.append(
    el('div', { class: 'grid-2' }, [
      el('div', {}, [wdlBar(ctx.report.boardHabits.earlyQueen.wdlQueenOut), el('p', { class: 'muted' }, ['Queen out by move 5'])]),
      el('div', {}, [wdlBar(ctx.report.boardHabits.earlyQueen.wdlQueenHome), el('p', { class: 'muted' }, ['Queen kept home'])]),
    ]),
  );

  section.append(
    el('h3', {}, ['Where your pieces get taken']),
    boardHeatmap(r.captureHeatmap),
  );

  section.append(
    el('h3', {}, ['Checkmates']),
    el('p', {}, [
      `Delivered: ${r.mates.deliveredTotal} (${Object.entries(r.mates.deliveredByPiece).map(([p, c]) => `${p}: ${c}`).join(', ') || 'none'})`,
    ]),
    el('p', {}, [
      `Received: ${r.mates.receivedTotal} (${Object.entries(r.mates.receivedByPiece).map(([p, c]) => `${p}: ${c}`).join(', ') || 'none'})`,
    ]),
  );

  return section;
}
