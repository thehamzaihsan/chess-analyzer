import { el } from './shared';
import { wdlBar } from '../svg';
import type { SectionContext } from './shared';

export function renderResultsSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'results' }, [el('h2', {}, ['Results'])]);
  const r = ctx.report.results;

  section.append(
    el('div', { class: 'grid-2' }, [
      el('div', {}, [el('h4', {}, ['By colour']), wdlBar(r.byColor.white), el('p', { class: 'muted' }, ['White']), wdlBar(r.byColor.black), el('p', { class: 'muted' }, ['Black'])]),
      el('div', {}, [
        el('h4', {}, ['By game length (moves)']),
        el(
          'table',
          {},
          [
            el('thead', {}, [el('tr', {}, [el('th', {}, ['Moves']), el('th', {}, ['Games']), el('th', {}, ['Score'])])]),
            el(
              'tbody',
              {},
              r.byLength.map((b) => el('tr', {}, [el('td', {}, [b.label]), el('td', {}, [String(b.games.length)]), el('td', {}, [`${b.wdl.scorePct.toFixed(1)}%`])])),
            ),
          ],
        ),
      ]),
    ]),
  );

  section.append(
    el('p', { class: 'muted' }, [
      `Median game length: ${r.medianLengths.game ?? '–'} moves. Median win length: ${r.medianLengths.win ?? '–'}. Median loss length: ${r.medianLengths.loss ?? '–'}.`,
    ]),
  );

  function terminationTable(title: string, rows: { category: string; count: number; pct: number }[]) {
    return el('div', {}, [
      el('h4', {}, [title]),
      el(
        'table',
        {},
        [
          el('thead', {}, [el('tr', {}, [el('th', {}, ['Termination']), el('th', {}, ['Count']), el('th', {}, ['%'])])]),
          el('tbody', {}, rows.map((row) => el('tr', {}, [el('td', {}, [row.category]), el('td', {}, [String(row.count)]), el('td', {}, [`${row.pct.toFixed(1)}%`])]))),
        ],
      ),
    ]);
  }

  section.append(
    el('div', { class: 'grid-2' }, [
      terminationTable('Games won by', r.wonBy),
      terminationTable('Games lost by', r.lostBy),
    ]),
  );
  section.append(terminationTable('Games drawn by', r.drawnBy));

  return section;
}
