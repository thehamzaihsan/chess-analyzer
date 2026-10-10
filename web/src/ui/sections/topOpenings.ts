import { el } from './shared';
import { wdlBar } from '../svg';
import type { SectionContext } from './shared';

export function renderTopOpeningsSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'top-openings' }, [el('h2', {}, ['Top openings'])]);
  const rows = ctx.report.topOpenings;
  const maxTotal = Math.max(...rows.map((r) => r.wdl.total), 1);

  const table = el(
    'div',
    {},
    rows.map((row) => {
      const href = row.ecoUrl ?? `https://www.chess.com/openings/${row.fallbackSlug}`;
      return el('div', { style: 'display:flex;align-items:center;gap:12px;padding:6px 0;border-bottom:1px solid var(--rule)' }, [
        el('a', { href, target: '_blank', rel: 'noreferrer', style: 'min-width:220px' }, [row.name]),
        wdlBar(row.wdl, 200, 10, maxTotal),
        el('span', { class: 'muted' }, [
          `${row.games.length} games · ${row.whiteCount} as White, ${row.blackCount} as Black · ${row.wdl.scorePct.toFixed(0)}% score`,
        ]),
      ]);
    }),
  );
  section.append(table);
  return section;
}
