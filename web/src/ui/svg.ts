const SVG_NS = 'http://www.w3.org/2000/svg';

export function svgEl<K extends keyof SVGElementTagNameMap>(
  tag: K,
  attrs: Record<string, string | number> = {},
): SVGElementTagNameMap[K] {
  const node = document.createElementNS(SVG_NS, tag);
  for (const [key, value] of Object.entries(attrs)) {
    const v = String(value);
    // CSS variables only resolve via style, not presentation attributes.
    if ((key === 'fill' || key === 'stroke') && v.startsWith('var(')) {
      node.style.setProperty(key, v);
    } else {
      node.setAttribute(key, v);
    }
  }
  return node;
}

export const COLORS = {
  win: 'var(--win)',
  loss: 'var(--loss)',
  draw: 'var(--draw)',
  warning: 'var(--warning)',
  ink: 'var(--ink)',
  muted: 'var(--muted)',
  rule: 'var(--rule-strong)',
  accent: 'var(--accent)',
};

export function resultColor(result: 'win' | 'draw' | 'loss'): string {
  return result === 'win' ? COLORS.win : result === 'loss' ? COLORS.loss : COLORS.draw;
}

export interface WdlLike {
  won: number;
  drawn: number;
  lost: number;
  total: number;
}

/** A horizontal stacked win/draw/loss bar, scaled to `scaleTotal` (defaults to its own total). */
export function wdlBar(wdl: WdlLike, width = 240, height = 14, scaleTotal?: number): SVGSVGElement {
  const total = scaleTotal ?? wdl.total;
  const svg = svgEl('svg', { width, height, viewBox: `0 0 ${width} ${height}` });
  if (total === 0) return svg;
  const scale = width / total;
  let x = 0;
  const segments: [number, string][] = [
    [wdl.won, COLORS.win],
    [wdl.drawn, COLORS.draw],
    [wdl.lost, COLORS.loss],
  ];
  for (const [count, color] of segments) {
    const w = count * scale;
    if (w > 0) {
      svg.append(svgEl('rect', { x, y: 0, width: w, height, fill: color }));
    }
    x += w;
  }
  return svg;
}

export interface LinePoint {
  x: number;
  y: number;
  color?: string;
  title?: string;
  onClick?: () => void;
}

export interface LineChartOptions {
  width?: number;
  height?: number;
  padding?: number;
  yMin?: number;
  yMax?: number;
  yTickCount?: number;
  formatYTick?: (value: number) => string;
  xTickCount?: number;
  formatXTick?: (value: number) => string;
  xAxisLabel?: string;
  yAxisLabel?: string;
  onRangeSelect?: (xMin: number, xMax: number) => void;
}

