import { el } from './shared';
import type { SectionContext } from './shared';

export function renderTimeManagementSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'time-management' }, [el('h2', {}, ['Time management'])]);
  const t = ctx.report.timeManagement;

  section.append(
    el('ul', {}, [
      t.avgClockShareUsed != null ? el('li', {}, [`You use ${(t.avgClockShareUsed * 100).toFixed(1)}% of your clock on average.`]) : '',
      el('li', {}, [`${t.gamesUnder30sMinClock} games came down to under 30 seconds on your clock.`]),
      el('li', {}, [`Won on time: ${t.wonOnTime}. Lost on time: ${t.lostOnTime}.`]),
      t.avgClockShareLeftInNonTimeLosses != null
        ? el('li', {}, [`In non-time losses, you still had ${(t.avgClockShareLeftInNonTimeLosses * 100).toFixed(1)}% of your clock left on average.`])
        : '',
      el('li', {}, [`${t.movesUnder2sPct.toFixed(1)}% of your moves take under 2 seconds.`]),
      t.errorsHastyOrTimeTroublePct != null
        ? el('li', {}, [`${t.errorsHastyOrTimeTroublePct.toFixed(1)}% of your mistakes and blunders were rushed or made in time trouble.`])
        : '',
    ].filter((n) => n !== '')),
  );

  const byMoveTable = el('table', {}, [
    el('thead', {}, [el('tr', {}, [el('th', {}, ['Moves']), el('th', {}, ['Avg seconds'])])]),
    el(
      'tbody',
      {},
      t.avgTimePerMoveByBand.map((b) => el('tr', {}, [el('td', {}, [b.band]), el('td', {}, [b.avgSeconds != null ? b.avgSeconds.toFixed(1) : '–'])])),
    ),
  ]);

  const byThinkTimeTable = el('table', {}, [
    el('thead', {}, [el('tr', {}, [el('th', {}, ['Think time']), el('th', {}, ['Moves']), el('th', {}, ['Mistake rate'])])]),
    el(
      'tbody',
      {},
      t.mistakeRateByThinkTime.map((b) =>
        el('tr', {}, [
          el('td', {}, [b.band]),
          el('td', {}, [String(b.moveCount)]),
          el('td', {}, [b.mistakeRate != null ? `${b.mistakeRate.toFixed(1)}%` : '–']),
        ]),
      ),
    ),
  ]);

  section.append(
    el('div', { class: 'grid-2' }, [
      el('div', {}, [el('h4', {}, ['Average time per move by phase']), byMoveTable]),
      el('div', {}, [el('h4', {}, ['Mistake rate by think time']), byThinkTimeTable]),
    ]),
  );

  return section;
}
