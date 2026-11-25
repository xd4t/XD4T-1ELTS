import './styles.css';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { openUrl } from '@tauri-apps/plugin-opener';
import donateQrUrl from './assets/donate-qr.png';
import type {
  ApiTestResult,
  BackupResultDto,
  DashboardState,
  GradeResponseDto,
  GradeResult,
  QuizAnswerInput,
  QuizQuestion,
  QuizSubmitResult,
  RestoreResultDto,
  RoadmapPreview,
  VocabCard,
  WidgetRuntimeInfo,
  WritingPrompt,
} from './types';

type ThemeId = 'pastel-male' | 'pastel-female' | 'mono-dark' | 'mono-light';

const THEME_OPTIONS: { id: ThemeId; label: string; sub: string; swatches: string[] }[] = [
  {
    id: 'pastel-male',
    label: 'Pastel Nam · Sky & Mint',
    sub: 'Dịu mắt, hiện đại (mặc định)',
    swatches: ['#2F6E89', '#52A68C', '#F2F7F6'],
  },
  {
    id: 'pastel-female',
    label: 'Pastel Nữ · Sakura & Peach',
    sub: 'Ấm áp, tinh tế, bo góc mềm',
    swatches: ['#B54E61', '#E59876', '#FDF6F5'],
  },
  {
    id: 'mono-dark',
    label: 'XD4T Mono · Đen',
    sub: 'Góc vuông 0px, tương phản cao',
    swatches: ['#0B0B0B', '#F4F4F4', '#2E2E2E'],
  },
  {
    id: 'mono-light',
    label: 'XD4T Mono · Trắng',
    sub: 'Góc vuông 0px, sáng sắc nét',
    swatches: ['#FFFFFF', '#111111', '#D4D4D4'],
  },
];

function normalizeTheme(raw: string): ThemeId {
  if (raw === 'dark' || raw === 'mono-dark') return 'mono-dark';
  if (raw === 'light' || raw === 'mono-light') return 'mono-light';
  if (raw === 'pastel-female') return 'pastel-female';
  return 'pastel-male';
}

function renderThemeCards(currentTheme: ThemeId, btnClass: string): string {
  return `<div class="theme-cards-grid">
    ${THEME_OPTIONS.map(
      (t) => `
      <button type="button" class="theme-card ${btnClass} ${currentTheme === t.id ? 'active' : ''}" data-pick-theme="${t.id}">
        <div class="theme-card-head">
          ${t.swatches.map((c) => `<span class="swatch-dot" style="background:${c}"></span>`).join('')}
          <span>${esc(t.label)}</span>
        </div>
        <div class="theme-card-sub">${esc(t.sub)}</div>
      </button>`
    ).join('')}
  </div>`;
}

let dash: DashboardState | null = null;
let widgetInfo: WidgetRuntimeInfo = { running: false, autostart: false, exe_found: true };
let activeTab: 'today' | 'writing' | 'vocab' | 'roadmap' = 'today';
let writingMode = 'paragraph_90';
let selectedPromptId = 'p90-1';
let hintOpen = false;
let lastGrade: GradeResult | null = null;
let lastEndpoint = '';
let isGrading = false;
let writingStatusMsg = '';
let writingStatusErr = false;
let errFormMsg = '';
let errFormErr = false;
let scrollToNewErr = false;
let dueCards: VocabCard[] = [];
let cardIdx = 0;
let cardFlipped = false;
let vocabSubMode: 'flashcard' | 'quiz' = 'flashcard';
let quizMode: 'mixed' | 'mcq' | 'typing' = 'mixed';
let quizPhase: 'setup' | 'playing' | 'summary' = 'setup';
let quizQuestions: QuizQuestion[] = [];
let quizIdx = 0;
let quizAnswers: QuizAnswerInput[] = [];
let quizFeedback: { correct: boolean; gaveUp: boolean; selectedOpt?: string; typedText?: string } | null = null;
let quizSummary: QuizSubmitResult | null = null;
let quizLoading = false;
let quizErrorMsg = '';
let lastSyncedWindowTitle = '';
let draftEssay = '';

let showApiKey = false;
let apiTestMsg = '';
let apiTestOk: boolean | null = null;
let isTestingApi = false;

let roadmapDraftMd = '';
let roadmapPreview: RoadmapPreview | null = null;
let roadmapStatusMsg = '';
let roadmapStatusOk: boolean | null = null;

let welcomeOpen = false;
let welcomeStep = 1;
let welcomeMsg = '';
let welcomeOk: boolean | null = null;
let welcomeKeyDraft = '';

let backupMsg = '';
let backupOk: boolean | null = null;
let widgetStatusMsg = '';

let pomoSecs = 15 * 60;
let pomoTimer: number | null = null;

