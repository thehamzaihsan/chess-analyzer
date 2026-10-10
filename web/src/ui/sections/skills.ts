import { el, engineControls } from './shared';
import type { SectionContext } from './shared';
import type { SkillScore } from '../../aggregate/skills';

function labelSuffix(label: SkillScore['label']): string {
  switch (label) {
    case 'A strength':
      return 'strength';
    case 'Holding steady':
      return 'steady';
    case 'Work on this':
      return 'work';
    default:
      return 'na';
  }
}

const SECTION_IDS: Record<string, string> = {
  opening: 'opening',
  tactics: 'tactics',
  ending: 'endgame',
  'advantage-capitalization': 'advantage-resourcefulness',
  resourcefulness: 'advantage-resourcefulness',
  'time-management': 'time-management',
};

function skillCard(skill: SkillScore, onNavigate: (sectionId: string) => void): HTMLElement {
  const width = skill.score != null ? Math.max(0, Math.min(100, skill.score)) : 0;
  const suffix = labelSuffix(skill.label);
  const sectionId = SECTION_IDS[skill.key] ?? '';
  return el(
    'a',
    {
      class: 'card skill-card',
      href: `#${sectionId}`,
      onClick: (e: Event) => {
        e.preventDefault();
        onNavigate(sectionId);
      },
    },
    [
      el('h4', {}, [skill.title]),
      el('div', { class: 'big-number' }, [skill.score != null ? skill.score.toFixed(0) : '–']),
      el('div', { class: `meter meter-${suffix}` }, [el('span', { style: `width:${width}%` })]),
      el('div', { class: `label-${suffix}` }, [skill.label]),
      el('p', { class: 'muted' }, [skill.explanation]),
    ],
  );
}

export function renderSkillsSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'skills' }, [el('h2', {}, ['Skill overview'])]);

  section.append(engineControls(ctx));
  if (!ctx.hasEngine) return section;

  section.append(
    el(
      'div',
      { class: 'cards' },
      ctx.report.skills.map((skill) => skillCard(skill, ctx.onNavigateToSection)),
    ),
  );

  return section;
}
