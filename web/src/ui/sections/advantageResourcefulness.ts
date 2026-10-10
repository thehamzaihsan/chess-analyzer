import { el, enginePlaceholder } from './shared';
import { accordion } from '../dom';
import { formatDate } from '../format';
import type { SectionContext } from './shared';

export function renderAdvantageResourcefulnessSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'advantage-resourcefulness' }, [
    el('h2', {}, ['Advantage capitalization & resourcefulness']),
  ]);
  const r = ctx.report.advantageResourcefulness;
  if (!ctx.hasEngine) section.append(enginePlaceholder(ctx, 'The capitalization and resourcefulness scores'));
  const score = (v: number | null) =>
    ctx.hasEngine ? el('div', { class: 'big-number' }, [v != null ? `${v.toFixed(0)}%` : '\u2013']) : '';
  const gameList = (title: string, games: typeof r.blownLeads) =>
    accordion(`${title} (${games.length})`, [
      games.length
        ? el(
            'ul',
            {},
            games.map((g) =>
              el('li', {}, [el('button', { onClick: () => ctx.onOpenGame(g) }, [`vs ${g.opponent}, ${formatDate(g.date)}`])]),
            ),
          )
        : el('p', { class: 'muted' }, ['None.']),
    ]);

  section.append(
    el('div', { class: 'grid-2' }, [
      el('div', {}, [
        el('h4', {}, ['Advantage capitalization']),
        score(r.advantageCapitalization.score),
        el('p', { class: 'muted' }, ['Share of games with a +3 pawn edge that you went on to win.']),
        gameList('Blown material leads', r.blownLeads),
      ]),
      el('div', {}, [
        el('h4', {}, ['Resourcefulness']),
        score(r.resourcefulness.score),
        el('p', { class: 'muted' }, ['Share of games down 3 pawns that you won or drew.']),
        gameList('Comebacks', r.comebacks),
      ]),
    ]),
  );

  return section;
}
