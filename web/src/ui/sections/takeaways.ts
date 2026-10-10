import { el } from './shared';
import type { SectionContext } from './shared';

export function renderTakeawaysSection(ctx: SectionContext): HTMLElement | null {
  if (ctx.report.takeaways.length === 0) return null;
  const section = el('div', { class: 'section', id: 'takeaways' }, [el('h2', {}, ['What stands out'])]);
  section.append(el('ol', { class: 'takeaway-list' }, ctx.report.takeaways.map((t) => el('li', {}, [t]))));
  return section;
}