/** A simple SVG line/dot chart over points already in data coordinates. */
export function lineChart(points: LinePoint[], opts: LineChartOptions = {}): SVGSVGElement {
  const width = opts.width ?? 700;
  const height = opts.height ?? 240;
  const padding = opts.padding ?? 24;
  const leftPadding = opts.yTickCount ? Math.max(padding, 52) : padding;
  const bottomPadding = opts.xTickCount ? 34 : padding;
  const svg = svgEl('svg', { width, height, viewBox: `0 0 ${width} ${height}` });
  if (points.length === 0) return svg;

  const xs = points.map((p) => p.x);
  const ys = points.map((p) => p.y);
  const xMin = Math.min(...xs);
  const xMax = Math.max(...xs);
  const yMin = opts.yMin ?? Math.min(...ys);
  const yMax = opts.yMax ?? Math.max(...ys);

  const sx = (x: number) =>
    xMax === xMin ? leftPadding : leftPadding + ((x - xMin) / (xMax - xMin)) * (width - leftPadding - padding);
  const sy = (y: number) =>
    yMax === yMin
      ? (height - bottomPadding) / 2
      : height - bottomPadding - ((y - yMin) / (yMax - yMin)) * (height - bottomPadding - padding);

  // Y-axis gridlines + labels
  const tickCount = opts.yTickCount ?? 0;
  for (let i = 0; i < tickCount; i++) {
    const value = yMin + ((yMax - yMin) * i) / (tickCount - 1 || 1);
    const y = sy(value);
    svg.append(
      svgEl('line', { x1: leftPadding, y1: y, x2: width - padding, y2: y, stroke: COLORS.rule, 'stroke-dasharray': '2,3' }),
    );
    const label = svgEl('text', { x: leftPadding - 8, y: y + 3, 'text-anchor': 'end', 'font-size': 11, fill: COLORS.muted });
    label.textContent = opts.formatYTick ? opts.formatYTick(value) : value.toFixed(0);
    svg.append(label);
  }
  if (opts.yAxisLabel) {
    const t = svgEl('text', { x: 12, y: (height - bottomPadding) / 2, 'text-anchor': 'middle', 'font-size': 11, fill: COLORS.muted, transform: `rotate(-90 12 ${(height - bottomPadding) / 2})` });
    t.textContent = opts.yAxisLabel;
    svg.append(t);
  }

  // X-axis ticks + labels
  const xTicks = opts.xTickCount ?? 0;
  for (let i = 0; i < xTicks; i++) {
    const value = xMin + ((xMax - xMin) * i) / (xTicks - 1 || 1);
    const x = sx(value);
    svg.append(
      svgEl('line', { x1: x, y1: height - bottomPadding, x2: x, y2: height - bottomPadding + 4, stroke: COLORS.muted }),
    );
    const label = svgEl('text', { x, y: height - bottomPadding + 16, 'text-anchor': 'middle', 'font-size': 11, fill: COLORS.muted });
    label.textContent = opts.formatXTick ? opts.formatXTick(value) : value.toFixed(0);
    svg.append(label);
  }
  if (opts.xAxisLabel) {
    const t = svgEl('text', { x: (leftPadding + width - padding) / 2, y: height - 4, 'text-anchor': 'middle', 'font-size': 11, fill: COLORS.muted });
    t.textContent = opts.xAxisLabel;
    svg.append(t);
  }

  // Axis line
  svg.append(
    svgEl('line', { x1: leftPadding, y1: height - bottomPadding, x2: width - padding, y2: height - bottomPadding, stroke: COLORS.rule }),
  );

  // Path
  const pathD = points.map((p, i) => `${i === 0 ? 'M' : 'L'}${sx(p.x)},${sy(p.y)}`).join(' ');
  svg.append(svgEl('path', { d: pathD, fill: 'none', stroke: COLORS.accent, 'stroke-width': 2, 'stroke-linejoin': 'round' }));

  // Dots
  for (const p of points) {
    const dot = svgEl('circle', {
      cx: sx(p.x),
      cy: sy(p.y),
      r: 3.5,
      fill: p.color ?? COLORS.ink,
    });
    if (p.title) {
      const titleEl = svgEl('title', {});
      titleEl.textContent = p.title;
      dot.append(titleEl);
    }
    if (p.onClick) {
      dot.style.cursor = 'pointer';
      dot.addEventListener('click', p.onClick);
    }
    svg.append(dot);
  }

  if (opts.onRangeSelect) {
    let startX: number | null = null;
    const selectionRect = svgEl('rect', {
      x: 0,
      y: 0,
      width: 0,
      height,
      fill: 'var(--accent-soft)',
    });
    selectionRect.style.display = 'none';
    svg.append(selectionRect);

    // Convert client px -> viewBox units (the SVG scales responsively).
    const toViewX = (clientX: number): number => {
      const rect = svg.getBoundingClientRect();
      return ((clientX - rect.left) / rect.width) * width;
    };
    const toDataX = (viewX: number): number => {
      const frac = (viewX - leftPadding) / (width - leftPadding - padding);
      return xMin + Math.max(0, Math.min(1, frac)) * (xMax - xMin);
    };

    svg.addEventListener('pointerdown', (e) => {
      if (e.button !== 0 || (e.target as Element).tagName === 'circle') return;
      startX = toViewX(e.clientX);
      svg.setPointerCapture(e.pointerId);
      selectionRect.style.display = 'block';
      selectionRect.setAttribute('x', String(startX));
      selectionRect.setAttribute('width', '0');
    });
    svg.addEventListener('pointermove', (e) => {
      if (startX == null) return;
      const x1 = toViewX(e.clientX);
      selectionRect.setAttribute('x', String(Math.min(startX, x1)));
      selectionRect.setAttribute('width', String(Math.abs(x1 - startX)));
    });
    const end = (e: PointerEvent) => {
      if (startX == null) return;
      const endX = toViewX(e.clientX);
      selectionRect.style.display = 'none';
      const s0 = startX;
      startX = null;
      if (Math.abs(endX - s0) > 8) {
        opts.onRangeSelect?.(toDataX(Math.min(s0, endX)), toDataX(Math.max(s0, endX)));
      }
    };
    svg.addEventListener('pointerup', end);
    svg.addEventListener('pointercancel', () => {
      startX = null;
      selectionRect.style.display = 'none';
    });
    svg.style.touchAction = 'pan-y';
    svg.style.cursor = 'crosshair';
  }

  return svg;
}

