export interface ChartSeries {
  name: string;
  values: number[];
}

export interface LineBarSpec {
  type: 'line' | 'bar';
  title?: string;
  unit?: string;
  labels: string[];
  series: ChartSeries[];
}

export interface PieChartPanel {
  label: string;
  slices: Array<{ name: string; value: number }>;
}

export interface PieSpec {
  type: 'pie';
  title?: string;
  charts: PieChartPanel[];
}

export interface TableSpec {
  type: 'table';
  title?: string;
  headers: string[];
  rows: Array<Array<string | number>>;
}

export interface ProcessSpec {
  type: 'process';
  title?: string;
  cycle?: boolean;
  steps: Array<{ title: string; detail?: string }>;
}

export interface MapZone {
  pos: 'NW' | 'N' | 'NE' | 'W' | 'C' | 'E' | 'SW' | 'S' | 'SE';
  name: string;
  kind?: string;
}

export interface MapSpec {
  type: 'map';
  title?: string;
  panels: Array<{ label: string; zones: MapZone[] }>;
}

export interface MixedSpec {
  type: 'mixed';
  title?: string;
  line: Omit<LineBarSpec, 'type'>;
  bar: Omit<LineBarSpec, 'type'>;
}

export type Task1ChartSpec =
  | LineBarSpec
  | PieSpec
  | TableSpec
  | ProcessSpec
  | MapSpec
  | MixedSpec;

const PALETTE = ['#2563eb', '#059669', '#d97706', '#dc2626', '#7c3aed', '#0891b2'];

