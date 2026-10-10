import { el } from './shared';
import { formatDate } from '../format';
import type { SectionContext } from './shared';

export function renderOpponentsSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'opponents' }, [el('h2', {}, ['Opponents'])]);
  const r = ctx.report.opponents;

  const gapTable = el('table', {}, [
    el('thead', {}, [el('tr', {}, [el('th', {}, ['Opponent strength']), el('th', {}, ['Games']), el('th', {}, ['W/D/L']), el('th', {}, ['Score'])])]),
    el(
      'tbody',
      {},
      r.byRatingGap.map((row) =>
        el('tr', {}, [
          el('td', {}, [row.label]),
          el('td', {}, [String(row.games.length)]),
          el('td', {}, [`${row.wdl.won}/${row.wdl.drawn}/${row.wdl.lost}`]),
          el('td', {}, [`${row.wdl.scorePct.toFixed(1)}%`]),
        ]),
      ),
    ),
  ]);
  section.append(gapTable);

  if (r.performanceRating != null) {
    section.append(el('p', {}, [`Performance rating: ${Math.round(r.performanceRating)}.`]));
  }

  section.append(
    el('h3', {}, ['Opponents played more than once']),
    el(
      'ul',
      {},
      r.repeatOpponents
        .slice(0, 10)
        .map((o) => el('li', {}, [`${o.name}: ${o.games.length} games, ${o.wdl.scorePct.toFixed(0)}% score`])),
    ),
  );

  section.append(
    el('div', { class: 'grid-2' }, [
      el('div', {}, [
        el('h4', {}, ['Best wins']),
        el(
          'ul',
          {},
          r.bestWins.map((g) =>
            el('li', {}, [el('button', { onClick: () => ctx.onOpenGame(g) }, [`vs ${g.opponent} (${g.opponentPreRating}), ${formatDate(g.date)}`])]),
          ),
        ),
      ]),
      el('div', {}, [
        el('h4', {}, ['Toughest losses']),
        el(
          'ul',
          {},
          r.toughestLosses.map((g) =>
            el('li', {}, [el('button', { onClick: () => ctx.onOpenGame(g) }, [`vs ${g.opponent} (${g.opponentPreRating}), ${formatDate(g.date)}`])]),
          ),
        ),
      ]),
    ]),
  );

  return section;
}