function extractApiKeyFromText(raw: string): string {
  const lines = raw.split(/\r?\n/);
  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const eqIdx = trimmed.indexOf('=');
    const candidate = eqIdx >= 0 ? trimmed.slice(eqIdx + 1).trim().replace(/^["']|["']$/g, '') : trimmed;
    if (candidate.startsWith('AIza') || candidate.startsWith('AQ.')) {
      return candidate;
    }
  }
  const match = raw.match(/(AIza[0-9A-Za-z_-]{20,}|AQ\.[0-9A-Za-z._-]{20,})/);
  return match ? match[1] : raw.trim();
}

function esc(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function getCurrentPrompt(): WritingPrompt | undefined {
  if (!dash) return undefined;
  return (
    dash.prompts.find((p) => p.id === selectedPromptId) ||
    dash.prompts.find((p) => p.mode === writingMode) ||
    dash.prompts[0]
  );
}

function renderLevelCards(is80: boolean, extraClass: string): string {
  return `
    <div class="level-cards">
      <button type="button" class="level-card ${extraClass} ${!is80 ? 'active' : ''}" data-pick-level="6.5">
        <div class="level-card-title">🎯 Lộ trình Tiêu chuẩn · Aim 6.5 (Vùng đệm 7.0)</div>
        <div class="level-card-desc">Dành cho nền 4.5–5.5 · Diệt lỗi gốc, nắm 5 khung Writing &amp; săn 30–34/40 câu (Khuyên dùng cho đa số người tự học)</div>
      </button>
      <button type="button" class="level-card ${extraClass} ${is80 ? 'active' : ''}" data-pick-level="8.0">
        <div class="level-card-title">🚀 Lộ trình Nâng cao · Aim 8.0 (Độ khó cao)</div>
        <div class="level-card-desc">Khuyến khích khi nền đã vững 6.5+ · Từ vựng C1–C2 &amp; Hedging, phản biện sâu, săn 36–38/40 câu</div>
      </button>
    </div>
    ${
      is80
        ? `<div class="level-warning-box">⚠️ Khuyến khích dành cho bạn đã vững nền Band 6.5+ (Listening/Reading đã đạt ≥ 27–30/40 câu, nắm chắc ngữ pháp câu phức). Nếu đang ở mức 5.0–6.0, bạn nên chọn Lộ trình Tiêu chuẩn 6.5 trước để xây gốc vững chắc và không bị ngợp!</div>`
        : ''
    }
  `;
}

function isAim80Mode(): boolean {
  return (dash?.settings.target_band ?? 7.0) >= 7.5;
}

function normalizeQuizWord(s: string): string {
  return s
    .toLowerCase()
    .trim()
    .replace(/[.,!?;:]+$/g, '')
    .replace(/\s+/g, ' ');
}

function isTypingAnswerMatch(input: string, target: string): boolean {
  const normIn = normalizeQuizWord(input);
  if (!normIn) return false;
  const normTarget = normalizeQuizWord(target);
  if (normIn === normTarget) return true;
  const primary = normalizeQuizWord(target.split('(')[0].split('/')[0]);
  return Boolean(primary) && normIn === primary;
}

function renderClozeHtml(sentence: string): string {
  return esc(sentence).replace('________', '<span class="quiz-blank">________</span>');
}

function renderQuizBox(): string {
  if (!dash) return '';
  const qs = dash.quiz_status;

  if (quizPhase === 'setup') {
    const lastText = qs.last_quiz_at
      ? `${esc(qs.last_quiz_at)}${qs.last_quiz_score ? ` (${esc(qs.last_quiz_score)})` : ''}`
      : 'Chưa làm bài nào';
    return `
      <div class="box">
        <div class="box-title">
          <span>QUIZ KIỂM TRA TRÍ NHỚ TỪ VỰNG (100% OFFLINE · TỰ ĐẨY TỪ QUÊN VỀ SRS)</span>
          <span>${qs.quiz_due ? 'ĐẾN LỊCH KIỂM TRA' : 'SẴN SÀNG'}</span>
        </div>
        <div class="quiz-setup-stats">
          <span>📅 Lần kiểm tra gần nhất: <b>${lastText}</b></span>
          <span>📊 Tổng số bài Quiz đã làm: <b class="mono-num">${qs.total_quizzes}</b></span>
        </div>
        <div class="quiz-mode-grid">
          <button type="button" class="quiz-mode-card ${quizMode === 'mixed' ? 'active' : ''}" data-qmode="mixed">
            <div class="quiz-mode-title">⚡ Hỗn hợp (Khuyên dùng)</div>
            <div class="quiz-mode-sub">7 câu trắc nghiệm + 3 câu gõ từ cuối bài</div>
          </button>
          <button type="button" class="quiz-mode-card ${quizMode === 'mcq' ? 'active' : ''}" data-qmode="mcq">
            <div class="quiz-mode-title">🔘 Trắc nghiệm 4 đáp án</div>
            <div class="quiz-mode-sub">Xen kẽ chọn nghĩa &amp; điền ngữ cảnh</div>
          </button>
          <button type="button" class="quiz-mode-card ${quizMode === 'typing' ? 'active' : ''}" data-qmode="typing">
            <div class="quiz-mode-title">⌨️ Gõ từ Active Recall</div>
            <div class="quiz-mode-sub">Nhìn nghĩa &amp; gợi ý chữ cái đầu để gõ lại</div>
          </button>
        </div>
        ${quizErrorMsg ? `<div class="status-msg err" style="margin-bottom:10px;">${esc(quizErrorMsg)}</div>` : ''}
        <div style="display:flex;align-items:center;gap:10px;flex-wrap:wrap;">
          <button class="btn btn-primary" id="startQuizBtn" ${quizLoading ? 'disabled' : ''}>
            ${quizLoading ? 'ĐANG TẠO BÀI QUIZ...' : 'BẮT ĐẦU QUIZ 10 CÂU →'}
          </button>
          <span style="font-size:11.5px;color:var(--m-muted);">Từ nào bạn chọn sai hoặc bấm <b>Quên từ này</b> sẽ tự động reset về đầu hàng đợi SRS hôm nay.</span>
        </div>
      </div>`;
  }

  if (quizPhase === 'playing') {
    const q = quizQuestions[quizIdx];
    if (!q) return '';
    const total = quizQuestions.length || 1;
    const pct = Math.round(((quizIdx + (quizFeedback ? 1 : 0)) / total) * 100);
    const typeLabel =
      q.question_type === 'cloze_mcq'
        ? 'ĐIỀN NGỮ CẢNH'
        : q.question_type === 'typing'
          ? 'GÕ TỪ ACTIVE RECALL'
          : 'TRẮC NGHIỆM NGHĨA';

    return `
      <div class="box">
        <div class="quiz-stage">
          <div class="quiz-head">
            <span>CÂU ${quizIdx + 1} / ${total} · ${esc(q.topic.toUpperCase())}</span>
            <span class="quiz-type-badge">${typeLabel}</span>
          </div>
          <div class="bar-track"><div class="bar-fill" style="width:${pct}%"></div></div>
          <div class="quiz-prompt">${esc(q.prompt_text)}</div>
          ${q.cloze_sentence ? `<div class="quiz-cloze">"${renderClozeHtml(q.cloze_sentence)}"</div>` : ''}

          ${
            q.question_type === 'typing'
              ? `
                <div style="font-size:12px;color:var(--m-muted);margin-bottom:4px;">Gợi ý: <b class="mono-num">${esc(q.hint)}</b></div>
                <div class="quiz-typing-row">
                  <input type="text" class="quiz-typing-input" id="quizTypingInput"
                    placeholder="Nhập từ/cụm từ tiếng Anh..."
                    value="${esc(quizFeedback?.typedText || '')}"
                    ${quizFeedback ? 'disabled' : ''} />
                  ${
                    !quizFeedback
                      ? `<button class="btn btn-primary" id="quizCheckTypingBtn" type="button">KIỂM TRA (Enter)</button>
                         <button class="btn" id="quizGiveUpBtn" type="button">❓ Quên từ này</button>`
                      : ''
                  }
                </div>`
              : `
                <div class="quiz-options-grid">
                  ${q.options
                    .map((opt, i) => {
                      let cls = 'quiz-opt-btn';
                      if (quizFeedback) {
                        if (opt === q.word) cls += ' correct';
                        else if (quizFeedback.selectedOpt === opt) cls += ' wrong';
                      }
                      return `<button type="button" class="${cls}" data-quiz-opt="${i}" ${quizFeedback ? 'disabled' : ''}>${String.fromCharCode(65 + i)}. ${esc(opt)}</button>`;
                    })
                    .join('')}
                </div>
                ${
                  !quizFeedback
                    ? `<div style="margin-top:10px;">
                         <button class="btn btn-sm" id="quizGiveUpBtn" type="button">❓ Quên từ này (đẩy về SRS)</button>
                       </div>`
                    : ''
                }`
          }

          ${
            quizFeedback
              ? `
                <div class="quiz-feedback ${quizFeedback.correct ? 'ok' : 'err'}">
                  <div style="font-weight:700;margin-bottom:4px;">
                    ${quizFeedback.correct ? '✓ Chính xác!' : quizFeedback.gaveUp ? '↺ Đã ghi nhận quên từ này' : '✕ Chưa chính xác'}
                    — Đáp án: <b>${esc(q.word)}</b>
                  </div>
                  <div>Nghĩa: <b>${esc(q.meaning)}</b></div>
                  <div style="color:var(--m-muted);margin-top:2px;">Ví dụ: "${esc(q.example)}"</div>
                  ${!quizFeedback.correct ? `<div class="quiz-srs-badge">↺ Đã đánh dấu học lại trong SRS</div>` : ''}
                  <div style="margin-top:10px;">
                    <button class="btn btn-primary" id="quizNextBtn" type="button">
                      ${quizIdx + 1 >= total ? 'XEM KẾT QUẢ →' : 'CÂU TIẾP THEO →'}
                    </button>
                  </div>
                </div>`
              : ''
          }
        </div>
      </div>`;
  }

  if (quizPhase === 'summary' && quizSummary) {
    const forgotten = quizSummary.forgotten_cards;
    return `
      <div class="box">
        <div class="quiz-summary-hero">
          <div class="quiz-summary-score mono-num">${quizSummary.correct_count} / ${quizSummary.total_questions} · ${quizSummary.accuracy_pct}%</div>
          <div style="font-size:12px;color:var(--m-muted);">
            ✓ Đã lưu lịch sử Quiz${quizSummary.marked_quest_done ? ' · ✓ Đã tự động tick xong nhiệm vụ Quiz hôm nay!' : ''}
          </div>
        </div>
        ${
          forgotten.length > 0
            ? `
              <div style="font-size:12.5px;font-weight:700;margin-bottom:4px;">
                Các từ bạn vừa quên (${forgotten.length}) — Đã tự động đẩy về hàng đợi SRS học lại hôm nay:
              </div>
              <div class="quiz-forgotten-list">
                ${forgotten
                  .map(
                    (c) => `
                  <div class="quiz-forgotten-item">
                    <div><b>${esc(c.word)}</b> — ${esc(c.meaning)}</div>
                    <div style="font-size:11px;color:var(--m-muted);">"${esc(c.example)}"</div>
                  </div>`
                  )
                  .join('')}
              </div>
              <div style="display:flex;gap:8px;flex-wrap:wrap;">
                <button class="btn btn-primary" id="quizReviewForgottenBtn" type="button">↺ ÔN LẠI NGAY ${forgotten.length} TỪ VỪA QUÊN →</button>
                <button class="btn" id="quizRestartBtn" type="button">LÀM BÀI QUIZ MỚI</button>
              </div>`
            : `
              <div style="text-align:center;padding:8px 0 12px;font-size:12.5px;color:var(--m-ok);font-weight:700;">
                🎉 Xuất sắc! Bạn nhớ trọn vẹn tất cả từ vựng trong bài kiểm tra này.
              </div>
              <div style="text-align:center;">
                <button class="btn btn-primary" id="quizRestartBtn" type="button">LÀM BÀI QUIZ MỚI</button>
              </div>`
        }
      </div>`;
  }

  return '';
}

function renderApp() {
  if (!dash) return;
  const curTheme = normalizeTheme(dash.settings.theme);
  const isAim80 = dash.settings.target_band >= 7.5;
  const winTitle = isAim80 ? 'XD4T // IELTS 8.0' : 'XD4T // IELTS 6.5';
  if (lastSyncedWindowTitle !== winTitle) {
    lastSyncedWindowTitle = winTitle;
    document.title = winTitle;
    try {
      void getCurrentWindow().setTitle(winTitle);
    } catch {
      // Ignore outside Tauri window context
    }
  }
  document.documentElement.className = `theme-${curTheme}`;
  const doneQuests = dash.quests.filter((q) => q.done).length;
  const totalQuests = dash.quests.length || 1;
  const questPct = Math.round((doneQuests / totalQuests) * 100);

  const curPrompt = getCurrentPrompt();
  const modePrompts = dash.prompts.filter((p) => p.mode === writingMode);

  const phaseNumbers = Array.from(new Set(dash.roadmap.map((r) => r.phase))).sort((a, b) => a - b);
  const pct = (arr: typeof dash extends null ? never : NonNullable<typeof dash>['roadmap']) =>
    arr.length ? Math.round((arr.filter((x) => x.done).length / arr.length) * 100) : 0;

  const curCard = dueCards[cardIdx];

  const root = document.getElementById('app')!;
  root.innerHTML = `
  <div class="app-shell theme-${curTheme}" id="shell">
    <div class="titlebar">
      <div class="tb-left">
        <div class="brand-pill">${isAim80 ? 'XD4T // IELTS 8.0' : 'XD4T // IELTS 6.5'}</div>
        <div class="stat-pill"><span>THI ${esc(dash.settings.exam_date)}:</span> <b class="mono-num">${dash.days_until_exam}d</b></div>
        <div class="stat-pill"><span>STREAK:</span> <b class="mono-num">${dash.streak} ngày</b></div>
        <div class="stat-pill"><span>API:</span> <b>${dash.settings.has_api_key ? 'SẴN SÀNG' : 'CHƯA CÓ KEY'}</b></div>
      </div>
      <div class="tb-right">
        <button class="btn btn-sm mono-num" id="pomoBtn" title="Bật/tạm dừng đếm ngược 15 phút">⏱ ${formatPomo(pomoSecs)}</button>
        <select class="theme-select" id="themeSelect" title="Đổi giao diện tức thì">
          ${THEME_OPTIONS.map(
            (t) => `<option value="${t.id}" ${curTheme === t.id ? 'selected' : ''}>🎨 ${esc(t.label)}</option>`
          ).join('')}
        </select>
        <button class="btn btn-sm" id="openSettingsBtn">CÀI ĐẶT</button>
      </div>
    </div>

    <div class="nav">
      <button class="nav-tab ${activeTab === 'today' ? 'active' : ''}" data-tab="today">[1] HÔM NAY</button>
      <button class="nav-tab ${activeTab === 'writing' ? 'active' : ''}" data-tab="writing">[2] WRITING + AI</button>
      <button class="nav-tab ${activeTab === 'vocab' ? 'active' : ''}" data-tab="vocab">[3] TỪ VỰNG SRS (${dueCards.length})</button>
      <button class="nav-tab ${activeTab === 'roadmap' ? 'active' : ''}" data-tab="roadmap">[4] LỘ TRÌNH & SỔ LỖI</button>
      <div class="nav-meta">Mục tiêu Band ${dash.settings.target_band.toFixed(1)}</div>
    </div>

    <div class="content">
      <!-- TAB 1: TODAY -->
      <div class="panel ${activeTab === 'today' ? 'active' : ''}" id="tab-today">
        ${
          !dash.settings.has_api_key
            ? `<div class="api-banner">
                <span>Chưa có Gemini API Key — bấm <b>Cài đặt</b> để nạp miễn phí hoặc dùng <b>Copy-Prompt</b></span>
                <button class="btn btn-sm" id="bannerOpenSettingsBtn">NẠP API KEY</button>
              </div>`
            : ''
        }
        <div class="band-section-head">
          <span>TIẾN ĐỘ 4 KỸ NĂNG (MỤC TIÊU ${dash.settings.target_band.toFixed(1)} · BẤM THẺ ĐỂ NHẬP ĐIỂM)</span>
          <button class="btn btn-sm btn-primary" id="openScoreTab1Btn" type="button">+ Nhập điểm</button>
        </div>
        <div class="grid-4" style="margin-bottom:12px;">
          ${dash.skill_bands
            .map((b) => {
              const fill = Math.min(100, Math.max(10, Math.round((b.current / 9.0) * 100)));
              return `
              <div class="band-card" data-band-skill="${esc(b.skill)}" tabindex="0" role="button" title="Bấm để nhập điểm ${esc(b.skill)}">
                <div class="band-name">${esc(b.skill)}${b.skill === 'Writing' ? ' ★' : ''}</div>
                <div class="band-val">${b.current.toFixed(1)} <small>→ ${b.target.toFixed(1)}</small></div>
                <div class="bar-track"><div class="bar-fill" style="width:${fill}%"></div></div>
              </div>`;
            })
            .join('')}
        </div>

        <div class="grid-2">
          <div class="box">
            <div class="box-title">
              <span>NHIỆM VỤ HÔM NAY (30 PHÚT · BẤM ĐỂ TICK)</span>
              <span>${doneQuests}/${totalQuests} XONG</span>
            </div>
            <div class="bar-track" style="margin-bottom:10px;"><div class="bar-fill" style="width:${questPct}%"></div></div>
            ${dash.quests
              .map(
                (q) => `
              <div class="quest ${q.done ? 'done' : ''}" data-quest-id="${q.id}" tabindex="0" role="button">
                <div class="q-check">${q.done ? '✓' : ''}</div>
                <div>
                  <div class="q-title">${esc(q.title)}</div>
                  <div class="q-meta">${esc(q.meta)}</div>
                </div>
                <div class="q-tag">${String(q.minutes).padStart(2, '0')} PHÚT</div>
              </div>`
              )
              .join('')}
          </div>

          <div class="box">
            <div class="box-title">
              <span>HEATMAP CHUỖI HỌC (10 TUẦN)</span>
              <span>30p/ngày</span>
            </div>
            <div style="font-size:11.5px;color:var(--m-muted);">
              Mỗi ô = 1 ngày. Hoàn thành tối thiểu 1 nhiệm vụ (≥10 phút) để sáng ô & giữ streak.
            </div>
            <div class="heatmap">
              ${dash.heatmap
                .map((d) => {
                  const lvl =
                    d.minutes >= 30 ? 4 : d.minutes >= 20 ? 3 : d.minutes >= 10 ? 2 : d.minutes > 0 ? 1 : 0;
                  return `<div class="hm-cell ${lvl ? 'hm-' + lvl : ''}" title="${d.date}: ${d.minutes} phút"></div>`;
                })
                .join('')}
            </div>
            <div style="display:flex;justify-content:space-between;align-items:center;margin-top:10px;font-size:11px;color:var(--m-muted);">
              <span>Ít</span>
              <div style="display:flex;gap:3px;">
                <span style="width:10px;height:10px;background:var(--m-heat-0);border:1px solid var(--m-line);display:inline-block;"></span>
                <span style="width:10px;height:10px;background:var(--m-heat-1);border:1px solid var(--m-line);display:inline-block;"></span>
                <span style="width:10px;height:10px;background:var(--m-heat-2);border:1px solid var(--m-line);display:inline-block;"></span>
                <span style="width:10px;height:10px;background:var(--m-heat-3);border:1px solid var(--m-line);display:inline-block;"></span>
                <span style="width:10px;height:10px;background:var(--m-heat-4);border:1px solid var(--m-line);display:inline-block;"></span>
              </div>
              <span>Nhiều</span>
            </div>
          </div>
        </div>
        ${
          dash.quiz_status.quiz_due
            ? `<div class="quiz-due-banner">
                <span>📝 <b>Nhắc lịch từ vựng:</b> ${esc(dash.quiz_status.quiz_due_reason)}</span>
                <button class="btn btn-sm btn-primary" id="bannerStartQuizBtn" type="button">LÀM QUIZ 10 CÂU →</button>
              </div>`
            : ''
        }
      </div>

      <!-- TAB 2: WRITING + AI -->
      <div class="panel ${activeTab === 'writing' ? 'active' : ''}" id="tab-writing">
        <div class="mode-row">
          <button class="btn btn-sm mode-btn ${writingMode === 'paragraph_90' ? 'active' : ''}" data-wmode="paragraph_90">ĐOẠN 90 TỪ</button>
          <button class="btn btn-sm mode-btn ${writingMode === 'task1' ? 'active' : ''}" data-wmode="task1">TASK 1 (150+)</button>
          <button class="btn btn-sm mode-btn ${writingMode === 'task2' ? 'active' : ''}" data-wmode="task2">TASK 2 (250+)</button>
          <select class="select-input" id="promptSelect">
            ${modePrompts
              .map(
                (p) =>
                  `<option value="${esc(p.id)}" ${curPrompt?.id === p.id ? 'selected' : ''}>${esc(p.title)}</option>`
              )
              .join('')}
          </select>
          <button class="btn btn-sm" style="margin-left:auto;" id="toggleHintBtn">+ GỢI Ý KHUNG BÀI ${isAim80 ? '8.0' : '6.5'}</button>
        </div>

        <div class="grid-2">
          <div class="box">
            <div class="box-title">
              <span>ĐỀ BÀI & KHUNG VIẾT</span>
              <span id="wordCount">0 / ${curPrompt?.target_words || 90} từ</span>
            </div>
            <div class="prompt-box"><b>Đề:</b> ${esc(curPrompt?.prompt || '')}</div>
            <div class="hint-box ${hintOpen ? 'open' : ''}" id="hintBox">${esc(curPrompt?.outline_hint || '')}</div>
            <textarea class="editor" id="essayInput" placeholder="Nhập bài viết tiếng Anh của bạn tại đây..."></textarea>
            <div class="checklist-3m">
              <div style="font-weight:700;color:var(--m-muted);margin-bottom:5px;text-transform:uppercase;">CHECKLIST SOÁT LỖI 3 PHÚT TRƯỚC KHI NỘP:</div>
              <div style="display:grid;grid-template-columns:1fr 1fr;gap:4px;">
                ${dash.settings.writing_checklist
                  .map(
                    (item, idx) =>
                      `<label style="display:flex;gap:5px;align-items:flex-start;cursor:pointer;"><input type="checkbox" class="chk-3m" /> <span>${idx + 1}. ${esc(item)}</span></label>`
                  )
                  .join('')}
              </div>
            </div>
            <div style="display:flex;justify-content:space-between;align-items:center;margin-top:8px;gap:6px;flex-wrap:wrap;">
              <button class="btn btn-sm" id="copyPromptBtn">COPY PROMPT CHẤM AI</button>
              <button class="btn btn-primary" id="gradeBtn" ${isGrading ? 'disabled' : ''}>${isGrading ? 'ĐANG CHẤM AI...' : `CHẤM BAND ${isAim80 ? '8.0' : '6.5'} NGAY →`}</button>
            </div>
            <div id="writingStatus" class="status-msg ${writingStatusErr ? 'err' : writingStatusMsg ? 'ok' : ''}" style="margin-top:6px;">${esc(writingStatusMsg)}</div>
          </div>

          <div class="box" id="gradePanel">
            <div class="box-title">
              <span>KẾT QUẢ CHẤM AI (4 TIÊU CHÍ)</span>
              <span>ƯỚC LƯỢNG: <b>${lastGrade ? lastGrade.overall.toFixed(1) : '—'}</b></span>
            </div>
            <div class="score-grid">
              <div class="score-box"><b>${lastGrade ? lastGrade.ta.toFixed(1) : '—'}</b><span>TA</span></div>
              <div class="score-box"><b>${lastGrade ? lastGrade.cc.toFixed(1) : '—'}</b><span>CC</span></div>
              <div class="score-box"><b>${lastGrade ? lastGrade.lr.toFixed(1) : '—'}</b><span>LR</span></div>
              <div class="score-box"><b>${lastGrade ? lastGrade.gra.toFixed(1) : '—'}</b><span>GRA</span></div>
            </div>
            ${
              lastGrade
                ? `
              ${lastGrade.summary_vi ? `<div style="font-size:11.5px;margin-bottom:8px;"><b>Nhận xét:</b> ${esc(lastGrade.summary_vi)}</div>` : ''}
              <div style="font-size:11.5px;margin-bottom:8px;">
                <b>Sửa lỗi trực tiếp:</b>
                <div style="border:1px solid var(--m-line);background:var(--m-bg);padding:8px;margin-top:4px;line-height:1.65;">
                  ${lastGrade.corrections
                    .map(
                      (c) =>
                        `<div style="margin-bottom:4px;"><span class="err-del">${esc(c.wrong)}</span> → <span class="ok-add">${esc(c.right)}</span> <span style="color:var(--m-muted);font-size:10.5px;">[${esc(c.category)}: ${esc(c.reason)}]</span></div>`
                    )
                    .join('') || 'Không có lỗi đáng kể.'}
                </div>
              </div>
              <div style="font-size:11.5px;">
                <b>Bài mẫu nâng Band ${isAim80 ? '7.5–8.0' : '6.5'}:</b>
                <div style="border:1px solid var(--m-line);background:var(--m-bg);padding:8px;margin-top:4px;color:var(--m-muted);">
                  ${esc(lastGrade.band65_rewrite)}
                </div>
              </div>
              <div style="margin-top:8px;display:flex;justify-content:space-between;align-items:center;">
                <span style="font-size:10.5px;color:var(--m-muted);">${esc(lastEndpoint)}</span>
                <button class="btn btn-sm" id="saveErrorsBtn">+ LƯU LỖI VÀO SỔ</button>
              </div>`
                : `<div style="color:var(--m-muted);font-size:11.5px;">Viết bài ở khung bên trái rồi bấm <b>CHẤM BAND ${isAim80 ? '8.0' : '6.5'} NGAY →</b> (hoặc dùng <b>COPY PROMPT CHẤM AI</b> miễn phí).</div>`
            }
          </div>
        </div>
      </div>

      <!-- TAB 3: VOCAB SRS -->
      <div class="panel ${activeTab === 'vocab' ? 'active' : ''}" id="tab-vocab">
        <div class="vocab-sub-toggle">
          <button type="button" class="vocab-sub-btn ${vocabSubMode === 'flashcard' ? 'active' : ''}" data-vsub="flashcard">
            📇 ÔN FLASHCARD (SM-2)
          </button>
          <button type="button" class="vocab-sub-btn ${vocabSubMode === 'quiz' ? 'active' : ''}" data-vsub="quiz">
            📝 QUIZ KIỂM TRA TRÍ NHỚ
            ${dash.quiz_status.quiz_due ? '<span class="quiz-due-pill">ĐẾN LỊCH</span>' : ''}
          </button>
        </div>
        ${
          vocabSubMode === 'quiz'
            ? renderQuizBox()
            : `<div class="box">
                <div class="box-title">
                  <span>FLASHCARD SRS (THUẬT TOÁN SM-2 · OFFLINE · ${dash.total_vocab_count} THẺ)</span>
                  <span>${curCard ? `THẺ ${cardIdx + 1} / ${dueCards.length} CẦN ÔN HÔM NAY` : 'ĐÃ ÔN HẾT HÔM NAY ✓'}</span>
                </div>
                ${
                  curCard
                    ? `
                  <div class="fc-stage" id="fcStage" tabindex="0" role="button">
                    <span class="fc-topic">${esc(curCard.topic.toUpperCase())} · ${isAim80 ? 'BAND 8.0 (C1–C2)' : 'BAND 6.5'} · REPS ${curCard.reps}</span>
                    <div class="fc-word">${esc(curCard.word)}</div>
                    ${!cardFlipped ? `<div style="font-size:11px;color:var(--m-muted);">(Bấm vào thẻ để xem nghĩa & ví dụ)</div>` : ''}
                    <div class="fc-back ${cardFlipped ? 'open' : ''}">
                      <div style="font-weight:700;margin-bottom:4px;">${esc(curCard.meaning)}</div>
                      <div style="font-size:11.5px;color:var(--m-muted);">"${esc(curCard.example)}"</div>
                    </div>
                  </div>
                  <div class="srs-row">
                    <button class="btn srs-btn" data-q="0">[1] QUÊN · 10p</button>
                    <button class="btn srs-btn" data-q="1">[2] KHÓ · 1 ngày</button>
                    <button class="btn srs-btn" data-q="2">[3] NHỚ · 3 ngày</button>
                    <button class="btn btn-primary srs-btn" data-q="3">[4] DỄ · 7 ngày</button>
                  </div>`
                    : `<div style="padding:28px;text-align:center;color:var(--m-muted);">Tuyệt vời! Bạn đã hoàn thành toàn bộ thẻ đến hạn hôm nay.</div>`
                }
              </div>`
        }
      </div>

      <!-- TAB 4: ROADMAP & ERROR LOG -->
      <div class="panel ${activeTab === 'roadmap' ? 'active' : ''}" id="tab-roadmap">
        <div class="grid-2">
          <div class="box">
            <div class="box-title">
              <span>LỘ TRÌNH (${dash.roadmap.length} TUẦN · ${phaseNumbers.length} GIAI ĐOẠN)</span>
              <div style="display:flex;align-items:center;gap:8px;">
                <span>${dash.roadmap.filter((r) => r.done).length} / ${dash.roadmap.length}</span>
                <button class="btn btn-sm btn-primary" id="openRoadmapModalBtn">+ NẠP LỘ TRÌNH .MD</button>
              </div>
            </div>
            ${phaseNumbers
              .map((ph, idx) => {
                const items = dash!.roadmap.filter((r) => r.phase === ph);
                const label = dash!.settings.phase_labels[String(ph)] || `GĐ${ph} · Giai đoạn ${ph}`;
                const mb = idx === phaseNumbers.length - 1 ? '10px' : '6px';
                return `<div style="margin-bottom:${mb};">
                  <div style="display:flex;justify-content:space-between;font-size:11px;margin-bottom:2px;"><b>${esc(label)}</b><span>${pct(items)}%</span></div>
                  <div class="bar-track"><div class="bar-fill" style="width:${pct(items)}%"></div></div>
                </div>`;
              })
              .join('')}
            <div class="roadmap-list">
              ${dash.roadmap
                .map(
                  (r) => `
                <div class="rm-item ${r.done ? 'done' : ''}" data-rm-id="${r.id}" tabindex="0" role="button">
                  <div class="rm-check">${r.done ? '✓' : ''}</div>
                  <b class="rm-week">${esc(r.week_label)}:</b>
                  <span class="rm-title">${esc(r.title)}</span>
                </div>`
                )
                .join('')}
            </div>
            <div style="display:flex;gap:6px;flex-wrap:wrap;">
              <button class="btn btn-sm" id="openStudy4Btn">↗ MỞ STUDY4</button>
              <button class="btn btn-sm" id="openIotBtn">↗ MỞ IELTS ONLINE TESTS</button>
              <button class="btn btn-sm btn-primary" id="openScoreModalBtn">+ NHẬP ĐIỂM ĐỀ VỪA LÀM</button>
            </div>
          </div>

          <div class="box">
            <div class="box-title">
              <span>SỔ LỖI HAY MẮC (TỰ GOM TỪ WRITING)</span>
              <span>${dash.error_logs.length} MỤC</span>
            </div>
            <div style="display:flex;gap:4px;margin-bottom:8px;flex-wrap:wrap;">
              <input class="text-input" id="errCat" placeholder="Loại lỗi" style="width:95px;" value="Ngữ pháp" />
              <input class="text-input" id="errWrong" placeholder="Viết sai..." style="flex:1;min-width:90px;" />
              <input class="text-input" id="errRight" placeholder="Sửa chuẩn..." style="flex:1;min-width:90px;" />
              <button class="btn btn-sm" id="addErrBtn">+ THÊM</button>
            </div>
            <div id="errFormMsg" style="font-size:11.5px;margin-bottom:6px;${errFormMsg ? `color:${errFormErr ? 'var(--m-err)' : 'var(--m-ok)'};` : 'display:none;'}">${esc(errFormMsg)}</div>
            <div class="log-table-wrap">
              <table class="log-table">
                <thead>
                  <tr><th>Loại lỗi</th><th>Hay viết sai</th><th>Sửa chuẩn</th><th>Lần</th></tr>
                </thead>
                <tbody>
                  ${dash.error_logs
                    .map(
                      (e) =>
                        `<tr><td>${esc(e.category)}</td><td class="err-del">${esc(e.wrong_text)}</td><td class="ok-add">${esc(e.right_text)}</td><td>${e.count}</td></tr>`
                    )
                    .join('')}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="tray-strip">
      <div class="tray-badge">
        <span class="dot-live"></span>
        <span><b>SYSTEM TRAY:</b> Chạy ngầm cùng Windows · Nhắc lúc ${esc(dash.settings.reminder_time)} nếu chưa học</span>
      </div>
      <div style="color:var(--m-muted);">Rust + Tauri v2 + SQLite · Module track: ielts</div>
    </div>

    <!-- SETTINGS MODAL -->
    <div class="modal-backdrop" id="settingsModal">
      <div class="modal-box modal-wide">
        <div class="box-title"><span>CÀI ĐẶT HỆ THỐNG & DỮ LIỆU</span><button class="btn btn-sm" id="closeSettingsBtn">✕</button></div>

        <div class="form-row">
          <label>Chọn giao diện (4 phong cách · đổi tức thì)</label>
          ${renderThemeCards(curTheme, 'settings-theme-card')}
        </div>

        <div class="form-row">
          <label>Cấp độ lộ trình (Đổi tức thì giữa Tiêu chuẩn 6.5 &amp; Nâng cao 8.0)</label>
          ${renderLevelCards(isAim80, 'settings-level-card')}
        </div>

        <div class="step-card">
          <div class="step-head"><span>Dữ liệu học tập & Widget Taskbar Windows</span></div>
          <div class="data-actions-row">
            <button class="btn btn-sm" id="exportBackupBtn" type="button">💾 Xuất file sao lưu (.db)</button>
            <button class="btn btn-sm" id="importBackupBtn" type="button">📂 Khôi phục từ file (.db)</button>
          </div>
          ${backupMsg ? `<div class="status-msg ${backupOk === true ? 'ok' : backupOk === false ? 'err' : ''}">${esc(backupMsg)}</div>` : ''}
          <label class="switch-row" style="margin-top:8px;">
            <input type="checkbox" id="setWidgetEnabled" ${widgetInfo.running || widgetInfo.autostart ? 'checked' : ''} />
            <span>Hiện thanh Widget nhỏ trên Taskbar Windows (tự khởi động cùng máy)</span>
          </label>
          ${widgetStatusMsg ? `<div class="status-msg">${esc(widgetStatusMsg)}</div>` : ''}
        </div>

        <details class="guide-box guide-details">
          <summary>❓ Hướng dẫn lấy Gemini API Key miễn phí (3 bước)</summary>
          1. Bấm nút bên dưới để mở Google AI Studio và đăng nhập Google.<br/>
          2. Chọn <b>Create API key</b> → copy chuỗi bắt đầu bằng <code>AIza...</code>.<br/>
          3. Dán vào ô bên dưới hoặc nạp từ file <code>.txt/.env</code>, rồi bấm <b>Kiểm tra kết nối API</b>.
          <div style="margin-top:6px;">
            <button class="btn btn-sm" id="openAiStudioBtn" type="button">↗ LẤY KEY MIỄN PHÍ TẠI GOOGLE AI STUDIO</button>
          </div>
        </details>
        <div class="form-row">
          <label>Gemini API Key ${dash.settings.has_api_key ? `(Đang lưu: ${esc(dash.settings.api_key_masked)})` : '(Chưa có)'}</label>
          <div class="input-group">
            <input type="${showApiKey ? 'text' : 'password'}" id="setKey" placeholder="Dán key AIza... hoặc để trống để giữ key cũ" />
            <button class="btn btn-sm" id="toggleKeyVisBtn" type="button">${showApiKey ? 'ẨN' : 'HIỆN'}</button>
          </div>
          <div style="display:flex;gap:6px;margin-top:6px;flex-wrap:wrap;">
            <input type="file" id="apiKeyFileInput" accept=".txt,.env,.key" style="display:none;" />
            <button class="btn btn-sm" id="loadKeyFileBtn" type="button">NẠP KEY TỪ FILE .TXT/.ENV</button>
            <button class="btn btn-sm" id="testApiBtn" type="button" ${isTestingApi ? 'disabled' : ''}>${isTestingApi ? 'ĐANG KIỂM TRA...' : 'KIỂM TRA KẾT NỐI API'}</button>
            <button class="btn btn-sm" id="clearKeyBtn" type="button" ${!dash.settings.has_api_key ? 'disabled' : ''}>XÓA KEY</button>
          </div>
          ${apiTestMsg ? `<div class="status-msg ${apiTestOk === true ? 'ok' : apiTestOk === false ? 'err' : ''}">${esc(apiTestMsg)}</div>` : ''}
        </div>
        <div class="form-row">
          <label>Model Gemini</label>
          <select id="setModel">
            <option value="gemini-3-flash-preview" ${dash.settings.gemini_model === 'gemini-3-flash-preview' ? 'selected' : ''}>gemini-3-flash-preview (Khuyên dùng)</option>
            <option value="gemini-3.1-flash-lite-preview" ${dash.settings.gemini_model === 'gemini-3.1-flash-lite-preview' ? 'selected' : ''}>gemini-3.1-flash-lite-preview (Nhanh)</option>
            <option value="gemini-2.5-flash" ${dash.settings.gemini_model === 'gemini-2.5-flash' ? 'selected' : ''}>gemini-2.5-flash</option>
          </select>
        </div>
        <div class="form-row">
          <label>Ngày thi dự kiến (YYYY-MM-DD)</label>
          <input type="text" id="setExamDate" value="${esc(dash.settings.exam_date)}" />
        </div>
        <div class="form-row">
          <label>Giờ nhắc học mỗi ngày (HH:MM)</label>
          <input type="text" id="setReminder" value="${esc(dash.settings.reminder_time)}" />
        </div>
        <div class="form-row" style="display:flex;align-items:center;gap:8px;">
          <input type="checkbox" id="setAutostart" style="width:auto;" ${dash.settings.autostart ? 'checked' : ''} />
          <label for="setAutostart" style="margin:0;cursor:pointer;">Tự khởi động cùng Windows</label>
        </div>
        <details class="donate-details" id="settingsDonateDetails">
          <summary>☕ Thích app này? Ủng hộ tác giả (VietQR)</summary>
          <div class="donate-details-body">
            <div class="donate-qr-frame donate-qr-frame-sm">
              <img src="${donateQrUrl}" alt="VietQR MB Bank 9888888881808" />
            </div>
            <div class="donate-details-info">
              <div class="donate-details-thanks">Cảm ơn bạn đã đồng hành cùng XD4T IELTS! App hoàn toàn miễn phí &amp; không quảng cáo.</div>
              <div class="donate-bank-row">
                <code>MB Bank · 9888888881808 · PHAM XUAN DAT</code>
                <button class="btn btn-sm" id="settingsCopyStkBtn" type="button">Copy STK</button>
              </div>
            </div>
          </div>
        </details>

        <div class="modal-sticky-footer">
          <div style="display:flex;gap:6px;flex-wrap:wrap;">
            <button class="btn btn-sm" id="settingsToRoadmapBtn" type="button">+ NẠP LỘ TRÌNH .MD</button>
            <button class="btn btn-sm" id="reopenWelcomeBtn" type="button">Mở lại hướng dẫn bắt đầu</button>
          </div>
          <div style="display:flex;gap:6px;">
            <button class="btn" id="cancelSettingsBtn">HỦY</button>
            <button class="btn btn-primary" id="saveSettingsBtn">LƯU CÀI ĐẶT</button>
          </div>
        </div>
      </div>
    </div>

    <!-- WELCOME WIZARD MODAL -->
    <div class="modal-backdrop ${welcomeOpen ? 'open' : ''}" id="welcomeModal">
      <div class="modal-box modal-wide">
        <div class="box-title">
          <span>${welcomeStep === 4 ? 'CHÀO MỪNG TỚI XD4T IELTS · HOÀN TẤT THIẾT LẬP' : `CHÀO MỪNG TỚI XD4T IELTS · THIẾT LẬP NHANH (${welcomeStep}/3)`}</span>
          <button class="btn btn-sm" id="skipWelcomeBtn" type="button">Bỏ qua</button>
        </div>

        <div class="wizard-steps">
          <div class="wizard-step-pill ${welcomeStep === 1 ? 'active' : welcomeStep > 1 ? 'done' : ''}">1. Chọn phong cách</div>
          <div class="wizard-step-pill ${welcomeStep === 2 ? 'active' : welcomeStep > 2 ? 'done' : ''}">2. Lộ trình học</div>
          <div class="wizard-step-pill ${welcomeStep === 3 ? 'active' : welcomeStep > 3 ? 'done' : ''}">3. Chấm Writing AI</div>
        </div>

        ${
          welcomeStep === 1
            ? `
          <div class="step-card">
            <div class="step-head"><span>Bước 1 — Chọn phong cách bạn thích (bấm là đổi ngay)</span></div>
            <div style="font-size:11.5px;color:var(--m-muted);margin-bottom:8px;">
              Bạn có thể đổi lại bất cứ lúc nào ngay trên thanh trên cùng của ứng dụng.
            </div>
            ${renderThemeCards(curTheme, 'welcome-theme-card')}
          </div>
          <div style="display:flex;justify-content:flex-end;gap:8px;margin-top:12px;">
            <button class="btn btn-primary" id="welcomeNext1Btn" type="button">Tiếp tục →</button>
          </div>`
            : welcomeStep === 2
            ? `
          <div class="step-card">
            <div class="step-head"><span>Bước 2 — Chọn cấp độ &amp; lộ trình học của bạn</span></div>
            ${renderLevelCards(isAim80, 'welcome-level-card')}
            <div style="font-size:11.5px;color:var(--m-muted);margin-bottom:10px;">
              App đã tích hợp sẵn cả 2 lộ trình 24 tuần chuẩn (xoay vòng T2–CN + KPI từng tuần). Nếu muốn cá nhân hóa thêm theo file riêng, bạn có thể nhờ ChatGPT/Gemini tạo file <code>.md</code> và nạp vào app:
            </div>
            <div style="display:flex;gap:8px;flex-wrap:wrap;">
              <button class="btn btn-sm" id="welcomeCopyPromptBtn" type="button">📋 Copy câu lệnh gửi ChatGPT / Gemini</button>
              <button class="btn btn-sm" id="welcomeOpenRoadmapBtn" type="button">📂 Mở trình nạp file .md</button>
              <button class="btn btn-sm btn-primary" id="welcomeUseSampleBtn" type="button">✓ Dùng lộ trình 24 tuần (${isAim80 ? 'Aim 8.0' : 'Aim 6.5'})</button>
            </div>
            ${welcomeMsg ? `<div class="status-msg ${welcomeOk === true ? 'ok' : welcomeOk === false ? 'err' : ''}">${esc(welcomeMsg)}</div>` : ''}
          </div>
          <div style="display:flex;justify-content:space-between;gap:8px;margin-top:12px;">
            <button class="btn" id="welcomePrev2Btn" type="button">← Quay lại</button>
            <button class="btn btn-primary" id="welcomeNext2Btn" type="button">Tiếp tục →</button>
          </div>`
            : welcomeStep === 3
            ? `
          <div class="step-card">
            <div class="step-head"><span>Bước 3 — Chấm Writing bằng AI (tùy chọn)</span></div>
            <div style="font-size:11.5px;color:var(--m-muted);margin-bottom:8px;">
              Bạn có thể gắn <b>Google Gemini API Key miễn phí</b> để app tự chấm Writing 4 tiêu chí, hoặc bỏ qua bước này để học hoàn toàn miễn phí (vẫn có nút Copy Prompt gửi ChatGPT/Gemini web).
            </div>
            <div style="margin-bottom:8px;">
              <button class="btn btn-sm" id="welcomeOpenAiStudioBtn" type="button">↗ Mở Google AI Studio lấy key miễn phí</button>
            </div>
            <div class="input-group">
              <input type="password" id="welcomeKeyInput" placeholder="Dán Gemini API Key (AIza...) hoặc để trống nếu chưa dùng" value="${esc(welcomeKeyDraft)}" />
              <button class="btn btn-sm" id="welcomeTestKeyBtn" type="button" ${isTestingApi ? 'disabled' : ''}>${isTestingApi ? 'Đang thử...' : 'Kiểm tra kết nối'}</button>
            </div>
            ${welcomeMsg ? `<div class="status-msg ${welcomeOk === true ? 'ok' : welcomeOk === false ? 'err' : ''}">${esc(welcomeMsg)}</div>` : ''}
          </div>
          <div style="display:flex;justify-content:space-between;gap:8px;margin-top:12px;flex-wrap:wrap;">
            <button class="btn" id="welcomePrev3Btn" type="button">← Quay lại</button>
            <button class="btn btn-primary" id="welcomeNext3Btn" type="button">Hoàn tất thiết lập →</button>
          </div>`
            : `
          <div class="welcome-finish-grid">
            <div class="welcome-finish-main">
              <div class="welcome-finish-title">🎉 Xong rồi! Sẵn sàng chinh phục Band ${isAim80 ? '8.0' : '6.5+'}</div>
              <div class="welcome-finish-desc">
                Mọi thứ đã sẵn sàng trên máy bạn. Từ giờ mỗi ngày chỉ cần 30 phút đều đặn nhé.
              </div>
              <div class="welcome-summary-box">
                <div class="welcome-summary-row">
                  <span>Giao diện đang chọn</span>
                  <b>${esc(THEME_OPTIONS.find((t) => t.id === curTheme)?.label || curTheme)}</b>
                </div>
                <div class="welcome-summary-row">
                  <span>Lộ trình hiện tại</span>
                  <b>${dash.roadmap.length} tuần · Mục tiêu Band ${dash.settings.target_band.toFixed(1)}</b>
                </div>
                <div class="welcome-summary-row">
                  <span>Chấm Writing AI</span>
                  <b>${dash.settings.has_api_key || welcomeKeyDraft.trim() ? '✓ Đã gắn Gemini API Key' : 'Chưa gắn key (dùng Prompt miễn phí)'}</b>
                </div>
              </div>
              <button class="btn btn-primary welcome-start-btn" id="finishWelcomeBtn" type="button">Bắt đầu học ngay →</button>
              <div class="welcome-finish-note">Màn hình này chỉ hiện 1 lần duy nhất sau khi thiết lập.</div>
            </div>

            <div class="welcome-finish-donate">
              <div class="donate-badge">☕ Tùy chọn · Mời cà phê tác giả</div>
              <div class="donate-qr-frame">
                <img src="${donateQrUrl}" alt="VietQR MB Bank 9888888881808" />
              </div>
              <div class="donate-caption">
                App hoàn toàn miễn phí &amp; không quảng cáo. Nếu thấy hữu ích, mời tác giả ly cà phê nhé!
              </div>
              <div class="donate-bank-row">
                <code>MB · 9888888881808</code>
                <button class="btn btn-sm" id="welcomeCopyStkBtn" type="button">Copy</button>
              </div>
            </div>
          </div>`
        }
      </div>
    </div>

    <!-- ROADMAP IMPORT MODAL -->
    <div class="modal-backdrop" id="roadmapModal">
      <div class="modal-box modal-wide">
        <div class="box-title"><span>NẠP LỘ TRÌNH CÁ NHÂN (.MD)</span><button class="btn btn-sm" id="closeRoadmapBtn">✕</button></div>

        <div class="step-card">
          <div class="step-head"><span>Bước 1 — Tạo file lộ trình bằng AI của bạn</span></div>
          <div style="font-size:11.5px;color:var(--m-muted);margin-bottom:8px;">
            Copy prompt phỏng vấn 5 câu bên dưới và dán vào ChatGPT / Claude / Gemini web. AI sẽ tự sinh ra file <code>lo-trinh.md</code> đúng chuẩn 100% cho app.
          </div>
          <div style="display:flex;gap:6px;flex-wrap:wrap;">
            <button class="btn btn-sm btn-primary" id="copyRoadmapPromptBtn">COPY PROMPT TẠO LỘ TRÌNH CHO AI</button>
            <button class="btn btn-sm" id="loadSampleRoadmapBtn">XEM MẪU CHUẨN</button>
            <button class="btn btn-sm" id="exportCurrentRoadmapBtn">XUẤT LỘ TRÌNH HIỆN TẠI (.MD)</button>
          </div>
        </div>

        <div class="step-card">
          <div class="step-head">
            <span>Bước 2 — Nạp file .md hoặc dán trực tiếp Markdown</span>
            <div>
              <input type="file" id="roadmapFileInput" accept=".md,.txt" style="display:none;" />
              <button class="btn btn-sm" id="pickRoadmapFileBtn">CHỌN FILE .MD TỪ MÁY</button>
            </div>
          </div>
          <textarea class="editor" id="roadmapMdInput" style="min-height:145px;" placeholder="Dán nội dung lo-trinh.md tại đây hoặc bấm [CHỌN FILE .MD TỪ MÁY]...">${esc(roadmapDraftMd)}</textarea>
        </div>

        <div class="step-card">
          <div class="step-head">
            <span>Bước 3 — Xem trước (Preview) & Áp dụng</span>
            <button class="btn btn-sm" id="previewRoadmapBtn">PHÂN TÍCH & XEM TRƯỚC</button>
          </div>
          ${
            roadmapPreview
              ? `
            <div class="preview-grid">
              <div class="preview-stat"><span>Ngày thi</span><b>${esc(roadmapPreview.exam_date)}</b></div>
              <div class="preview-stat"><span>Mục tiêu</span><b>${roadmapPreview.overall_current.toFixed(1)} → ${roadmapPreview.overall_target.toFixed(1)}</b></div>
              <div class="preview-stat"><span>Nhiệm vụ/ngày</span><b>${roadmapPreview.quests_count} mục (${roadmapPreview.daily_minutes}p)</b></div>
              <div class="preview-stat"><span>Lộ trình tuần</span><b>${roadmapPreview.weeks_count} tuần (${roadmapPreview.phases_count} GĐ)</b></div>
              <div class="preview-stat"><span>Checklist Writing</span><b>${roadmapPreview.checklist_count} mục</b></div>
              <div class="preview-stat"><span>Điểm yếu AI soi</span><b>${roadmapPreview.focus_notes_count} nhóm</b></div>
              <div class="preview-stat"><span>Từ vựng SRS</span><b>${roadmapPreview.vocab_count} thẻ</b></div>
              <div class="preview-stat"><span>Sổ lỗi ban đầu</span><b>${roadmapPreview.error_logs_count} lỗi</b></div>
            </div>
            <div style="font-size:11px;color:var(--m-muted);margin-bottom:4px;">
              <b>4 Band:</b> ${roadmapPreview.skill_bands.map((b) => `${esc(b.skill)} ${b.current.toFixed(1)}→${b.target.toFixed(1)}`).join(' · ') || 'Giữ nguyên'}
            </div>
            ${
              roadmapPreview.warnings.length
                ? `<div style="font-size:11px;color:var(--m-warn);margin-top:4px;">⚠ ${roadmapPreview.warnings.map((w) => esc(w)).join(' | ')}</div>`
                : ''
            }`
              : `<div style="font-size:11px;color:var(--m-muted);">Bấm <b>[PHÂN TÍCH & XEM TRƯỚC]</b> để kiểm tra dữ liệu trước khi ghi vào SQLite.</div>`
          }
          <div style="display:flex;gap:14px;flex-wrap:wrap;margin-top:8px;font-size:11px;">
            <label style="display:flex;align-items:center;gap:6px;cursor:pointer;">
              <input type="checkbox" id="chkReplaceRoadmap" checked />
              <span>Thay thế lộ trình & nhiệm vụ cũ</span>
            </label>
            <label style="display:flex;align-items:center;gap:6px;cursor:pointer;">
              <input type="checkbox" id="chkMergeVocabErrors" checked />
              <span>Gộp thêm từ vựng SRS & sổ lỗi (giữ lịch sử cũ)</span>
            </label>
          </div>
          ${roadmapStatusMsg ? `<div class="status-msg ${roadmapStatusOk === true ? 'ok' : roadmapStatusOk === false ? 'err' : ''}">${esc(roadmapStatusMsg)}</div>` : ''}
        </div>

        <div style="display:flex;justify-content:flex-end;gap:6px;margin-top:10px;">
          <button class="btn" id="cancelRoadmapBtn">ĐÓNG</button>
          <button class="btn btn-primary" id="applyRoadmapBtn">ÁP DỤNG LỘ TRÌNH NGAY</button>
        </div>
      </div>
    </div>

    <!-- SCORE MODAL -->
    <div class="modal-backdrop" id="scoreModal">
      <div class="modal-box">
        <div class="box-title"><span>NHẬP ĐIỂM ĐỀ VỪA LÀM</span><button class="btn btn-sm" id="closeScoreBtn">✕</button></div>
        <div class="form-row">
          <label>Kỹ năng</label>
          <select id="scSkill">
            <option value="Listening">Listening</option>
            <option value="Reading">Reading</option>
            <option value="Writing">Writing</option>
            <option value="Speaking">Speaking</option>
          </select>
        </div>
        <div class="form-row">
          <label>Nguồn đề (VD: Study4 Cam 15 Test 1)</label>
          <input type="text" id="scSource" value="Study4 Cambridge" />
        </div>
        <div class="form-row">
          <label>Band điểm (0.0 – 9.0)</label>
          <input type="number" step="0.5" min="0" max="9" id="scVal" value="5.5" />
        </div>
        <div class="form-row">
          <label>Ghi chú</label>
          <input type="text" id="scNote" placeholder="VD: Đúng 23/40 câu" />
        </div>
        <div style="display:flex;justify-content:flex-end;gap:6px;margin-top:14px;">
          <button class="btn" id="cancelScoreBtn">HỦY</button>
          <button class="btn btn-primary" id="saveScoreBtn">LƯU ĐIỂM</button>
        </div>
      </div>
    </div>
  </div>`;

  bindEvents();
}

function formatPomo(s: number): string {
  const m = String(Math.floor(s / 60)).padStart(2, '0');
  const sec = String(s % 60).padStart(2, '0');
  return `${m}:${sec}`;
}

function togglePomodoro(forceStart = false) {
  if (pomoTimer !== null && !forceStart) {
    window.clearInterval(pomoTimer);
    pomoTimer = null;
    const btn = document.getElementById('pomoBtn');
    if (btn) btn.textContent = `⏱ ${formatPomo(pomoSecs)}`;
    return;
  }
  if (pomoTimer !== null) return;
  pomoTimer = window.setInterval(async () => {
    if (pomoSecs <= 1) {
      window.clearInterval(pomoTimer!);
      pomoTimer = null;
      pomoSecs = 15 * 60;
      dash = await invoke<DashboardState>('add_pomodoro_minutes', { minutes: 15 });
      renderApp();
      return;
    }
    pomoSecs -= 1;
    const btn = document.getElementById('pomoBtn');
    if (btn) btn.textContent = `⏱ ${formatPomo(pomoSecs)}`;
  }, 1000);
}

async function finishOnboarding(extraKey?: string) {
  if (!dash) return;
  dash.settings = await invoke('update_settings', {
    geminiApiKey: extraKey && extraKey.trim() ? extraKey.trim() : null,
    onboardingDone: true,
  });
  dash = await invoke<DashboardState>('get_dashboard');
  welcomeOpen = false;
  welcomeMsg = '';
  welcomeOk = null;
  renderApp();
}

function bindEvents() {
  document.querySelectorAll<HTMLButtonElement>('.nav-tab').forEach((btn) => {
    btn.onclick = () => {
      activeTab = btn.dataset.tab as typeof activeTab;
      renderApp();
    };
  });

  document.getElementById('pomoBtn')!.onclick = () => togglePomodoro(false);

  const themeSelect = document.getElementById('themeSelect') as HTMLSelectElement | null;
  if (themeSelect) {
    themeSelect.onchange = async () => {
      if (!dash) return;
      dash.settings = await invoke('update_settings', { theme: themeSelect.value });
      renderApp();
    };
  }

  const bindActivate = (el: HTMLElement, handler: () => void | Promise<void>) => {
    el.onclick = () => { void handler(); };
    el.onkeydown = (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        void handler();
      }
    };
  };

  document.querySelectorAll<HTMLElement>('[data-quest-id]').forEach((el) => {
    bindActivate(el, async () => {
      const id = Number(el.dataset.questId);
      dash = await invoke<DashboardState>('toggle_quest', { id });
      renderApp();
    });
  });

  // Writing tab
  document.querySelectorAll<HTMLButtonElement>('[data-wmode]').forEach((btn) => {
    btn.onclick = () => {
      writingMode = btn.dataset.wmode!;
      const first = dash?.prompts.find((p) => p.mode === writingMode);
      if (first) selectedPromptId = first.id;
      renderApp();
    };
  });

  const promptSel = document.getElementById('promptSelect') as HTMLSelectElement | null;
  if (promptSel) {
    promptSel.onchange = () => {
      selectedPromptId = promptSel.value;
      renderApp();
    };
  }

  const toggleHintBtn = document.getElementById('toggleHintBtn');
  if (toggleHintBtn) {
    toggleHintBtn.onclick = () => {
      hintOpen = !hintOpen;
      document.getElementById('hintBox')?.classList.toggle('open', hintOpen);
    };
  }

  const essayInput = document.getElementById('essayInput') as HTMLTextAreaElement | null;
  if (essayInput) {
    essayInput.value = draftEssay;
    const updateWC = () => {
      draftEssay = essayInput.value;
      const txt = essayInput.value.trim();
      const n = txt ? txt.split(/\s+/).length : 0;
      const target = getCurrentPrompt()?.target_words || 90;
      const wc = document.getElementById('wordCount');
      if (wc) wc.textContent = `${n} / ${target} từ`;
    };
    essayInput.oninput = updateWC;
    updateWC();
  }

  const copyPromptBtn = document.getElementById('copyPromptBtn');
  if (copyPromptBtn) {
    copyPromptBtn.onclick = async () => {
      const p = getCurrentPrompt();
      const content = (document.getElementById('essayInput') as HTMLTextAreaElement)?.value || '';
      const focusNotes = (dash?.settings.writing_focus_notes || [])
        .map((n, i) => `${i + 1}. ${n}`)
        .join('\n');
      const targetLabel = isAim80Mode()
        ? 'bản viết lại Band 8.0 (ưu tiên collocations C1–C2, hedging học thuật & câu phức sắc sảo)'
        : 'bản viết lại Band 6.5';
      const rubric = `Hãy đóng vai Giám khảo IELTS Writing chấm bài dưới đây theo 4 tiêu chí (TA, CC, LR, GRA) và trả kết quả rõ ràng bằng tiếng Việt + ${targetLabel}.\n\n[ĐIỂM YẾU CẦN SOI KỸ]:\n${focusNotes}\n\n[CHẾ ĐỘ]: ${writingMode}\n[ĐỀ BÀI]: ${p?.prompt || ''}\n\n[BÀI VIẾT]:\n${content}`;
      await navigator.clipboard.writeText(rubric);
      writingStatusMsg = '✓ Đã copy prompt + bài viết vào Clipboard! Dán vào ChatGPT/Gemini web để chấm miễn phí.';
      writingStatusErr = false;
      const st = document.getElementById('writingStatus');
      if (st) {
        st.textContent = writingStatusMsg;
        st.className = 'status-msg ok';
      }
    };
  }

  const gradeBtn = document.getElementById('gradeBtn');
  if (gradeBtn) {
    gradeBtn.onclick = async () => {
      const content = (document.getElementById('essayInput') as HTMLTextAreaElement)?.value.trim() || '';
      if (content.length < 20) {
        writingStatusMsg = 'Hãy viết ít nhất vài câu trước khi chấm.';
        writingStatusErr = true;
        const st = document.getElementById('writingStatus');
        if (st) {
          st.textContent = writingStatusMsg;
          st.className = 'status-msg err';
        }
        return;
      }
      isGrading = true;
      writingStatusMsg = '';
      writingStatusErr = false;
      renderApp();
      // Preserve textarea content across re-render
      const restored = document.getElementById('essayInput') as HTMLTextAreaElement | null;
      if (restored) restored.value = content;
      try {
        const p = getCurrentPrompt();
        const res = await invoke<GradeResponseDto>('grade_writing', {
          mode: writingMode,
          prompt: p?.prompt || '',
          content,
        });
        lastGrade = res.result;
        lastEndpoint = res.endpoint;
        dash = await invoke<DashboardState>('get_dashboard');
      } catch (err) {
        writingStatusMsg = `Lỗi: ${String(err)}`;
        writingStatusErr = true;
      } finally {
        isGrading = false;
        renderApp();
        const after = document.getElementById('essayInput') as HTMLTextAreaElement | null;
        if (after) after.value = content;
      }
    };
  }

  const saveErrorsBtn = document.getElementById('saveErrorsBtn');
  if (saveErrorsBtn && lastGrade) {
    saveErrorsBtn.onclick = async () => {
      if (!lastGrade) return;
      dash = await invoke<DashboardState>('save_corrections_to_errors', {
        corrections: lastGrade.corrections,
      });
      saveErrorsBtn.textContent = `✓ ĐÃ LƯU ${lastGrade.corrections.length} LỖI VÀO SỔ`;
    };
  }

  // Vocab SRS & Quiz
  const startQuizRun = async () => {
    quizLoading = true;
    quizErrorMsg = '';
    renderApp();
    try {
      const questions = await invoke<QuizQuestion[]>('generate_vocab_quiz', {
        count: 10,
        mode: quizMode,
      });
      if (!questions.length) {
        quizErrorMsg = 'Chưa có đủ thẻ từ vựng để tạo bài Quiz.';
        quizPhase = 'setup';
      } else {
        quizQuestions = questions;
        quizIdx = 0;
        quizAnswers = [];
        quizFeedback = null;
        quizSummary = null;
        quizPhase = 'playing';
      }
    } catch (err) {
      quizErrorMsg = `Không thể tạo bài Quiz: ${String(err)}`;
      quizPhase = 'setup';
    } finally {
      quizLoading = false;
      renderApp();
      document.getElementById('quizTypingInput')?.focus();
    }
  };

  const bannerStartQuizBtn = document.getElementById('bannerStartQuizBtn');
  if (bannerStartQuizBtn) {
    bannerStartQuizBtn.onclick = () => {
      activeTab = 'vocab';
      vocabSubMode = 'quiz';
      void startQuizRun();
    };
  }

  document.querySelectorAll<HTMLButtonElement>('[data-vsub]').forEach((btn) => {
    btn.onclick = () => {
      vocabSubMode = btn.dataset.vsub as 'flashcard' | 'quiz';
      renderApp();
    };
  });

  document.querySelectorAll<HTMLButtonElement>('[data-qmode]').forEach((btn) => {
    btn.onclick = () => {
      quizMode = btn.dataset.qmode as 'mixed' | 'mcq' | 'typing';
      renderApp();
    };
  });

  const startQuizBtn = document.getElementById('startQuizBtn');
  if (startQuizBtn) {
    startQuizBtn.onclick = () => {
      void startQuizRun();
    };
  }

  const recordQuizAnswer = (correct: boolean, gaveUp: boolean, selectedOpt?: string, typedText?: string) => {
    const q = quizQuestions[quizIdx];
    if (!q || quizFeedback) return;
    quizAnswers.push({ card_id: q.card_id, correct, gave_up: gaveUp });
    quizFeedback = { correct, gaveUp, selectedOpt, typedText };
    renderApp();
    document.getElementById('quizNextBtn')?.focus();
  };

  document.querySelectorAll<HTMLButtonElement>('[data-quiz-opt]').forEach((btn) => {
    btn.onclick = () => {
      const q = quizQuestions[quizIdx];
      if (!q || quizFeedback) return;
      const opt = q.options[Number(btn.dataset.quizOpt)];
      recordQuizAnswer(opt === q.word, false, opt);
    };
  });

  const quizGiveUpBtn = document.getElementById('quizGiveUpBtn');
  if (quizGiveUpBtn) {
    quizGiveUpBtn.onclick = () => {
      const typed = (document.getElementById('quizTypingInput') as HTMLInputElement | null)?.value;
      recordQuizAnswer(false, true, undefined, typed);
    };
  }

  const quizCheckTypingBtn = document.getElementById('quizCheckTypingBtn');
  const quizTypingInput = document.getElementById('quizTypingInput') as HTMLInputElement | null;
  const checkTyping = () => {
    const q = quizQuestions[quizIdx];
    if (!q || quizFeedback || !quizTypingInput) return;
    const val = quizTypingInput.value;
    if (!val.trim()) {
      quizTypingInput.focus();
      return;
    }
    recordQuizAnswer(isTypingAnswerMatch(val, q.word), false, undefined, val);
  };
  if (quizCheckTypingBtn) {
    quizCheckTypingBtn.onclick = checkTyping;
  }
  if (quizTypingInput && !quizFeedback) {
    quizTypingInput.focus();
    quizTypingInput.onkeydown = (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        checkTyping();
      }
    };
  }

  const quizNextBtn = document.getElementById('quizNextBtn');
  if (quizNextBtn) {
    const goNext = async () => {
      if (quizIdx + 1 < quizQuestions.length) {
        quizIdx += 1;
        quizFeedback = null;
        renderApp();
        document.getElementById('quizTypingInput')?.focus();
      } else {
        quizSummary = await invoke<QuizSubmitResult>('submit_vocab_quiz', {
          mode: quizMode,
          results: quizAnswers,
        });
        dueCards = await invoke<VocabCard[]>('get_due_vocab');
        dash = await invoke<DashboardState>('get_dashboard');
        quizPhase = 'summary';
        quizFeedback = null;
        renderApp();
      }
    };
    quizNextBtn.onclick = () => { void goNext(); };
    quizNextBtn.onkeydown = (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        void goNext();
      }
    };
  }

  const quizReviewForgottenBtn = document.getElementById('quizReviewForgottenBtn');
  if (quizReviewForgottenBtn) {
    quizReviewForgottenBtn.onclick = async () => {
      dueCards = await invoke<VocabCard[]>('get_due_vocab');
      cardIdx = 0;
      cardFlipped = false;
      vocabSubMode = 'flashcard';
      quizPhase = 'setup';
      renderApp();
    };
  }

  const quizRestartBtn = document.getElementById('quizRestartBtn');
  if (quizRestartBtn) {
    quizRestartBtn.onclick = () => {
      quizPhase = 'setup';
      quizFeedback = null;
      quizSummary = null;
      renderApp();
    };
  }

  const fcStage = document.getElementById('fcStage');
  if (fcStage) {
    bindActivate(fcStage, () => {
      cardFlipped = !cardFlipped;
      renderApp();
    });
  }

  document.querySelectorAll<HTMLButtonElement>('.srs-btn').forEach((btn) => {
    btn.onclick = async () => {
      const card = dueCards[cardIdx];
      if (!card) return;
      const quality = Number(btn.dataset.q);
      await invoke('review_vocab', { cardId: card.id, quality });
      if (quality === 0) {
        dueCards.push(dueCards.splice(cardIdx, 1)[0]);
      } else {
        dueCards.splice(cardIdx, 1);
      }
      if (cardIdx >= dueCards.length) cardIdx = 0;
      cardFlipped = false;
      dash = await invoke<DashboardState>('get_dashboard');
      renderApp();
    };
  });

  // Roadmap & Error Log
  document.querySelectorAll<HTMLElement>('[data-rm-id]').forEach((el) => {
    bindActivate(el, async () => {
      const id = Number(el.dataset.rmId);
      dash = await invoke<DashboardState>('toggle_roadmap_item', { id });
      renderApp();
    });
  });

  document.getElementById('openStudy4Btn')!.onclick = () => openUrl('https://study4.com/tests/ielts/');
  document.getElementById('openIotBtn')!.onclick = () => openUrl('https://ieltsonlinetests.com/');

  document.getElementById('addErrBtn')!.onclick = async () => {
    const category = (document.getElementById('errCat') as HTMLInputElement).value.trim() || 'Ngữ pháp';
    const wrongInput = document.getElementById('errWrong') as HTMLInputElement;
    const rightInput = document.getElementById('errRight') as HTMLInputElement;
    const wrongText = wrongInput.value.trim();
    const rightText = rightInput.value.trim();
    const msgEl = document.getElementById('errFormMsg');
    if (!wrongText) {
      if (msgEl) {
        msgEl.textContent = '✕ Vui lòng nhập nội dung lỗi (Viết sai...).';
        msgEl.style.color = 'var(--m-err)';
        msgEl.style.display = 'block';
      }
      wrongInput.focus();
      return;
    }
    if (!rightText) {
      if (msgEl) {
        msgEl.textContent = '✕ Vui lòng nhập cách sửa chuẩn (Sửa chuẩn...).';
        msgEl.style.color = 'var(--m-err)';
        msgEl.style.display = 'block';
      }
      rightInput.focus();
      return;
    }
    errFormMsg = '✓ Đã thêm vào Sổ lỗi';
    errFormErr = false;
    scrollToNewErr = true;
    dash = await invoke<DashboardState>('add_error_log', { category, wrongText, rightText });
    renderApp();
  };

  if (scrollToNewErr) {
    scrollToNewErr = false;
    document.querySelector('.log-table-wrap')?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }

  const settingsDonateDetails = document.getElementById('settingsDonateDetails') as HTMLDetailsElement | null;
  if (settingsDonateDetails) {
    settingsDonateDetails.addEventListener('toggle', () => {
      if (settingsDonateDetails.open) {
        settingsDonateDetails.scrollIntoView({ behavior: 'smooth', block: 'end' });
      }
    });
  }

  // Modals
  const settingsModal = document.getElementById('settingsModal')!;
  const openSettings = () => settingsModal.classList.add('open');
  document.getElementById('openSettingsBtn')!.onclick = openSettings;
  const bannerOpenSettingsBtn = document.getElementById('bannerOpenSettingsBtn');
  if (bannerOpenSettingsBtn) bannerOpenSettingsBtn.onclick = openSettings;
  document.getElementById('closeSettingsBtn')!.onclick = () => settingsModal.classList.remove('open');
  document.getElementById('cancelSettingsBtn')!.onclick = () => settingsModal.classList.remove('open');
  settingsModal.onclick = (e) => {
    if (e.target === settingsModal) settingsModal.classList.remove('open');
  };

  document.getElementById('openAiStudioBtn')!.onclick = () => openUrl('https://aistudio.google.com/apikey');

  document.getElementById('toggleKeyVisBtn')!.onclick = () => {
    const typed = (document.getElementById('setKey') as HTMLInputElement).value;
    showApiKey = !showApiKey;
    renderApp();
    document.getElementById('settingsModal')!.classList.add('open');
    (document.getElementById('setKey') as HTMLInputElement).value = typed;
  };

  const apiKeyFileInput = document.getElementById('apiKeyFileInput') as HTMLInputElement;
  document.getElementById('loadKeyFileBtn')!.onclick = () => apiKeyFileInput.click();
  apiKeyFileInput.onchange = async () => {
    const file = apiKeyFileInput.files?.[0];
    if (!file) return;
    const text = await file.text();
    const extracted = extractApiKeyFromText(text);
    const keyInput = document.getElementById('setKey') as HTMLInputElement;
    keyInput.value = extracted;
    apiTestMsg = extracted
      ? `✓ Đã nạp key từ file (${file.name}). Bấm [KIỂM TRA KẾT NỐI API] hoặc [LƯU CÀI ĐẶT].`
      : '✕ Không tìm thấy API Key hợp lệ trong file.';
    apiTestOk = Boolean(extracted);
    renderApp();
    document.getElementById('settingsModal')!.classList.add('open');
    (document.getElementById('setKey') as HTMLInputElement).value = extracted;
  };

  document.getElementById('testApiBtn')!.onclick = async () => {
    const typedKey = (document.getElementById('setKey') as HTMLInputElement).value.trim();
    const modelVal = (document.getElementById('setModel') as HTMLSelectElement).value;
    isTestingApi = true;
    apiTestMsg = 'Đang kiểm tra kết nối tới Google Gemini API...';
    apiTestOk = null;
    renderApp();
    document.getElementById('settingsModal')!.classList.add('open');
    (document.getElementById('setKey') as HTMLInputElement).value = typedKey;
    try {
      const res = await invoke<ApiTestResult>('test_gemini_connection', {
        apiKey: typedKey || null,
        model: modelVal,
      });
      apiTestMsg = res.message;
      apiTestOk = res.ok;
    } catch (err) {
      apiTestMsg = `✕ ${String(err)}`;
      apiTestOk = false;
    } finally {
      isTestingApi = false;
      renderApp();
      document.getElementById('settingsModal')!.classList.add('open');
      (document.getElementById('setKey') as HTMLInputElement).value = typedKey;
    }
  };

  document.getElementById('clearKeyBtn')!.onclick = async () => {
    await invoke('clear_gemini_api_key');
    dash = await invoke<DashboardState>('get_dashboard');
    apiTestMsg = '✓ Đã xóa Gemini API Key khỏi SQLite.';
    apiTestOk = true;
    renderApp();
    document.getElementById('settingsModal')!.classList.add('open');
  };

  document.getElementById('settingsToRoadmapBtn')!.onclick = () => {
    settingsModal.classList.remove('open');
    document.getElementById('roadmapModal')!.classList.add('open');
  };

  document.getElementById('reopenWelcomeBtn')!.onclick = () => {
    settingsModal.classList.remove('open');
    welcomeOpen = true;
    welcomeStep = 1;
    welcomeMsg = '';
    welcomeOk = null;
    renderApp();
  };

  document.querySelectorAll<HTMLButtonElement>('.settings-theme-card').forEach((btn) => {
    btn.onclick = async () => {
      if (!dash) return;
      const picked = btn.dataset.pickTheme!;
      dash.settings = await invoke('update_settings', { theme: picked });
      renderApp();
      document.getElementById('settingsModal')!.classList.add('open');
    };
  });

  document.getElementById('exportBackupBtn')!.onclick = async () => {
    try {
      const res = await invoke<BackupResultDto>('export_db_backup');
      backupMsg = res.message;
      backupOk = res.cancelled ? null : true;
    } catch (err) {
      backupMsg = `✕ ${String(err)}`;
      backupOk = false;
    }
    renderApp();
    document.getElementById('settingsModal')!.classList.add('open');
  };

  document.getElementById('importBackupBtn')!.onclick = async () => {
    try {
      const res = await invoke<RestoreResultDto>('import_db_backup');
      backupMsg = res.message;
      backupOk = res.cancelled ? null : true;
      if (res.dashboard) {
        dash = res.dashboard;
        dueCards = await invoke<VocabCard[]>('get_due_vocab');
        cardIdx = 0;
        cardFlipped = false;
      }
    } catch (err) {
      backupMsg = `✕ ${String(err)}`;
      backupOk = false;
    }
    renderApp();
    document.getElementById('settingsModal')!.classList.add('open');
  };

  const widgetChk = document.getElementById('setWidgetEnabled') as HTMLInputElement | null;
  if (widgetChk) {
    widgetChk.onchange = async () => {
      try {
        widgetInfo = await invoke<WidgetRuntimeInfo>('set_widget_enabled', {
          enabled: widgetChk.checked,
        });
        widgetStatusMsg = widgetInfo.running
          ? '✓ Thanh Widget Taskbar đang bật và tự khởi động cùng Windows.'
          : '✓ Đã tắt thanh Widget Taskbar.';
      } catch (err) {
        widgetStatusMsg = `✕ ${String(err)}`;
        widgetChk.checked = !widgetChk.checked;
      }
      renderApp();
      document.getElementById('settingsModal')!.classList.add('open');
    };
  }

  // Welcome Wizard events
  const skipWelcomeBtn = document.getElementById('skipWelcomeBtn');
  if (skipWelcomeBtn) {
    skipWelcomeBtn.onclick = () => finishOnboarding();
  }

  document.querySelectorAll<HTMLButtonElement>('.welcome-theme-card').forEach((btn) => {
    btn.onclick = async () => {
      if (!dash) return;
      const picked = btn.dataset.pickTheme!;
      dash.settings = await invoke('update_settings', { theme: picked });
      renderApp();
    };
  });

  document.querySelectorAll<HTMLButtonElement>('.welcome-level-card').forEach((btn) => {
    btn.onclick = async () => {
      const level = btn.dataset.pickLevel!;
      dash = await invoke<DashboardState>('set_study_level', { level });
      dueCards = await invoke<VocabCard[]>('get_due_vocab');
      cardIdx = 0;
      cardFlipped = false;
      welcomeMsg =
        level === '8.0'
          ? '✓ Đã chuyển sang Lộ trình Nâng cao Aim 8.0 (nạp sẵn 40 từ C1–C2 & lộ trình 24 tuần KPI nâng cao).'
          : '✓ Đã chọn Lộ trình Tiêu chuẩn Aim 6.5 (Vùng đệm 7.0 · 24 tuần KPI chuẩn).';
      welcomeOk = true;
      renderApp();
    };
  });

  document.querySelectorAll<HTMLButtonElement>('.settings-level-card').forEach((btn) => {
    btn.onclick = async () => {
      const level = btn.dataset.pickLevel!;
      dash = await invoke<DashboardState>('set_study_level', { level });
      dueCards = await invoke<VocabCard[]>('get_due_vocab');
      cardIdx = 0;
      cardFlipped = false;
      renderApp();
      document.getElementById('settingsModal')?.classList.add('open');
    };
  });

  const welcomeNext1Btn = document.getElementById('welcomeNext1Btn');
  if (welcomeNext1Btn) {
    welcomeNext1Btn.onclick = () => {
      welcomeStep = 2;
      welcomeMsg = '';
      welcomeOk = null;
      renderApp();
    };
  }

  const welcomePrev2Btn = document.getElementById('welcomePrev2Btn');
  if (welcomePrev2Btn) {
    welcomePrev2Btn.onclick = () => {
      welcomeStep = 1;
      welcomeMsg = '';
      welcomeOk = null;
      renderApp();
    };
  }

  const welcomeNext2Btn = document.getElementById('welcomeNext2Btn');
  if (welcomeNext2Btn) {
    welcomeNext2Btn.onclick = () => {
      welcomeStep = 3;
      welcomeMsg = '';
      welcomeOk = null;
      renderApp();
    };
  }

  const welcomeCopyPromptBtn = document.getElementById('welcomeCopyPromptBtn');
  if (welcomeCopyPromptBtn) {
    welcomeCopyPromptBtn.onclick = async () => {
      const promptText = await invoke<string>('get_roadmap_prompt');
      await navigator.clipboard.writeText(promptText);
      welcomeMsg = '✓ Đã copy câu lệnh! Bạn có thể dán vào ChatGPT / Gemini để tạo lộ trình riêng.';
      welcomeOk = true;
      renderApp();
    };
  }

  const welcomeOpenRoadmapBtn = document.getElementById('welcomeOpenRoadmapBtn');
  if (welcomeOpenRoadmapBtn) {
    welcomeOpenRoadmapBtn.onclick = () => {
      welcomeOpen = false;
      renderApp();
      document.getElementById('roadmapModal')!.classList.add('open');
    };
  }

  const welcomeUseSampleBtn = document.getElementById('welcomeUseSampleBtn');
  if (welcomeUseSampleBtn) {
    welcomeUseSampleBtn.onclick = () => {
      welcomeMsg = `✓ Đang dùng sẵn Lộ trình mẫu 24 tuần (${isAim80Mode() ? 'Band 8.0' : 'Band 6.5'}).`;
      welcomeOk = true;
      welcomeStep = 3;
      renderApp();
    };
  }

  const welcomePrev3Btn = document.getElementById('welcomePrev3Btn');
  if (welcomePrev3Btn) {
    welcomePrev3Btn.onclick = () => {
      welcomeStep = 2;
      welcomeMsg = '';
      welcomeOk = null;
      renderApp();
    };
  }

  const welcomeOpenAiStudioBtn = document.getElementById('welcomeOpenAiStudioBtn');
  if (welcomeOpenAiStudioBtn) {
    welcomeOpenAiStudioBtn.onclick = () => openUrl('https://aistudio.google.com/apikey');
  }

  const welcomeKeyInput = document.getElementById('welcomeKeyInput') as HTMLInputElement | null;
  if (welcomeKeyInput) {
    welcomeKeyInput.oninput = () => {
      welcomeKeyDraft = welcomeKeyInput.value;
    };
  }

  const welcomeTestKeyBtn = document.getElementById('welcomeTestKeyBtn');
  if (welcomeTestKeyBtn) {
    welcomeTestKeyBtn.onclick = async () => {
      const typed = (document.getElementById('welcomeKeyInput') as HTMLInputElement)?.value.trim() || '';
      welcomeKeyDraft = typed;
      isTestingApi = true;
      welcomeMsg = 'Đang kiểm tra kết nối tới Google Gemini API...';
      welcomeOk = null;
      renderApp();
      try {
        const res = await invoke<ApiTestResult>('test_gemini_connection', {
          apiKey: typed || null,
          model: dash?.settings.gemini_model || 'gemini-3-flash-preview',
        });
        welcomeMsg = res.message;
        welcomeOk = res.ok;
      } catch (err) {
        welcomeMsg = `✕ ${String(err)}`;
        welcomeOk = false;
      } finally {
        isTestingApi = false;
        renderApp();
      }
    };
  }

  const welcomeNext3Btn = document.getElementById('welcomeNext3Btn');
  if (welcomeNext3Btn) {
    welcomeNext3Btn.onclick = () => {
      const typed = (document.getElementById('welcomeKeyInput') as HTMLInputElement)?.value.trim() || '';
      if (typed) welcomeKeyDraft = typed;
      welcomeStep = 4;
      welcomeMsg = '';
      welcomeOk = null;
      renderApp();
    };
  }

  const finishWelcomeBtn = document.getElementById('finishWelcomeBtn');
  if (finishWelcomeBtn) {
    finishWelcomeBtn.onclick = () => {
      finishOnboarding(welcomeKeyDraft);
    };
  }

  const bindCopyStk = (btnId: string) => {
    const btn = document.getElementById(btnId);
    if (!btn) return;
    btn.onclick = async () => {
      await navigator.clipboard.writeText('9888888881808');
      const prev = btn.textContent || 'Copy';
      btn.textContent = 'Đã copy!';
      window.setTimeout(() => {
        btn.textContent = prev;
      }, 2000);
    };
  };
  bindCopyStk('welcomeCopyStkBtn');
  bindCopyStk('settingsCopyStkBtn');

  document.getElementById('saveSettingsBtn')!.onclick = async () => {
    const keyVal = (document.getElementById('setKey') as HTMLInputElement).value.trim();
    const modelVal = (document.getElementById('setModel') as HTMLSelectElement).value;
    const examVal = (document.getElementById('setExamDate') as HTMLInputElement).value.trim();
    const remVal = (document.getElementById('setReminder') as HTMLInputElement).value.trim();
    const autoVal = (document.getElementById('setAutostart') as HTMLInputElement).checked;
    await invoke('update_settings', {
      geminiApiKey: keyVal ? keyVal : null,
      geminiModel: modelVal,
      examDate: examVal,
      reminderTime: remVal,
      autostart: autoVal,
    });
    dash = await invoke<DashboardState>('get_dashboard');
    settingsModal.classList.remove('open');
    renderApp();
  };

  // Roadmap Import Modal
  const roadmapModal = document.getElementById('roadmapModal')!;
  const openRoadmapBtn = document.getElementById('openRoadmapModalBtn');
  if (openRoadmapBtn) openRoadmapBtn.onclick = () => roadmapModal.classList.add('open');
  document.getElementById('closeRoadmapBtn')!.onclick = () => roadmapModal.classList.remove('open');
  document.getElementById('cancelRoadmapBtn')!.onclick = () => roadmapModal.classList.remove('open');
  roadmapModal.onclick = (e) => {
    if (e.target === roadmapModal) roadmapModal.classList.remove('open');
  };

  const roadmapMdInput = document.getElementById('roadmapMdInput') as HTMLTextAreaElement;
  roadmapMdInput.oninput = () => {
    roadmapDraftMd = roadmapMdInput.value;
  };

  document.getElementById('copyRoadmapPromptBtn')!.onclick = async () => {
    const promptText = await invoke<string>('get_roadmap_prompt');
    await navigator.clipboard.writeText(promptText);
    roadmapStatusMsg = '✓ Đã copy! Dán vào ChatGPT/Claude/Gemini để tạo lo-trinh.md';
    roadmapStatusOk = true;
    renderApp();
    document.getElementById('roadmapModal')!.classList.add('open');
  };

  document.getElementById('loadSampleRoadmapBtn')!.onclick = async () => {
    roadmapDraftMd = await invoke<string>('get_sample_roadmap_md');
    roadmapPreview = await invoke<RoadmapPreview>('preview_roadmap_md', { markdown: roadmapDraftMd });
    roadmapStatusMsg = '✓ Đã điền mẫu chuẩn XD4T-1ELTS & phân tích sẵn.';
    roadmapStatusOk = true;
    renderApp();
    document.getElementById('roadmapModal')!.classList.add('open');
  };

  document.getElementById('exportCurrentRoadmapBtn')!.onclick = async () => {
    roadmapDraftMd = await invoke<string>('export_current_roadmap_md');
    roadmapPreview = await invoke<RoadmapPreview>('preview_roadmap_md', { markdown: roadmapDraftMd });
    roadmapStatusMsg = '✓ Đã xuất lộ trình hiện tại ra khung Markdown.';
    roadmapStatusOk = true;
    renderApp();
    document.getElementById('roadmapModal')!.classList.add('open');
  };

  const roadmapFileInput = document.getElementById('roadmapFileInput') as HTMLInputElement;
  document.getElementById('pickRoadmapFileBtn')!.onclick = () => roadmapFileInput.click();
  roadmapFileInput.onchange = async () => {
    const file = roadmapFileInput.files?.[0];
    if (!file) return;
    roadmapDraftMd = await file.text();
    try {
      roadmapPreview = await invoke<RoadmapPreview>('preview_roadmap_md', { markdown: roadmapDraftMd });
      roadmapStatusMsg = `✓ Đã đọc file ${file.name} (${roadmapPreview.weeks_count} tuần).`;
      roadmapStatusOk = true;
    } catch (err) {
      roadmapPreview = null;
      roadmapStatusMsg = `✕ ${String(err)}`;
      roadmapStatusOk = false;
    }
    renderApp();
    document.getElementById('roadmapModal')!.classList.add('open');
  };

  document.getElementById('previewRoadmapBtn')!.onclick = async () => {
    roadmapDraftMd = roadmapMdInput.value;
    try {
      roadmapPreview = await invoke<RoadmapPreview>('preview_roadmap_md', { markdown: roadmapDraftMd });
      roadmapStatusMsg = '✓ Phân tích thành công! Kiểm tra bảng xem trước rồi bấm [ÁP DỤNG LỘ TRÌNH NGAY].';
      roadmapStatusOk = true;
    } catch (err) {
      roadmapPreview = null;
      roadmapStatusMsg = `✕ ${String(err)}`;
      roadmapStatusOk = false;
    }
    renderApp();
    document.getElementById('roadmapModal')!.classList.add('open');
  };

  document.getElementById('applyRoadmapBtn')!.onclick = async () => {
    roadmapDraftMd = roadmapMdInput.value;
    const replaceExisting = (document.getElementById('chkReplaceRoadmap') as HTMLInputElement).checked;
    const mergeVocabErrors = (document.getElementById('chkMergeVocabErrors') as HTMLInputElement).checked;
    try {
      dash = await invoke<DashboardState>('import_roadmap_md', {
        markdown: roadmapDraftMd,
        replaceExisting,
        mergeVocabErrors,
      });
      dueCards = await invoke<VocabCard[]>('get_due_vocab');
      cardIdx = 0;
      cardFlipped = false;
      roadmapStatusMsg = '✓ Đã áp dụng lộ trình mới!';
      roadmapStatusOk = true;
      roadmapModal.classList.remove('open');
      renderApp();
    } catch (err) {
      roadmapStatusMsg = `✕ ${String(err)}`;
      roadmapStatusOk = false;
      renderApp();
      document.getElementById('roadmapModal')!.classList.add('open');
    }
  };

  const scoreModal = document.getElementById('scoreModal')!;
  const openScoreForSkill = (skill?: string) => {
    if (skill) {
      const sel = document.getElementById('scSkill') as HTMLSelectElement | null;
      if (sel) sel.value = skill;
    }
    scoreModal.classList.add('open');
  };
  document.getElementById('openScoreModalBtn')!.onclick = () => openScoreForSkill();
  const openScoreTab1Btn = document.getElementById('openScoreTab1Btn');
  if (openScoreTab1Btn) openScoreTab1Btn.onclick = () => openScoreForSkill();
  document.querySelectorAll<HTMLElement>('[data-band-skill]').forEach((el) => {
    bindActivate(el, () => openScoreForSkill(el.dataset.bandSkill));
  });
  document.getElementById('closeScoreBtn')!.onclick = () => scoreModal.classList.remove('open');
  document.getElementById('cancelScoreBtn')!.onclick = () => scoreModal.classList.remove('open');
  scoreModal.onclick = (e) => {
    if (e.target === scoreModal) scoreModal.classList.remove('open');
  };
  document.getElementById('saveScoreBtn')!.onclick = async () => {
    const skill = (document.getElementById('scSkill') as HTMLSelectElement).value;
    const source = (document.getElementById('scSource') as HTMLInputElement).value.trim() || 'Practice';
    const score = parseFloat((document.getElementById('scVal') as HTMLInputElement).value) || 5.0;
    const note = (document.getElementById('scNote') as HTMLInputElement).value.trim();
    dash = await invoke<DashboardState>('add_practice_score', { skill, source, score, note });
    scoreModal.classList.remove('open');
    renderApp();
  };
}

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    const welcomeEl = document.getElementById('welcomeModal');
    if (welcomeEl?.classList.contains('open')) {
      welcomeOpen = false;
      void finishOnboarding();
    }
    document.querySelectorAll('.modal-backdrop.open').forEach((m) => m.classList.remove('open'));
  }
});

async function init() {
  dash = await invoke<DashboardState>('get_dashboard');
  dueCards = await invoke<VocabCard[]>('get_due_vocab');
  try {
    widgetInfo = await invoke<WidgetRuntimeInfo>('get_widget_runtime_info');
  } catch {
    widgetInfo = { running: false, autostart: false, exe_found: false };
  }
  if (dash && !dash.settings.onboarding_done) {
    welcomeOpen = true;
    welcomeStep = 1;
  }
  renderApp();

  await listen('start-pomodoro-15', () => {
    pomoSecs = 15 * 60;
    togglePomodoro(true);
  });
}

init();
