import { el } from './shared';
import { wdlBar } from '../svg';
import { slugifyOpeningName } from '../../aggregate/topOpenings';
import type { SectionContext } from './shared';
import type { OpeningVerdict } from '../../aggregate/openings';

const VERDICT_TEXT: Record<OpeningVerdict, string> = {
  strong: 'Strong',
  'good-not-converting': 'Good positions, not converting',
  'worse-converting': 'Worse positions, converting',
  'needs-study': 'Needs study',
  level: 'Level',
};

function interpretLabel(dir: 'ahead' | 'level' | 'behind' | null): string {
  if (dir === 'ahead') return 'ahead';
  if (dir === 'behind') return 'behind';
  if (dir === 'level') return 'level';
  return 'not enough data';
}

export function renderOpeningSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'opening' }, [el('h2', {}, ['Opening'])]);
  const evalReport = ctx.report.openingsEval;

  section.append(
    ctx.hasEngine
      ? el('p', {}, [
          `Average evaluation after move 15: overall you are ${interpretLabel(evalReport.interpret(evalReport.overall))}`,
          `, as White ${interpretLabel(evalReport.interpret(evalReport.white))}`,
          `, as Black ${interpretLabel(evalReport.interpret(evalReport.black))}.`,
        ])
      : el('p', { class: 'muted' }, [
          'Evaluations after move 15 and opening verdicts need engine analysis (run it from the Overview tab). Results below are based on game outcomes only.',
        ]),
  );

  const cards = ctx.report.openingCards.map((card) => {
    const needsLink = !ctx.hasEngine || card.verdict !== 'strong';
    return el('div', { class: 'card' }, [
      el('h4', {}, [`${card.name} (${card.eco})`]),
      el('p', { class: 'muted' }, [`${card.color}, ${card.games.length} games`]),
      wdlBar(card.wdl),
      el('p', {}, [
        `Score ${card.wdl.scorePct.toFixed(1)}%`,
        card.evalAfter15 != null ? `, eval after 15 ${(card.evalAfter15 / 100).toFixed(2)}` : '',
      ]),
      ctx.hasEngine ? el('p', {}, [VERDICT_TEXT[card.verdict]]) : '',
      needsLink
        ? el(
            'a',
            { href: `https://www.chess.com/openings/${slugifyOpeningName(card.name)}`, target: '_blank', rel: 'noreferrer' },
            ['Read about this opening →'],
          )
        : '',
    ]);
  });
  section.append(el('div', { class: 'cards' }, cards));

  const ecoTable = el('table', {}, [
    el('thead', {}, [
      el('tr', {}, [el('th', {}, ['ECO']), el('th', {}, ['Games']), el('th', {}, ['W/D/L']), el('th', {}, ['Eval @15']), el('th', {}, ['Score'])]),
    ]),
    el(
      'tbody',
      {},
      ctx.report.openingEcoTable.map((row) =>
        el('tr', {}, [
          el('td', {}, [row.eco]),
          el('td', {}, [String(row.games)]),
          el('td', {}, [wdlBar(row.wdl, 100, 10)]),
          el('td', {}, [row.evalAfter15 != null ? (row.evalAfter15 / 100).toFixed(2) : '–']),
          el('td', {}, [`${row.wdl.scorePct.toFixed(1)}%`]),
        ]),
      ),
    ),
  ]);
  section.append(el('h3', {}, ['By ECO code']), el('div', { class: 'hscroll' }, [ecoTable]));

  section.append(
    el('h3', {}, ['How to improve your openings']),
    el('p', { class: 'muted' }, [
      'Develop knights and bishops before moving the same piece twice, castle early, contest the center with pawns, and avoid early queen sorties that invite your opponent to develop with tempo.',
    ]),
  );

  function nameTable(title: string, rows: { name: string; eco: string; wdl: { won: number; drawn: number; lost: number; total: number; scorePct: number } }[]) {
    const top = rows.slice(0, 8);
    return el('div', {}, [
      el('h4', {}, [title]),
      el('table', {}, [
        el('thead', {}, [el('tr', {}, [el('th', {}, ['Opening']), el('th', {}, ['Games']), el('th', {}, ['Score'])])]),
        el(
          'tbody',
          {},
          top.map((r) =>
            el('tr', {}, [el('td', {}, [`${r.name} (${r.eco})`]), el('td', {}, [String(r.wdl.total)]), el('td', {}, [`${r.wdl.scorePct.toFixed(1)}%`])]),
          ),
        ),
      ]),
      rows.length > 8 ? el('p', { class: 'muted' }, [`+${rows.length - 8} more`]) : '',
    ]);
  }

  section.append(
    el('div', { class: 'grid-2' }, [
      nameTable('As White', ctx.report.openingsByColor.white),
      nameTable('As Black', ctx.report.openingsByColor.black),
    ]),
  );

  function moveTable(title: string, rows: { move: string; wdl: { won: number; drawn: number; lost: number; total: number; scorePct: number } }[]) {
    return el('div', {}, [
      el('h4', {}, [title]),
      el('table', {}, [
        el('thead', {}, [el('tr', {}, [el('th', {}, ['Move']), el('th', {}, ['Games']), el('th', {}, ['Score'])])]),
        el(
          'tbody',
          {},
          rows.map((r) => el('tr', {}, [el('td', {}, [r.move]), el('td', {}, [String(r.wdl.total)]), el('td', {}, [`${r.wdl.scorePct.toFixed(1)}%`])])),
        ),
      ]),
    ]);
  }

  section.append(
    el('div', { class: 'grid-2' }, [
      moveTable('First move as White', ctx.report.firstMoveAsWhite),
      el('div', {}, [moveTable('Reply to 1.e4 as Black', ctx.report.replyToE4), moveTable('Reply to 1.d4 as Black', ctx.report.replyToD4)]),
    ]),
  );

  return section;
}
