import { el } from './shared';
import { wdlBar } from '../svg';
import { interpretScore, type WDL } from '../../aggregate/winrate';
import type { SectionContext } from './shared';

function row(label: string, wdl: WDL): HTMLElement {
  const interpretation = interpretScore(wdl.scorePct);
  return el('div', {}, [
    el('h4', {}, [label]),
    wdlBar(wdl),
    el('p', { class: 'muted' }, [
      `${interpretation}: you win ${wdl.scorePct.toFixed(1)}% of ${wdl.total} games (${wdl.won}W ${wdl.drawn}D ${wdl.lost}L).`,
    ]),
  ]);
}

export function renderWinRateSection(ctx: SectionContext): HTMLElement {
  const r = ctx.report.winRate;
  const section = el('div', { class: 'section', id: 'win-rate' }, [el('h2', {}, ['Win rate'])]);

  section.append(el('div', { class: 'grid-2' }, [row('White', r.white), row('Black', r.black)]));
  section.append(row('Last 40 games', r.last40));

  if (r.last40.scorePct >= 60) {
    section.append(el('p', { class: 'muted' }, ['Your last 40 games are well above even — your rating may still be catching up.']));
  }

  return section;
}
