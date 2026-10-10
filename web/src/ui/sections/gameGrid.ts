import { el } from './shared';
import { formatDate } from '../format';
import type { SectionContext } from './shared';

export function renderGameGridSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'games' }, [el('h2', {}, ['Every game at a glance'])]);

  const sorted = [...ctx.games].sort((a, b) => a.index - b.index);
  const grid = el(
    'div',
    { class: 'game-grid' },
    sorted.map((g) =>
      el('button', {
        class: `game-square ${g.result}`,
        'aria-label': `${formatDate(g.date)} vs ${g.opponent}, ${g.result}`,
        onClick: () => ctx.onOpenGame(g),
      }),
    ),
  );

  section.append(grid);
  return section;
}
