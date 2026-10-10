import { el, enginePlaceholder } from './shared';
import { barChart, COLORS } from '../svg';
import { formatDate } from '../format';
import type { SectionContext } from './shared';

export function renderTacticsSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'tactics' }, [el('h2', {}, ['Tactics'])]);
  const t = ctx.report.tactics;

  if (!ctx.hasEngine || t.errorRates.gamesCounted === 0) {
    section.append(enginePlaceholder(ctx, 'Inaccuracies, mistakes and blunders'));
    return section;
  }

  section.append(
    el('div', { class: 'grid-2' }, [
      el('div', {}, [
        el('h4', {}, ['Inaccuracies per game']),
        barChart(
          [
            { label: 'You', value: t.errorRates.myPerGame.inaccuracies, color: COLORS.muted },
            { label: 'Opponent', value: t.errorRates.opponentPerGame.inaccuracies, color: COLORS.rule },
          ],
          { width: 220, height: 140 },
        ),
      ]),
      el('div', {}, [
        el('h4', {}, ['Mistakes per game']),
        barChart(
          [
            { label: 'You', value: t.errorRates.myPerGame.mistakes, color: COLORS.warning },
            { label: 'Opponent', value: t.errorRates.opponentPerGame.mistakes, color: COLORS.rule },
          ],
          { width: 220, height: 140 },
        ),
      ]),
      el('div', {}, [
        el('h4', {}, ['Blunders per game']),
        barChart(
          [
            { label: 'You', value: t.errorRates.myPerGame.blunders, color: COLORS.loss },
            { label: 'Opponent', value: t.errorRates.opponentPerGame.blunders, color: COLORS.rule },
          ],
          { width: 220, height: 140 },
        ),
      ]),
    ]),
  );

  section.append(
    el('p', {}, [
      `You average ${t.errorRates.myPerGame.blunders.toFixed(2)} blunders per game versus ${t.errorRates.opponentPerGame.blunders.toFixed(2)} for your opponents.`,
    ]),
  );

  const phaseTable = el('table', {}, [
    el('thead', {}, [el('tr', {}, [el('th', {}, ['Phase']), el('th', {}, ['Your accuracy']), el('th', {}, ['Your blunders']), el('th', {}, ['Opponent blunders'])])]),
    el(
      'tbody',
      {},
      t.accuracyByPhase.map((p) =>
        el('tr', {}, [
          el('td', {}, [p.phase]),
          el('td', {}, [p.myAccuracy != null ? p.myAccuracy.toFixed(1) : '–']),
          el('td', {}, [String(p.myBlunders)]),
          el('td', {}, [String(p.opponentBlunders)]),
        ]),
      ),
    ),
  ]);
  section.append(el('h3', {}, ['Accuracy by phase']), phaseTable, el('p', { class: 'muted' }, ['Endgame accuracy tends to run high: fewer pieces mean fewer ways to go wrong.']));

  section.append(
    el('h3', {}, ['Missed punishments']),
    el('p', {}, [
      `You missed punishing ${t.missedPunishments.count} of your opponents' ${t.missedPunishments.opponentBlunderCount} blunders (${t.missedPunishments.pct.toFixed(1)}%).`,
    ]),
  );

  section.append(
    el('h3', {}, ['Your biggest blunders']),
    el(
      'ul',
      {},
      t.biggestBlunders.map((b) =>
        el('li', {}, [
          el('button', { onClick: () => ctx.onOpenGame(b.game) }, [
            `Move ${b.moveNumber} ${b.san} vs ${b.game.opponent}, ${formatDate(b.date)} — lost ${b.winPctLost.toFixed(0)}% win chance`,
          ]),
        ]),
      ),
    ),
  );

  return section;
}
