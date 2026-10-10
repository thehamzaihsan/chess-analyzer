import { el } from './shared';
import { wdlBar } from '../svg';
import type { SectionContext } from './shared';

export function renderWhenYouPlaySection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'when-you-play' }, [el('h2', {}, ['When you play'])]);
  const r = ctx.report.whenYouPlay;

  function wdlRows(rows: { label: string; wdl: { won: number; drawn: number; lost: number; total: number; scorePct: number } }[]) {
    return el('table', {}, [
      el('thead', {}, [el('tr', {}, [el('th', {}, ['']), el('th', {}, ['Games']), el('th', {}, ['Score'])])]),
      el(
        'tbody',
        {},
        rows.map((row) => el('tr', {}, [el('td', {}, [row.label]), el('td', {}, [String(row.wdl.total)]), el('td', {}, [`${row.wdl.scorePct.toFixed(1)}%`])])),
      ),
    ]);
  }

  section.append(
    el('div', { class: 'grid-2' }, [
      el('div', {}, [el('h4', {}, ['By weekday']), wdlRows(r.byWeekday.map((w) => ({ label: w.weekday, wdl: w.wdl })))]),
      el('div', {}, [el('h4', {}, ['By time of day']), wdlRows(r.byTimeOfDay.map((w) => ({ label: w.timeOfDay, wdl: w.wdl })))]),
    ]),
  );

  section.append(
    el('h4', {}, ['By game number in a sitting']),
    wdlRows(r.byGameNumberInSitting.map((g) => ({ label: g.gameNumber === 10 ? '10+' : String(g.gameNumber), wdl: g.wdl }))),
  );

  section.append(
    el('h4', {}, ['After a result']),
    el('div', { class: 'grid-2' }, [
      el('div', {}, [wdlBar(r.afterResult.afterWin), el('p', { class: 'muted' }, ['After a win'])]),
      el('div', {}, [wdlBar(r.afterResult.afterLoss), el('p', { class: 'muted' }, ['After a loss'])]),
    ]),
    el('div', {}, [wdlBar(r.afterResult.afterTwoLosses), el('p', { class: 'muted' }, ['After two losses in a row'])]),
  );

  section.append(
    el('p', { class: 'muted' }, [`${r.daysPlayed} days played, averaging ${r.gamesPerDay.toFixed(1)} games per day.`]),
  );

  return section;
}
