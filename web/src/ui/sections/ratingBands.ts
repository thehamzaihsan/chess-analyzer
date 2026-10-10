import { el } from './shared';
import { stackedBarChart } from '../svg';
import type { SectionContext } from './shared';

export function renderRatingBandsSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'result-by-rating' }, [
    el('h2', {}, ['Result by opponent rating']),
  ]);
  const rows = ctx.report.ratingBands;

  section.append(
    stackedBarChart(rows.map((r) => ({ label: r.label, wdl: r.wdl }))),
  );

  const table = el('table', {}, [
    el('thead', {}, [el('tr', {}, [el('th', {}, ['Rating band']), el('th', {}, ['W']), el('th', {}, ['D']), el('th', {}, ['L']), el('th', {}, ['Score'])])]),
    el(
      'tbody',
      {},
      rows.map((r) =>
        el('tr', {}, [
          el('td', {}, [r.label]),
          el('td', {}, [String(r.wdl.won)]),
          el('td', {}, [String(r.wdl.drawn)]),
          el('td', {}, [String(r.wdl.lost)]),
          el('td', {}, [`${r.wdl.scorePct.toFixed(1)}%`]),
        ]),
      ),
    ),
  ]);
  section.append(el('div', { class: 'hscroll' }, [table]));
  return section;
}