export interface BarDatum {
  label: string;
  value: number;
  color?: string;
}

export function barChart(data: BarDatum[], opts: { width?: number; height?: number } = {}): SVGSVGElement {
  const width = opts.width ?? 480;
  const height = opts.height ?? 180;
  const padding = 28;
  const svg = svgEl('svg', { width, height, viewBox: `0 0 ${width} ${height}` });
  if (data.length === 0) return svg;
  const max = Math.max(...data.map((d) => d.value), 1);
  const barWidth = (width - 2 * padding) / data.length;

  data.forEach((d, i) => {
    const barHeight = (d.value / max) * (height - 2 * padding);
    const x = padding + i * barWidth;
    const y = height - padding - barHeight;
    svg.append(
      svgEl('rect', {
        x: x + barWidth * 0.1,
        y,
        width: barWidth * 0.8,
        height: barHeight,
        fill: d.color ?? COLORS.ink,
      }),
    );
    const label = svgEl('text', {
      x: x + barWidth / 2,
      y: height - padding + 14,
      'text-anchor': 'middle',
      'font-size': 10,
      fill: COLORS.muted,
    });
    label.textContent = d.label;
    svg.append(label);
  });

  return svg;
}

export interface StackedBarGroup {
  label: string;
  wdl: WdlLike;
}

export function stackedBarChart(groups: StackedBarGroup[], opts: { width?: number; height?: number } = {}): SVGSVGElement {
  const width = opts.width ?? 560;
  const height = opts.height ?? 200;
  const padding = 32;
  const svg = svgEl('svg', { width, height, viewBox: `0 0 ${width} ${height}` });
  if (groups.length === 0) return svg;
  const max = Math.max(...groups.map((g) => g.wdl.total), 1);
  const barWidth = (width - 2 * padding) / groups.length;

  groups.forEach((g, i) => {
    const x = padding + i * barWidth;
    const totalHeight = (g.wdl.total / max) * (height - 2 * padding);
    let y = height - padding;
    const segments: [number, string][] = [
      [g.wdl.won, COLORS.win],
      [g.wdl.drawn, COLORS.draw],
      [g.wdl.lost, COLORS.loss],
    ];
    for (const [count, color] of segments) {
      const segHeight = g.wdl.total ? (count / g.wdl.total) * totalHeight : 0;
      y -= segHeight;
      if (segHeight > 0) {
        svg.append(svgEl('rect', { x: x + barWidth * 0.15, y, width: barWidth * 0.7, height: segHeight, fill: color }));
      }
    }
    const label = svgEl('text', {
      x: x + barWidth / 2,
      y: height - padding + 14,
      'text-anchor': 'middle',
      'font-size': 10,
      fill: COLORS.muted,
    });
    label.textContent = g.label;
    svg.append(label);
  });

  return svg;
}

const FILES = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];

/** An 8x8 heatmap of square -> count, orientation from White's perspective (a1 bottom-left). */
export function boardHeatmap(counts: Record<string, number>, size = 280): SVGSVGElement {
  const cell = size / 8;
  const svg = svgEl('svg', { width: size, height: size, viewBox: `0 0 ${size} ${size}` });
  const max = Math.max(...Object.values(counts), 1);
  for (let rank = 8; rank >= 1; rank--) {
    for (let fileIdx = 0; fileIdx < 8; fileIdx++) {
      const square = `${FILES[fileIdx]}${rank}`;
      const count = counts[square] ?? 0;
      const x = fileIdx * cell;
      const y = (8 - rank) * cell;
      const isLight = (fileIdx + rank) % 2 === 1;
      const base = isLight ? '#CBDAD2' : '#54786F';
      svg.append(svgEl('rect', { x, y, width: cell, height: cell, fill: base }));
      if (count > 0) {
        const opacity = 0.15 + 0.7 * (count / max);
        svg.append(
          svgEl('rect', { x, y, width: cell, height: cell, fill: `rgba(181,70,47,${opacity.toFixed(2)})` }),
        );
        const label = svgEl('text', {
          x: x + cell / 2,
          y: y + cell / 2 + 4,
          'text-anchor': 'middle',
          'font-size': 11,
          fill: '#18222B',
        });
        label.textContent = String(count);
        svg.append(label);
      }
    }
  }
  return svg;
}
