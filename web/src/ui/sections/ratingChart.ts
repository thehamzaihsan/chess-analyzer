import { el } from './shared';
import { lineChart, resultColor } from '../svg';
import { mainTimeControl } from '../../aggregate/ratings';
import { formatDate } from '../format';
import type { SectionContext } from './shared';
import type { AnalyzedGame } from '../../aggregate/ratings';

let zoomRange: { min: number; max: number } | null = null;

function monthKey(d: Date): string {
  return `${d.getUTCFullYear()}-${String(d.getUTCMonth() + 1).padStart(2, '0')}`;
}

function wdlTable(groups: Map<string, AnalyzedGame[]>, firstColumnLabel: string): HTMLElement {
  const rows = [...groups.entries()].map(([key, games]) => {
    const won = games.filter((g) => g.result === 'win').length;
    const drawn = games.filter((g) => g.result === 'draw').length;
    const lost = games.filter((g) => g.result === 'loss').length;
    return el('tr', {}, [
      el('td', {}, [key]),
      el('td', {}, [String(won)]),
      el('td', {}, [String(drawn)]),
      el('td', {}, [String(lost)]),
    ]);
  });
  return el('table', {}, [
    el('thead', {}, [
      el('tr', {}, [el('th', {}, [firstColumnLabel]), el('th', {}, ['W']), el('th', {}, ['D']), el('th', {}, ['L'])]),
    ]),
    el('tbody', {}, rows),
  ]);
}

export function renderRatingChartSection(ctx: SectionContext): HTMLElement {
  const section = el('div', { class: 'section', id: 'rating-over-time' }, [el('h2', {}, ['Rating over time'])]);
  const mtc = mainTimeControl(ctx.games);
  if (!mtc) {
    section.append(el('p', { class: 'muted' }, ['Not enough rated games to chart.']));
    return section;
  }

  const seriesGames = ctx.games
    .filter((g) => g.timeControl.label === mtc && g.myElo != null)
    .sort((a, b) => a.index - b.index);

  const visible = zoomRange
    ? seriesGames.filter((g) => g.index >= zoomRange!.min && g.index <= zoomRange!.max)
    : seriesGames;

  const ratings = visible.map((g) => g.myElo as number);
  const rawMin = ratings.length ? Math.min(...ratings) : 0;
  const rawMax = ratings.length ? Math.max(...ratings) : 0;
  const yPad = Math.max(10, Math.round((rawMax - rawMin) * 0.1));
  const yMin = Math.floor((rawMin - yPad) / 10) * 10;
  const yMax = Math.ceil((rawMax + yPad) / 10) * 10;

  const dateByIndex = new Map(seriesGames.map((g) => [g.index, g.date]));
  const nearestDate = (index: number): Date | null => {
    let best: Date | null = null;
    let bestDist = Infinity;
    for (const [i, d] of dateByIndex) {
      const dist = Math.abs(i - index);
      if (dist < bestDist && d) {
        best = d;
        bestDist = dist;
      }
    }
    return best;
  };

  const chart = lineChart(
    visible.map((g) => ({
      x: g.index,
      y: g.myElo as number,
      color: resultColor(g.result),
      title: `${formatDate(g.date)} vs ${g.opponent}: ${g.myElo} (${g.result})`,
      onClick: () => ctx.onOpenGame(g),
    })),
    {
      width: 880,
      height: 340,
      padding: 16,
      yMin,
      yMax,
      yTickCount: 6,
      formatYTick: (v) => Math.round(v).toString(),
      yAxisLabel: 'Rating',
      xTickCount: 5,
      formatXTick: (v) => {
        const d = nearestDate(Math.round(v));
        return d ? d.toLocaleDateString(undefined, { month: 'short', year: '2-digit' }) : '';
      },
      xAxisLabel: 'Date',
      onRangeSelect: (xMin, xMax) => {
        zoomRange = { min: Math.round(xMin), max: Math.round(xMax) };
        rerender();
      },
    },
  );

  const resetButton = el('button', { onClick: () => { zoomRange = null; rerender(); } }, ['Show all games']);

  const wrapper = el('div', {}, []);
  function rerender() {
    const fresh = renderRatingChartSection(ctx);
    wrapper.replaceWith(fresh);
  }

  section.append(
    el('p', { class: 'muted' }, [`Main time control: ${mtc}. Drag across the chart to zoom in.`]),
    chart,
    el('div', {}, [resetButton]),
  );

  const byMonth = new Map<string, AnalyzedGame[]>();
  for (const g of seriesGames) {
    if (!g.date) continue;
    const key = monthKey(g.date);
    if (!byMonth.has(key)) byMonth.set(key, []);
    byMonth.get(key)!.push(g);
  }

  const byTimeControl = new Map<string, AnalyzedGame[]>();
  for (const g of ctx.games) {
    if (!byTimeControl.has(g.timeControl.label)) byTimeControl.set(g.timeControl.label, []);
    byTimeControl.get(g.timeControl.label)!.push(g);
  }

  section.append(
    el('div', { class: 'grid-2' }, [
      el('div', {}, [el('h4', {}, ['By month']), wdlTable(byMonth, 'Month')]),
      el('div', {}, [el('h4', {}, ['By time control']), wdlTable(byTimeControl, 'Time control')]),
    ]),
  );

  wrapper.append(section);
  return wrapper;
}
