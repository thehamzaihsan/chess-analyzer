import { el, enginePlaceholder } from './shared';
import { wdlBar } from '../svg';
import type { SectionContext } from './shared';

export function renderEndgameSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'endgame' }, [el('h2', {}, ['Endgame'])]);
  const r = ctx.report.endgame;

  if (!ctx.hasEngine) {
    section.append(enginePlaceholder(ctx, 'Endgame results by position'));
    return section;
  }
  if (r.gamesReachedEndgame === 0) {
    section.append(el('p', { class: 'muted' }, ['None of the engine-checked games reached an endgame.']));
    return section;
  }

  section.append(
    el('p', {}, [`${r.gamesReachedEndgame} games reached an endgame.`]),
    wdlBar(r.overallWdl),
  );

  const table = el('table', {}, [
    el('thead', {}, [el('tr', {}, [el('th', {}, ['Entered']), el('th', {}, ['Games']), el('th', {}, ['W/D/L']), el('th', {}, ['Score'])])]),
    el(
      'tbody',
      {},
      r.byEntryState.map((row) =>
        el('tr', {}, [
          el('td', {}, [row.state]),
          el('td', {}, [String(row.games.length)]),
          el('td', {}, [`${row.wdl.won}/${row.wdl.drawn}/${row.wdl.lost}`]),
          el('td', {}, [`${row.wdl.scorePct.toFixed(1)}%`]),
        ]),
      ),
    ),
  ]);
  section.append(table);

  if (r.atLeastDeservedRate != null) {
    section.append(
      el('p', {}, [`You scored at least what the position deserved in ${r.atLeastDeservedRate.toFixed(1)}% of endgames.`]),
    );
  }

  if (r.showAdvice) {
    section.append(
      el('p', { class: 'muted' }, [
        'Your winning endgames convert under 80% of the time. Drilling basic mates (king + queen/rook) and pawn-race calculation should help.',
      ]),
    );
  }

  return section;
}