function escHtml(s: unknown): string {
  return String(s ?? '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export function parseChartSpec(raw: string | undefined | null): Task1ChartSpec | null {
  if (!raw || !raw.trim()) return null;
  try {
    const obj = JSON.parse(raw);
    if (!obj || typeof obj !== 'object' || !obj.type) return null;
    return obj as Task1ChartSpec;
  } catch {
    return null;
  }
}

function safeNum(x: unknown): number {
  const n = Number(x);
  return Number.isFinite(n) ? n : 0;
}

function niceMax(values: number[]): number {
  const maxVal = Math.max(1, ...values.map(safeNum));
  if (maxVal <= 10) return Math.ceil(maxVal);
  if (maxVal <= 100) return Math.ceil(maxVal / 10) * 10;
  const pow = Math.pow(10, Math.floor(Math.log10(maxVal)));
  return Math.ceil(maxVal / (pow / 2)) * (pow / 2);
}

function renderLineSvg(spec: Omit<LineBarSpec, 'type'>, compact = false): string {
  const labels = spec.labels || [];
  const series = spec.series || [];
  if (!labels.length || !series.length) return '';

  const w = 520;
  const h = compact ? 155 : 210;
  const padL = 44;
  const padR = 18;
  const padT = 18;
  const padB = 34;
  const pw = w - padL - padR;
  const ph = h - padT - padB;
  const allVals = series.flatMap((s) => (s.values || []).map(safeNum));
  const yMax = niceMax(allVals);

  let grid = '';
  for (let i = 0; i <= 4; i++) {
    const y = padT + (ph * i) / 4;
    const val = Math.round(((4 - i) * yMax) / 4);
    grid += `<line x1="${padL}" y1="${y.toFixed(1)}" x2="${w - padR}" y2="${y.toFixed(1)}" stroke="currentColor" stroke-opacity="0.14" stroke-dasharray="3,3" />`;
    grid += `<text x="${padL - 6}" y="${(y + 3).toFixed(1)}" text-anchor="end" font-size="10" fill="currentColor" opacity="0.65">${escHtml(String(val))}</text>`;
  }

  const stepX = labels.length > 1 ? pw / (labels.length - 1) : 0;
  let xLabels = '';
  labels.forEach((lb, i) => {
    const x = labels.length > 1 ? padL + i * stepX : padL + pw / 2;
    xLabels += `<text x="${x.toFixed(1)}" y="${h - 10}" text-anchor="middle" font-size="10.5" fill="currentColor" opacity="0.8">${escHtml(lb)}</text>`;
  });

  let paths = '';
  series.forEach((s, si) => {
    const color = PALETTE[si % PALETTE.length];
    const pts = labels.map((_, i) => {
      const x = labels.length > 1 ? padL + i * stepX : padL + pw / 2;
      const v = safeNum(s.values?.[i]);
      const y = padT + ph - (v / yMax) * ph;
      return { x, y, v };
    });
    const d = pts.map((p, idx) => `${idx === 0 ? 'M' : 'L'}${p.x.toFixed(1)},${p.y.toFixed(1)}`).join(' ');
    paths += `<path d="${d}" fill="none" stroke="${color}" stroke-width="2.4" stroke-linejoin="round" />`;
    pts.forEach((p) => {
      paths += `<circle cx="${p.x.toFixed(1)}" cy="${p.y.toFixed(1)}" r="3.4" fill="${color}" stroke="#fff" stroke-width="1" />`;
      paths += `<text x="${p.x.toFixed(1)}" y="${(p.y - 6).toFixed(1)}" text-anchor="middle" font-size="9.5" font-weight="700" fill="var(--m-fg)">${escHtml(String(p.v))}</text>`;
    });
  });

  const unitTag = spec.unit
    ? `<text x="${padL}" y="11" font-size="9.5" fill="currentColor" opacity="0.7">${escHtml(spec.unit)}</text>`
    : '';

  const legend = `<div class="chart-legend">${series
    .map(
      (s, si) =>
        `<span class="chart-legend-item"><i style="background:${PALETTE[si % PALETTE.length]}"></i>${escHtml(s.name)}</span>`
    )
    .join('')}</div>`;

  return `
    ${spec.title ? `<div class="chart-sub-title">${escHtml(spec.title)}</div>` : ''}
    <svg class="task1-svg" viewBox="0 0 ${w} ${h}" role="img" aria-label="${escHtml(spec.title || 'Line graph')}">
      ${unitTag}
      ${grid}
      <line x1="${padL}" y1="${padT}" x2="${padL}" y2="${padT + ph}" stroke="currentColor" stroke-opacity="0.35" />
      <line x1="${padL}" y1="${padT + ph}" x2="${w - padR}" y2="${padT + ph}" stroke="currentColor" stroke-opacity="0.35" />
      ${xLabels}
      ${paths}
    </svg>
    ${legend}
  `;
}

function renderBarSvg(spec: Omit<LineBarSpec, 'type'>, compact = false): string {
  const labels = spec.labels || [];
  const series = spec.series || [];
  if (!labels.length || !series.length) return '';

  const w = 520;
  const h = compact ? 155 : 210;
  const padL = 44;
  const padR = 14;
  const padT = 18;
  const padB = 34;
  const pw = w - padL - padR;
  const ph = h - padT - padB;
  const allVals = series.flatMap((s) => (s.values || []).map(safeNum));
  const yMax = niceMax(allVals);

  let grid = '';
  for (let i = 0; i <= 4; i++) {
    const y = padT + (ph * i) / 4;
    const val = Math.round(((4 - i) * yMax) / 4);
    grid += `<line x1="${padL}" y1="${y.toFixed(1)}" x2="${w - padR}" y2="${y.toFixed(1)}" stroke="currentColor" stroke-opacity="0.14" stroke-dasharray="3,3" />`;
    grid += `<text x="${padL - 6}" y="${(y + 3).toFixed(1)}" text-anchor="end" font-size="10" fill="currentColor" opacity="0.65">${escHtml(String(val))}</text>`;
  }

  const groupW = pw / labels.length;
  const barGroupW = Math.min(groupW * 0.72, series.length * 26);
  const barW = Math.max(8, barGroupW / series.length - 3);
  let bars = '';

  labels.forEach((lb, gi) => {
    const gx = padL + gi * groupW + (groupW - barGroupW) / 2;
    series.forEach((s, si) => {
      const val = safeNum(s.values?.[gi]);
      const bh = (val / yMax) * ph;
      const bx = gx + si * (barW + 3);
      const by = padT + ph - bh;
      const color = PALETTE[si % PALETTE.length];
      bars += `<rect x="${bx.toFixed(1)}" y="${by.toFixed(1)}" width="${barW.toFixed(1)}" height="${bh.toFixed(1)}" rx="2" fill="${color}" />`;
      bars += `<text x="${(bx + barW / 2).toFixed(1)}" y="${(by - 4).toFixed(1)}" text-anchor="middle" font-size="9" font-weight="700" fill="currentColor">${escHtml(String(val))}</text>`;
    });
    bars += `<text x="${(padL + (gi + 0.5) * groupW).toFixed(1)}" y="${h - 10}" text-anchor="middle" font-size="10.5" fill="currentColor" opacity="0.82">${escHtml(lb)}</text>`;
  });

  const unitTag = spec.unit
    ? `<text x="${padL}" y="11" font-size="9.5" fill="currentColor" opacity="0.7">${escHtml(spec.unit)}</text>`
    : '';

  const legend = `<div class="chart-legend">${series
    .map(
      (s, si) =>
        `<span class="chart-legend-item"><i style="background:${PALETTE[si % PALETTE.length]}"></i>${escHtml(s.name)}</span>`
    )
    .join('')}</div>`;

  return `
    ${spec.title ? `<div class="chart-sub-title">${escHtml(spec.title)}</div>` : ''}
    <svg class="task1-svg" viewBox="0 0 ${w} ${h}" role="img" aria-label="${escHtml(spec.title || 'Bar chart')}">
      ${unitTag}
      ${grid}
      <line x1="${padL}" y1="${padT}" x2="${padL}" y2="${padT + ph}" stroke="currentColor" stroke-opacity="0.35" />
      <line x1="${padL}" y1="${padT + ph}" x2="${w - padR}" y2="${padT + ph}" stroke="currentColor" stroke-opacity="0.35" />
      ${bars}
    </svg>
    ${legend}
  `;
}

function renderPieSvg(spec: PieSpec): string {
  const panels = spec.charts || [];
  if (!panels.length) return '';

  const categoryNames = Array.from(
    new Set(panels.flatMap((p) => p.slices.map((s) => s.name)))
  );

  const piesHtml = panels
    .map((panel) => {
      const total = panel.slices.reduce((a, b) => a + safeNum(b.value), 0) || 100;
      const cx = 75;
      const cy = 75;
      const r = 62;
      let startAngle = -Math.PI / 2;
      let arcs = '';

      panel.slices.forEach((slice) => {
        const idx = categoryNames.indexOf(slice.name);
        const color = PALETTE[(idx >= 0 ? idx : 0) % PALETTE.length];
        const sv = safeNum(slice.value);
        const frac = sv / total;
        const endAngle = startAngle + frac * Math.PI * 2;
        const x1 = cx + r * Math.cos(startAngle);
        const y1 = cy + r * Math.sin(startAngle);
        const x2 = cx + r * Math.cos(endAngle);
        const y2 = cy + r * Math.sin(endAngle);
        const largeArc = frac > 0.5 ? 1 : 0;
        arcs += `<path d="M${cx},${cy} L${x1.toFixed(1)},${y1.toFixed(1)} A${r},${r} 0 ${largeArc},1 ${x2.toFixed(1)},${y2.toFixed(1)} Z" fill="${color}" stroke="var(--m-bg)" stroke-width="1.5" />`;

        if (frac >= 0.06) {
          const mid = (startAngle + endAngle) / 2;
          const lx = cx + r * 0.65 * Math.cos(mid);
          const ly = cy + r * 0.65 * Math.sin(mid) + 3;
          arcs += `<text x="${lx.toFixed(1)}" y="${ly.toFixed(1)}" text-anchor="middle" font-size="10" font-weight="700" fill="#fff" style="paint-order:stroke;stroke:rgba(0,0,0,0.58);stroke-width:2.5px;stroke-linejoin:round;">${escHtml(String(sv))}%</text>`;
        }
        startAngle = endAngle;
      });

      return `
        <div class="pie-panel">
          <div class="pie-panel-label">${escHtml(panel.label)}</div>
          <svg width="112" height="112" viewBox="0 0 150 150" role="img" aria-label="${escHtml(panel.label)}">
            ${arcs}
            <circle cx="${cx}" cy="${cy}" r="22" fill="var(--m-bg)" />
          </svg>
        </div>
      `;
    })
    .join('');

  const legend = `<div class="chart-legend">${categoryNames
    .map(
      (name, idx) =>
        `<span class="chart-legend-item"><i style="background:${PALETTE[idx % PALETTE.length]}"></i>${escHtml(name)}</span>`
    )
    .join('')}</div>`;

  return `
    ${spec.title ? `<div class="chart-sub-title">${escHtml(spec.title)}</div>` : ''}
    <div class="pie-panels-row">${piesHtml}</div>
    ${legend}
  `;
}

function renderTableHtml(spec: TableSpec): string {
  return `
    ${spec.title ? `<div class="chart-sub-title">${escHtml(spec.title)}</div>` : ''}
    <div class="chart-table-wrap">
      <table class="chart-table">
        <thead><tr>${(spec.headers || []).map((h) => `<th>${escHtml(h)}</th>`).join('')}</tr></thead>
        <tbody>
          ${(spec.rows || [])
            .map((row) => `<tr>${row.map((c) => `<td>${escHtml(c)}</td>`).join('')}</tr>`)
            .join('')}
        </tbody>
      </table>
    </div>
  `;
}

function renderProcessHtml(spec: ProcessSpec): string {
  const steps = spec.steps || [];
  return `
    ${spec.title ? `<div class="chart-sub-title">${escHtml(spec.title)}</div>` : ''}
    <div class="process-flow">
      ${steps
        .map(
          (st, idx) => `
          <div class="process-node">
            <span class="process-step-num">${idx + 1}</span>
            <div class="process-step-title">${escHtml(st.title)}</div>
            ${st.detail ? `<div class="process-step-detail">${escHtml(st.detail)}</div>` : ''}
          </div>
          ${idx < steps.length - 1 ? '<span class="process-arrow">→</span>' : spec.cycle ? '<span class="process-arrow" title="Chu trình lặp">↺</span>' : ''}
        `
        )
        .join('')}
    </div>
  `;
}

function renderMapHtml(spec: MapSpec): string {
  const posCoords: Record<MapZone['pos'], [number, number]> = {
    NW: [52, 42],
    N: [130, 32],
    NE: [208, 42],
    W: [48, 86],
    C: [130, 86],
    E: [212, 86],
    SW: [52, 130],
    S: [130, 142],
    SE: [208, 130],
  };

  const panelsHtml = (spec.panels || [])
    .map((panel) => {
      const items = (panel.zones || [])
        .map((z) => {
          const [x, y] = posCoords[z.pos] || [130, 86];
          const fill =
            z.kind === 'green'
              ? 'rgba(16,185,129,0.22)'
              : z.kind === 'water'
              ? 'rgba(37,99,235,0.2)'
              : z.kind === 'transit'
              ? 'rgba(217,119,6,0.24)'
              : 'rgba(148,163,184,0.22)';
          return `
            <rect x="${x - 38}" y="${y - 16}" width="76" height="32" rx="6" fill="${fill}" stroke="currentColor" stroke-opacity="0.45" />
            <text x="${x}" y="${y + 3}" text-anchor="middle" font-size="8.8" font-weight="600" fill="currentColor">${escHtml(z.name)}</text>
          `;
        })
        .join('');

      return `
        <div class="map-panel">
          <div class="pie-panel-label">${escHtml(panel.label)}</div>
          <svg class="map-svg" viewBox="0 0 260 168" role="img" aria-label="${escHtml(panel.label)}">
            <rect x="4" y="4" width="252" height="160" rx="8" fill="none" stroke="currentColor" stroke-opacity="0.25" />
            <text x="130" y="14" text-anchor="middle" font-size="8.5" font-weight="700" fill="currentColor" opacity="0.6">N</text>
            <text x="130" y="160" text-anchor="middle" font-size="8.5" font-weight="700" fill="currentColor" opacity="0.6">S</text>
            <text x="11" y="88" text-anchor="middle" font-size="8.5" font-weight="700" fill="currentColor" opacity="0.6">W</text>
            <text x="249" y="88" text-anchor="middle" font-size="8.5" font-weight="700" fill="currentColor" opacity="0.6">E</text>
            ${items}
          </svg>
        </div>
      `;
    })
    .join('');

  return `
    ${spec.title ? `<div class="chart-sub-title">${escHtml(spec.title)}</div>` : ''}
    <div class="map-panels-row">${panelsHtml}</div>
  `;
}

export function renderChartInnerHtml(spec: Task1ChartSpec): string {
  switch (spec.type) {
    case 'line':
      return renderLineSvg(spec);
    case 'bar':
      return renderBarSvg(spec);
    case 'pie':
      return renderPieSvg(spec);
    case 'table':
      return renderTableHtml(spec);
    case 'process':
      return renderProcessHtml(spec);
    case 'map':
      return renderMapHtml(spec);
    case 'mixed':
      return `
        ${spec.title ? `<div class="chart-sub-title">${escHtml(spec.title)}</div>` : ''}
        <div class="mixed-charts-grid">
          <div>${renderLineSvg(spec.line, true)}</div>
          <div>${renderBarSvg(spec.bar, true)}</div>
        </div>
      `;
  }
}

export function renderTask1VisualHtml(
  chartSpecJson: string | undefined | null,
  customImageDataUrl?: string | null
): string {
  if (customImageDataUrl) {
    return `
      <div class="task1-visual-box">
        <div class="task1-visual-bar">
          <span>🖼 Hình/biểu đồ đề ngoài đã nạp</span>
          <div style="display:flex;gap:6px;">
            <button type="button" class="btn btn-sm" id="zoomChartBtn">🔍 Phóng to biểu đồ</button>
            <button type="button" class="btn btn-sm" id="clearCustomTask1ImgBtn">✕ Xóa ảnh ngoài</button>
          </div>
        </div>
        <img src="${escHtml(customImageDataUrl)}" alt="Custom Task 1 Chart" class="task1-custom-img" />
      </div>
    `;
  }

  const spec = parseChartSpec(chartSpecJson);
  if (!spec) {
    return `
      <div class="task1-visual-box" style="padding:8px 10px;">
        <div class="task1-visual-bar" style="margin-bottom:0;">
          <span class="muted">Nếu bạn luyện đề Task 1 bên ngoài, có thể nạp ảnh biểu đồ vào đây:</span>
          <button type="button" class="btn btn-sm" id="uploadTask1ImgBtn">🖼 Tải ảnh đề ngoài</button>
        </div>
      </div>
    `;
  }

  return `
    <div class="task1-visual-box">
      <div class="task1-visual-bar">
        <span>📊 Số liệu / Sơ đồ đề bài Task 1</span>
        <div style="display:flex;gap:6px;">
          <button type="button" class="btn btn-sm" id="zoomChartBtn">🔍 Phóng to biểu đồ</button>
          <button type="button" class="btn btn-sm" id="uploadTask1ImgBtn">🖼 Tải ảnh đề ngoài</button>
        </div>
      </div>
      <div class="task1-visual-body">${renderChartInnerHtml(spec)}</div>
    </div>
  `;
}

export function summarizeChartSpecForPrompt(chartSpecJson: string | undefined | null): string {
  const spec = parseChartSpec(chartSpecJson);
  if (!spec) return '';
  switch (spec.type) {
    case 'line':
    case 'bar': {
      const unit = spec.unit ? ` (${spec.unit})` : '';
      const lines = spec.series.map(
        (s) => `${s.name}: ${spec.labels.map((l, i) => `${l}=${s.values[i] ?? ''}`).join(', ')}`
      );
      return `[CHART DATA - ${spec.type.toUpperCase()}${unit}]: ${spec.title || ''}\n${lines.join('\n')}`;
    }
    case 'pie': {
      const parts = spec.charts.map(
        (c) => `${c.label}: ${c.slices.map((s) => `${s.name} ${s.value}%`).join(', ')}`
      );
      return `[CHART DATA - PIE]: ${spec.title || ''}\n${parts.join('\n')}`;
    }
    case 'table': {
      const header = spec.headers.join(' | ');
      const rows = spec.rows.map((r) => r.join(' | ')).join('\n');
      return `[CHART DATA - TABLE]: ${spec.title || ''}\n${header}\n${rows}`;
    }
    case 'process': {
      const steps = spec.steps
        .map((s, i) => `${i + 1}. ${s.title}${s.detail ? ` (${s.detail})` : ''}`)
        .join(' -> ');
      return `[CHART DATA - PROCESS]: ${spec.title || ''}\n${steps}`;
    }
    case 'map': {
      const panels = spec.panels
        .map((p) => `${p.label}: ${p.zones.map((z) => `[${z.pos}] ${z.name}`).join('; ')}`)
        .join('\n');
      return `[CHART DATA - MAP]: ${spec.title || ''}\n${panels}`;
    }
    case 'mixed': {
      const l = spec.line.series
        .map((s) => `${s.name}: ${spec.line.labels.map((lb, i) => `${lb}=${s.values[i]}`).join(', ')}`)
        .join('; ');
      const b = spec.bar.series
        .map((s) => `${s.name}: ${spec.bar.labels.map((lb, i) => `${lb}=${s.values[i]}`).join(', ')}`)
        .join('; ');
      return `[CHART DATA - MIXED LINE+BAR]: ${spec.title || ''}\nLine (${spec.line.unit || ''}): ${l}\nBar (${spec.bar.unit || ''}): ${b}`;
    }
  }
}
