import './styles.css';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { openUrl } from '@tauri-apps/plugin-opener';
import pkg from '../package.json';
import donateQrUrl from './assets/donate-qr.png';
import {
  computeWordDiff,
  esc,
  evaluateErrorFixAnswer,
  extractDictationErrorPairs,
  highlightWordInExample,
  initTtsVoices,
  isTtsSupported,
  makeCardClozeHtml,
  speakText,
} from './diff';
import {
  parseChartSpec,
  renderChartInnerHtml,
  renderTask1VisualHtml,
  summarizeChartSpecForPrompt,
} from './charts';
import type {
  ApiTestResult,
  BackupResultDto,
  DashboardState,
  DictationResultInput,
  DictationSentence,
  DictationSubmitResult,
  ErrorDrillQuestion,
  ErrorDrillResultInput,
  ErrorDrillSubmitResult,
  ErrorFixAiGrade,
  GradeResponseDto,
  GradeResult,
  MicroCheckAnswerInput,
  MicroCheckItem,
  MicroCheckSubmitResult,
  QuizAnswerInput,
  QuizQuestion,
  QuizSubmitResult,
  RefillPackResult,
  RestoreResultDto,
  RoadmapPreview,
  SpeakingCue,
  SpeakingGradeResponse,
  VocabCard,
  WidgetRuntimeInfo,
  WritingPrompt,
} from './types';

const APP_VERSION = pkg.version;
const IELTS_TOPICS = [
  'Environment',
  'Technology & AI',
  'Education',
  'Health',
  'Economy',
  'Urbanization',
  'Media',
  'Crime & Law',
  'Work',
  'Culture',
  'Transport',
  'Globalization',
];

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

// --- App State ---
let dash: DashboardState | null = null;
let widgetInfo: WidgetRuntimeInfo = { running: false, autostart: false, exe_found: true };
let activeTab: 'today' | 'writing' | 'vocab' | 'roadmap' = 'today';

// Writing State
let writingMode = 'paragraph_90';
let selectedPromptId = 'p90-1';
let hintOpen = false;
let top3Collapsed = false;
let lastGrade: GradeResult | null = null;
let lastEndpoint = '';
let isGrading = false;
let writingStatusMsg = '';
let writingStatusErr = false;
let draftEssay = '';

// Tab 4 State
let roadmapRightTab: 'errors' | 'paraphrase' = 'errors';
let errFormMsg = '';
let errFormErr = false;
let paraFormMsg = '';
let paraFormErr = false;
let scrollToNewErr = false;

// Tab 3 Flashcard State
let dueCards: VocabCard[] = [];
let cardIdx = 0;
let cardFlipped = false;
let vocabSubMode: 'flashcard' | 'quiz' | 'dictation' = 'flashcard';
let cardFaceMode: 'word' | 'cloze' | 'listen' = 'word';
let onlyLeechFilter = false;
let leechFixCardId: number | null = null;
let leechFixDraft = '';
let leechFixMsg = '';
let addVocabOpen = false;
let addVocabWord = '';
let addVocabMeaning = '';
let addVocabExample = '';
let addVocabTopic = 'Environment';
let addVocabMsg = '';

// Tab 3 Quiz State
let quizPillMode: 'vocab' | 'fix' | 'micro' = 'vocab';
let quizMode: 'mixed' | 'mcq' | 'typing' = 'mixed';
let quizPhase: 'setup' | 'playing' | 'summary' = 'setup';
let quizQuestions: QuizQuestion[] = [];
let quizIdx = 0;
let quizAnswers: QuizAnswerInput[] = [];
let quizFeedback: { correct: boolean; gaveUp: boolean; selectedOpt?: string; typedText?: string } | null = null;
let quizSummary: QuizSubmitResult | null = null;
let quizLoading = false;
let quizSubmitting = false;
let quizErrorMsg = '';

type ErrorFixFeedbackState = {
  correct: boolean;
  target: string;
  matchReason?: string;
  diffHtml?: string;
  selfAccepted?: boolean;
  aiChecking?: boolean;
  aiFeedback?: string;
  aiError?: string;
};

// Error-Driven Mastery Drill State
let errorDrillBatch: ErrorDrillQuestion[] = [];
let errorDrillIdx = 0;
let errorDrillInput = '';
let errorDrillAnswers: ErrorDrillResultInput[] = [];
let errorDrillFeedback: (ErrorFixFeedbackState & {
  selectedOpt?: string;
  rationaleVi: string;
  streakAfter?: number;
  masteredNow?: boolean;
}) | null = null;
let errorDrillSummary: ErrorDrillSubmitResult | null = null;
let errorDrillLoading = false;

// Flashcard safety & Undo state
let confirmDeleteCardId: number | null = null;
let confirmDeleteTimer: number | null = null;
let undoHideCard: { id: number; word: string } | null = null;
let undoHideTimer: number | null = null;

// Micro-Check State
let microQuestions: MicroCheckItem[] = [];
let microIdx = 0;
let microAnswers: MicroCheckAnswerInput[] = [];
let microInput = '';
let microFeedback: (ErrorFixFeedbackState & {
  gaveUp: boolean;
  note: string;
  isErrorFix?: boolean;
}) | null = null;
let microSummary: MicroCheckSubmitResult | null = null;
let microSecsLeft = 120;
let microTimer: number | null = null;
let microTimeExpired = false;

// Dictation State
let dictationBatch: DictationSentence[] = [];
let dictationIdx = 0;
let dictationRate = 0.85;
let dictationInput = '';
let dictationHintWords = 0;
let dictationAutoPlay = true;
let dictationChecked: {
  pct: number;
  tokensHtml: string;
  wrongWords: string[];
  pairs: { wrong: string; right: string }[];
  pushed: boolean;
} | null = null;
let dictationResults: DictationResultInput[] = [];
let dictationSummary: DictationSubmitResult | null = null;

// Speaking Studio State
type SpeakingPlanKey = 'real' | '432' | 'p3';
const SPEAKING_PLANS: Record<SpeakingPlanKey, { label: string; stages: { name: string; secs: number; record: boolean }[] }> = {
  real: {
    label: '🎯 Thi thật (Part 2 + Part 3)',
    stages: [
      { name: 'Chuẩn bị', secs: 60, record: false },
      { name: 'Part 2', secs: 120, record: true },
      { name: 'Part 3 · C1', secs: 45, record: true },
      { name: 'Part 3 · C2', secs: 45, record: true },
    ],
  },
  '432': {
    label: '⚡ Phản xạ 4-3-2',
    stages: [
      { name: 'Chuẩn bị', secs: 60, record: false },
      { name: 'Nói 1', secs: 120, record: true },
      { name: 'Nói 2', secs: 90, record: true },
      { name: 'Nói 3', secs: 60, record: true },
    ],
  },
  p3: {
    label: 'Part 3 · 2×45s',
    stages: [
      { name: 'Câu 1', secs: 45, record: true },
      { name: 'Câu 2', secs: 45, record: true },
    ],
  },
};

let studioOpen = false;
let studioSource: 'bank' | 'custom' = 'bank';
let studioCue: SpeakingCue | null = null;
let customCueTitle = 'Describe a time you solved a difficult problem.';
let customPart3 = 'Why do some people avoid complex problems?\nHow can schools teach analytical thinking?';
let studioPrepNotes = '';
let studioPlanKey: SpeakingPlanKey = 'real';
let studioStageIdx = 0;
let studioSecsLeft = 60;
let studioTimer: number | null = null;
let studioRunning = false;
let mediaRecorder: MediaRecorder | null = null;
let mediaStream: MediaStream | null = null;
let currentChunks: BlobPart[] = [];
let recordedBlobs: { stageName: string; blob: Blob; url: string }[] = [];
let selectedClipIdx = 0;
let speakingHistoryOpen = false;
let studioPlaybackAudio: HTMLAudioElement | null = null;
let isGradingSpeaking = false;
let speakingGradeOut: SpeakingGradeResponse | null = null;
let speakingStatusMsg = '';
let speakingStatusErr = false;
let manualCheckOpen = false;
let manualChecks = [false, false, false, false, false];
let manualBandFc = '6.0';
let manualBandLr = '6.0';
let manualBandGra = '5.5';
let manualBandP = '6.0';
let manualNote = '';
let manualTranscript = '';

let aiPackOpen = false;
let aiPackTopic = 'Environment';
let aiPackLoading = false;
let aiPackResult: RefillPackResult | null = null;
let aiPackError = '';
let shortcutsModalOpen = false;
let generatingAiDrillErrId: number | null = null;
let autoRefillTriggeredThisSession = false;
let autoRefillToastMsg = '';
let chartZoomOpen = false;
let customTask1ImageDataUrl = '';

let lastSyncedWindowTitle = '';
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

// Modal A11y Focus Trap & Trigger Restoration State
let lastModalTriggerEl: HTMLElement | null = null;
let lastModalTriggerKey: string | null = null;
let pendingModalFocus = false;
let pendingRestoreFocus = false;

function getActiveDialogEl(): HTMLElement | null {
  const candidates = Array.from(
    document.querySelectorAll<HTMLElement>('.modal-backdrop.open .modal-box, .studio-overlay.on, .v-drawer.on')
  );
  let bestEl: HTMLElement | null = null;
  let bestZ = -Infinity;
  for (const el of candidates) {
    const layer = el.closest<HTMLElement>('.modal-backdrop') || el;
    const parsed = Number.parseInt(window.getComputedStyle(layer).zIndex, 10);
    const z = Number.isFinite(parsed) ? parsed : 0;
    if (z >= bestZ) {
      bestZ = z;
      bestEl = el;
    }
  }
  return bestEl;
}

function describeTriggerElement(el: HTMLElement | null): string | null {
  if (!el || el === document.body) return null;
  if (el.id) return `#${CSS.escape(el.id)}`;
  const attrNames = ['data-band-skill', 'data-go', 'data-tab', 'data-vsub', 'data-rtab'];
  for (const attr of attrNames) {
    const val = el.getAttribute(attr);
    if (val) return `[${attr}="${CSS.escape(val)}"]`;
  }
  return null;
}

function rememberModalTrigger(): void {
  if (getActiveDialogEl()) return;
  const active = document.activeElement as HTMLElement | null;
  if (active && active !== document.body && document.getElementById('app')?.contains(active)) {
    lastModalTriggerEl = active;
    lastModalTriggerKey = describeTriggerElement(active);
  } else {
    lastModalTriggerEl = null;
    lastModalTriggerKey = null;
  }
}

function getFocusableElements(container: HTMLElement): HTMLElement[] {
  const selector =
    'button:not([disabled]), [href], input:not([disabled]):not([type="hidden"]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"]), summary';
  return Array.from(container.querySelectorAll<HTMLElement>(selector)).filter(
    (el) => el.offsetParent !== null || el.getClientRects().length > 0
  );
}

function focusActiveDialog(): void {
  const dlg = getActiveDialogEl();
  if (!dlg) return;
  const focusables = getFocusableElements(dlg);
  if (focusables.length > 0) {
    focusables[0].focus();
  } else {
    dlg.focus();
  }
}

function restoreModalTriggerFocus(): void {
  const candidate =
    lastModalTriggerEl && lastModalTriggerEl.isConnected
      ? lastModalTriggerEl
      : lastModalTriggerKey
        ? document.querySelector<HTMLElement>(lastModalTriggerKey)
        : null;
  lastModalTriggerEl = null;
  lastModalTriggerKey = null;
  if (candidate && candidate.isConnected) {
    candidate.focus();
  }
}

function openClassModal(modalEl: HTMLElement): void {
  rememberModalTrigger();
  modalEl.classList.add('open');
  focusActiveDialog();
}

function closeClassModal(modalEl: HTMLElement): void {
  modalEl.classList.remove('open');
  if (!getActiveDialogEl()) {
    restoreModalTriggerFocus();
  }
}

function applyThemeInPlace(rawTheme: string): void {
  const nextTheme = normalizeTheme(rawTheme);
  document.documentElement.className = `theme-${nextTheme}`;
  const shell = document.getElementById('shell');
  if (shell) shell.className = `app-shell theme-${nextTheme}`;
  const themeSel = document.getElementById('themeSelect') as HTMLSelectElement | null;
  if (themeSel) themeSel.value = nextTheme;
  document.querySelectorAll<HTMLButtonElement>('.settings-theme-card, .welcome-theme-card').forEach((btn) => {
    btn.classList.toggle('active', btn.dataset.pickTheme === nextTheme);
  });
}

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

function getCurrentPrompt(): WritingPrompt | undefined {
  if (!dash) return undefined;
  return (
    dash.prompts.find((p) => p.id === selectedPromptId) ||
    dash.prompts.find((p) => p.mode === writingMode) ||
    dash.prompts[0]
  );
}

function isAim80Mode(): boolean {
  return (dash?.settings.target_band ?? 7.0) >= 7.5;
}

function renderLevelCards(is80: boolean, extraClass: string): string {
  return `
    <div class="level-cards">
      <button type="button" class="level-card ${extraClass} ${!is80 ? 'active' : ''}" data-pick-level="6.5">
        <div class="level-card-title">🎯 Lộ trình Tiêu chuẩn · Aim 6.5 (Vùng đệm 7.0)</div>
        <div class="level-card-desc">Dành cho nền 4.5–5.5 · Diệt lỗi gốc, nắm 5 khung Writing &amp; săn 30–34/40 câu</div>
      </button>
      <button type="button" class="level-card ${extraClass} ${is80 ? 'active' : ''}" data-pick-level="8.0">
        <div class="level-card-title">🚀 Lộ trình Nâng cao · Aim 8.0 (Độ khó cao)</div>
        <div class="level-card-desc">Khuyến khích khi nền đã vững 6.5+ · Từ vựng C1–C2 &amp; Hedging, săn 36–39/40 câu</div>
      </button>
    </div>
    ${
      is80
        ? `<div class="level-warning-box">⚠️ Khuyến khích dành cho bạn đã vững nền Band 6.5+ (Listening/Reading ≥ 27–30/40 câu). Nếu đang ở mức 5.0–6.0, nên chọn Aim 6.5 trước.</div>`
        : ''
    }
  `;
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

function questTargetFromHint(hint: string, title: string): string {
  switch (hint) {
    case 'srs':
      return 'vocab:flash';
    case 'micro_check':
      return 'vocab:quiz:micro';
    case 'quiz':
      return 'vocab:quiz:vocab';
    case 'dictation':
      return 'vocab:dict';
    case 'speaking':
      return 'speak';
    case 'writing':
      return 'writing';
    case 'paraphrase':
      return 'roadmap:para';
    case 'study4':
      return 'study4';
    default: {
      const low = title.toLowerCase();
      if (low.includes('nói') || low.includes('speaking')) return 'speak';
      if (low.includes('nghe chép') || low.includes('dictation')) return 'vocab:dict';
      if (low.includes('micro')) return 'vocab:quiz:micro';
      if (low.includes('quiz')) return 'vocab:quiz:vocab';
      if (low.includes('writing') || low.includes('viết')) return 'writing';
      if (low.includes('paraphrase')) return 'roadmap:para';
      if (low.includes('study4')) return 'study4';
      return 'vocab:flash';
    }
  }
}

function fmtMMSS(secs: number): string {
  const s = Math.max(0, Math.floor(secs));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
}

function stopMicroTimer(): void {
  if (microTimer !== null) {
    window.clearInterval(microTimer);
    microTimer = null;
  }
}

function stopStudioTimer(): void {
  if (studioTimer !== null) {
    window.clearInterval(studioTimer);
    studioTimer = null;
  }
}

function revokeRecordedBlobs(): void {
  for (const item of recordedBlobs) {
    try {
      URL.revokeObjectURL(item.url);
    } catch {
      // ignore
    }
  }
  recordedBlobs = [];
  selectedClipIdx = 0;
}

function stopMediaRecorderIfActive(): void {
  if (mediaRecorder && mediaRecorder.state !== 'inactive') {
    try {
      mediaRecorder.stop();
    } catch {
      // ignore
    }
  }
  if (mediaStream) {
    mediaStream.getTracks().forEach((t) => t.stop());
    mediaStream = null;
  }
}

async function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => {
      const res = String(reader.result || '');
      const comma = res.indexOf(',');
      resolve(comma >= 0 ? res.slice(comma + 1) : res);
    };
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(blob);
  });
}

async function startRecordingForStage(stageName: string): Promise<void> {
  if (!navigator.mediaDevices?.getUserMedia || typeof MediaRecorder === 'undefined') {
    speakingStatusMsg = 'Trình duyệt không hỗ trợ ghi âm trực tiếp. Bạn vẫn có thể luyện bấm giờ & dùng Tự chấm 5 mục.';
    speakingStatusErr = false;
    return;
  }
  try {
    if (!mediaStream) {
      mediaStream = await navigator.mediaDevices.getUserMedia({ audio: true });
    }
    const mime = MediaRecorder.isTypeSupported('audio/webm;codecs=opus')
      ? 'audio/webm;codecs=opus'
      : 'audio/webm';
    currentChunks = [];
    mediaRecorder = new MediaRecorder(mediaStream, {
      mimeType: mime,
      audioBitsPerSecond: 32000,
    });
    mediaRecorder.ondataavailable = (ev) => {
      if (ev.data && ev.data.size > 0) currentChunks.push(ev.data);
    };
    mediaRecorder.onstop = () => {
      if (currentChunks.length > 0) {
        const blob = new Blob(currentChunks, { type: mime });
        const url = URL.createObjectURL(blob);
        recordedBlobs.push({ stageName, blob, url });
        selectedClipIdx = recordedBlobs.length - 1;
      }
      currentChunks = [];
      renderApp();
    };
    mediaRecorder.start(500);
  } catch (err) {
    speakingStatusMsg = `Không truy cập được Micro (${String(err)}). Đang chạy chế độ bấm giờ không ghi âm.`;
    speakingStatusErr = false;
  }
}

function finishCurrentRecordingClip(): void {
  if (mediaRecorder && mediaRecorder.state === 'recording') {
    try {
      mediaRecorder.stop();
    } catch {
      // ignore
    }
  }
}

async function loadSpeakingCue(randomNext = false): Promise<void> {
  if (!dash) return;
  try {
    studioCue = await invoke<SpeakingCue>('get_speaking_cue', {
      weekNum: dash.current_week,
      randomNext,
      excludeId: randomNext && studioCue ? studioCue.id : null,
    });
  } catch (err) {
    speakingStatusMsg = `Không tải được Cue Card: ${String(err)}`;
    speakingStatusErr = true;
  }
}

async function openSpeakingStudio(): Promise<void> {
  rememberModalTrigger();
  stopMicroTimer();
  window.speechSynthesis?.cancel();
  studioOpen = true;
  speakingGradeOut = null;
  speakingStatusMsg = '';
  speakingStatusErr = false;
  manualCheckOpen = false;
  if (!studioCue) {
    await loadSpeakingCue(false);
  }
  pendingModalFocus = true;
  renderApp();
}

function closeSpeakingStudio(): void {
  if (studioRunning) {
    stopStudioTimer();
    studioRunning = false;
    finishCurrentRecordingClip();
  }
  stopMediaRecorderIfActive();
  studioPlaybackAudio?.pause();
  studioPlaybackAudio = null;
  revokeRecordedBlobs();
  studioOpen = false;
  pendingRestoreFocus = true;
  renderApp();
}

function validateSpeakingCustomCue(): boolean {
  if (studioSource === 'custom' && !customCueTitle.trim()) {
    speakingStatusMsg = '✕ Vui lòng nhập đề Cue Card Part 2 vào ô "Đề của tôi" trước.';
    speakingStatusErr = true;
    renderApp();
    document.getElementById('customCueIn')?.focus();
    return false;
  }
  return true;
}

async function switchStudyLevel(level: string): Promise<void> {
  stopMicroTimer();
  window.speechSynthesis?.cancel();
  dash = await invoke<DashboardState>('set_study_level', { level });
  dictationBatch = [];
  dictationIdx = 0;
  dictationInput = '';
  dictationHintWords = 0;
  dictationChecked = null;
  dictationResults = [];
  dictationSummary = null;
  studioCue = null;
  speakingGradeOut = null;
  errorDrillBatch = [];
  errorDrillIdx = 0;
  errorDrillInput = '';
  errorDrillAnswers = [];
  errorDrillFeedback = null;
  errorDrillSummary = null;
  microQuestions = [];
  microIdx = 0;
  microAnswers = [];
  microInput = '';
  microFeedback = null;
  microSummary = null;
  microTimeExpired = false;
  lastGrade = null;
  lastEndpoint = '';
  const matchPrompt = dash.prompts.find((p) => p.mode === writingMode) || dash.prompts[0];
  if (matchPrompt) {
    writingMode = matchPrompt.mode;
    selectedPromptId = matchPrompt.id;
  }
  dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
  cardIdx = 0;
  cardFlipped = false;
}

function openAiPackModal(): void {
  rememberModalTrigger();
  const weekIdx = dash ? (Math.max(1, dash.current_week) - 1) % IELTS_TOPICS.length : 0;
  aiPackTopic = IELTS_TOPICS[weekIdx];
  aiPackResult = null;
  aiPackError = '';
  aiPackOpen = true;
  pendingModalFocus = true;
  renderApp();
}

async function goToTarget(target: string): Promise<void> {
  stopMicroTimer();
  if (target === 'study4') {
    void openUrl('https://study4.com/tests/ielts/');
    return;
  }
  const [t, s, m] = target.split(':');
  if (t === 'speak') {
    await openSpeakingStudio();
    return;
  }
  if (t === 'writing') {
    activeTab = 'writing';
    renderApp();
    return;
  }
  if (t === 'roadmap') {
    activeTab = 'roadmap';
    roadmapRightTab = s === 'para' ? 'paraphrase' : 'errors';
    renderApp();
    return;
  }
  if (t === 'vocab') {
    activeTab = 'vocab';
    if (s === 'dict') {
      vocabSubMode = 'dictation';
      if (!dictationBatch.length) {
        dictationBatch = await invoke<DictationSentence[]>('get_dictation_batch', { count: 5 });
        dictationIdx = 0;
        dictationChecked = null;
        dictationResults = [];
        dictationSummary = null;
      }
    } else if (s === 'quiz') {
      vocabSubMode = 'quiz';
      if (m === 'fix') {
        quizPillMode = 'fix';
        await loadErrorDrills();
      } else if (m === 'micro') {
        quizPillMode = 'micro';
        await startMicroCheck();
      } else {
        quizPillMode = 'vocab';
      }
    } else {
      vocabSubMode = 'flashcard';
      onlyLeechFilter = m === 'leech';
      [dueCards, dash] = await Promise.all([
        invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter }),
        invoke<DashboardState>('get_dashboard'),
      ]);
      cardIdx = 0;
      cardFlipped = false;
    }
    renderApp();
  }
}

async function loadErrorDrills(): Promise<void> {
  errorDrillLoading = true;
  renderApp();
  try {
    errorDrillBatch = await invoke<ErrorDrillQuestion[]>('generate_error_drills', { count: 5 });
    errorDrillIdx = 0;
    errorDrillInput = '';
    errorDrillAnswers = [];
    errorDrillFeedback = null;
    errorDrillSummary = null;
  } finally {
    errorDrillLoading = false;
  }
}

async function finishMicroEarly(): Promise<void> {
  stopMicroTimer();
  microTimeExpired = false;
  microSummary = await invoke<MicroCheckSubmitResult>('submit_micro_check', { results: microAnswers });
  dash = await invoke<DashboardState>('get_dashboard');
  dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
  microFeedback = null;
  renderApp();
}

async function startMicroCheck(): Promise<void> {
  stopMicroTimer();
  microQuestions = await invoke<MicroCheckItem[]>('generate_micro_check');
  microIdx = 0;
  microAnswers = [];
  microInput = '';
  microFeedback = null;
  microSummary = null;
  microSecsLeft = 120;
  microTimeExpired = false;
  microTimer = window.setInterval(() => {
    if (microSecsLeft > 1) {
      microSecsLeft -= 1;
      const el = document.getElementById('microTimerDisplay');
      if (el) el.textContent = `⏱ ${fmtMMSS(microSecsLeft)}`;
    } else {
      microSecsLeft = 0;
      microTimeExpired = true;
      stopMicroTimer();
      const el = document.getElementById('microTimerDisplay');
      if (el) el.textContent = '⏱ 0:00';
      const host = document.getElementById('microExpiredHost');
      if (host) {
        host.innerHTML = `
          <div class="micro-expired-banner">
            <span>⏱ <b>Hết 2 phút</b> — Bấm Xem kết quả hoặc làm nốt câu đang dở</span>
            <button type="button" class="btn btn-sm btn-primary" id="microFinishEarlyBtn">Xem kết quả ngay →</button>
          </div>`;
        document.getElementById('microFinishEarlyBtn')?.addEventListener('click', () => {
          void finishMicroEarly();
        });
      }
    }
  }, 1000);
}

// --- Renderers for Tab 3 Sub-modes ---
function renderFlashcardMode(curTheme: ThemeId): string {
  if (!dash) return '';
  const isAim80 = isAim80Mode();
  const curCard = dueCards[cardIdx];
  const ttsOk = isTtsSupported();
  const accent = dash.settings.tts_accent || 'en-GB';
  const isLeech = curCard ? curCard.lapses >= 3 : false;
  const leechCount = Math.max(dash.leech_count, dueCards.filter((c) => c.lapses >= 3).length);

  return `
    <div class="v-toolbar">
      <span class="label-uc">Mặt thẻ</span>
      <div class="seg" role="group" aria-label="Kiểu mặt thẻ">
        <button type="button" aria-pressed="${cardFaceMode === 'word'}" data-face="word">Mặt từ</button>
        <button type="button" aria-pressed="${cardFaceMode === 'cloze'}" data-face="cloze">Câu đục lỗ</button>
        <button type="button" aria-pressed="${cardFaceMode === 'listen'}" data-face="listen">🎧 Nghe trước</button>
      </div>
      <button type="button" class="chip" id="leechChip" aria-pressed="${onlyLeechFilter}">
        🔥 Từ cứng đầu (${leechCount})
      </button>
      <span class="sp"></span>
      <span class="small muted">Giọng: Cài đặt › ${esc(accent)}</span>
      <button type="button" class="btn btn-primary" id="openAddVocabBtn">+ Thêm từ</button>
    </div>

    <div class="stage-wrap">
      ${
        undoHideCard
          ? `<div class="fc-undo-toast">
              <span>🙈 Đã ẩn thẻ <b>“${esc(undoHideCard.word)}”</b> khỏi hàng đợi.</span>
              <button type="button" class="btn btn-sm btn-primary" id="undoHideCardBtn">↺ Hoàn tác (Undo)</button>
            </div>`
          : ''
      }
      ${
        curCard
          ? `
        <div class="fc-card" data-theme-ref="${curTheme}">
          <div class="fc-corner">
            <span class="row" style="gap:6px;">
              ${
                isLeech
                  ? `<span class="tag leech" title="Quên ≥ 3 lần">🔥 Từ cứng đầu · quên ${curCard.lapses} lần</span>`
                  : `<span class="tag soft">${esc(curCard.topic.toUpperCase())} · ${isAim80 ? 'BAND 8.0' : 'BAND 6.5'} · REPS ${curCard.reps}</span>`
              }
              ${curCard.source === 'ai_pack' ? `<span class="tag ai-pack">✨ Mới nạp</span>` : ''}
            </span>
            <span class="row" style="gap:6px;align-items:center;">
              ${
                isLeech
                  ? `<button type="button" class="btn btn-sm" id="openLeechFixBtn" style="border-color:var(--v16-leech);color:var(--v16-leech);font-weight:700;">✍️ Chữa từ này <kbd>C</kbd></button>`
                  : ''
              }
              <button type="button" class="btn ghost ibtn" id="hideVocabBtn" title="Ẩn thẻ khỏi hàng đợi (có Hoàn tác)">🙈</button>
              <span class="fc-danger-zone">
                <button type="button" class="btn ghost ibtn ${confirmDeleteCardId === curCard.id ? 'btn-danger-confirm' : ''}" id="deleteVocabBtn" title="${confirmDeleteCardId === curCard.id ? 'Bấm lại để xác nhận xóa vĩnh viễn' : 'Xóa thẻ'}">
                  ${confirmDeleteCardId === curCard.id ? 'Xác nhận xóa?' : '🗑'}
                </button>
              </span>
            </span>
          </div>

          ${
            cardFaceMode === 'cloze'
              ? `
                <div class="fc-ex">${makeCardClozeHtml(curCard)}</div>
                ${
                  cardFlipped
                    ? `<div class="fc-word-lg" style="margin-top:6px;">${esc(curCard.word)} ${ttsOk ? `<button type="button" class="btn ibtn" data-say="${esc(curCard.word)}" title="Đọc từ [R]">🔊 <kbd>R</kbd></button>` : ''}</div>
                       <div class="small muted">${esc(curCard.meaning)}</div>`
                    : `<div class="small muted">Nhớ lại từ còn thiếu rồi bấm Lật thẻ <kbd>Space</kbd></div>`
                }
              `
              : cardFaceMode === 'listen'
                ? `
                  ${
                    ttsOk
                      ? `<button type="button" class="btn btn-primary lg" id="listenReplayBtn" data-say="${esc(curCard.word)}">🔊 Nghe lại <kbd style="color:inherit;border-color:currentColor;">R</kbd></button>`
                      : `<div class="small muted">Thiết bị chưa bật TTS — lật thẻ để xem chữ</div>`
                  }
                  <div class="${cardFlipped ? '' : 'masked'}">
                    <div class="fc-word-lg">${esc(curCard.word)}</div>
                    <div class="small muted" style="margin-top:4px;">${esc(curCard.meaning)}</div>
                  </div>
                  <div class="fc-ex ${cardFlipped ? '' : 'masked'}">
                    ${highlightWordInExample(curCard.example, curCard.word)}
                    ${ttsOk && cardFlipped ? `<button type="button" class="btn ghost ibtn" data-say="${esc(curCard.example)}" title="Đọc câu [E]">🔉 <kbd>E</kbd></button>` : ''}
                  </div>
                  ${!cardFlipped ? `<div class="small muted">Mặt chữ ẩn tới khi lật thẻ</div>` : ''}
                `
                : `
                  <div class="row" style="justify-content:center;">
                    <span class="fc-word-lg">${esc(curCard.word)}</span>
                    ${ttsOk ? `<button type="button" class="btn ibtn" data-say="${esc(curCard.word)}" title="Đọc từ [R]">🔊 <kbd>R</kbd></button>` : ''}
                  </div>
                  ${
                    cardFlipped
                      ? `<div style="font-weight:700;font-size:15px;">${esc(curCard.meaning)}</div>
                         <div class="fc-ex">
                           ${highlightWordInExample(curCard.example, curCard.word)}
                           ${ttsOk ? `<button type="button" class="btn ghost ibtn" data-say="${esc(curCard.example)}" title="Đọc câu [E]">🔉 <kbd>E</kbd></button>` : ''}
                         </div>`
                      : `<div class="small muted">(Bấm Lật thẻ hoặc nhấn Space để xem nghĩa &amp; ví dụ)</div>`
                  }
                `
          }

          <div class="row" style="margin-top:6px;">
            <button type="button" class="btn" id="flipCardBtn">${cardFlipped ? 'Úp thẻ' : 'Lật thẻ'} <kbd>Space</kbd></button>
            <span class="small muted">Thẻ ${cardIdx + 1} / ${dueCards.length}</span>
          </div>
          <div class="srs-row" style="width:100%;max-width:560px;">
            <button type="button" class="btn srs-btn" data-q="0">[1] QUÊN · 10p</button>
            <button type="button" class="btn srs-btn" data-q="1">[2] KHÓ · 1 ngày</button>
            <button type="button" class="btn srs-btn" data-q="2">[3] NHỚ · 3 ngày</button>
            <button type="button" class="btn btn-primary srs-btn" data-q="3">[4] DỄ · 7 ngày</button>
          </div>
        </div>

        ${
          leechFixCardId === curCard.id
            ? `
          <div class="fixbox" id="fixBox">
            <div class="row" style="justify-content:space-between;">
              <b class="small">✍️ Tự đặt 1 câu mới với “${esc(curCard.word)}”</b>
              <span class="small muted">→ reset lượt quên &amp; lưu thành câu đục lỗ mới</span>
            </div>
            <textarea class="text-input" id="leechSentenceInput" rows="2" style="width:100%;margin-top:8px;" placeholder="Ví dụ: Planting more trees can help ${esc(curCard.word)} urban heat...">${esc(leechFixDraft)}</textarea>
            ${leechFixMsg ? `<div class="status-msg err">${esc(leechFixMsg)}</div>` : ''}
            <div class="row" style="justify-content:flex-end;margin-top:8px;">
              <button type="button" class="btn btn-sm" id="cancelLeechFixBtn">Bỏ</button>
              <button type="button" class="btn btn-sm btn-primary" id="saveLeechFixBtn">Lưu thẻ Cloze</button>
            </div>
          </div>`
            : ''
        }
      `
          : `
        <div class="box" style="padding:30px;text-align:center;color:var(--m-muted);">
          ${
            onlyLeechFilter
              ? '🎉 Tuyệt vời! Không còn từ cứng đầu (quên ≥ 3 lần) nào cần chữa.'
              : '🎉 Tuyệt vời! Bạn đã ôn hết toàn bộ thẻ đến hạn hôm nay.'
          }
          ${
            onlyLeechFilter
              ? `<div style="margin-top:10px;"><button type="button" class="btn btn-sm" id="clearLeechFilterBtn">Quay lại ôn tất cả thẻ</button></div>`
              : ''
          }
        </div>`
      }
    </div>`;
}

function buildBestWordDiffHtml(
  userInput: string,
  variants: string[],
  fallbackTarget: string
): string {
  const candidates = variants.length > 0 ? variants : [fallbackTarget];
  let best = computeWordDiff(userInput, candidates[0]);
  for (let i = 1; i < candidates.length; i++) {
    const cur = computeWordDiff(userInput, candidates[i]);
    if (cur.pct > best.pct) best = cur;
  }
  return best.tokens
    .map((t) =>
      t.kind === 'ok'
        ? `<span class="w-ok">${esc(t.word)}</span>`
        : t.kind === 'miss'
          ? `<span class="w-fix">${esc(t.word)}</span>`
          : `<span class="w-bad">${esc(t.word)}</span>`
    )
    .join(' ');
}

function renderErrorFixFallbackBlock(
  fb: ErrorFixFeedbackState,
  selfBtnId: string,
  aiBtnId: string
): string {
  const hasKey = Boolean(dash?.settings.has_api_key);
  return `
    ${fb.diffHtml ? `<div class="diff-box" style="margin-top:8px;font-size:14px;line-height:1.7;padding:8px 10px;">${fb.diffHtml}</div>` : ''}
    ${fb.aiFeedback ? `<div class="small" style="margin-top:6px;"><b>AI nhận xét:</b> ${esc(fb.aiFeedback)}</div>` : ''}
    ${fb.aiError ? `<div class="small" style="margin-top:6px;color:var(--m-err);">${esc(fb.aiError)}</div>` : ''}
    ${
      !fb.correct
        ? `<div class="row" style="gap:8px;margin-top:8px;flex-wrap:wrap;">
            <button type="button" class="btn btn-sm" id="${selfBtnId}">✓ Tôi sửa đúng nghĩa (Tính đúng)</button>
            ${
              hasKey
                ? `<button type="button" class="btn btn-sm" id="${aiBtnId}" ${fb.aiChecking ? 'disabled' : ''}>${fb.aiChecking ? '⏳ Đang nhờ AI chấm...' : '✨ Nhờ AI chấm câu sửa'}</button>`
                : ''
            }
          </div>`
        : ''
    }`;
}

function renderErrorDrillQuiz(): string {
  if (errorDrillSummary) {
    return `
      <div class="stage-wrap box">
        <div class="quiz-summary-hero">
          <div class="quiz-summary-score mono-num">${errorDrillSummary.correct_count} / ${errorDrillSummary.total_questions} · ${errorDrillSummary.accuracy_pct}%</div>
          <div class="small muted">
            ✓ Đã cập nhật tiến độ trị lỗi thực chiến${errorDrillSummary.marked_quest_done ? ' · ✓ Đã tự động tick xong nhiệm vụ Quiz hôm nay!' : ''}
          </div>
          ${
            errorDrillSummary.newly_mastered.length > 0
              ? `<div class="tag mastered" style="margin-top:8px;">✅ Đã trị dứt điểm: ${errorDrillSummary.newly_mastered.map((c) => esc(c)).join(', ')}</div>`
              : ''
          }
        </div>
        <div style="text-align:center;">
          <button type="button" class="btn btn-primary" id="restartErrDrillBtn">LUYỆN LƯỢT LỖI MỚI → <kbd style="color:inherit;border-color:currentColor;">Enter</kbd></button>
        </div>
      </div>`;
  }
  const q = errorDrillBatch[errorDrillIdx];
  if (!q) {
    return `
      <div class="stage-wrap box" style="text-align:center;padding:24px;">
        <div class="small muted" style="margin-bottom:10px;">Luyện sửa lỗi thực chiến qua Trắc nghiệm IELTS, Mini-Case Reading/Listening/Speaking &amp; Điền từ tiếng Anh (đúng 3 lần khác ngữ cảnh → trị dứt điểm).</div>
        <button type="button" class="btn btn-primary" id="startErrDrillBtn" ${errorDrillLoading ? 'disabled' : ''}>
          ${errorDrillLoading ? 'ĐANG TẢI BÀI TẬP...' : 'BẮT ĐẦU LUYỆN LỖI SAI (5 CÂU) →'} <kbd style="color:inherit;border-color:currentColor;">Enter</kbd>
        </button>
      </div>`;
  }
  const tot = errorDrillBatch.length || 1;
  const pct = Math.round(((errorDrillIdx + (errorDrillFeedback ? 1 : 0)) / tot) * 100);
  const modeLabel =
    q.drill_mode === 'TYPING'
      ? 'TYPING · ĐIỀN TỪ'
      : q.drill_mode === 'MINI_CASE'
        ? 'MINI-CASE · PHÒNG THI'
        : 'GRAMMAR MCQ · 4 ĐÁP ÁN';

  return `
    <div class="stage-wrap box">
      <div class="row small muted" style="justify-content:space-between;flex-wrap:wrap;gap:6px;">
        <span>Câu ${errorDrillIdx + 1}/${tot} · <span class="src-badge">${esc(q.category)}</span> · <span class="tag soft">${esc(q.context_tag)}</span></span>
        <span class="row" style="gap:6px;">
          <span class="quiz-type-badge">${modeLabel}</span>
          <span class="tag ${q.status === 'mastered' ? 'mastered' : 'active-err'}">${q.status === 'mastered' ? '✅ Đã trị' : `🔴 Trị ${q.streak_distinct}/3`}</span>
        </span>
      </div>
      <div class="bar-track" style="margin:8px 0 10px;"><div class="bar-fill" style="width:${pct}%"></div></div>
      <div class="drill-stem-box">${q.drill_mode === 'TYPING' ? renderClozeHtml(q.stem) : esc(q.stem)}</div>

      ${
        q.drill_mode === 'TYPING'
          ? `
            <div class="quiz-typing-row">
              <input type="text" class="quiz-typing-input" id="errDrillInput" placeholder="Gõ 1–2 từ tiếng Anh chuẩn..." value="${esc(errorDrillInput)}" ${errorDrillFeedback ? 'disabled' : ''} />
              ${
                !errorDrillFeedback
                  ? `<button type="button" class="btn btn-primary" id="checkErrDrillBtn">Kiểm tra <kbd style="color:inherit;border-color:currentColor;">Enter</kbd></button>`
                  : ''
              }
            </div>`
          : `
            <div class="quiz-options-grid">
              ${q.options
                .map((opt, i) => {
                  let cls = 'quiz-opt-btn';
                  if (errorDrillFeedback) {
                    if (opt === q.answer) cls += ' correct';
                    else if (errorDrillFeedback.selectedOpt === opt) cls += ' wrong';
                  }
                  return `<button type="button" class="${cls}" data-edrill-opt="${i}" ${errorDrillFeedback ? 'disabled' : ''}>
                    <kbd>${i + 1}</kbd> ${String.fromCharCode(65 + i)}. ${esc(opt)}
                  </button>`;
                })
                .join('')}
            </div>`
      }

      ${
        errorDrillFeedback
          ? `<div class="quiz-feedback ${errorDrillFeedback.correct ? 'ok' : 'err'}" style="margin-top:10px;">
              <div>
                <b>${errorDrillFeedback.correct ? '✓ Chính xác!' : '✗ Chưa đúng'}</b>
                ${errorDrillFeedback.matchReason ? ` <span class="small muted">(${esc(errorDrillFeedback.matchReason)})</span>` : ''}
                — Đáp án: <b>${esc(errorDrillFeedback.target)}</b>
              </div>
              ${errorDrillFeedback.rationaleVi ? `<div class="small" style="margin-top:4px;">💡 ${esc(errorDrillFeedback.rationaleVi)}</div>` : ''}
              ${
                errorDrillFeedback.masteredNow
                  ? `<div class="tag mastered" style="margin-top:6px;">✅ Xuất sắc! Đã trị dứt điểm lỗi này (3 ngữ cảnh liên tiếp)!</div>`
                  : typeof errorDrillFeedback.streakAfter === 'number'
                    ? `<div class="small muted" style="margin-top:4px;">Tiến độ trị lỗi: <b>${errorDrillFeedback.streakAfter}/3</b> ngữ cảnh khác nhau</div>`
                    : ''
              }
              ${
                q.drill_mode === 'TYPING' && !errorDrillFeedback.correct
                  ? renderErrorFixFallbackBlock(errorDrillFeedback, 'errDrillSelfAcceptBtn', 'errDrillAiGradeBtn')
                  : ''
              }
              <div style="margin-top:10px;">
                <button type="button" class="btn btn-primary" id="nextErrDrillBtn">
                  ${errorDrillIdx + 1 >= tot ? 'Xem tổng kết →' : 'Câu tiếp →'} <kbd style="color:inherit;border-color:currentColor;">Enter</kbd>
                </button>
              </div>
            </div>`
          : ''
      }
    </div>`;
}

function renderMicroCheck(): string {
  if (microSummary) {
    return `
      <div class="stage-wrap box">
        <div class="quiz-summary-hero">
          <div class="quiz-summary-score mono-num">${microSummary.correct_count} / ${microSummary.total_questions} · ${microSummary.accuracy_pct}%</div>
          <div class="small muted">✓ Đã hoàn thành Micro-Check 2 phút${microSummary.marked_quest_done ? ' · ✓ Đã tự động tick xong nhiệm vụ Micro-Check hôm nay!' : ''}</div>
        </div>
        <div style="text-align:center;">
          <button type="button" class="btn btn-primary" id="restartMicroBtn">LÀM LẠI MICRO-CHECK MỚI → <kbd style="color:inherit;border-color:currentColor;">Enter</kbd></button>
        </div>
      </div>`;
  }
  const q = microQuestions[microIdx];
  if (!q) {
    return `
      <div class="stage-wrap box" style="text-align:center;padding:24px;">
        <div class="small muted" style="margin-bottom:10px;">Trộn nhanh 5 câu (2 từ dễ quên + 2 câu Sổ lỗi + 1 cặp Paraphrase) để rà trí nhớ trong 2 phút.</div>
        <button type="button" class="btn btn-primary" id="startMicroBtn">⚡ BẮT ĐẦU MICRO-CHECK 5 CÂU (2P) → <kbd style="color:inherit;border-color:currentColor;">Enter</kbd></button>
      </div>`;
  }
  const tot = microQuestions.length || 5;
  const pct = Math.round(((microIdx + (microFeedback ? 1 : 0)) / tot) * 100);
  return `
    <div class="stage-wrap box">
      <div class="row small muted" style="justify-content:space-between;">
        <span>Câu ${microIdx + 1}/${tot} · <span class="src-badge">${esc(q.source_label)}</span></span>
        <span id="microTimerDisplay" class="mono-num">⏱ ${fmtMMSS(microSecsLeft)}</span>
      </div>
      <div class="bar-track" style="margin:8px 0 12px;"><div class="bar-fill" style="width:${pct}%"></div></div>
      <div id="microExpiredHost">
        ${
          microTimeExpired
            ? `<div class="micro-expired-banner">
                <span>⏱ <b>Hết 2 phút</b> — Bấm Xem kết quả hoặc làm nốt câu đang dở</span>
                <button type="button" class="btn btn-sm btn-primary" id="microFinishEarlyBtn">Xem kết quả ngay →</button>
              </div>`
            : ''
        }
      </div>
      ${
        q.item_type === 'error_fix'
          ? `<div class="label-uc">Sửa cụm sai này</div>
             <div style="font-size:16px;margin:4px 0 12px;"><s style="color:var(--m-err);">${esc(q.prompt_text)}</s></div>`
          : `<div style="font-size:15px;font-weight:700;margin:4px 0 10px;">${esc(q.prompt_text)}</div>`
      }
      ${
        q.item_type === 'paraphrase' && q.options.length > 0
          ? `<div class="quiz-options-grid">
              ${q.options
                .map(
                  (opt, i) => `
                <button type="button" class="quiz-opt-btn ${microFeedback ? (opt === q.answer_text ? 'correct' : '') : ''}" data-micro-opt="${i}" ${microFeedback ? 'disabled' : ''}>
                  <kbd>${i + 1}</kbd> ${String.fromCharCode(65 + i)}. ${esc(opt)}
                </button>`
                )
                .join('')}
            </div>`
          : `<div class="small muted" style="margin-bottom:4px;">Gợi ý: <b class="mono-num">${esc(q.hint)}</b></div>
             <textarea class="text-input" id="microInput" rows="2" style="width:100%;font-size:14px;" placeholder="Gõ đáp án tiếng Anh..." ${microFeedback ? 'disabled' : ''}>${esc(microInput)}</textarea>
             <div class="row" style="justify-content:flex-end;margin-top:8px;">
               ${
                 !microFeedback
                   ? `<button type="button" class="btn btn-sm" id="microGiveUpBtn">❓ Quên / Xem đáp án <kbd>Ctrl+H</kbd></button>
                      <button type="button" class="btn btn-primary" id="microCheckBtn">Kiểm tra <kbd style="color:inherit;border-color:currentColor;">Enter</kbd></button>`
                   : ''
               }
             </div>`
      }
      ${
        microFeedback
          ? `<div class="quiz-feedback ${microFeedback.correct ? 'ok' : 'err'}">
              <div><b>${microFeedback.correct ? '✓ Chính xác!' : microFeedback.gaveUp ? '↺ Đã đẩy về SRS' : '✗ Chưa khớp'}</b>${microFeedback.matchReason ? ` <span class="small muted">(${esc(microFeedback.matchReason)})</span>` : ''} — Đáp án: <b>${esc(microFeedback.target)}</b></div>
              ${microFeedback.note ? `<div class="small muted" style="margin-top:3px;">${esc(microFeedback.note)}</div>` : ''}
              ${
                microFeedback.isErrorFix && !microFeedback.gaveUp
                  ? renderErrorFixFallbackBlock(microFeedback, 'microSelfAcceptBtn', 'microAiGradeBtn')
                  : ''
              }
              <div style="margin-top:8px;"><button type="button" class="btn btn-primary btn-sm" id="microNextBtn">${microIdx + 1 >= tot ? 'Hoàn thành Micro-Check →' : 'Câu tiếp →'} <kbd style="color:inherit;border-color:currentColor;">Enter</kbd></button></div>
            </div>`
          : ''
      }
    </div>`;
}

function renderQuizBox(): string {
  if (!dash) return '';
  const qs = dash.quiz_status;

  const pillsHtml = `
    <div class="quizpills" role="group" aria-label="Chế độ Quiz">
      <button type="button" class="pill" aria-pressed="${quizPillMode === 'vocab'}" data-qpill="vocab">📝 Quiz Từ vựng</button>
      <button type="button" class="pill" aria-pressed="${quizPillMode === 'fix'}" data-qpill="fix">🔧 Luyện Lỗi Sai (Thực chiến)</button>
      <button type="button" class="pill" aria-pressed="${quizPillMode === 'micro'}" data-qpill="micro">⚡ Micro-Check 5 câu (2p)</button>
    </div>`;

  if (quizPillMode === 'fix') {
    return `${pillsHtml}${renderErrorDrillQuiz()}`;
  }
  if (quizPillMode === 'micro') {
    return `${pillsHtml}${renderMicroCheck()}`;
  }

  if (quizPhase === 'setup') {
    const lastText = qs.last_quiz_at
      ? `${esc(qs.last_quiz_at)}${qs.last_quiz_score ? ` (${esc(qs.last_quiz_score)})` : ''}`
      : 'Chưa làm bài nào';
    return `${pillsHtml}
      <div class="stage-wrap box">
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
            ${quizLoading ? 'ĐANG TẠO BÀI QUIZ...' : 'BẮT ĐẦU QUIZ 10 CÂU →'} <kbd style="color:inherit;border-color:currentColor;">Enter</kbd>
          </button>
          <span class="small muted">Từ nào sai hoặc bấm <b>Quên từ này</b> sẽ tăng số lần quên &amp; đẩy về đầu hàng đợi SRS.</span>
        </div>
      </div>`;
  }

  if (quizPhase === 'playing') {
    const q = quizQuestions[quizIdx];
    if (!q) return pillsHtml;
    const total = quizQuestions.length || 1;
    const pct = Math.round(((quizIdx + (quizFeedback ? 1 : 0)) / total) * 100);
    const typeLabel =
      q.question_type === 'cloze_mcq'
        ? 'ĐIỀN NGỮ CẢNH'
        : q.question_type === 'typing'
          ? 'GÕ TỪ ACTIVE RECALL'
          : 'TRẮC NGHIỆM NGHĨA';

    return `${pillsHtml}
      <div class="stage-wrap box">
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
                <div class="small muted" style="margin-bottom:4px;">Gợi ý: <b class="mono-num">${esc(q.hint)}</b></div>
                <div class="quiz-typing-row">
                  <input type="text" class="quiz-typing-input" id="quizTypingInput"
                    placeholder="Nhập từ/cụm từ tiếng Anh..."
                    value="${esc(quizFeedback?.typedText || '')}"
                    ${quizFeedback ? 'disabled' : ''} />
                  ${
                    !quizFeedback
                      ? `<button class="btn btn-primary" id="quizCheckTypingBtn" type="button">KIỂM TRA <kbd style="color:inherit;border-color:currentColor;">Enter</kbd></button>
                         <button class="btn" id="quizGiveUpBtn" type="button">❓ Quên từ này <kbd>Ctrl+H</kbd></button>`
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
                      return `<button type="button" class="${cls}" data-quiz-opt="${i}" ${quizFeedback ? 'disabled' : ''}><kbd>${i + 1}</kbd> ${String.fromCharCode(65 + i)}. ${esc(opt)}</button>`;
                    })
                    .join('')}
                </div>
                ${
                  !quizFeedback
                    ? `<div style="margin-top:10px;"><button class="btn btn-sm" id="quizGiveUpBtn" type="button">❓ Quên từ này (đẩy về SRS) <kbd>F</kbd></button></div>`
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
                  <div class="small muted" style="margin-top:2px;">Ví dụ: "${esc(q.example)}"</div>
                  ${!quizFeedback.correct ? `<div class="quiz-srs-badge">↺ Đã đánh dấu học lại trong SRS</div>` : ''}
                  <div style="margin-top:10px;">
                    <button class="btn btn-primary" id="quizNextBtn" type="button" ${quizSubmitting ? 'disabled' : ''}>
                      ${quizSubmitting ? 'ĐANG LƯU...' : quizIdx + 1 >= total ? 'XEM KẾT QUẢ →' : 'CÂU TIẾP THEO →'}
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
    return `${pillsHtml}
      <div class="stage-wrap box">
        <div class="quiz-summary-hero">
          <div class="quiz-summary-score mono-num">${quizSummary.correct_count} / ${quizSummary.total_questions} · ${quizSummary.accuracy_pct}%</div>
          <div class="small muted">
            ✓ Đã lưu lịch sử Quiz${quizSummary.marked_quest_done ? ' · ✓ Đã tự động tick xong nhiệm vụ Quiz hôm nay!' : ''}
          </div>
        </div>
        ${
          forgotten.length > 0
            ? `
              <div style="font-size:12.5px;font-weight:700;margin-bottom:4px;">
                Các từ bạn vừa quên (${forgotten.length}) — Đã tự động đẩy về hàng đợi SRS:
              </div>
              <div class="quiz-forgotten-list">
                ${forgotten
                  .map(
                    (c) => `
                  <div class="quiz-forgotten-item">
                    <div><b>${esc(c.word)}</b> — ${esc(c.meaning)}</div>
                    <div class="small muted">"${esc(c.example)}"</div>
                  </div>`
                  )
                  .join('')}
              </div>
              <div class="row" style="flex-wrap:wrap;">
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

  return pillsHtml;
}

function renderDictationMode(): string {
  if (dictationSummary) {
    return `
      <div class="stage-wrap box">
        <div class="quiz-summary-hero">
          <div class="quiz-summary-score mono-num">Trung bình ${dictationSummary.avg_accuracy}% · ${dictationSummary.total_sentences} câu</div>
          <div class="small muted">✓ Đã lưu phiên Nghe chép câu${dictationSummary.marked_quest_done ? ' · ✓ Đã tự động tick xong nhiệm vụ Dictation hôm nay!' : ''}</div>
        </div>
        <div style="text-align:center;">
          <button type="button" class="btn btn-primary" id="dictNewBatchBtn">LUYỆN 5 CÂU TIẾP THEO →</button>
        </div>
      </div>`;
  }

  const cur = dictationBatch[dictationIdx];
  if (!cur) {
    return `
      <div class="stage-wrap box" style="text-align:center;padding:28px;">
        <div class="small muted" style="margin-bottom:10px;">Luyện nghe chép câu IELTS 100% offline theo bẫy số nhiều -s, đuôi -ed, con số và từ đổi ý.</div>
        <button type="button" class="btn btn-primary" id="loadDictBatchBtn">🎧 TẢI 5 CÂU NGHE CHÉP NGAY →</button>
      </div>`;
  }

  const tot = dictationBatch.length || 5;
  const pctBar = Math.round(((dictationIdx + (dictationChecked ? 1 : 0)) / tot) * 100);
  const words = cur.sentence.split(/\s+/);
  const hintPreview =
    dictationHintWords > 0
      ? `💡 Gợi ý (${dictationHintWords} từ): <b>${esc(words.slice(0, dictationHintWords).join(' '))}...</b>`
      : '';

  return `
    <div class="v-toolbar">
      <span class="label-uc">Tốc độ</span>
      <div class="seg" role="group" aria-label="Tốc độ đọc">
        <button type="button" aria-pressed="${dictationRate === 0.85}" data-rate="0.85">0.85x</button>
        <button type="button" aria-pressed="${dictationRate === 1}" data-rate="1">1.0x</button>
        <button type="button" aria-pressed="${dictationRate === 1.15}" data-rate="1.15">1.15x</button>
      </div>
      <button type="button" class="chip" id="dictAutoPlayChip" aria-pressed="${dictationAutoPlay}" title="Tự phát âm khi sang câu mới">🔊 Tự đọc câu mới</button>
      <span class="sp"></span>
      <span class="small">Câu ${dictationIdx + 1} / ${tot} · ${esc(cur.section)} · <span class="tag soft">${esc(cur.trap_focus)}</span> ${cur.source === 'ai_pack' ? '<span class="tag ai-pack">✨ Mới nạp</span>' : ''}</span>
    </div>

    <div class="stage-wrap box" style="display:grid;gap:12px;">
      <div class="bar-track"><div class="bar-fill" style="width:${pctBar}%"></div></div>
      <div style="text-align:center;">
        <button type="button" class="btn btn-primary lg" id="dictPlayBtn">🔊 Nghe câu <kbd style="color:inherit;border-color:currentColor;">Ctrl+Space</kbd></button>
      </div>
      <div>
        <div class="label-uc" style="margin-bottom:4px;">Gõ lại nguyên câu nghe được</div>
        <textarea class="text-input" id="dictInput" rows="2" style="width:100%;font-size:14px;" placeholder="Gõ lại câu bạn nghe được...">${esc(dictationInput)}</textarea>
      </div>
      ${hintPreview ? `<div class="small muted">${hintPreview}</div>` : ''}
      <div class="row" style="justify-content:flex-end;flex-wrap:wrap;">
        <button type="button" class="btn" id="dictHintBtn">💡 Xem gợi ý 1 từ <kbd>Ctrl+H</kbd></button>
        <button type="button" class="btn btn-primary" id="dictCheckBtn">Kiểm tra <kbd style="color:inherit;border-color:currentColor;">Enter</kbd></button>
      </div>
        ${
          dictationChecked
            ? `
          <div>
            <div class="row" style="justify-content:space-between;margin-bottom:6px;">
              <span class="label-uc">Kết quả từng từ</span>
              <span><span class="dict-pct">${dictationChecked.pct}%</span> <span class="small muted">chính xác</span></span>
            </div>
            <div class="diff-box">${dictationChecked.tokensHtml}</div>
            <div class="small muted" style="margin-top:6px;">Nghĩa: ${esc(cur.meaning_vi)}</div>
            <div class="row" style="justify-content:space-between;margin-top:8px;flex-wrap:wrap;">
              <span class="small muted"><span class="w-ok">xanh</span> = đúng · <span class="w-bad">gạch</span> = bạn gõ sai/thừa · <span class="w-fix">đỏ</span> = đáp án/thiếu</span>
              <div class="row">
                <button type="button" class="btn btn-sm" id="pushDictErrBtn" ${dictationChecked.pushed || !dictationChecked.pairs.length ? 'disabled' : ''}>
                  ${dictationChecked.pushed ? `✓ Đã đẩy ${dictationChecked.pairs.length} lỗi + Drill` : '+ Đẩy vào Sổ lỗi Listening'} <kbd>Ctrl+L</kbd>
                </button>
                <button type="button" class="btn btn-sm btn-primary" id="dictNextBtn">
                  ${dictationIdx + 1 >= tot ? 'Hoàn thành & Lưu buổi nghe →' : 'Câu tiếp →'} <kbd style="color:inherit;border-color:currentColor;">Enter</kbd>
                </button>
              </div>
            </div>
          </div>`
            : ''
        }
    </div>`;
}

// --- Speaking Studio Renderer ---
function renderSpeakingStudio(): string {
  if (!dash) return '';
  const isAim80 = isAim80Mode();
  const plan = SPEAKING_PLANS[studioPlanKey];
  const p3Lines =
    studioSource === 'bank' && studioCue
      ? studioCue.part3_questions.slice(0, 2)
      : customPart3
          .split('\n')
          .map((s) => s.trim())
          .filter(Boolean)
          .slice(0, 2);

  return `
    <div class="v-backdrop ${studioOpen || addVocabOpen ? 'on' : ''}" id="vBackdrop"></div>
    <div class="studio-overlay ${studioOpen ? 'on' : ''}" id="speakStudioModal" role="dialog" aria-modal="true" aria-label="Phòng Luyện Nói" tabindex="-1">
      <div class="studio-head">
        <b>🎙️ Phòng Luyện Nói</b>
        <span class="small muted">Phiên tự lưu metadata · audio chỉ giữ tạm trong phiên</span>
        <span class="sp"></span>
        <button type="button" class="btn ghost ibtn" id="closeStudioBtn" title="Đóng">✕</button>
      </div>
      <div class="sbody">
        <!-- Left column: Cue Card + Part 3 + Prep Notes -->
        <div class="scol">
          <div class="row" style="justify-content:space-between;flex-wrap:wrap;">
            <div class="seg" role="group" aria-label="Nguồn đề">
              <button type="button" aria-pressed="${studioSource === 'bank'}" data-stsrc="bank">Đề theo tuần</button>
              <button type="button" aria-pressed="${studioSource === 'custom'}" data-stsrc="custom">Đề của tôi</button>
            </div>
            <button type="button" class="btn btn-sm" id="swapCueBtn" ${studioSource === 'custom' ? 'disabled' : ''}>↻ Đổi đề</button>
          </div>
          ${
            studioSource === 'bank' && studioCue
              ? `
            <div class="small muted">Tuần ${studioCue.week_num} · Cue Card #${studioCue.id}/48 — Track ${esc(studioCue.level)} (${esc(studioCue.topic)})</div>
            <div class="cue-card">
              <div class="label-uc">Part 2 · Cue Card</div>
              <h4>${esc(studioCue.cue_title)}</h4>
              <div class="small muted">You should say:</div>
              <ul class="small">
                ${studioCue.cue_bullets.map((b) => `<li>${esc(b)}</li>`).join('')}
              </ul>
            </div>
            <div>
              <div class="label-uc">Part 3 · 2 câu nối tiếp</div>
              <ol class="small" style="padding-left:18px;margin:5px 0;">
                ${p3Lines.map((q) => `<li>${esc(q)}</li>`).join('')}
              </ol>
              <div class="small muted">
                ${
                  isAim80
                    ? `<b>5 cụm C1–C2 mục tiêu:</b> ${studioCue.target_phrases.map((p) => `<code>${esc(p)}</code>`).join(' · ')}`
                    : `Khung <b>A.R.E.A</b>: Answer → Reason → Example → Alternative`
                }
              </div>
            </div>`
              : `
            <div class="small muted">Tự dán đề Part 2 &amp; Part 3 của bạn</div>
            <label class="small"><b>Cue Card Part 2</b>
              <input type="text" class="text-input" id="customCueIn" style="width:100%;margin-top:4px;" value="${esc(customCueTitle)}" />
            </label>
            <label class="small"><b>Part 3 (mỗi câu 1 dòng)</b>
              <textarea class="text-input" id="customP3In" rows="3" style="width:100%;margin-top:4px;">${esc(customPart3)}</textarea>
            </label>`
          }
          <label class="small">
            <span class="label-uc" style="display:block;margin-bottom:4px;">Ghi chú chuẩn bị (1 phút)</span>
            <textarea class="text-input" id="studioPrepIn" rows="3" style="width:100%;" placeholder="Ghi nhanh từ khóa, ý chính A.R.E.A...">${esc(studioPrepNotes)}</textarea>
          </label>
        </div>

        <!-- Right column: Timer, MediaRecorder, AI Grading + Fallback -->
        <div class="scol">
          <div class="row" style="justify-content:space-between;flex-wrap:wrap;">
            <div class="seg" role="group" aria-label="Chế độ luyện nói">
              <button type="button" aria-pressed="${studioPlanKey === 'real'}" data-stplan="real">🎯 Thi thật</button>
              <button type="button" aria-pressed="${studioPlanKey === '432'}" data-stplan="432">⚡ Phản xạ 4-3-2</button>
              <button type="button" aria-pressed="${studioPlanKey === 'p3'}" data-stplan="p3">Part 3 · 2×45s</button>
            </div>
            <span class="small"><span class="recdot"></span> ${recordedBlobs.length} đoạn đã ghi</span>
          </div>

          <div class="stages-grid" style="grid-template-columns:repeat(${plan.stages.length},1fr);">
            ${plan.stages
              .map(
                (st, idx) => `
              <div class="stage-pill ${idx === studioStageIdx ? 'cur' : idx < studioStageIdx ? 'done' : ''}">
                <b>${esc(st.name)}</b>${fmtMMSS(st.secs)}
              </div>`
              )
              .join('')}
          </div>

          <div class="bigtime" id="studioTimerDisplay">${fmtMMSS(studioSecsLeft)}</div>

          <div class="ctrls">
            <button type="button" class="btn btn-primary" id="studioGoBtn">${studioRunning ? '⏸ Dừng' : '● Bắt đầu'} <kbd style="color:inherit;border-color:currentColor;">Ctrl+Space</kbd></button>
            <button type="button" class="btn" id="studioNextStageBtn">⏭ Chặng sau <kbd>Alt+N</kbd></button>
            <button type="button" class="btn" id="studioReplayBtn" ${recordedBlobs.length === 0 ? 'disabled' : ''}>▶ Nghe lại</button>
            <button type="button" class="btn" id="studioSaveAudioBtn" title="Tải file ghi âm (.webm) về máy" ${recordedBlobs.length === 0 ? 'disabled' : ''}>💾 Lưu audio</button>
          </div>
          ${
            recordedBlobs.length >= 2
              ? `<div class="clip-pills">
                  ${recordedBlobs
                    .map(
                      (c, i) => `
                    <button type="button" class="pill" aria-pressed="${selectedClipIdx === i}" data-pick-clip="${i}">
                      Chặng ${i + 1}: ${esc(c.stageName)}
                    </button>`
                    )
                    .join('')}
                </div>`
              : ''
          }

          <div class="speak-score-box">
            <div class="row" style="justify-content:space-between;flex-wrap:wrap;gap:6px;">
              <span class="label-uc">Chấm điểm Speaking</span>
              <span class="row" style="gap:6px;flex-wrap:wrap;">
                ${
                  dash.speaking_sessions.length > 0
                    ? `<button type="button" class="btn btn-sm" id="toggleSpeakHistoryBtn" title="Xem lịch sử ${dash.speaking_sessions.length} phiên Speaking">📜 Lịch sử (${dash.speaking_sessions.length})</button>`
                    : ''
                }
                <button type="button" class="btn btn-primary btn-sm" id="gradeSpeakingAiBtn" title="Chấm bằng Gemini Audio (Ctrl+Enter)" ${isGradingSpeaking ? 'disabled' : ''}>
                  ${isGradingSpeaking ? 'ĐANG CHẤM...' : '✨ Chấm AI'} <kbd style="color:inherit;border-color:currentColor;">Ctrl+Enter</kbd>
                </button>
              </span>
            </div>
            ${speakingStatusMsg ? `<div class="status-msg ${speakingStatusErr ? 'err' : 'ok'}">${esc(speakingStatusMsg)}</div>` : ''}
            ${
              speakingGradeOut
                ? `
              <div class="speak-bands">
                <div class="speak-band"><span class="small muted">FC</span><b>${speakingGradeOut.session.fc_band.toFixed(1)}</b></div>
                <div class="speak-band"><span class="small muted">LR</span><b>${speakingGradeOut.session.lr_band.toFixed(1)}</b></div>
                <div class="speak-band"><span class="small muted">GRA</span><b>${speakingGradeOut.session.gra_band.toFixed(1)}</b></div>
                <div class="speak-band"><span class="small muted">P*</span><b>${speakingGradeOut.session.p_band.toFixed(1)}</b></div>
              </div>
              ${
                speakingGradeOut.session.wpm_end > 0
                  ? `<div class="speak-kv"><span>Tốc độ nói</span><b>${speakingGradeOut.session.wpm_start} → ${speakingGradeOut.session.wpm_end} WPM</b></div>
                     <div class="speak-kv"><span>Từ đệm (uh, like…)</span><b>${speakingGradeOut.session.fillers_start} → ${speakingGradeOut.session.fillers_end}</b></div>`
                  : ''
              }
              <div class="speak-kv"><span>Nhận xét tiến bộ</span><b style="color:var(--m-ok);text-align:right;">${esc(speakingGradeOut.session.progress_note)}</b></div>
              ${
                speakingGradeOut.top_errors.length
                  ? `<div class="small" style="margin-top:6px;"><b>${speakingGradeOut.top_errors.length} lỗi → Đã đẩy Sổ lỗi</b> <span class="tag soft">Speaking</span>
                      <ol style="margin:4px 0 0;padding-left:18px;">
                        ${speakingGradeOut.top_errors.map((e) => `<li>“${esc(e.wrong)}” → <b>${esc(e.fix)}</b></li>`).join('')}
                      </ol>
                    </div>`
                  : ''
              }
              <div class="small muted" style="margin-top:4px;">* P ước lượng từ audio, chỉ tham khảo.</div>`
                : ''
            }

            <div class="speak-fallback">
              <span class="small muted">Chưa có API key? Lưu audio ở trên rồi dùng:</span>
              <button type="button" class="btn btn-sm" id="fallbackCopyPromptBtn">📋 Copy Prompt + Rubric</button>
              <button type="button" class="btn btn-sm" id="toggleManualCheckBtn">✓ Tự chấm 5 mục</button>
            </div>

            ${
              manualCheckOpen
                ? `
              <div style="margin-top:8px;padding-top:8px;border-top:1px solid var(--m-line);font-size:11.5px;">
                <div class="label-uc" style="margin-bottom:4px;">Checklist tự chấm 5 mục</div>
                ${[
                  'Trả lời đủ 4 gạch đầu dòng Cue Card (≥ 90s)',
                  'Nói mạch lạc, hạn chế ậm ừ (uh/um/like)',
                  'Dùng ≥ 3 cụm từ vựng/collocations mục tiêu',
                  'Chú ý chia thì quá khứ & âm cuối -s/-ed',
                  'Part 3 trả lời theo cấu trúc A.R.E.A',
                ]
                  .map(
                    (lbl, i) => `
                  <label style="display:flex;gap:6px;align-items:center;margin:3px 0;cursor:pointer;">
                    <input type="checkbox" class="manual-spk-chk" data-mchk="${i}" ${manualChecks[i] ? 'checked' : ''} />
                    <span>${esc(lbl)}</span>
                  </label>`
                  )
                  .join('')}
                <div class="speak-manual-bands">
                  <label>FC<input type="number" step="0.5" min="1" max="9" class="text-input" id="mBandFc" value="${esc(manualBandFc)}" style="width:100%;" /></label>
                  <label>LR<input type="number" step="0.5" min="1" max="9" class="text-input" id="mBandLr" value="${esc(manualBandLr)}" style="width:100%;" /></label>
                  <label>GRA<input type="number" step="0.5" min="1" max="9" class="text-input" id="mBandGra" value="${esc(manualBandGra)}" style="width:100%;" /></label>
                  <label>P<input type="number" step="0.5" min="1" max="9" class="text-input" id="mBandP" value="${esc(manualBandP)}" style="width:100%;" /></label>
                </div>
                <input type="text" class="text-input" id="mSpkNote" style="width:100%;margin-top:6px;" placeholder="Ghi chú tiến bộ hoặc dán nhận xét AI..." value="${esc(manualNote)}" />
                <textarea class="text-input" id="mSpkTranscript" rows="2" style="width:100%;margin-top:6px;" placeholder="Transcript / từ khóa (tùy chọn)...">${esc(manualTranscript)}</textarea>
                <div style="text-align:right;margin-top:6px;">
                  <button type="button" class="btn btn-sm btn-primary" id="saveManualSpeakBtn">Lưu phiên Speaking →</button>
                </div>
              </div>`
                : ''
            }

            ${
              dash.speaking_sessions.length > 0
                ? `<div class="small muted" style="margin-top:8px;padding-top:6px;border-top:1px solid var(--m-line);">
                    Gần nhất: <b>Band ${dash.speaking_sessions[0].overall_band.toFixed(1)}</b> · ${esc(dash.speaking_sessions[0].cue_title)} (${esc(dash.speaking_sessions[0].created_at)}) · Tổng ${dash.speaking_sessions.length} phiên
                  </div>`
                : ''
            }
            ${
              speakingHistoryOpen && dash.speaking_sessions.length > 0
                ? `<div class="speak-history-list">
                    ${dash.speaking_sessions
                      .map(
                        (s) => `
                      <div class="speak-history-item">
                        <div class="row" style="justify-content:space-between;gap:6px;">
                          <b>Band ${s.overall_band.toFixed(1)} <span class="muted" style="font-weight:400;">(FC ${s.fc_band.toFixed(1)} · LR ${s.lr_band.toFixed(1)} · GRA ${s.gra_band.toFixed(1)} · P ${s.p_band.toFixed(1)})</span></b>
                          <span class="muted">${esc(s.created_at)}</span>
                        </div>
                        <div style="margin-top:2px;"><b>${esc(s.cue_title)}</b>${s.wpm_end > 0 ? ` · <span class="mono-num">${s.wpm_start}→${s.wpm_end} WPM</span>` : ''}</div>
                        ${s.progress_note ? `<div class="muted" style="margin-top:2px;">${esc(s.progress_note)}</div>` : ''}
                      </div>`
                      )
                      .join('')}
                  </div>`
                : ''
            }
          </div>
        </div>
      </div>
    </div>

    <!-- Drawer + Thêm từ (Sentence Mining) -->
    <aside class="v-drawer ${addVocabOpen ? 'on' : ''}" id="addVocabDrawer" role="dialog" aria-modal="true" aria-label="Thêm từ" tabindex="-1">
      <div class="row" style="justify-content:space-between;">
        <b>+ Thêm từ (Sentence Mining)</b>
        <button type="button" class="btn ghost ibtn" id="closeAddVocabBtn">✕</button>
      </div>
      <label class="small">
        <span class="label-uc" style="display:block;margin-bottom:4px;">Từ / Cụm từ</span>
        <input type="text" class="text-input" id="addWordIn" style="width:100%;" placeholder="exacerbate" value="${esc(addVocabWord)}" />
      </label>
      <label class="small">
        <span class="label-uc" style="display:block;margin-bottom:4px;">Nghĩa tiếng Việt</span>
        <input type="text" class="text-input" id="addMeaningIn" style="width:100%;" placeholder="làm trầm trọng thêm" value="${esc(addVocabMeaning)}" />
      </label>
      <label class="small">
        <span class="label-uc" style="display:block;margin-bottom:4px;">Câu ví dụ (dán từ Study4)</span>
        <textarea class="text-input" id="addExampleIn" rows="3" style="width:100%;" placeholder="Rising temperatures could exacerbate water shortages.">${esc(addVocabExample)}</textarea>
      </label>
      <div class="row">
        <label class="small" style="flex:1;">
          <span class="label-uc" style="display:block;margin-bottom:4px;">Chủ đề</span>
          <select class="select-input" id="addTopicSel" style="width:100%;">
            ${['Environment', 'Education', 'Technology', 'Work', 'Health', 'Society', 'Study4 Reading']
              .map((t) => `<option value="${t}" ${addVocabTopic === t ? 'selected' : ''}>${t}</option>`)
              .join('')}
          </select>
        </label>
        <label class="small" style="flex:1;">
          <span class="label-uc" style="display:block;margin-bottom:4px;">Tuần</span>
          <select class="select-input" id="addWeekSel" style="width:100%;">
            <option value="${dash.current_week}">Tuần ${dash.current_week} (hiện tại)</option>
            <option value="${Math.min(24, dash.current_week + 1)}">Tuần ${Math.min(24, dash.current_week + 1)}</option>
          </select>
        </label>
      </div>
      <div class="small muted">Thẻ tự tạo hiển thị cả ở Mặt từ và Câu đục lỗ. Âm thanh dùng TTS, không tốn dung lượng.</div>
      ${addVocabMsg ? `<div class="status-msg err">${esc(addVocabMsg)}</div>` : ''}
      <div class="row" style="margin-top:auto;justify-content:flex-end;">
        <button type="button" class="btn" id="cancelAddVocabBtn">Hủy</button>
        <button type="button" class="btn btn-primary" id="saveAddVocabBtn">Lưu thẻ</button>
      </div>
    </aside>
  `;
}

// --- Main App Render ---
function renderApp() {
  if (!dash) return;
  const curTheme = normalizeTheme(dash.settings.theme);
  const isAim80 = isAim80Mode();
  const winTitle = isAim80 ? 'XD4T // IELTS 8.0' : 'XD4T // IELTS 6.5';
  if (lastSyncedWindowTitle !== winTitle) {
    lastSyncedWindowTitle = winTitle;
    document.title = winTitle;
    try {
      void getCurrentWindow().setTitle(winTitle).catch(() => {});
    } catch {
      // Ignore outside Tauri
    }
  }
  document.documentElement.className = `theme-${curTheme}`;

  const coreQuests = dash.quests.filter((q) => q.tier !== 'bonus');
  const bonusQuests = dash.quests.filter((q) => q.tier === 'bonus');
  const coreDone = coreQuests.filter((q) => q.done).length;
  const coreTotal = coreQuests.length || 1;
  const corePct = Math.round((coreDone / coreTotal) * 100);
  const remainingCoreMins = coreQuests
    .filter((q) => !q.done)
    .reduce((acc, q) => acc + q.minutes, 0);

  const curPrompt = getCurrentPrompt();
  const modePrompts = dash.prompts.filter((p) => p.mode === writingMode);
  const top3Errors = dash.error_logs
    .filter((e) => e.category.toLowerCase().includes('writing'))
    .slice(0, 3);

  const phaseNumbers = Array.from(new Set(dash.roadmap.map((r) => r.phase))).sort((a, b) => a - b);
  const pct = (arr: NonNullable<typeof dash>['roadmap']) =>
    arr.length ? Math.round((arr.filter((x) => x.done).length / arr.length) * 100) : 0;

  const root = document.getElementById('app')!;
  root.innerHTML = `
  <div class="app-shell theme-${curTheme}" id="shell">
    <div class="titlebar">
      <div class="tb-left">
        <div class="brand-pill">${isAim80 ? 'XD4T // IELTS 8.0' : 'XD4T // IELTS 6.5'}</div>
        <div class="stat-pill"><span>THI ${esc(dash.settings.exam_date)}:</span> <b class="mono-num">${dash.days_until_exam}d</b></div>
        <div class="stat-pill"><span>STREAK:</span> <b class="mono-num">${dash.streak} ngày</b></div>
        ${
          dash.settings.has_api_key
            ? `<div class="stat-pill" id="topbarApiStatus"><span>API:</span> <b>SẴN SÀNG</b></div>`
            : `<button type="button" class="stat-pill stat-pill-btn" id="bannerOpenSettingsBtn" title="Chưa có Gemini API Key — bấm để nạp miễn phí hoặc dùng Copy-Prompt"><span>API:</span> <b>Chưa có · + Nạp Key</b></button>`
        }
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
      <button class="nav-tab ${activeTab === 'vocab' ? 'active' : ''}" data-tab="vocab">[3] TỪ VỰNG &amp; NGHE (${onlyLeechFilter ? dash.due_vocab_count : dueCards.length})</button>
      <button class="nav-tab ${activeTab === 'roadmap' ? 'active' : ''}" data-tab="roadmap">[4] LỘ TRÌNH &amp; SỔ LỖI</button>
      <div class="nav-meta">Tuần ${dash.current_week} · Track ${isAim80 ? '8.0' : '6.5'} · Mục tiêu Band ${dash.settings.target_band.toFixed(1)}</div>
    </div>

    <div class="content">
      <!-- TAB 1: TODAY -->
      <div class="panel ${activeTab === 'today' ? 'active' : ''}" id="tab-today">
        ${
          autoRefillToastMsg
            ? `<div class="api-banner" role="status">
                <span>${esc(autoRefillToastMsg)}</span>
                <button class="btn btn-sm" id="dismissAutoRefillToastBtn" type="button">Đóng</button>
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

        <div class="grid1">
          <div class="box">
            <div class="grp">
              <span class="label-uc">CORE ≤ 30 PHÚT · TÍNH STREAK</span>
              <span class="tag ok">${coreDone}/${coreTotal} xong</span>
              <span class="sp"></span>
              <span class="small muted">${remainingCoreMins > 0 ? `Còn ${remainingCoreMins}p Core` : '✓ Đủ nhịp Core hôm nay'}</span>
            </div>
            <div class="bar-track" style="margin-bottom:10px;"><div class="bar-fill" style="width:${corePct}%"></div></div>
            ${coreQuests
              .map((q, idx) => {
                const target = questTargetFromHint(q.action_hint, q.title);
                const isStudy4 = target === 'study4';
                const isPrimary = idx === coreQuests.length - 1;
                return `
                <div class="task-row ${q.done ? 'done' : ''}">
                  <div class="task-toggle" data-quest-id="${q.id}" tabindex="0" role="button">
                    <div class="q-check">${q.done ? '✓' : ''}</div>
                    <span class="task-min">${String(q.minutes).padStart(2, '0')}p</span>
                    <div class="task-body">
                      <div class="task-title">${esc(q.title)}</div>
                      <div class="task-meta">${esc(q.meta)}</div>
                    </div>
                  </div>
                  ${
                    q.done
                      ? `<span class="tag ok">✓ Xong</span>`
                      : isStudy4
                        ? `<button type="button" class="btn btn-sm ${isPrimary ? 'btn-primary' : ''}" data-go="study4">Mở Study4 ↗</button>`
                        : `<button type="button" class="btn btn-sm ${isPrimary ? 'btn-primary' : ''}" data-go="${target}">Mở ngay →</button>`
                  }
                </div>`;
              })
              .join('')}

            ${
              bonusQuests.length > 0
                ? `
              <details class="bonus">
                <summary>
                  <span class="label-uc">BONUS · TÙY CHỌN</span>
                  <span class="small muted">${bonusQuests.length} việc · không tính streak</span>
                </summary>
                ${bonusQuests
                  .map((q) => {
                    const target = questTargetFromHint(q.action_hint, q.title);
                    const isStudy4 = target === 'study4';
                    return `
                    <div class="task-row ${q.done ? 'done' : ''}">
                      <div class="task-toggle" data-quest-id="${q.id}" tabindex="0" role="button">
                        <div class="q-check">${q.done ? '✓' : ''}</div>
                        <span class="task-min">${String(q.minutes).padStart(2, '0')}p</span>
                        <div class="task-body">
                          <div class="task-title">${esc(q.title)}</div>
                          <div class="task-meta">${esc(q.meta)}</div>
                        </div>
                      </div>
                      <button type="button" class="btn btn-sm" data-go="${target}">${isStudy4 ? 'Mở Study4 ↗' : 'Mở ngay →'}</button>
                    </div>`;
                  })
                  .join('')}
              </details>`
                : ''
            }
          </div>

          <div style="display:grid;gap:12px;align-content:start;">
            <div class="box">
              <div class="box-title">
                <span>HEATMAP &amp; STREAK CORE</span>
                <span>${dash.streak} ngày</span>
              </div>
              <div class="small muted">Hoàn thành ≥ 1 nhiệm vụ Core để giữ chuỗi học.</div>
              <div class="heatmap">
                ${dash.heatmap
                  .map((d) => {
                    const lvl =
                      d.minutes >= 30 ? 4 : d.minutes >= 20 ? 3 : d.minutes >= 10 ? 2 : d.minutes > 0 ? 1 : 0;
                    return `<div class="hm-cell ${lvl ? 'hm-' + lvl : ''}" title="${d.date}: ${d.minutes} phút"></div>`;
                  })
                  .join('')}
              </div>
            </div>

            <div class="box">
              <div class="label-uc" style="margin-bottom:8px;">Quick Actions (1 chạm)</div>
              <div class="qa-grid">
                <button type="button" class="btn" data-go="speak" title="Phòng Luyện Nói (Alt+S)"><span class="qa-label">🎙️ Phòng Luyện Nói</span> <kbd>Alt+S</kbd></button>
                <button type="button" class="btn" data-go="vocab:dict" title="Nghe chép câu (Dictation)"><span class="qa-label">🎧 Nghe chép câu</span></button>
                <button type="button" class="btn" data-go="vocab:quiz:micro" title="Micro-Check 2 phút"><span class="qa-label">⚡ Micro-Check 2p</span></button>
                <button type="button" class="btn" id="openAiPackTab1Btn" title="Nạp thêm bài (AI)"><span class="qa-label">✨ Nạp thêm bài (AI)</span></button>
              </div>
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
        ${
          top3Errors.length > 0
            ? `
          <div class="top3-bar ${top3Collapsed ? 'collapsed' : ''}" id="top3Bar">
            <span>⚠️</span>
            <div class="top3-body">
              <div class="row">
                <b class="small">Rà nhanh Top 3 lỗi Writing bạn hay mắc trước khi viết</b>
                <span class="small muted">· lọc riêng Writing từ Sổ lỗi</span>
              </div>
              <ol>
                ${top3Errors
                  .map(
                    (e, i) =>
                      `<li><b>#${i + 1}</b> ${esc(e.category)}: <i>${esc(e.wrong_text)}</i> → ${esc(e.right_text)} <span class="muted">· ${e.count} lần</span></li>`
                  )
                  .join('')}
              </ol>
            </div>
            <button type="button" class="btn btn-sm ghost top3-toggle-btn" id="toggleTop3Btn" aria-expanded="${!top3Collapsed}" title="Thu gọn [Alt+T]">${top3Collapsed ? '▾' : '▴'} <kbd>Alt+T</kbd></button>
          </div>`
            : ''
        }

        <div class="mode-row">
          <button class="btn btn-sm mode-btn ${writingMode === 'paragraph_90' ? 'active' : ''}" data-wmode="paragraph_90">ĐOẠN 90 TỪ</button>
          <button class="btn btn-sm mode-btn ${writingMode === 'task1' ? 'active' : ''}" data-wmode="task1">TASK 1 (150+)</button>
          <button class="btn btn-sm mode-btn ${writingMode === 'task2' ? 'active' : ''}" data-wmode="task2">TASK 2 (250+)</button>
          <select class="select-input" id="promptSelect">
            ${modePrompts
              .map(
                (p) =>
                  `<option value="${esc(p.id)}" ${curPrompt?.id === p.id ? 'selected' : ''}>${p.source === 'ai_pack' ? '✨ [Mới nạp] ' : ''}${esc(p.title)}</option>`
              )
              .join('')}
          </select>
          <button class="btn btn-sm" style="margin-left:auto;" id="toggleHintBtn">+ GỢI Ý KHUNG BÀI ${isAim80 ? '8.0' : '6.5'}</button>
        </div>

        <div class="grid-2">
          <div class="box ${writingMode === 'task1' || Boolean(curPrompt?.chart_spec_json) || Boolean(customTask1ImageDataUrl) ? 'writing-task1-active' : ''}">
            <div class="box-title">
              <span>ĐỀ BÀI &amp; KHUNG VIẾT ${curPrompt?.source === 'ai_pack' ? '<span class="tag ai-pack">✨ Mới nạp</span>' : ''}</span>
              <span id="wordCount">0 / ${curPrompt?.target_words || 90} từ</span>
            </div>
            <div class="prompt-box"><b>Đề:</b> ${esc(curPrompt?.prompt || '')}</div>
            <input type="file" id="task1ImgInput" accept="image/*" style="display:none;" />
            ${
              writingMode === 'task1' || Boolean(curPrompt?.chart_spec_json) || Boolean(customTask1ImageDataUrl)
                ? renderTask1VisualHtml(curPrompt?.chart_spec_json, customTask1ImageDataUrl)
                : ''
            }
            ${
              curPrompt?.target_vocab && curPrompt.target_vocab.length > 0
                ? `<div class="target-vocab-row"><span class="muted">Từ mục tiêu:</span> ${curPrompt.target_vocab.map((v) => `<span class="tag soft">${esc(v)}</span>`).join('')}</div>`
                : ''
            }
            <div class="hint-box ${hintOpen ? 'open' : ''}" id="hintBox">${esc(curPrompt?.outline_hint || '')}</div>
            <textarea class="editor" id="essayInput" placeholder="Nhập bài viết tiếng Anh của bạn tại đây..."></textarea>
            ${
              lastGrade && lastGrade.repeated_top_errors && lastGrade.repeated_top_errors.length > 0
                ? `
              <div class="repeat-alert" role="status">
                <span>⚠️</span>
                <div>
                  <b>Cảnh báo: Bài này bạn vừa lặp lại lỗi trong Top 3</b>
                  <ul style="margin:4px 0 6px;padding-left:18px;">
                    ${lastGrade.repeated_top_errors.map((r) => `<li>${esc(r)}</li>`).join('')}
                  </ul>
                  <button type="button" class="btn btn-sm" data-go="vocab:quiz:fix">🔧 Luyện sửa lỗi ngay →</button>
                </div>
              </div>`
                : ''
            }
            <details class="checklist-3m" ${writingMode === 'task1' || Boolean(curPrompt?.chart_spec_json) || Boolean(customTask1ImageDataUrl) ? '' : 'open'}>
              <summary>CHECKLIST SOÁT LỖI 3 PHÚT TRƯỚC KHI NỘP (${dash.settings.writing_checklist.length} MỤC)</summary>
              <div class="checklist-grid" style="display:grid;grid-template-columns:1fr 1fr;gap:4px;">
                ${dash.settings.writing_checklist
                  .map(
                    (item, idx) =>
                      `<label style="display:flex;gap:5px;align-items:flex-start;cursor:pointer;"><input type="checkbox" class="chk-3m" /> <span>${idx + 1}. ${esc(item)}</span></label>`
                  )
                  .join('')}
              </div>
            </details>
            <div style="display:flex;justify-content:space-between;align-items:center;margin-top:8px;gap:6px;flex-wrap:wrap;">
              <div style="display:flex;gap:6px;flex-wrap:wrap;">
                <button class="btn btn-sm" id="copyPromptBtn">COPY PROMPT CHẤM AI</button>
                <button class="btn btn-sm" id="markWritingDoneBtn" title="Đánh dấu đã viết xong (cho người dùng Copy Prompt miễn phí)">✓ Đánh dấu đã viết (Copy Prompt)</button>
              </div>
              <button class="btn btn-primary" id="gradeBtn" ${isGrading ? 'disabled' : ''}>${isGrading ? 'ĐANG CHẤM AI...' : `CHẤM BAND ${isAim80 ? '8.0' : '6.5'} NGAY →`} <kbd style="color:inherit;border-color:currentColor;">Ctrl+Enter</kbd></button>
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
                <div class="grade-result-box">
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
                <div class="grade-result-box muted-copy">
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

      <!-- TAB 3: TỪ VỰNG & NGHE -->
      <div class="panel ${activeTab === 'vocab' ? 'active' : ''}" id="tab-vocab">
        <div class="row" style="margin-bottom:12px;flex-wrap:wrap;justify-content:space-between;gap:8px;">
          <div class="seg" role="group" aria-label="Sub-mode Từ vựng và Nghe">
            <button type="button" aria-pressed="${vocabSubMode === 'flashcard'}" data-vsub="flashcard">📇 Flashcard</button>
            <button type="button" aria-pressed="${vocabSubMode === 'quiz'}" data-vsub="quiz">📝 Quiz &amp; Sửa lỗi ${dash.quiz_status.quiz_due ? '· ●' : ''}</button>
            <button type="button" aria-pressed="${vocabSubMode === 'dictation'}" data-vsub="dictation">🎧 Nghe chép câu</button>
          </div>
          <button type="button" class="btn btn-sm" id="openAiPackTab3Btn">✨ Nạp thêm bài (AI)</button>
        </div>
        ${
          vocabSubMode === 'quiz'
            ? renderQuizBox()
            : vocabSubMode === 'dictation'
              ? renderDictationMode()
              : renderFlashcardMode(curTheme)
        }
      </div>

      <!-- TAB 4: ROADMAP, ERROR LOG & PARAPHRASE -->
      <div class="panel ${activeTab === 'roadmap' ? 'active' : ''}" id="tab-roadmap">
        <div class="grid4">
          <div class="box">
            <div class="box-title">
              <span>LỘ TRÌNH (${dash.roadmap.length} TUẦN · ${phaseNumbers.length} GĐ)</span>
              <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap;">
                <span>${dash.roadmap.filter((r) => r.done).length}/${dash.roadmap.length}</span>
                <button class="btn btn-sm" id="openSampleMdBtn" type="button">📄 Mẫu .md v1.6</button>
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
            <div class="colhead" role="tablist">
              <button type="button" class="subtab" role="tab" aria-selected="${roadmapRightTab === 'errors'}" data-rtab="errors">SỔ LỖI HAY MẮC (${dash.error_logs.length})</button>
              <button type="button" class="subtab" role="tab" aria-selected="${roadmapRightTab === 'paraphrase'}" data-rtab="paraphrase">BẢNG PARAPHRASE READING (${dash.paraphrase_pairs.length})</button>
            </div>

            ${
              roadmapRightTab === 'errors'
                ? `
              <div style="display:flex;gap:4px;margin-bottom:8px;flex-wrap:wrap;">
                <input class="text-input" id="errCat" placeholder="Nguồn · Loại" style="width:120px;" value="Writing · Ngữ pháp" />
                <input class="text-input" id="errWrong" placeholder="Viết / nghe sai..." style="flex:1;min-width:90px;" />
                <input class="text-input" id="errRight" placeholder="Sửa chuẩn..." style="flex:1;min-width:90px;" />
                <button class="btn btn-sm" id="addErrBtn">+ THÊM</button>
              </div>
              <div id="errFormMsg" style="font-size:11.5px;margin-bottom:6px;${errFormMsg ? `color:${errFormErr ? 'var(--m-err)' : 'var(--m-ok)'};` : 'display:none;'}">${esc(errFormMsg)}</div>
              <div class="log-table-wrap">
                <table class="log-table">
                  <thead>
                    <tr><th>Nguồn / Loại</th><th>Hay mắc</th><th>Sửa chuẩn</th><th>Trạng thái trị lỗi</th><th>Lần</th></tr>
                  </thead>
                  <tbody>
                    ${
                      dash.error_logs.length > 0
                        ? dash.error_logs
                            .map(
                              (e) =>
                                `<tr>
                            <td>
                              <span class="tag soft">${esc(e.category)}</span>
                              <div class="small muted" style="margin-top:2px;">${esc(e.drill_mode)}</div>
                            </td>
                            <td class="err-del">${esc(e.wrong_text)}</td>
                            <td class="ok-add">${esc(e.right_text)}</td>
                            <td>
                              ${
                                e.status === 'mastered'
                                  ? `<span class="tag mastered">✅ Đã trị dứt điểm</span>`
                                  : `<span class="tag active-err">🔴 Đang trị · ${e.streak_distinct}/3</span>`
                              }
                              ${
                                e.drill_count === 0
                                  ? `<div style="margin-top:4px;"><button type="button" class="btn btn-sm" data-gen-err-drills="${e.id}" ${generatingAiDrillErrId === e.id ? 'disabled' : ''}>${generatingAiDrillErrId === e.id ? '⏳ Đang sinh...' : '✨ Sinh 3 bài tập AI'}</button></div>`
                                  : ''
                              }
                            </td>
                            <td>${e.count}</td>
                          </tr>`
                            )
                            .join('')
                        : `<tr><td colspan="5" class="table-empty-cell">Chưa có lỗi nào trong Sổ lỗi. Hãy nhập lỗi sai &amp; cách sửa chuẩn ở trên hoặc bấm <b>+ LƯU LỖI VÀO SỔ</b> sau khi chấm bài.</td></tr>`
                    }
                  </tbody>
                </table>
              </div>`
                : `
              <div style="display:flex;gap:4px;margin-bottom:8px;flex-wrap:wrap;">
                <input class="text-input" id="paraQTerm" placeholder="Từ câu hỏi" style="flex:1;min-width:95px;" />
                <input class="text-input" id="paraPassage" placeholder="Trong bài đọc" style="flex:1;min-width:110px;" />
                <input class="text-input" id="paraSource" placeholder="Cam / Study4" style="width:110px;" />
                <input class="text-input" id="paraTrap" placeholder="Ghi chú / Bẫy" style="flex:1;min-width:100px;" />
                <button type="button" class="btn btn-sm btn-primary" id="addParaBtn">+ THÊM</button>
              </div>
              <div id="paraFormMsg" class="status-msg ${paraFormErr ? 'err' : paraFormMsg ? 'ok' : ''}" style="margin-bottom:6px;${paraFormMsg ? '' : 'display:none;'}">${esc(paraFormMsg)}</div>
              <div class="log-table-wrap" style="max-height:360px;">
                <table class="para-table">
                  <colgroup>
                    <col style="width:22%" />
                    <col style="width:26%" />
                    <col style="width:22%" />
                    <col style="width:30%" />
                  </colgroup>
                  <thead>
                    <tr><th>Từ câu hỏi</th><th>Trong bài đọc</th><th>Nguồn / Dạng</th><th>Ghi chú / Bẫy</th></tr>
                  </thead>
                  <tbody>
                    ${
                      dash.paraphrase_pairs.length > 0
                        ? dash.paraphrase_pairs
                            .map(
                              (p) => `
                      <tr>
                        <td><b>${esc(p.question_term)}</b> ${p.source === 'ai_pack' ? '<span class="tag ai-pack">✨ Mới nạp</span>' : ''}</td>
                        <td>${esc(p.passage_paraphrase)}</td>
                        <td class="small">${esc(p.source_type)}</td>
                        <td>
                          <div class="row" style="justify-content:space-between;align-items:flex-start;">
                            <span class="muted">${esc(p.trap_note || '—')}</span>
                            <button type="button" class="btn ghost ibtn" data-del-para="${p.id}" title="Xóa">🗑</button>
                          </div>
                        </td>
                      </tr>`
                            )
                            .join('')
                        : `<tr><td colspan="4" class="table-empty-cell">Chưa có cặp Paraphrase nào. Hãy thêm từ câu hỏi &amp; cụm tương ứng trong bài đọc ở trên để tự động nuôi vào Micro-Check 2 phút.</td></tr>`
                    }
                  </tbody>
                </table>
              </div>
              <div class="small muted" style="margin-top:6px;">Các cặp Paraphrase tự động nuôi vào câu hỏi Micro-Check 2 phút.</div>`
            }
          </div>
        </div>
      </div>
    </div>

    <div class="tray-strip">
      <div class="tray-badge">
        <span class="dot-live"></span>
        <span><b>SYSTEM TRAY:</b> Chạy ngầm cùng Windows · Nhắc lúc ${esc(dash.settings.reminder_time)} nếu chưa học</span>
      </div>
      <div class="row" style="gap:10px;">
        <button type="button" class="btn btn-sm" id="openShortcutsBtn">⌨️ Phím tắt <kbd>?</kbd></button>
        <span style="color:var(--m-muted);">XD4T-1ELTS v${esc(APP_VERSION)} · Rust + Tauri v2 + SQLite</span>
      </div>
    </div>

    ${renderSpeakingStudio()}

    <!-- AI CONTENT PACK REFILL MODAL -->
    <div class="modal-backdrop ${aiPackOpen ? 'open' : ''}" id="aiPackModal">
      <div class="modal-box modal-wide" role="dialog" aria-modal="true" aria-labelledby="aiPackModalTitle" tabindex="-1">
        <div class="box-title"><span id="aiPackModalTitle">✨ NẠP GÓI CHỦ ĐỀ MỚI (GEMINI OFFLINE REFILL)</span><button class="btn btn-sm" id="closeAiPackBtn">✕</button></div>
        <div class="small muted" style="margin-bottom:8px;">
          Chỉ <b>1 HTTP Request duy nhất</b> nạp trọn <b>25 bài liên kết nhau</b> (+10 Collocations · +5 Dictation · +2 Đề Writing · +5 Paraphrase · +3 Luyện lỗi bám Top 3) cho Track <b>${isAim80 ? '8.0' : '6.5'}</b> để học offline mãi mãi.
        </div>
        ${
          !dash.settings.has_api_key
            ? `<div class="status-msg err" style="margin-bottom:10px;">
                Gắn Gemini API Key miễn phí trong Cài đặt để nạp thêm bài không giới hạn (hiện app đã có sẵn 120 từ · 48 Dictation · 48 Speaking · 60 Luyện lỗi · 24 đề Writing offline!).
                <div style="margin-top:6px;"><button type="button" class="btn btn-sm btn-primary" id="aiPackOpenSettingsBtn">Mở Cài đặt gắn Key →</button></div>
              </div>`
            : ''
        }
        <div class="label-uc">Chọn 1 trong 12 chủ đề IELTS (Gợi ý Tuần ${dash.current_week}: <b>${esc(IELTS_TOPICS[(Math.max(1, dash.current_week) - 1) % IELTS_TOPICS.length])}</b>)</div>
        <div class="topic-chips-grid">
          ${IELTS_TOPICS.map(
            (tp) => `<button type="button" class="topic-chip ${aiPackTopic === tp ? 'active' : ''}" data-pick-topic="${esc(tp)}">${esc(tp)}</button>`
          ).join('')}
        </div>
        <div class="form-row" style="margin-top:8px;">
          <label>Hoặc nhập chủ đề tùy ý (VD: Space Exploration, Bioethics...)</label>
          <input type="text" id="aiPackTopicIn" value="${esc(aiPackTopic)}" placeholder="Nhập chủ đề tiếng Anh..." />
        </div>
        ${aiPackError ? `<div class="status-msg err">${esc(aiPackError)}</div>` : ''}
        ${
          aiPackResult
            ? `<div class="status-msg ok">
                ✓ Đã nạp gói <b>${esc(aiPackResult.topic)}</b> (Track ${esc(aiPackResult.level)}):
                <b>+${aiPackResult.vocab_added}</b> từ · <b>+${aiPackResult.dictations_added}</b> Dictation · <b>+${aiPackResult.prompts_added}</b> Writing · <b>+${aiPackResult.paraphrases_added}</b> Paraphrase · <b>+${aiPackResult.drills_added}</b> Luyện lỗi!
              </div>`
            : ''
        }
        <div style="display:flex;justify-content:flex-end;gap:8px;margin-top:12px;">
          <button type="button" class="btn" id="cancelAiPackBtn">ĐÓNG</button>
          <button type="button" class="btn btn-primary" id="runAiPackBtn" ${aiPackLoading || !dash.settings.has_api_key ? 'disabled' : ''}>
            ${aiPackLoading ? '⏳ ĐANG NẠP 25 BÀI TỪ GEMINI...' : '✨ Nạp gói 25 bài (1 request) →'}
          </button>
        </div>
      </div>
    </div>

    <!-- KEYBOARD CHEATSHEET OVERLAY -->
    <div class="modal-backdrop ${shortcutsModalOpen ? 'open' : ''}" id="shortcutsModal">
      <div class="modal-box modal-wide" role="dialog" aria-modal="true" aria-labelledby="shortcutsModalTitle" tabindex="-1">
        <div class="box-title"><span id="shortcutsModalTitle">⌨️ BẢNG PHÍM TẮT LIÊN TỤC (0 CHẠM CHUỘT)</span><button class="btn btn-sm" id="closeShortcutsBtn">✕</button></div>
        <div class="kbd-grid">
          <div class="kbd-section">
            <div class="kbd-section-title">Toàn ứng dụng</div>
            <div class="kbd-row"><span>Chuyển 4 tab chính</span><span><kbd>Alt+1..4</kbd></span></div>
            <div class="kbd-row"><span>Mở / Đóng Phòng Luyện Nói</span><span><kbd>Alt+S</kbd></span></div>
            <div class="kbd-row"><span>Bật / Tắt bảng phím tắt này</span><span><kbd>?</kbd></span></div>
            <div class="kbd-row"><span>Đóng modal / drawer / bỏ focus</span><span><kbd>Esc</kbd></span></div>
          </div>
          <div class="kbd-section">
            <div class="kbd-section-title">Flashcard SRS</div>
            <div class="kbd-row"><span>Lật / Úp thẻ</span><span><kbd>Space</kbd> / <kbd>Enter</kbd></span></div>
            <div class="kbd-row"><span>Chấm Quên / Khó / Nhớ / Dễ</span><span><kbd>1</kbd> <kbd>2</kbd> <kbd>3</kbd> <kbd>4</kbd></span></div>
            <div class="kbd-row"><span>Đọc từ / Đọc ví dụ</span><span><kbd>R</kbd> / <kbd>E</kbd></span></div>
            <div class="kbd-row"><span>Chữa từ cứng đầu (Lưu: Ctrl+Enter)</span><span><kbd>C</kbd></span></div>
          </div>
          <div class="kbd-section">
            <div class="kbd-section-title">Quiz, Luyện Lỗi &amp; Micro-Check</div>
            <div class="kbd-row"><span>Chọn đáp án MCQ</span><span><kbd>1..4</kbd> / <kbd>A..D</kbd></span></div>
            <div class="kbd-row"><span>Nhịp đôi: Chấm → Câu tiếp</span><span><kbd>Enter</kbd> → <kbd>Enter</kbd></span></div>
            <div class="kbd-row"><span>Quên / Xem đáp án</span><span><kbd>F</kbd> / <kbd>Ctrl+H</kbd></span></div>
            <div class="kbd-row"><span>Tự xác nhận đúng / AI chấm sửa</span><span><kbd>Ctrl+Y</kbd> / <kbd>Ctrl+I</kbd></span></div>
          </div>
          <div class="kbd-section">
            <div class="kbd-section-title">Dictation, Writing &amp; Speaking</div>
            <div class="kbd-row"><span>Nghe lại Dictation / Chạy Speaking</span><span><kbd>Ctrl+Space</kbd></span></div>
            <div class="kbd-row"><span>Tốc độ Dictation 0.85x / 1.0x / 1.15x</span><span><kbd>Ctrl+1..3</kbd></span></div>
            <div class="kbd-row"><span>Đẩy lỗi Listening / Sang chặng Nói</span><span><kbd>Ctrl+L</kbd> / <kbd>Alt+N</kbd></span></div>
            <div class="kbd-row"><span>Chấm AI Writing/Speaking · Top 3</span><span><kbd>Ctrl+Enter</kbd> / <kbd>Alt+T</kbd></span></div>
          </div>
        </div>
      </div>
    </div>

    <!-- SETTINGS MODAL -->
    <div class="modal-backdrop" id="settingsModal">
      <div class="modal-box modal-wide" role="dialog" aria-modal="true" aria-labelledby="settingsModalTitle" tabindex="-1">
        <div class="box-title"><span id="settingsModalTitle">CÀI ĐẶT HỆ THỐNG &amp; DỮ LIỆU</span><button class="btn btn-sm" id="closeSettingsBtn">✕</button></div>

        <div class="form-row">
          <label>Chọn giao diện (4 phong cách · đổi tức thì)</label>
          ${renderThemeCards(curTheme, 'settings-theme-card')}
        </div>

        <div class="form-row">
          <label>Cấp độ lộ trình (Đổi tức thì giữa Tiêu chuẩn 6.5 &amp; Nâng cao 8.0)</label>
          ${renderLevelCards(isAim80, 'settings-level-card')}
        </div>

        <div class="form-row">
          <label>Giọng đọc TTS (Flashcard &amp; Nghe chép câu Dictation)</label>
          <select id="setTtsAccent">
            <option value="en-GB" ${dash.settings.tts_accent !== 'en-US' ? 'selected' : ''}>🇬🇧 Anh – Anh (en-GB)</option>
            <option value="en-US" ${dash.settings.tts_accent === 'en-US' ? 'selected' : ''}>🇺🇸 Anh – Mỹ (en-US)</option>
          </select>
        </div>

        <div class="step-card">
          <div class="step-head"><span>Dữ liệu học tập &amp; Widget Taskbar Windows</span></div>
          <div class="data-actions-row">
            <button class="btn btn-sm" id="exportBackupBtn" type="button">💾 Xuất file sao lưu (.db)</button>
            <button class="btn btn-sm" id="importBackupBtn" type="button">📂 Khôi phục từ file (.db)</button>
          </div>
          <div id="backupStatusMsg" class="status-msg ${backupOk === true ? 'ok' : backupOk === false ? 'err' : ''}" style="${backupMsg ? '' : 'display:none;'}">${esc(backupMsg)}</div>
          <label class="switch-row" style="margin-top:8px;">
            <input type="checkbox" id="setWidgetEnabled" ${widgetInfo.running || widgetInfo.autostart ? 'checked' : ''} />
            <span>Hiện thanh Widget nhỏ trên Taskbar Windows (tự khởi động cùng máy)</span>
          </label>
          <div id="widgetStatusMsg" class="status-msg" style="${widgetStatusMsg ? '' : 'display:none;'}">${esc(widgetStatusMsg)}</div>
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
          <label id="setKeyLabel">Gemini API Key ${dash.settings.has_api_key ? `(Đang lưu: ${esc(dash.settings.api_key_masked)})` : '(Chưa có)'}</label>
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
          <div id="apiTestMsg" class="status-msg ${apiTestOk === true ? 'ok' : apiTestOk === false ? 'err' : ''}" style="${apiTestMsg ? '' : 'display:none;'}">${esc(apiTestMsg)}</div>
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
        <div class="form-row">
          <div style="display:flex;align-items:center;gap:8px;">
            <input type="checkbox" id="setAutoRefill" style="width:auto;" ${dash.settings.auto_refill_enabled !== false ? 'checked' : ''} />
            <label for="setAutoRefill" style="margin:0;cursor:pointer;">Tự động nạp gói chủ đề AI hàng tuần (Chủ Nhật / đủ tiến độ học)</label>
          </div>
          <div class="small muted" style="margin-top:3px;">
            ${
              dash.auto_refill_status?.last_refill_at
                ? `Lần nạp gần nhất: <b>${esc(dash.auto_refill_status.last_refill_at)}</b> · Chủ đề <b>${esc(dash.auto_refill_status.last_refill_topic || '—')}</b>`
                : 'Chưa tự nạp lần nào (sẽ tự kích hoạt vào Chủ Nhật hoặc khi học đủ tiến độ nếu có API Key).'
            }
          </div>
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
            <button class="btn btn-sm" id="settingsToAiPackBtn" type="button">✨ Nạp thêm bài (AI)</button>
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
      <div class="modal-box modal-wide" role="dialog" aria-modal="true" aria-labelledby="welcomeModalTitle" tabindex="-1">
        <div class="box-title">
          <span id="welcomeModalTitle">${welcomeStep === 4 ? 'CHÀO MỪNG TỚI XD4T IELTS · HOÀN TẤT THIẾT LẬP' : `CHÀO MỪNG TỚI XD4T IELTS · THIẾT LẬP NHANH (${welcomeStep}/3)`}</span>
          <button class="btn btn-sm" id="skipWelcomeBtn" type="button">Bỏ qua</button>
        </div>
        <div class="wizard-steps">
          <div class="wizard-step-pill ${welcomeStep === 1 ? 'active' : welcomeStep > 1 ? 'done' : ''}">1. Chọn phong cách</div>
          <div class="wizard-step-pill ${welcomeStep === 2 ? 'active' : welcomeStep > 2 ? 'done' : ''}">2. Lộ trình học</div>
          <div class="wizard-step-pill ${welcomeStep === 3 ? 'active' : welcomeStep > 3 ? 'done' : ''}">3. Chấm AI (Audio &amp; Writing)</div>
        </div>
        ${
          welcomeStep === 1
            ? `
          <div class="step-card">
            <div class="step-head"><span>Bước 1 — Chọn phong cách bạn thích (bấm là đổi ngay)</span></div>
            <div class="small muted" style="margin-bottom:8px;">Bạn có thể đổi lại bất cứ lúc nào ngay trên thanh trên cùng của ứng dụng.</div>
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
            <div class="small muted" style="margin-bottom:10px;">App tích hợp sẵn lộ trình 24 tuần chuẩn v1.6.0 (Core ≤ 30p/ngày + Study4 + Speaking Studio + Dictation).</div>
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
            <div class="step-head"><span>Bước 3 — Chấm Writing &amp; Speaking Audio bằng AI (tùy chọn)</span></div>
            <div class="small muted" style="margin-bottom:8px;">Gắn <b>Google Gemini API Key miễn phí</b> để app tự chấm Writing &amp; Speaking Audio, hoặc bỏ qua để học offline 100% + dùng nút Copy Prompt miễn phí.</div>
            <div style="margin-bottom:8px;"><button class="btn btn-sm" id="welcomeOpenAiStudioBtn" type="button">↗ Mở Google AI Studio lấy key miễn phí</button></div>
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
              <div class="welcome-finish-desc">Mọi thứ đã sẵn sàng trên máy bạn. Từ giờ mỗi ngày chỉ cần 30 phút Core đều đặn nhé.</div>
              <div class="welcome-summary-box">
                <div class="welcome-summary-row"><span>Giao diện đang chọn</span><b>${esc(THEME_OPTIONS.find((t) => t.id === curTheme)?.label || curTheme)}</b></div>
                <div class="welcome-summary-row"><span>Lộ trình hiện tại</span><b>${dash.roadmap.length} tuần · Mục tiêu Band ${dash.settings.target_band.toFixed(1)}</b></div>
                <div class="welcome-summary-row"><span>Chấm AI (Writing + Speaking)</span><b>${dash.settings.has_api_key || welcomeKeyDraft.trim() ? '✓ Đã gắn Gemini API Key' : 'Chưa gắn key (dùng Prompt miễn phí)'}</b></div>
              </div>
              <button class="btn btn-primary welcome-start-btn" id="finishWelcomeBtn" type="button">Bắt đầu học ngay →</button>
            </div>
            <div class="welcome-finish-donate">
              <div class="donate-badge">☕ Tùy chọn · Mời cà phê tác giả</div>
              <div class="donate-qr-frame"><img src="${donateQrUrl}" alt="VietQR MB Bank 9888888881808" /></div>
              <div class="donate-caption">App hoàn toàn miễn phí &amp; không quảng cáo.</div>
              <div class="donate-bank-row"><code>MB · 9888888881808</code><button class="btn btn-sm" id="welcomeCopyStkBtn" type="button">Copy</button></div>
            </div>
          </div>`
        }
      </div>
    </div>

    <!-- CHART ZOOM LIGHTBOX MODAL -->
    <div class="modal-backdrop ${chartZoomOpen ? 'open' : ''}" id="chartZoomModal">
      <div class="modal-box modal-wide" role="dialog" aria-modal="true" aria-labelledby="chartZoomModalTitle" tabindex="-1">
        <div class="box-title">
          <span id="chartZoomModalTitle">📊 XEM BIỂU ĐỒ / SƠ ĐỒ TASK 1</span>
          <button class="btn btn-sm" id="closeChartZoomBtn" type="button">✕ Đóng</button>
        </div>
        <div class="chart-zoom-body">
          ${
            customTask1ImageDataUrl
              ? `<img src="${esc(customTask1ImageDataUrl)}" alt="Task 1 Chart" style="max-width:100%;max-height:72vh;display:block;margin:0 auto;" />`
              : (() => {
                  const sp = parseChartSpec(curPrompt?.chart_spec_json);
                  return sp ? renderChartInnerHtml(sp) : '<div class="small muted">Chưa có biểu đồ.</div>';
                })()
          }
        </div>
      </div>
    </div>

    <!-- ROADMAP IMPORT MODAL -->
    <div class="modal-backdrop" id="roadmapModal">
      <div class="modal-box modal-wide" role="dialog" aria-modal="true" aria-labelledby="roadmapModalTitle" tabindex="-1">
        <div class="box-title"><span id="roadmapModalTitle">NẠP LỘ TRÌNH CÁ NHÂN (.MD)</span><button class="btn btn-sm" id="closeRoadmapBtn">✕</button></div>
        <div class="step-card">
          <div class="step-head"><span>Bước 1 — Tạo hoặc xem mẫu file lộ trình v1.6.0</span></div>
          <div class="small muted" style="margin-bottom:8px;">Hỗ trợ đầy đủ: <code>## Nhiệm vụ Core</code>, <code>## Bảng Paraphrase</code>, <code>## Sentence Mining</code>, <code>- [ ] Tuần X</code> và bảng Markdown.</div>
          <div style="display:flex;gap:6px;flex-wrap:wrap;">
            <button class="btn btn-sm btn-primary" id="copyRoadmapPromptBtn">COPY PROMPT TẠO LỘ TRÌNH CHO AI</button>
            <button class="btn btn-sm" id="loadSampleRoadmapBtn">XEM MẪU CHUẨN V1.6</button>
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
          <textarea class="editor" id="roadmapMdInput" style="min-height:140px;" placeholder="Dán nội dung lo-trinh.md tại đây...">${esc(roadmapDraftMd)}</textarea>
        </div>
        <div class="step-card">
          <div class="step-head">
            <span>Bước 3 — Xem trước (Preview) &amp; Áp dụng</span>
            <button class="btn btn-sm" id="previewRoadmapBtn">PHÂN TÍCH &amp; XEM TRƯỚC</button>
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
              <div class="preview-stat"><span>Bảng Paraphrase</span><b>${roadmapPreview.paraphrase_count} cặp</b></div>
              <div class="preview-stat"><span>Từ vựng SRS</span><b>${roadmapPreview.vocab_count} thẻ</b></div>
              <div class="preview-stat"><span>Sổ lỗi</span><b>${roadmapPreview.error_logs_count} lỗi</b></div>
            </div>
            ${roadmapPreview.warnings.length ? `<div style="font-size:11px;color:var(--m-warn);margin-top:4px;">⚠ ${roadmapPreview.warnings.map((w) => esc(w)).join(' | ')}</div>` : ''}`
              : `<div class="small muted">Bấm <b>[PHÂN TÍCH &amp; XEM TRƯỚC]</b> để kiểm tra dữ liệu trước khi ghi vào SQLite (trạng thái tuần đã tick sẽ được bảo toàn).</div>`
          }
          <div style="display:flex;gap:14px;flex-wrap:wrap;margin-top:8px;font-size:11px;">
            <label style="display:flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" id="chkReplaceRoadmap" checked /><span>Thay thế lộ trình tuần (vẫn giữ trạng thái ✓ cũ)</span></label>
            <label style="display:flex;align-items:center;gap:6px;cursor:pointer;"><input type="checkbox" id="chkMergeVocabErrors" checked /><span>Gộp thêm từ vựng, Paraphrase &amp; Sổ lỗi</span></label>
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
      <div class="modal-box" role="dialog" aria-modal="true" aria-labelledby="scoreModalTitle" tabindex="-1">
        <div class="box-title"><span id="scoreModalTitle">NHẬP ĐIỂM ĐỀ VỪA LÀM</span><button class="btn btn-sm" id="closeScoreBtn">✕</button></div>
        <div class="form-row"><label>Kỹ năng</label>
          <select id="scSkill"><option value="Listening">Listening</option><option value="Reading">Reading</option><option value="Writing">Writing</option><option value="Speaking">Speaking</option></select>
        </div>
        <div class="form-row"><label>Nguồn đề (VD: Study4 Cam 15 Test 1)</label><input type="text" id="scSource" value="Study4 Cambridge" /></div>
        <div class="form-row"><label>Band điểm (0.0 – 9.0)</label><input type="number" step="0.5" min="0" max="9" id="scVal" value="5.5" /></div>
        <div class="form-row"><label>Ghi chú</label><input type="text" id="scNote" placeholder="VD: Đúng 23/40 câu" /></div>
        <div style="display:flex;justify-content:flex-end;gap:6px;margin-top:14px;">
          <button class="btn" id="cancelScoreBtn">HỦY</button>
          <button class="btn btn-primary" id="saveScoreBtn">LƯU ĐIỂM</button>
        </div>
      </div>
    </div>
  </div>`;

  bindEvents();
  applyContextAutoFocus();
}

function applyContextAutoFocus(): void {
  const activeDialog = getActiveDialogEl();
  if (activeDialog) {
    if (pendingModalFocus) {
      pendingModalFocus = false;
      focusActiveDialog();
    }
    return;
  }
  if (pendingRestoreFocus) {
    pendingRestoreFocus = false;
    restoreModalTriggerFocus();
    return;
  }
  const anyOverlayOpen =
    studioOpen ||
    addVocabOpen ||
    welcomeOpen ||
    aiPackOpen ||
    shortcutsModalOpen ||
    Boolean(document.querySelector('.modal-backdrop.open'));
  if (anyOverlayOpen) return;

  if (activeTab === 'vocab' && vocabSubMode === 'dictation' && !dictationSummary && dictationBatch[dictationIdx] && !dictationChecked) {
    document.getElementById('dictInput')?.focus();
    return;
  }
  if (activeTab === 'vocab' && vocabSubMode === 'quiz') {
    if (quizPillMode === 'fix' && !errorDrillSummary && errorDrillBatch[errorDrillIdx]?.drill_mode === 'TYPING' && !errorDrillFeedback) {
      document.getElementById('errDrillInput')?.focus();
    } else if (
      quizPillMode === 'micro' &&
      !microSummary &&
      microQuestions[microIdx] &&
      microQuestions[microIdx].options.length === 0 &&
      !microFeedback
    ) {
      document.getElementById('microInput')?.focus();
    } else if (quizPillMode === 'vocab' && quizPhase === 'playing' && quizQuestions[quizIdx]?.question_type === 'typing' && !quizFeedback) {
      document.getElementById('quizTypingInput')?.focus();
    }
  }
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
  pendingRestoreFocus = true;
  renderApp();
}

function bindEvents() {
  const bindActivate = (el: HTMLElement, handler: () => void | Promise<void>) => {
    el.onclick = () => {
      void handler();
    };
    el.onkeydown = (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        void handler();
      }
    };
  };

  document.querySelectorAll<HTMLButtonElement>('.nav-tab').forEach((btn) => {
    btn.onclick = () => {
      window.speechSynthesis?.cancel();
      stopMicroTimer();
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
      applyThemeInPlace(dash.settings.theme);
    };
  }

  document.querySelectorAll<HTMLElement>('[data-quest-id]').forEach((el) => {
    bindActivate(el, async () => {
      const id = Number(el.dataset.questId);
      dash = await invoke<DashboardState>('toggle_quest', { id });
      renderApp();
    });
  });

  document.querySelectorAll<HTMLElement>('[data-go]').forEach((btn) => {
    btn.onclick = (e) => {
      e.stopPropagation();
      void goToTarget(btn.dataset.go!);
    };
  });

  // Tab 2: Writing
  const toggleTop3Btn = document.getElementById('toggleTop3Btn');
  if (toggleTop3Btn) {
    toggleTop3Btn.onclick = () => {
      top3Collapsed = !top3Collapsed;
      renderApp();
    };
  }

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
      customTask1ImageDataUrl = '';
      renderApp();
    };
  }

  document.getElementById('zoomChartBtn')?.addEventListener('click', () => {
    rememberModalTrigger();
    chartZoomOpen = true;
    pendingModalFocus = true;
    renderApp();
  });
  document.getElementById('closeChartZoomBtn')?.addEventListener('click', () => {
    chartZoomOpen = false;
    pendingRestoreFocus = true;
    renderApp();
  });
  document.getElementById('chartZoomModal')?.addEventListener('click', (e) => {
    if (e.target === document.getElementById('chartZoomModal')) {
      chartZoomOpen = false;
      pendingRestoreFocus = true;
      renderApp();
    }
  });
  document.getElementById('uploadTask1ImgBtn')?.addEventListener('click', () => {
    document.getElementById('task1ImgInput')?.click();
  });
  document.getElementById('task1ImgInput')?.addEventListener('change', (e) => {
    const file = (e.target as HTMLInputElement).files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = () => {
      customTask1ImageDataUrl = String(reader.result || '');
      renderApp();
    };
    reader.readAsDataURL(file);
  });
  document.getElementById('clearCustomTask1ImgBtn')?.addEventListener('click', () => {
    customTask1ImageDataUrl = '';
    renderApp();
  });
  document.getElementById('dismissAutoRefillToastBtn')?.addEventListener('click', () => {
    autoRefillToastMsg = '';
    renderApp();
  });

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
      const focusNotes = (dash?.settings.writing_focus_notes || []).map((n, i) => `${i + 1}. ${n}`).join('\n');
      const targetLabel = isAim80Mode()
        ? 'bản viết lại Band 8.0 (ưu tiên collocations C1–C2, hedging học thuật & câu phức sắc sảo)'
        : 'bản viết lại Band 6.5';
      const chartSummary = summarizeChartSpecForPrompt(p?.chart_spec_json);
      const chartPart = chartSummary ? `\n\n${chartSummary}` : '';
      const rubric = `Hãy đóng vai Giám khảo IELTS Writing chấm bài dưới đây theo 4 tiêu chí (TA, CC, LR, GRA) và trả kết quả rõ ràng bằng tiếng Việt + ${targetLabel}.\n\n[ĐIỂM YẾU CẦN SOI KỸ]:\n${focusNotes}\n\n[CHẾ ĐỘ]: ${writingMode}\n[ĐỀ BÀI]: ${p?.prompt || ''}${chartPart}\n\n[BÀI VIẾT]:\n${content}`;
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

  const markWritingDoneBtn = document.getElementById('markWritingDoneBtn');
  if (markWritingDoneBtn) {
    markWritingDoneBtn.onclick = async () => {
      const p = getCurrentPrompt();
      dash = await invoke<DashboardState>('mark_writing_practiced', {
        promptId: p?.id || null,
      });
      writingStatusMsg = '✓ Đã đánh dấu hoàn thành bài Writing hôm nay & cộng phút học!';
      writingStatusErr = false;
      renderApp();
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
      const restored = document.getElementById('essayInput') as HTMLTextAreaElement | null;
      if (restored) restored.value = content;
      try {
        const p = getCurrentPrompt();
        const chartSummary = summarizeChartSpecForPrompt(p?.chart_spec_json);
        const fullPrompt = chartSummary ? `${p?.prompt || ''}\n\n${chartSummary}` : p?.prompt || '';
        const res = await invoke<GradeResponseDto>('grade_writing', {
          mode: writingMode,
          prompt: fullPrompt,
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

  // Tab 3: Sub-modes & Flashcard
  document.querySelectorAll<HTMLButtonElement>('[data-vsub]').forEach((btn) => {
    btn.onclick = async () => {
      window.speechSynthesis?.cancel();
      stopMicroTimer();
      vocabSubMode = btn.dataset.vsub as typeof vocabSubMode;
      if (vocabSubMode === 'dictation' && !dictationBatch.length) {
        dictationBatch = await invoke<DictationSentence[]>('get_dictation_batch', { count: 5 });
        dictationIdx = 0;
      }
      renderApp();
    };
  });

  document.querySelectorAll<HTMLButtonElement>('[data-face]').forEach((btn) => {
    btn.onclick = () => {
      cardFaceMode = btn.dataset.face as typeof cardFaceMode;
      cardFlipped = false;
      renderApp();
    };
  });

  const leechChip = document.getElementById('leechChip');
  if (leechChip) {
    leechChip.onclick = async () => {
      onlyLeechFilter = !onlyLeechFilter;
      [dueCards, dash] = await Promise.all([
        invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter }),
        invoke<DashboardState>('get_dashboard'),
      ]);
      cardIdx = 0;
      cardFlipped = false;
      renderApp();
    };
  }

  const clearLeechFilterBtn = document.getElementById('clearLeechFilterBtn');
  if (clearLeechFilterBtn) {
    clearLeechFilterBtn.onclick = async () => {
      onlyLeechFilter = false;
      [dueCards, dash] = await Promise.all([
        invoke<VocabCard[]>('get_due_vocab', { onlyLeech: false }),
        invoke<DashboardState>('get_dashboard'),
      ]);
      cardIdx = 0;
      cardFlipped = false;
      renderApp();
    };
  }

  document.querySelectorAll<HTMLButtonElement>('[data-say]').forEach((btn) => {
    btn.onclick = (e) => {
      e.stopPropagation();
      const accent = dash?.settings.tts_accent || 'en-GB';
      speakText(btn.dataset.say || '', accent, 1.0);
    };
  });

  const flipCardBtn = document.getElementById('flipCardBtn');
  if (flipCardBtn) {
    flipCardBtn.onclick = () => {
      cardFlipped = !cardFlipped;
      renderApp();
    };
  }

  const openLeechFixBtn = document.getElementById('openLeechFixBtn');
  if (openLeechFixBtn) {
    openLeechFixBtn.onclick = () => {
      const card = dueCards[cardIdx];
      if (!card) return;
      leechFixCardId = card.id;
      leechFixDraft = '';
      leechFixMsg = '';
      renderApp();
      document.getElementById('leechSentenceInput')?.focus();
    };
  }

  const leechSentenceInput = document.getElementById('leechSentenceInput') as HTMLTextAreaElement | null;
  if (leechSentenceInput) {
    leechSentenceInput.oninput = () => {
      leechFixDraft = leechSentenceInput.value;
    };
  }

  const cancelLeechFixBtn = document.getElementById('cancelLeechFixBtn');
  if (cancelLeechFixBtn) {
    cancelLeechFixBtn.onclick = () => {
      leechFixCardId = null;
      leechFixDraft = '';
      leechFixMsg = '';
      renderApp();
    };
  }

  const saveLeechFixBtn = document.getElementById('saveLeechFixBtn');
  if (saveLeechFixBtn) {
    saveLeechFixBtn.onclick = async () => {
      const card = dueCards[cardIdx];
      if (!card) return;
      const sent = (document.getElementById('leechSentenceInput') as HTMLTextAreaElement)?.value || '';
      try {
        await invoke<VocabCard>('cure_leech_card', { cardId: card.id, userSentence: sent });
        leechFixCardId = null;
        leechFixDraft = '';
        leechFixMsg = '';
        dash = await invoke<DashboardState>('get_dashboard');
        dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
        cardIdx = 0;
        cardFlipped = false;
        renderApp();
      } catch (err) {
        leechFixMsg = String(err);
        renderApp();
      }
    };
  }

  const hideVocabBtn = document.getElementById('hideVocabBtn');
  if (hideVocabBtn) {
    hideVocabBtn.onclick = async () => {
      const card = dueCards[cardIdx];
      if (!card) return;
      confirmDeleteCardId = null;
      dash = await invoke<DashboardState>('toggle_hide_vocab_card', { cardId: card.id, hidden: true });
      dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
      if (cardIdx >= dueCards.length) cardIdx = 0;
      cardFlipped = false;
      undoHideCard = { id: card.id, word: card.word };
      if (undoHideTimer !== null) window.clearTimeout(undoHideTimer);
      undoHideTimer = window.setTimeout(() => {
        undoHideCard = null;
        undoHideTimer = null;
        if (activeTab === 'vocab' && vocabSubMode === 'flashcard') renderApp();
      }, 5000);
      renderApp();
    };
  }

  document.getElementById('undoHideCardBtn')?.addEventListener('click', async () => {
    if (!undoHideCard) return;
    const restoreId = undoHideCard.id;
    undoHideCard = null;
    if (undoHideTimer !== null) {
      window.clearTimeout(undoHideTimer);
      undoHideTimer = null;
    }
    dash = await invoke<DashboardState>('toggle_hide_vocab_card', { cardId: restoreId, hidden: false });
    dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
    const restoredIdx = dueCards.findIndex((c) => c.id === restoreId);
    if (restoredIdx >= 0) cardIdx = restoredIdx;
    cardFlipped = false;
    renderApp();
  });

  const deleteVocabBtn = document.getElementById('deleteVocabBtn');
  if (deleteVocabBtn) {
    deleteVocabBtn.onclick = async () => {
      const card = dueCards[cardIdx];
      if (!card) return;
      if (confirmDeleteCardId !== card.id) {
        confirmDeleteCardId = card.id;
        if (confirmDeleteTimer !== null) window.clearTimeout(confirmDeleteTimer);
        confirmDeleteTimer = window.setTimeout(() => {
          confirmDeleteCardId = null;
          confirmDeleteTimer = null;
          if (activeTab === 'vocab' && vocabSubMode === 'flashcard') renderApp();
        }, 4000);
        renderApp();
        return;
      }
      confirmDeleteCardId = null;
      if (confirmDeleteTimer !== null) {
        window.clearTimeout(confirmDeleteTimer);
        confirmDeleteTimer = null;
      }
      dash = await invoke<DashboardState>('delete_vocab_card', { cardId: card.id });
      dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
      if (cardIdx >= dueCards.length) cardIdx = 0;
      cardFlipped = false;
      renderApp();
    };
  }

  document.querySelectorAll<HTMLButtonElement>('.srs-btn').forEach((btn) => {
    btn.onclick = async () => {
      const card = dueCards[cardIdx];
      if (!card) return;
      confirmDeleteCardId = null;
      const quality = Number(btn.dataset.q);
      await invoke('review_vocab', { cardId: card.id, quality });
      dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
      if (quality === 0 && dueCards.length > 1) {
        cardIdx = (cardIdx + 1) % dueCards.length;
      } else if (cardIdx >= dueCards.length) {
        cardIdx = 0;
      }
      cardFlipped = false;
      leechFixCardId = null;
      dash = await invoke<DashboardState>('get_dashboard');
      renderApp();
    };
  });

  // Add Vocab Drawer
  const openAddVocabBtn = document.getElementById('openAddVocabBtn');
  if (openAddVocabBtn) {
    openAddVocabBtn.onclick = () => {
      rememberModalTrigger();
      addVocabOpen = true;
      addVocabMsg = '';
      renderApp();
      document.getElementById('addWordIn')?.focus();
    };
  }
  const closeDrawer = () => {
    addVocabOpen = false;
    pendingRestoreFocus = true;
    renderApp();
  };
  document.getElementById('closeAddVocabBtn')?.addEventListener('click', closeDrawer);
  document.getElementById('cancelAddVocabBtn')?.addEventListener('click', closeDrawer);

  document.getElementById('addWordIn')?.addEventListener('input', (e) => {
    addVocabWord = (e.target as HTMLInputElement).value;
  });
  document.getElementById('addMeaningIn')?.addEventListener('input', (e) => {
    addVocabMeaning = (e.target as HTMLInputElement).value;
  });
  document.getElementById('addExampleIn')?.addEventListener('input', (e) => {
    addVocabExample = (e.target as HTMLTextAreaElement).value;
  });
  document.getElementById('addTopicSel')?.addEventListener('change', (e) => {
    addVocabTopic = (e.target as HTMLSelectElement).value;
  });

  const saveAddVocabBtn = document.getElementById('saveAddVocabBtn');
  if (saveAddVocabBtn) {
    saveAddVocabBtn.onclick = async () => {
      try {
        await invoke<VocabCard>('add_vocab_card', {
          word: addVocabWord,
          meaning: addVocabMeaning,
          example: addVocabExample,
          topic: addVocabTopic,
        });
        addVocabWord = '';
        addVocabMeaning = '';
        addVocabExample = '';
        addVocabMsg = '';
        addVocabOpen = false;
        pendingRestoreFocus = true;
        dash = await invoke<DashboardState>('get_dashboard');
        dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
        cardIdx = 0;
        renderApp();
      } catch (err) {
        addVocabMsg = String(err);
        renderApp();
      }
    };
  }

  // Quiz Pills & Modes
  document.querySelectorAll<HTMLButtonElement>('[data-qpill]').forEach((btn) => {
    btn.onclick = async () => {
      const nextPill = btn.dataset.qpill as typeof quizPillMode;
      if (nextPill !== 'micro') {
        stopMicroTimer();
      }
      quizPillMode = nextPill;
      if (quizPillMode === 'fix' && !errorDrillBatch.length && !errorDrillSummary) {
        await loadErrorDrills();
      } else if (quizPillMode === 'micro' && !microQuestions.length && !microSummary) {
        await startMicroCheck();
      }
      renderApp();
    };
  });

  // Error-Driven Mastery Drills
  document.getElementById('startErrDrillBtn')?.addEventListener('click', async () => {
    await loadErrorDrills();
    renderApp();
  });
  document.getElementById('restartErrDrillBtn')?.addEventListener('click', async () => {
    await loadErrorDrills();
    renderApp();
  });

  const recordErrorDrillOutcome = (correct: boolean, selectedOpt?: string, matchReason?: string) => {
    const q = errorDrillBatch[errorDrillIdx];
    if (!q || errorDrillFeedback) return;
    errorDrillAnswers.push({
      error_log_id: q.error_log_id,
      drill_id: q.drill_id,
      context_tag: q.context_tag,
      correct,
    });
    const nextStreak = correct ? q.streak_distinct + 1 : 0;
    const masteredNow = correct && nextStreak >= 3 && q.status !== 'mastered';
    errorDrillFeedback = {
      correct,
      target: q.answer,
      selectedOpt,
      matchReason,
      rationaleVi: q.rationale_vi,
      streakAfter: nextStreak,
      masteredNow,
    };
    renderApp();
  };

  document.querySelectorAll<HTMLButtonElement>('[data-edrill-opt]').forEach((btn) => {
    btn.onclick = () => {
      const q = errorDrillBatch[errorDrillIdx];
      if (!q || errorDrillFeedback) return;
      const opt = q.options[Number(btn.dataset.edrillOpt)];
      recordErrorDrillOutcome(opt === q.answer, opt);
    };
  });

  const errDrillIn = document.getElementById('errDrillInput') as HTMLInputElement | null;
  const submitErrDrillTyping = () => {
    const q = errorDrillBatch[errorDrillIdx];
    if (!q || errorDrillFeedback || !errDrillIn) return;
    errorDrillInput = errDrillIn.value;
    const ok = isTypingAnswerMatch(errorDrillInput, q.answer);
    recordErrorDrillOutcome(ok, undefined, ok ? 'Chính xác' : undefined);
  };
  if (errDrillIn) {
    errDrillIn.oninput = () => {
      errorDrillInput = errDrillIn.value;
    };
  }
  document.getElementById('checkErrDrillBtn')?.addEventListener('click', submitErrDrillTyping);
  document.getElementById('errDrillSelfAcceptBtn')?.addEventListener('click', () => {
    if (!errorDrillFeedback || errorDrillFeedback.correct) return;
    errorDrillFeedback.correct = true;
    errorDrillFeedback.selfAccepted = true;
    errorDrillFeedback.matchReason = 'Tự xác nhận đúng';
    const last = errorDrillAnswers[errorDrillAnswers.length - 1];
    if (last) last.correct = true;
    renderApp();
  });
  document.getElementById('errDrillAiGradeBtn')?.addEventListener('click', async () => {
    const q = errorDrillBatch[errorDrillIdx];
    if (!q || !errorDrillFeedback || errorDrillFeedback.correct || errorDrillFeedback.aiChecking) return;
    errorDrillFeedback.aiChecking = true;
    errorDrillFeedback.aiError = undefined;
    renderApp();
    try {
      const res = await invoke<ErrorFixAiGrade>('grade_error_fix_with_ai', {
        wrongText: q.stem,
        expectedText: q.answer,
        userAnswer: errorDrillInput,
      });
      if (errorDrillFeedback) {
        errorDrillFeedback.aiChecking = false;
        errorDrillFeedback.aiFeedback = res.feedback;
        if (res.correct && !errorDrillFeedback.correct) {
          errorDrillFeedback.correct = true;
          errorDrillFeedback.matchReason = 'AI xác nhận đúng';
          const last = errorDrillAnswers[errorDrillAnswers.length - 1];
          if (last) last.correct = true;
        }
      }
    } catch (e) {
      if (errorDrillFeedback) {
        errorDrillFeedback.aiChecking = false;
        errorDrillFeedback.aiError = String(e);
      }
    }
    renderApp();
  });
  document.getElementById('nextErrDrillBtn')?.addEventListener('click', async () => {
    if (errorDrillIdx + 1 >= errorDrillBatch.length) {
      errorDrillSummary = await invoke<ErrorDrillSubmitResult>('submit_error_drill_session', {
        results: errorDrillAnswers,
      });
      dash = await invoke<DashboardState>('get_dashboard');
      errorDrillBatch = [];
      errorDrillFeedback = null;
    } else {
      errorDrillIdx += 1;
      errorDrillInput = '';
      errorDrillFeedback = null;
    }
    renderApp();
  });

  // Micro-Check
  document.getElementById('startMicroBtn')?.addEventListener('click', async () => {
    await startMicroCheck();
    renderApp();
  });
  document.getElementById('restartMicroBtn')?.addEventListener('click', async () => {
    await startMicroCheck();
    renderApp();
  });
  document.getElementById('microFinishEarlyBtn')?.addEventListener('click', () => {
    void finishMicroEarly();
  });
  document.querySelectorAll<HTMLButtonElement>('[data-micro-opt]').forEach((btn) => {
    btn.onclick = () => {
      const q = microQuestions[microIdx];
      if (!q || microFeedback) return;
      const opt = q.options[Number(btn.dataset.microOpt)];
      const ok = opt === q.answer_text;
      microAnswers.push({ item_type: q.item_type, ref_id: q.ref_id, correct: ok, gave_up: false });
      microFeedback = { correct: ok, gaveUp: false, target: q.answer_text, note: q.meaning_or_note };
      renderApp();
    };
  });
  const microInEl = document.getElementById('microInput') as HTMLTextAreaElement | null;
  const checkMicro = (gaveUp = false) => {
    const q = microQuestions[microIdx];
    if (!q || microFeedback) return;
    const val = microInEl?.value || '';
    microInput = val;
    let ok = false;
    let matchReason: string | undefined;
    let diffHtml: string | undefined;
    const isErrFix = q.item_type === 'error_fix';
    if (!gaveUp) {
      if (isErrFix) {
        const evalRes = evaluateErrorFixAnswer(val, q.prompt_text, q.answer_text);
        ok = evalRes.matched;
        matchReason = evalRes.matchReason;
        if (!ok) {
          diffHtml = buildBestWordDiffHtml(val, evalRes.variants, q.answer_text);
        }
      } else {
        ok = isTypingAnswerMatch(val, q.answer_text);
      }
    }
    microAnswers.push({ item_type: q.item_type, ref_id: q.ref_id, correct: ok, gave_up: gaveUp });
    microFeedback = {
      correct: ok,
      gaveUp,
      target: q.answer_text,
      note: q.meaning_or_note,
      matchReason,
      diffHtml,
      isErrorFix: isErrFix,
    };
    renderApp();
  };
  if (microInEl) {
    microInEl.oninput = () => {
      microInput = microInEl.value;
    };
  }
  document.getElementById('microCheckBtn')?.addEventListener('click', () => checkMicro(false));
  document.getElementById('microGiveUpBtn')?.addEventListener('click', () => checkMicro(true));
  document.getElementById('microSelfAcceptBtn')?.addEventListener('click', () => {
    if (!microFeedback || microFeedback.correct) return;
    microFeedback.correct = true;
    microFeedback.selfAccepted = true;
    microFeedback.matchReason = 'Tự xác nhận đúng nghĩa';
    const lastAns = microAnswers[microAnswers.length - 1];
    if (lastAns) lastAns.correct = true;
    renderApp();
  });
  document.getElementById('microAiGradeBtn')?.addEventListener('click', async () => {
    const q = microQuestions[microIdx];
    if (!q || !microFeedback || microFeedback.correct || microFeedback.aiChecking) return;
    microFeedback.aiChecking = true;
    microFeedback.aiError = undefined;
    renderApp();
    try {
      const res = await invoke<ErrorFixAiGrade>('grade_error_fix_with_ai', {
        wrongText: q.prompt_text,
        expectedText: q.answer_text,
        userAnswer: microInput,
      });
      if (microFeedback) {
        microFeedback.aiChecking = false;
        microFeedback.aiFeedback = res.feedback;
        if (res.correct && !microFeedback.correct) {
          microFeedback.correct = true;
          microFeedback.matchReason = 'AI xác nhận đúng';
          const lastAns = microAnswers[microAnswers.length - 1];
          if (lastAns) lastAns.correct = true;
        }
      }
    } catch (e) {
      if (microFeedback) {
        microFeedback.aiChecking = false;
        microFeedback.aiError = String(e);
      }
    }
    renderApp();
  });
  document.getElementById('microNextBtn')?.addEventListener('click', async () => {
    if (microIdx + 1 >= microQuestions.length) {
      stopMicroTimer();
      microSummary = await invoke<MicroCheckSubmitResult>('submit_micro_check', { results: microAnswers });
      dash = await invoke<DashboardState>('get_dashboard');
      dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
      microFeedback = null;
    } else {
      microIdx += 1;
      microInput = '';
      microFeedback = null;
    }
    renderApp();
  });

  // Vocab Quiz (10 questions)
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

  document.getElementById('bannerStartQuizBtn')?.addEventListener('click', () => {
    activeTab = 'vocab';
    vocabSubMode = 'quiz';
    quizPillMode = 'vocab';
    void startQuizRun();
  });
  document.querySelectorAll<HTMLButtonElement>('[data-qmode]').forEach((btn) => {
    btn.onclick = () => {
      quizMode = btn.dataset.qmode as typeof quizMode;
      renderApp();
    };
  });
  document.getElementById('startQuizBtn')?.addEventListener('click', () => {
    void startQuizRun();
  });

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

  document.getElementById('quizGiveUpBtn')?.addEventListener('click', () => {
    const typed = (document.getElementById('quizTypingInput') as HTMLInputElement | null)?.value;
    recordQuizAnswer(false, true, undefined, typed);
  });

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
  document.getElementById('quizCheckTypingBtn')?.addEventListener('click', checkTyping);
  if (quizTypingInput && !quizFeedback) {
    quizTypingInput.focus();
  }

  const quizNextBtn = document.getElementById('quizNextBtn');
  if (quizNextBtn) {
    const goNext = async () => {
      if (quizSubmitting) return;
      if (quizIdx + 1 < quizQuestions.length) {
        quizIdx += 1;
        quizFeedback = null;
        renderApp();
        document.getElementById('quizTypingInput')?.focus();
      } else {
        quizSubmitting = true;
        renderApp();
        try {
          quizSummary = await invoke<QuizSubmitResult>('submit_vocab_quiz', {
            mode: quizMode,
            results: quizAnswers,
          });
          dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
          dash = await invoke<DashboardState>('get_dashboard');
          quizPhase = 'summary';
          quizFeedback = null;
        } catch (err) {
          quizErrorMsg = `Không thể lưu kết quả Quiz: ${String(err)}`;
        } finally {
          quizSubmitting = false;
          renderApp();
        }
      }
    };
    quizNextBtn.onclick = () => {
      void goNext();
    };
  }

  document.getElementById('quizReviewForgottenBtn')?.addEventListener('click', async () => {
    dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: false });
    onlyLeechFilter = false;
    cardIdx = 0;
    cardFlipped = false;
    vocabSubMode = 'flashcard';
    quizPhase = 'setup';
    renderApp();
  });

  document.getElementById('quizRestartBtn')?.addEventListener('click', () => {
    quizPhase = 'setup';
    quizFeedback = null;
    quizSummary = null;
    renderApp();
  });

  // Dictation Mode
  document.getElementById('loadDictBatchBtn')?.addEventListener('click', async () => {
    dictationBatch = await invoke<DictationSentence[]>('get_dictation_batch', { count: 5 });
    dictationIdx = 0;
    dictationChecked = null;
    dictationResults = [];
    dictationSummary = null;
    renderApp();
    if (dictationAutoPlay && dictationBatch[0]) {
      speakText(dictationBatch[0].sentence, dash?.settings.tts_accent || 'en-GB', dictationRate);
    }
  });

  document.getElementById('dictNewBatchBtn')?.addEventListener('click', async () => {
    dictationBatch = await invoke<DictationSentence[]>('get_dictation_batch', { count: 5 });
    dictationIdx = 0;
    dictationInput = '';
    dictationHintWords = 0;
    dictationChecked = null;
    dictationResults = [];
    dictationSummary = null;
    renderApp();
    if (dictationAutoPlay && dictationBatch[0]) {
      speakText(dictationBatch[0].sentence, dash?.settings.tts_accent || 'en-GB', dictationRate);
    }
  });

  document.querySelectorAll<HTMLButtonElement>('[data-rate]').forEach((btn) => {
    btn.onclick = () => {
      dictationRate = Number(btn.dataset.rate);
      renderApp();
    };
  });

  document.getElementById('dictAutoPlayChip')?.addEventListener('click', () => {
    dictationAutoPlay = !dictationAutoPlay;
    renderApp();
  });

  document.getElementById('dictPlayBtn')?.addEventListener('click', () => {
    const cur = dictationBatch[dictationIdx];
    if (!cur) return;
    speakText(cur.sentence, dash?.settings.tts_accent || 'en-GB', dictationRate);
  });

  const dictInputEl = document.getElementById('dictInput') as HTMLTextAreaElement | null;
  const runDictCheck = () => {
    const cur = dictationBatch[dictationIdx];
    if (!cur) return;
    const text = dictInputEl ? dictInputEl.value : dictationInput;
    dictationInput = text;
    const { tokens, pct, wrongWords } = computeWordDiff(text, cur.sentence);
    const pairs = extractDictationErrorPairs(text, cur.sentence, 3);
    const tokensHtml = tokens
      .map((t) =>
        t.kind === 'ok'
          ? `<span class="w-ok">${esc(t.word)}</span>`
          : t.kind === 'miss'
            ? `<span class="w-fix">${esc(t.word)}</span>`
            : `<span class="w-bad">${esc(t.word)}</span>`
      )
      .join(' ');
    dictationChecked = { pct, tokensHtml, wrongWords, pairs, pushed: false };
    renderApp();
  };

  if (dictInputEl) {
    dictInputEl.oninput = () => {
      dictationInput = dictInputEl.value;
      if (dictationChecked) {
        dictationChecked = null;
      }
    };
  }
  document.getElementById('dictCheckBtn')?.addEventListener('click', runDictCheck);
  document.getElementById('dictHintBtn')?.addEventListener('click', () => {
    dictationHintWords += 1;
    renderApp();
  });

  document.getElementById('pushDictErrBtn')?.addEventListener('click', async () => {
    const cur = dictationBatch[dictationIdx];
    if (!cur || !dictationChecked || dictationChecked.pushed || !dictationChecked.pairs.length) return;
    dash = await invoke<DashboardState>('log_dictation_errors', {
      sentenceId: cur.id,
      trapFocus: cur.trap_focus,
      sentence: cur.sentence,
      pairs: dictationChecked.pairs,
    });
    dictationChecked.pushed = true;
    renderApp();
  });

  document.getElementById('dictNextBtn')?.addEventListener('click', async () => {
    const cur = dictationBatch[dictationIdx];
    if (!cur || !dictationChecked) return;
    dictationResults.push({
      sentence_id: cur.id,
      accuracy_pct: dictationChecked.pct,
      wrong_words: dictationChecked.wrongWords,
    });
    if (dictationIdx + 1 >= dictationBatch.length) {
      dictationSummary = await invoke<DictationSubmitResult>('submit_dictation_session', {
        results: dictationResults,
      });
      dash = await invoke<DashboardState>('get_dashboard');
      renderApp();
    } else {
      dictationIdx += 1;
      dictationInput = '';
      dictationHintWords = 0;
      dictationChecked = null;
      renderApp();
      const nextCur = dictationBatch[dictationIdx];
      if (dictationAutoPlay && nextCur) {
        speakText(nextCur.sentence, dash?.settings.tts_accent || 'en-GB', dictationRate);
      }
    }
  });

  // Tab 4: Roadmap, Error Log & Paraphrase
  document.querySelectorAll<HTMLElement>('[data-rm-id]').forEach((el) => {
    bindActivate(el, async () => {
      const id = Number(el.dataset.rmId);
      dash = await invoke<DashboardState>('toggle_roadmap_item', { id });
      renderApp();
    });
  });

  document.querySelectorAll<HTMLButtonElement>('[data-rtab]').forEach((btn) => {
    btn.onclick = () => {
      roadmapRightTab = btn.dataset.rtab as typeof roadmapRightTab;
      renderApp();
    };
  });

  document.getElementById('openStudy4Btn')!.onclick = () => openUrl('https://study4.com/tests/ielts/');
  document.getElementById('openIotBtn')!.onclick = () => openUrl('https://ieltsonlinetests.com/');

  document.getElementById('addErrBtn')?.addEventListener('click', async () => {
    const category = (document.getElementById('errCat') as HTMLInputElement).value.trim() || 'Ngữ pháp';
    const wrongInput = document.getElementById('errWrong') as HTMLInputElement;
    const rightInput = document.getElementById('errRight') as HTMLInputElement;
    const wrongText = wrongInput.value.trim();
    const rightText = rightInput.value.trim();
    if (!wrongText || !rightText) {
      errFormMsg = '✕ Vui lòng nhập đầy đủ Lỗi sai và Cách sửa chuẩn.';
      errFormErr = true;
      const msgEl = document.getElementById('errFormMsg');
      if (msgEl) {
        msgEl.textContent = errFormMsg;
        msgEl.style.color = 'var(--m-err)';
        msgEl.style.display = 'block';
      }
      (!wrongText ? wrongInput : rightInput).focus();
      return;
    }
    errFormMsg = '✓ Đã thêm vào Sổ lỗi';
    errFormErr = false;
    scrollToNewErr = true;
    dash = await invoke<DashboardState>('add_error_log', { category, wrongText, rightText });
    renderApp();
  });

  document.getElementById('addParaBtn')?.addEventListener('click', async () => {
    const qTermInput = document.getElementById('paraQTerm') as HTMLInputElement;
    const pPassInput = document.getElementById('paraPassage') as HTMLInputElement;
    const qTerm = qTermInput.value.trim();
    const pPass = pPassInput.value.trim();
    const pSrc = (document.getElementById('paraSource') as HTMLInputElement).value.trim() || 'Study4 Reading';
    const pTrap = (document.getElementById('paraTrap') as HTMLInputElement).value.trim();
    if (!qTerm || !pPass) {
      paraFormMsg = '✕ Nhập Từ câu hỏi và Trong bài đọc.';
      paraFormErr = true;
      const msgEl = document.getElementById('paraFormMsg');
      if (msgEl) {
        msgEl.textContent = paraFormMsg;
        msgEl.className = 'status-msg err';
        msgEl.style.display = 'block';
      }
      (!qTerm ? qTermInput : pPassInput).focus();
      return;
    }
    dash = await invoke<DashboardState>('add_paraphrase', {
      questionTerm: qTerm,
      passageParaphrase: pPass,
      sourceType: pSrc,
      trapNote: pTrap,
    });
    paraFormMsg = '✓ Đã lưu cặp Paraphrase (tự động đưa vào Micro-Check).';
    paraFormErr = false;
    renderApp();
  });

  document.querySelectorAll<HTMLButtonElement>('[data-del-para]').forEach((btn) => {
    btn.onclick = async () => {
      const id = Number(btn.dataset.delPara);
      dash = await invoke<DashboardState>('delete_paraphrase', { id });
      renderApp();
    };
  });

  document.querySelectorAll<HTMLButtonElement>('[data-gen-err-drills]').forEach((btn) => {
    btn.onclick = async () => {
      const errorLogId = Number(btn.dataset.genErrDrills);
      generatingAiDrillErrId = errorLogId;
      errFormMsg = '⏳ Đang nhờ Gemini sinh 3 bài tập thực chiến lưu offline...';
      errFormErr = false;
      renderApp();
      try {
        dash = await invoke<DashboardState>('generate_ai_drills_for_error', { errorLogId });
        errFormMsg = '✓ Đã sinh xong 3 bài tập thực chiến cho lỗi này (lưu offline vào Sổ lỗi)!';
        errFormErr = false;
      } catch (err) {
        errFormMsg = `✕ ${String(err)}`;
        errFormErr = true;
      } finally {
        generatingAiDrillErrId = null;
        renderApp();
      }
    };
  });

  if (scrollToNewErr) {
    scrollToNewErr = false;
    document.querySelector('.log-table-wrap')?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }

  // Speaking Studio Overlay
  document.getElementById('closeStudioBtn')?.addEventListener('click', closeSpeakingStudio);
  document.getElementById('vBackdrop')?.addEventListener('click', () => {
    if (addVocabOpen) {
      addVocabOpen = false;
      pendingRestoreFocus = true;
    }
    if (studioOpen) closeSpeakingStudio();
    else renderApp();
  });

  document.querySelectorAll<HTMLButtonElement>('[data-stsrc]').forEach((btn) => {
    btn.onclick = () => {
      studioSource = btn.dataset.stsrc as 'bank' | 'custom';
      renderApp();
    };
  });

  document.getElementById('swapCueBtn')?.addEventListener('click', async () => {
    await loadSpeakingCue(true);
    speakingGradeOut = null;
    renderApp();
  });

  document.getElementById('customCueIn')?.addEventListener('input', (e) => {
    customCueTitle = (e.target as HTMLInputElement).value;
  });
  document.getElementById('customP3In')?.addEventListener('input', (e) => {
    customPart3 = (e.target as HTMLTextAreaElement).value;
  });
  document.getElementById('studioPrepIn')?.addEventListener('input', (e) => {
    studioPrepNotes = (e.target as HTMLTextAreaElement).value;
  });

  document.querySelectorAll<HTMLButtonElement>('[data-stplan]').forEach((btn) => {
    btn.onclick = () => {
      stopStudioTimer();
      studioRunning = false;
      finishCurrentRecordingClip();
      revokeRecordedBlobs();
      studioPlanKey = btn.dataset.stplan as SpeakingPlanKey;
      studioStageIdx = 0;
      studioSecsLeft = SPEAKING_PLANS[studioPlanKey].stages[0].secs;
      renderApp();
    };
  });

  const advanceStudioStage = async () => {
    const plan = SPEAKING_PLANS[studioPlanKey];
    finishCurrentRecordingClip();
    if (studioStageIdx < plan.stages.length - 1) {
      studioStageIdx += 1;
      studioSecsLeft = plan.stages[studioStageIdx].secs;
      if (studioRunning && plan.stages[studioStageIdx].record) {
        await startRecordingForStage(plan.stages[studioStageIdx].name);
      }
    } else {
      stopStudioTimer();
      studioRunning = false;
      studioSecsLeft = 0;
    }
    renderApp();
  };

  document.getElementById('studioGoBtn')?.addEventListener('click', async () => {
    const plan = SPEAKING_PLANS[studioPlanKey];
    if (studioRunning) {
      stopStudioTimer();
      studioRunning = false;
      finishCurrentRecordingClip();
      renderApp();
      return;
    }
    if (!validateSpeakingCustomCue()) return;
    if (studioStageIdx === 0 && studioSecsLeft === plan.stages[0].secs && recordedBlobs.length > 0) {
      revokeRecordedBlobs();
    }
    studioRunning = true;
    const curStage = plan.stages[studioStageIdx];
    if (curStage?.record) {
      await startRecordingForStage(curStage.name);
    }
    studioTimer = window.setInterval(async () => {
      if (studioSecsLeft > 1) {
        studioSecsLeft -= 1;
        const el = document.getElementById('studioTimerDisplay');
        if (el) el.textContent = fmtMMSS(studioSecsLeft);
      } else {
        await advanceStudioStage();
      }
    }, 1000);
    renderApp();
  });

  document.getElementById('studioNextStageBtn')?.addEventListener('click', () => {
    void advanceStudioStage();
  });

  document.querySelectorAll<HTMLButtonElement>('[data-pick-clip]').forEach((btn) => {
    btn.onclick = () => {
      selectedClipIdx = Number(btn.dataset.pickClip);
      renderApp();
    };
  });

  document.getElementById('toggleSpeakHistoryBtn')?.addEventListener('click', () => {
    speakingHistoryOpen = !speakingHistoryOpen;
    renderApp();
  });

  document.getElementById('studioReplayBtn')?.addEventListener('click', () => {
    const clip = recordedBlobs[selectedClipIdx] || recordedBlobs[recordedBlobs.length - 1];
    if (!clip) return;
    if (studioPlaybackAudio) {
      studioPlaybackAudio.pause();
    }
    studioPlaybackAudio = new Audio(clip.url);
    void studioPlaybackAudio.play();
  });

  const downloadLastAudio = () => {
    const clip = recordedBlobs[selectedClipIdx] || recordedBlobs[recordedBlobs.length - 1];
    if (!clip) return;
    const a = document.createElement('a');
    a.href = clip.url;
    a.download = `xd4t-speaking-${Date.now()}.webm`;
    a.click();
  };
  document.getElementById('studioSaveAudioBtn')?.addEventListener('click', downloadLastAudio);

  document.getElementById('gradeSpeakingAiBtn')?.addEventListener('click', async () => {
    if (!dash || !validateSpeakingCustomCue()) return;
    finishCurrentRecordingClip();
    await new Promise((r) => setTimeout(r, 250));
    if (!recordedBlobs.length) {
      speakingStatusMsg = 'Hãy bấm Bắt đầu và nói ở chặng ghi âm trước khi bấm Chấm AI.';
      speakingStatusErr = true;
      renderApp();
      return;
    }
    isGradingSpeaking = true;
    speakingStatusMsg = 'Đang gửi audio nén (.webm/opus) tới Gemini Audio...';
    speakingStatusErr = false;
    renderApp();
    try {
      const picks =
        recordedBlobs.length >= 2
          ? [recordedBlobs[0], recordedBlobs[recordedBlobs.length - 1]]
          : [recordedBlobs[0]];
      const b64List = await Promise.all(picks.map((p) => blobToBase64(p.blob)));
      const cueTitle =
        studioSource === 'bank' && studioCue ? studioCue.cue_title : customCueTitle;
      const p3 =
        studioSource === 'bank' && studioCue
          ? studioCue.part3_questions.slice(0, 2)
          : customPart3.split('\n').filter(Boolean);
      const track = isAim80Mode() ? '8.0' : '6.5';

      speakingGradeOut = await invoke<SpeakingGradeResponse>('grade_speaking_audio', {
        cueId: studioSource === 'bank' && studioCue ? studioCue.id : null,
        cueTitle,
        part3Questions: p3,
        mode: studioPlanKey,
        track,
        audioB64List: b64List,
        mimeType: 'audio/webm',
      });
      speakingStatusMsg = `✓ Đã chấm xong (${speakingGradeOut.endpoint})${speakingGradeOut.marked_quest_done ? ' · ✓ Đã tick xong nhiệm vụ Speaking hôm nay!' : ''}`;
      speakingStatusErr = false;
      dash = await invoke<DashboardState>('get_dashboard');
    } catch (err) {
      speakingStatusMsg = String(err);
      speakingStatusErr = true;
    } finally {
      isGradingSpeaking = false;
      renderApp();
    }
  });

  document.getElementById('fallbackCopyPromptBtn')?.addEventListener('click', async () => {
    if (!validateSpeakingCustomCue()) return;
    const cueTitle = studioSource === 'bank' && studioCue ? studioCue.cue_title : customCueTitle;
    const p3 =
      studioSource === 'bank' && studioCue
        ? studioCue.part3_questions.slice(0, 2).join('\n- ')
        : customPart3;
    const track = isAim80Mode() ? '8.0' : '6.5';
    const prompt = `Hãy đóng vai Giám khảo IELTS Speaking chấm bản ghi âm/bài nói của tôi theo mục tiêu Band ${track}.\n\n[PART 2 CUE CARD]: ${cueTitle}\n[PART 3]:\n- ${p3}\n[GHI CHÚ CHUẨN BỊ]: ${studioPrepNotes}\n\nHãy chấm 4 tiêu chí (FC, LR, GRA, P ước lượng), tính WPM, đếm từ đệm (uh/um/like) và liệt kê 3 lỗi ngữ pháp/từ vựng cần sửa ngay bằng tiếng Việt.`;
    await navigator.clipboard.writeText(prompt);
    speakingStatusMsg = '✓ Đã copy Prompt + Cue Card + Rubric! Hãy tải file ghi âm và gửi kèm lên AI Studio / ChatGPT.';
    speakingStatusErr = false;
    renderApp();
  });

  document.getElementById('toggleManualCheckBtn')?.addEventListener('click', () => {
    manualCheckOpen = !manualCheckOpen;
    renderApp();
  });
  document.querySelectorAll<HTMLInputElement>('.manual-spk-chk').forEach((chk) => {
    chk.onchange = () => {
      manualChecks[Number(chk.dataset.mchk)] = chk.checked;
    };
  });
  document.getElementById('mBandFc')?.addEventListener('input', (e) => {
    manualBandFc = (e.target as HTMLInputElement).value;
  });
  document.getElementById('mBandLr')?.addEventListener('input', (e) => {
    manualBandLr = (e.target as HTMLInputElement).value;
  });
  document.getElementById('mBandGra')?.addEventListener('input', (e) => {
    manualBandGra = (e.target as HTMLInputElement).value;
  });
  document.getElementById('mBandP')?.addEventListener('input', (e) => {
    manualBandP = (e.target as HTMLInputElement).value;
  });
  document.getElementById('mSpkNote')?.addEventListener('input', (e) => {
    manualNote = (e.target as HTMLInputElement).value;
  });
  document.getElementById('mSpkTranscript')?.addEventListener('input', (e) => {
    manualTranscript = (e.target as HTMLTextAreaElement).value;
  });

  document.getElementById('saveManualSpeakBtn')?.addEventListener('click', async () => {
    if (!validateSpeakingCustomCue()) return;
    const cueTitle = studioSource === 'bank' && studioCue ? studioCue.cue_title : customCueTitle;
    const checkedN = manualChecks.filter(Boolean).length;
    const note = manualNote.trim() || `Tự chấm ${checkedN}/5 mục checklist Speaking`;
    speakingGradeOut = await invoke<SpeakingGradeResponse>('save_speaking_manual_session', {
      cueId: studioSource === 'bank' && studioCue ? studioCue.id : null,
      cueTitle,
      track: isAim80Mode() ? '8.0' : '6.5',
      mode: studioPlanKey,
      fcBand: parseFloat(manualBandFc) || 6.0,
      lrBand: parseFloat(manualBandLr) || 6.0,
      graBand: parseFloat(manualBandGra) || 5.5,
      pBand: parseFloat(manualBandP) || 6.0,
      progressNote: note,
      transcript: manualTranscript.trim(),
      feedbackJson: JSON.stringify({ checklist: manualChecks }),
      errors: [],
    });
    dash = await invoke<DashboardState>('get_dashboard');
    speakingStatusMsg = `✓ Đã lưu phiên Speaking tự chấm${speakingGradeOut.marked_quest_done ? ' · ✓ Đã tick xong nhiệm vụ Speaking hôm nay!' : ''}`;
    speakingStatusErr = false;
    manualCheckOpen = false;
    renderApp();
  });

  // Settings, Welcome, Roadmap, Score Modals
  const settingsModal = document.getElementById('settingsModal')!;
  const openSettings = () => openClassModal(settingsModal);
  const closeSettings = () => closeClassModal(settingsModal);
  document.getElementById('openSettingsBtn')!.onclick = openSettings;
  document.getElementById('bannerOpenSettingsBtn')?.addEventListener('click', openSettings);
  document.getElementById('closeSettingsBtn')!.onclick = closeSettings;
  document.getElementById('cancelSettingsBtn')!.onclick = closeSettings;
  settingsModal.onclick = (e) => {
    if (e.target === settingsModal) closeSettings();
  };
  document.getElementById('openAiStudioBtn')!.onclick = () => openUrl('https://aistudio.google.com/apikey');

  const updateApiTestDom = (msg: string, ok: boolean | null) => {
    apiTestMsg = msg;
    apiTestOk = ok;
    const el = document.getElementById('apiTestMsg');
    if (el) {
      el.textContent = msg;
      el.className = `status-msg ${ok === true ? 'ok' : ok === false ? 'err' : ''}`.trim();
      el.style.display = msg ? 'block' : 'none';
    }
  };

  document.getElementById('toggleKeyVisBtn')!.onclick = () => {
    showApiKey = !showApiKey;
    const keyInput = document.getElementById('setKey') as HTMLInputElement;
    const btn = document.getElementById('toggleKeyVisBtn')!;
    keyInput.type = showApiKey ? 'text' : 'password';
    btn.textContent = showApiKey ? 'ẨN' : 'HIỆN';
  };

  const apiKeyFileInput = document.getElementById('apiKeyFileInput') as HTMLInputElement;
  document.getElementById('loadKeyFileBtn')!.onclick = () => apiKeyFileInput.click();
  apiKeyFileInput.onchange = async () => {
    const file = apiKeyFileInput.files?.[0];
    if (!file) return;
    const text = await file.text();
    const extracted = extractApiKeyFromText(text);
    updateApiTestDom(
      extracted
        ? `✓ Đã nạp key từ file (${file.name}). Bấm [KIỂM TRA KẾT NỐI API] hoặc [LƯU CÀI ĐẶT].`
        : '✕ Không tìm thấy API Key hợp lệ trong file.',
      Boolean(extracted)
    );
    (document.getElementById('setKey') as HTMLInputElement).value = extracted;
  };

  document.getElementById('testApiBtn')!.onclick = async () => {
    const typedKey = (document.getElementById('setKey') as HTMLInputElement).value.trim();
    const modelVal = (document.getElementById('setModel') as HTMLSelectElement).value;
    const testBtn = document.getElementById('testApiBtn') as HTMLButtonElement;
    isTestingApi = true;
    testBtn.disabled = true;
    testBtn.textContent = 'ĐANG KIỂM TRA...';
    updateApiTestDom('Đang kiểm tra kết nối tới Google Gemini API...', null);
    try {
      const res = await invoke<ApiTestResult>('test_gemini_connection', {
        apiKey: typedKey || null,
        model: modelVal,
      });
      updateApiTestDom(res.message, res.ok);
    } catch (err) {
      updateApiTestDom(`✕ ${String(err)}`, false);
    } finally {
      isTestingApi = false;
      testBtn.disabled = false;
      testBtn.textContent = 'KIỂM TRA KẾT NỐI API';
    }
  };

  document.getElementById('clearKeyBtn')!.onclick = async () => {
    await invoke('clear_gemini_api_key');
    dash = await invoke<DashboardState>('get_dashboard');
    updateApiTestDom('✓ Đã xóa Gemini API Key khỏi SQLite.', true);
    const lbl = document.getElementById('setKeyLabel');
    if (lbl) lbl.textContent = 'Gemini API Key (Chưa có)';
    const clearBtn = document.getElementById('clearKeyBtn') as HTMLButtonElement | null;
    if (clearBtn) clearBtn.disabled = true;
    const topbarStatus = document.getElementById('topbarApiStatus');
    if (topbarStatus) {
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = 'stat-pill stat-pill-btn';
      btn.id = 'bannerOpenSettingsBtn';
      btn.title = 'Chưa có Gemini API Key — bấm để nạp miễn phí hoặc dùng Copy-Prompt';
      btn.innerHTML = '<span>API:</span> <b>Chưa có · + Nạp Key</b>';
      btn.onclick = openSettings;
      topbarStatus.replaceWith(btn);
    }
  };

  document.getElementById('settingsToRoadmapBtn')!.onclick = () => {
    settingsModal.classList.remove('open');
    openClassModal(document.getElementById('roadmapModal')!);
  };

  document.getElementById('reopenWelcomeBtn')!.onclick = () => {
    settingsModal.classList.remove('open');
    welcomeOpen = true;
    welcomeStep = 1;
    welcomeMsg = '';
    welcomeOk = null;
    pendingModalFocus = true;
    renderApp();
  };

  document.querySelectorAll<HTMLButtonElement>('.settings-theme-card').forEach((btn) => {
    btn.onclick = async () => {
      if (!dash) return;
      dash.settings = await invoke('update_settings', { theme: btn.dataset.pickTheme! });
      applyThemeInPlace(dash.settings.theme);
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
    const el = document.getElementById('backupStatusMsg');
    if (el) {
      el.textContent = backupMsg;
      el.className = `status-msg ${backupOk === true ? 'ok' : backupOk === false ? 'err' : ''}`.trim();
      el.style.display = backupMsg ? 'block' : 'none';
    }
  };

  document.getElementById('importBackupBtn')!.onclick = async () => {
    const prevScroll = settingsModal.querySelector<HTMLElement>('.modal-box')?.scrollTop || 0;
    try {
      const res = await invoke<RestoreResultDto>('import_db_backup');
      backupMsg = res.message;
      backupOk = res.cancelled ? null : true;
      if (res.dashboard) {
        dash = res.dashboard;
        dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
        cardIdx = 0;
        cardFlipped = false;
      }
    } catch (err) {
      backupMsg = `✕ ${String(err)}`;
      backupOk = false;
    }
    renderApp();
    const reopened = document.getElementById('settingsModal')!;
    reopened.classList.add('open');
    const box = reopened.querySelector<HTMLElement>('.modal-box');
    if (box) box.scrollTop = prevScroll;
  };

  const widgetChk = document.getElementById('setWidgetEnabled') as HTMLInputElement | null;
  if (widgetChk) {
    widgetChk.onchange = async () => {
      try {
        widgetInfo = await invoke<WidgetRuntimeInfo>('set_widget_enabled', { enabled: widgetChk.checked });
        widgetStatusMsg = widgetInfo.running
          ? '✓ Thanh Widget Taskbar đang bật và tự khởi động cùng Windows.'
          : '✓ Đã tắt thanh Widget Taskbar.';
      } catch (err) {
        widgetStatusMsg = `✕ ${String(err)}`;
        widgetChk.checked = !widgetChk.checked;
      }
      const el = document.getElementById('widgetStatusMsg');
      if (el) {
        el.textContent = widgetStatusMsg;
        el.style.display = widgetStatusMsg ? 'block' : 'none';
      }
    };
  }

  document.querySelectorAll<HTMLButtonElement>('.settings-level-card').forEach((btn) => {
    btn.onclick = async () => {
      const prevScroll = settingsModal.querySelector<HTMLElement>('.modal-box')?.scrollTop || 0;
      await switchStudyLevel(btn.dataset.pickLevel!);
      renderApp();
      const reopened = document.getElementById('settingsModal');
      if (reopened) {
        reopened.classList.add('open');
        const box = reopened.querySelector<HTMLElement>('.modal-box');
        if (box) box.scrollTop = prevScroll;
      }
    };
  });

  document.getElementById('saveSettingsBtn')!.onclick = async () => {
    const keyVal = (document.getElementById('setKey') as HTMLInputElement).value.trim();
    const modelVal = (document.getElementById('setModel') as HTMLSelectElement).value;
    const examVal = (document.getElementById('setExamDate') as HTMLInputElement).value.trim();
    const remVal = (document.getElementById('setReminder') as HTMLInputElement).value.trim();
    const autoVal = (document.getElementById('setAutostart') as HTMLInputElement).checked;
    const ttsVal = (document.getElementById('setTtsAccent') as HTMLSelectElement).value;
    const autoRefillVal = (document.getElementById('setAutoRefill') as HTMLInputElement)?.checked ?? true;
    await invoke('update_settings', {
      geminiApiKey: keyVal ? keyVal : null,
      geminiModel: modelVal,
      examDate: examVal,
      reminderTime: remVal,
      autostart: autoVal,
      ttsAccent: ttsVal,
      autoRefillEnabled: autoRefillVal,
    });
    dash = await invoke<DashboardState>('get_dashboard');
    settingsModal.classList.remove('open');
    pendingRestoreFocus = true;
    renderApp();
  };

  // Welcome Wizard Events
  document.getElementById('skipWelcomeBtn')?.addEventListener('click', () => finishOnboarding());
  document.querySelectorAll<HTMLButtonElement>('.welcome-theme-card').forEach((btn) => {
    btn.onclick = async () => {
      if (!dash) return;
      dash.settings = await invoke('update_settings', { theme: btn.dataset.pickTheme! });
      applyThemeInPlace(dash.settings.theme);
    };
  });
  document.querySelectorAll<HTMLButtonElement>('.welcome-level-card').forEach((btn) => {
    btn.onclick = async () => {
      const level = btn.dataset.pickLevel!;
      await switchStudyLevel(level);
      welcomeMsg =
        level === '8.0'
          ? '✓ Đã chuyển sang Lộ trình Nâng cao Aim 8.0.'
          : '✓ Đã chọn Lộ trình Tiêu chuẩn Aim 6.5 (Vùng đệm 7.0).';
      welcomeOk = true;
      renderApp();
    };
  });
  document.getElementById('welcomeNext1Btn')?.addEventListener('click', () => {
    welcomeStep = 2;
    welcomeMsg = '';
    welcomeOk = null;
    renderApp();
  });
  document.getElementById('welcomePrev2Btn')?.addEventListener('click', () => {
    welcomeStep = 1;
    welcomeMsg = '';
    welcomeOk = null;
    renderApp();
  });
  document.getElementById('welcomeNext2Btn')?.addEventListener('click', () => {
    welcomeStep = 3;
    welcomeMsg = '';
    welcomeOk = null;
    renderApp();
  });
  document.getElementById('welcomeCopyPromptBtn')?.addEventListener('click', async () => {
    const promptText = await invoke<string>('get_roadmap_prompt');
    await navigator.clipboard.writeText(promptText);
    welcomeMsg = '✓ Đã copy câu lệnh! Dán vào ChatGPT / Gemini để tạo lộ trình.';
    welcomeOk = true;
    renderApp();
  });
  document.getElementById('welcomeOpenRoadmapBtn')?.addEventListener('click', () => {
    welcomeOpen = false;
    renderApp();
    document.getElementById('roadmapModal')!.classList.add('open');
  });
  document.getElementById('welcomeUseSampleBtn')?.addEventListener('click', () => {
    welcomeMsg = `✓ Đang dùng sẵn Lộ trình 24 tuần (${isAim80Mode() ? 'Aim 8.0' : 'Aim 6.5'}).`;
    welcomeOk = true;
    welcomeStep = 3;
    renderApp();
  });
  document.getElementById('welcomePrev3Btn')?.addEventListener('click', () => {
    welcomeStep = 2;
    welcomeMsg = '';
    welcomeOk = null;
    renderApp();
  });
  document.getElementById('welcomeOpenAiStudioBtn')?.addEventListener('click', () => openUrl('https://aistudio.google.com/apikey'));
  document.getElementById('welcomeKeyInput')?.addEventListener('input', (e) => {
    welcomeKeyDraft = (e.target as HTMLInputElement).value;
  });
  document.getElementById('welcomeTestKeyBtn')?.addEventListener('click', async () => {
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
  });
  document.getElementById('welcomeNext3Btn')?.addEventListener('click', () => {
    const typed = (document.getElementById('welcomeKeyInput') as HTMLInputElement)?.value.trim() || '';
    if (typed) welcomeKeyDraft = typed;
    welcomeStep = 4;
    welcomeMsg = '';
    welcomeOk = null;
    renderApp();
  });
  document.getElementById('finishWelcomeBtn')?.addEventListener('click', () => finishOnboarding(welcomeKeyDraft));

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

  // Roadmap Import Modal
  const roadmapModal = document.getElementById('roadmapModal')!;
  const closeRoadmap = () => closeClassModal(roadmapModal);
  document.getElementById('openRoadmapModalBtn')?.addEventListener('click', () => openClassModal(roadmapModal));
  document.getElementById('openSampleMdBtn')?.addEventListener('click', async () => {
    rememberModalTrigger();
    roadmapDraftMd = await invoke<string>('get_sample_roadmap_md');
    roadmapPreview = await invoke<RoadmapPreview>('preview_roadmap_md', { markdown: roadmapDraftMd });
    roadmapStatusMsg = '✓ Đã nạp sẵn mẫu chuẩn XD4T-1ELTS v1.6.0 để bạn xem / chỉnh sửa.';
    roadmapStatusOk = true;
    renderApp();
    const reopened = document.getElementById('roadmapModal')!;
    reopened.classList.add('open');
    focusActiveDialog();
  });
  document.getElementById('closeRoadmapBtn')!.onclick = closeRoadmap;
  document.getElementById('cancelRoadmapBtn')!.onclick = closeRoadmap;
  roadmapModal.onclick = (e) => {
    if (e.target === roadmapModal) closeRoadmap();
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
    roadmapStatusMsg = '✓ Đã điền mẫu chuẩn XD4T-1ELTS v1.6.0 & phân tích sẵn.';
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
      dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
      cardIdx = 0;
      cardFlipped = false;
      roadmapStatusMsg = '✓ Đã áp dụng lộ trình mới!';
      roadmapStatusOk = true;
      roadmapModal.classList.remove('open');
      pendingRestoreFocus = true;
      renderApp();
    } catch (err) {
      roadmapStatusMsg = `✕ ${String(err)}`;
      roadmapStatusOk = false;
      renderApp();
      document.getElementById('roadmapModal')!.classList.add('open');
    }
  };

  // Score Modal
  const scoreModal = document.getElementById('scoreModal')!;
  const openScoreForSkill = (skill?: string) => {
    if (skill) {
      const sel = document.getElementById('scSkill') as HTMLSelectElement | null;
      if (sel) sel.value = skill;
    }
    openClassModal(scoreModal);
  };
  const closeScoreModal = () => closeClassModal(scoreModal);
  document.getElementById('openScoreModalBtn')!.onclick = () => openScoreForSkill();
  document.getElementById('openScoreTab1Btn')?.addEventListener('click', () => openScoreForSkill());
  document.querySelectorAll<HTMLElement>('[data-band-skill]').forEach((el) => {
    bindActivate(el, () => openScoreForSkill(el.dataset.bandSkill));
  });
  document.getElementById('closeScoreBtn')!.onclick = closeScoreModal;
  document.getElementById('cancelScoreBtn')!.onclick = closeScoreModal;
  scoreModal.onclick = (e) => {
    if (e.target === scoreModal) closeScoreModal();
  };
  document.getElementById('saveScoreBtn')!.onclick = async () => {
    const skill = (document.getElementById('scSkill') as HTMLSelectElement).value;
    const source = (document.getElementById('scSource') as HTMLInputElement).value.trim() || 'Practice';
    const score = parseFloat((document.getElementById('scVal') as HTMLInputElement).value) || 5.0;
    const note = (document.getElementById('scNote') as HTMLInputElement).value.trim();
    dash = await invoke<DashboardState>('add_practice_score', { skill, source, score, note });
    scoreModal.classList.remove('open');
    pendingRestoreFocus = true;
    renderApp();
  };

  // AI Content Pack Modal
  document.getElementById('openAiPackTab1Btn')?.addEventListener('click', openAiPackModal);
  document.getElementById('openAiPackTab3Btn')?.addEventListener('click', openAiPackModal);
  document.getElementById('settingsToAiPackBtn')?.addEventListener('click', () => {
    settingsModal.classList.remove('open');
    openAiPackModal();
  });
  const closeAiPack = () => {
    aiPackOpen = false;
    pendingRestoreFocus = true;
    renderApp();
  };
  document.getElementById('closeAiPackBtn')?.addEventListener('click', closeAiPack);
  document.getElementById('cancelAiPackBtn')?.addEventListener('click', closeAiPack);
  document.getElementById('aiPackModal')?.addEventListener('click', (e) => {
    if (e.target === document.getElementById('aiPackModal')) closeAiPack();
  });
  document.getElementById('aiPackOpenSettingsBtn')?.addEventListener('click', () => {
    aiPackOpen = false;
    renderApp();
    document.getElementById('settingsModal')?.classList.add('open');
  });
  document.querySelectorAll<HTMLButtonElement>('[data-pick-topic]').forEach((btn) => {
    btn.onclick = () => {
      aiPackTopic = btn.dataset.pickTopic || 'Environment';
      renderApp();
    };
  });
  document.getElementById('aiPackTopicIn')?.addEventListener('input', (e) => {
    aiPackTopic = (e.target as HTMLInputElement).value;
  });
  document.getElementById('runAiPackBtn')?.addEventListener('click', async () => {
    if (aiPackLoading || !aiPackTopic.trim()) return;
    aiPackLoading = true;
    aiPackError = '';
    aiPackResult = null;
    renderApp();
    try {
      aiPackResult = await invoke<RefillPackResult>('refill_content_pack', { topic: aiPackTopic.trim() });
      dash = await invoke<DashboardState>('get_dashboard');
      dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
      dictationBatch = [];
      errorDrillBatch = [];
    } catch (err) {
      aiPackError = String(err);
    } finally {
      aiPackLoading = false;
      renderApp();
    }
  });

  // Keyboard Cheatsheet Overlay
  document.getElementById('openShortcutsBtn')?.addEventListener('click', () => {
    rememberModalTrigger();
    shortcutsModalOpen = true;
    pendingModalFocus = true;
    renderApp();
  });
  const closeShortcuts = () => {
    shortcutsModalOpen = false;
    pendingRestoreFocus = true;
    renderApp();
  };
  document.getElementById('closeShortcutsBtn')?.addEventListener('click', closeShortcuts);
  document.getElementById('shortcutsModal')?.addEventListener('click', (e) => {
    if (e.target === document.getElementById('shortcutsModal')) closeShortcuts();
  });
}

// Unified Keyboard-First Dispatcher (0-mouse-click study flow)
document.addEventListener('keydown', (e) => {
  const activeDialog = getActiveDialogEl();
  if (e.key === 'Tab' && activeDialog) {
    const focusables = getFocusableElements(activeDialog);
    if (focusables.length === 0) {
      e.preventDefault();
      activeDialog.focus();
      return;
    }
    const first = focusables[0];
    const last = focusables[focusables.length - 1];
    const cur = document.activeElement as HTMLElement | null;
    if (e.shiftKey) {
      if (!cur || cur === first || !activeDialog.contains(cur)) {
        e.preventDefault();
        last.focus();
      }
    } else if (!cur || cur === last || !activeDialog.contains(cur)) {
      e.preventDefault();
      first.focus();
    }
    return;
  }

  const target = e.target as HTMLElement | null;
  const activeTag = target?.tagName;
  const isTyping = Boolean(
    target && (activeTag === 'INPUT' || activeTag === 'TEXTAREA' || activeTag === 'SELECT' || target.isContentEditable)
  );
  const hasModalBackdrop = Boolean(document.querySelector('.modal-backdrop.open'));

  // 1. Escape closes overlays/modals/studio or blurs input
  if (e.key === 'Escape') {
    if (chartZoomOpen) {
      chartZoomOpen = false;
      pendingRestoreFocus = true;
      renderApp();
      return;
    }
    if (shortcutsModalOpen) {
      shortcutsModalOpen = false;
      pendingRestoreFocus = true;
      renderApp();
      return;
    }
    if (aiPackOpen) {
      aiPackOpen = false;
      pendingRestoreFocus = true;
      renderApp();
      return;
    }
    const welcomeEl = document.getElementById('welcomeModal');
    if (welcomeEl?.classList.contains('open')) {
      welcomeOpen = false;
      void finishOnboarding();
      return;
    }
    if (addVocabOpen) {
      addVocabOpen = false;
      pendingRestoreFocus = true;
      renderApp();
      return;
    }
    if (studioOpen) {
      closeSpeakingStudio();
      return;
    }
    if (hasModalBackdrop) {
      document.querySelectorAll('.modal-backdrop.open').forEach((m) => m.classList.remove('open'));
      restoreModalTriggerFocus();
      return;
    }
    if (isTyping) {
      target?.blur();
    }
    return;
  }

  // 2. Global '?' cheatsheet toggle
  if ((e.key === '?' || (e.shiftKey && e.key === '/')) && !isTyping && !e.ctrlKey && !e.altKey && !e.metaKey) {
    e.preventDefault();
    if (!shortcutsModalOpen) {
      rememberModalTrigger();
      shortcutsModalOpen = true;
      pendingModalFocus = true;
    } else {
      shortcutsModalOpen = false;
      pendingRestoreFocus = true;
    }
    renderApp();
    return;
  }

  // 3. Global Alt+1..4 tabs & Alt+S Speaking Studio
  if (e.altKey && !e.ctrlKey && !e.metaKey) {
    const k = e.key.toLowerCase();
    if (k === '1' || k === '2' || k === '3' || k === '4') {
      e.preventDefault();
      window.speechSynthesis?.cancel();
      stopMicroTimer();
      shortcutsModalOpen = false;
      aiPackOpen = false;
      document.querySelectorAll('.modal-backdrop.open').forEach((m) => m.classList.remove('open'));
      const tabMap: Record<string, typeof activeTab> = {
        '1': 'today',
        '2': 'writing',
        '3': 'vocab',
        '4': 'roadmap',
      };
      activeTab = tabMap[k];
      renderApp();
      return;
    }
    if (k === 's') {
      e.preventDefault();
      if (studioOpen) closeSpeakingStudio();
      else void openSpeakingStudio();
      return;
    }
    if (k === 't' && activeTab === 'writing' && !studioOpen && !hasModalBackdrop) {
      e.preventDefault();
      top3Collapsed = !top3Collapsed;
      renderApp();
      return;
    }
    if (k === 'n' && studioOpen) {
      e.preventDefault();
      document.getElementById('studioNextStageBtn')?.click();
      return;
    }
  }

  // 4. Speaking Studio shortcuts (when open)
  if (studioOpen) {
    if (e.ctrlKey && e.code === 'Space') {
      e.preventDefault();
      document.getElementById('studioGoBtn')?.click();
      return;
    }
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      document.getElementById('gradeSpeakingAiBtn')?.click();
      return;
    }
    return;
  }

  if (hasModalBackdrop || addVocabOpen || shortcutsModalOpen || aiPackOpen || welcomeOpen) {
    return;
  }

  // 5. Tab 2 Writing shortcuts
  if (activeTab === 'writing') {
    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      document.getElementById('gradeBtn')?.click();
    }
    return;
  }

  // 6. Tab 3 Vocabulary, Quiz, Error Drills, Micro-Check & Dictation
  if (activeTab !== 'vocab') return;

  // 6a. Flashcard SRS
  if (vocabSubMode === 'flashcard') {
    if (leechFixCardId !== null) {
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        document.getElementById('saveLeechFixBtn')?.click();
      }
      return;
    }
    const cur = dueCards[cardIdx];
    if (e.ctrlKey && e.code === 'Space') {
      e.preventDefault();
      if (cur) speakText(cur.word, dash?.settings.tts_accent || 'en-GB', 1.0);
      return;
    }
    if (isTyping || !cur) return;
    const low = e.key.toLowerCase();
    if (e.code === 'Space' || e.key === 'Enter') {
      e.preventDefault();
      cardFlipped = !cardFlipped;
      renderApp();
    } else if (e.key === '1' || e.key === '2' || e.key === '3' || e.key === '4') {
      e.preventDefault();
      const qVal = Number(e.key) - 1;
      document.querySelector<HTMLButtonElement>(`.srs-btn[data-q="${qVal}"]`)?.click();
    } else if (low === 'r' || low === 'v') {
      e.preventDefault();
      speakText(cur.word, dash?.settings.tts_accent || 'en-GB', 1.0);
    } else if (low === 'e') {
      e.preventDefault();
      speakText(cur.example, dash?.settings.tts_accent || 'en-GB', 1.0);
    } else if (low === 'c') {
      const fixBtn = document.getElementById('openLeechFixBtn');
      if (fixBtn) {
        e.preventDefault();
        fixBtn.click();
      }
    }
    return;
  }

  // 6b. Dictation
  if (vocabSubMode === 'dictation') {
    const cur = dictationBatch[dictationIdx];
    if (e.ctrlKey && e.code === 'Space') {
      e.preventDefault();
      if (cur) speakText(cur.sentence, dash?.settings.tts_accent || 'en-GB', dictationRate);
      return;
    }
    if (e.ctrlKey && (e.key === '1' || e.key === '2' || e.key === '3')) {
      e.preventDefault();
      const rateMap: Record<string, number> = { '1': 0.85, '2': 1, '3': 1.15 };
      dictationRate = rateMap[e.key];
      renderApp();
      if (cur) speakText(cur.sentence, dash?.settings.tts_accent || 'en-GB', dictationRate);
      return;
    }
    if (e.ctrlKey && e.key.toLowerCase() === 'h') {
      e.preventDefault();
      document.getElementById('dictHintBtn')?.click();
      return;
    }
    if (e.ctrlKey && e.key.toLowerCase() === 'l') {
      e.preventDefault();
      document.getElementById('pushDictErrBtn')?.click();
      return;
    }
    if (e.key === 'Enter' && !e.shiftKey) {
      if (dictationSummary) {
        e.preventDefault();
        document.getElementById('dictNewBatchBtn')?.click();
      } else if (!cur) {
        e.preventDefault();
        document.getElementById('loadDictBatchBtn')?.click();
      } else if (!dictationChecked) {
        e.preventDefault();
        document.getElementById('dictCheckBtn')?.click();
      } else {
        e.preventDefault();
        document.getElementById('dictNextBtn')?.click();
      }
    }
    return;
  }

  // 6c. Quiz / Error Drill / Micro-Check
  if (vocabSubMode === 'quiz') {
    const keyLow = e.key.toLowerCase();
    const optIndexFromKey = (): number => {
      if (e.key >= '1' && e.key <= '4') return Number(e.key) - 1;
      if (keyLow >= 'a' && keyLow <= 'd') return keyLow.charCodeAt(0) - 97;
      return -1;
    };

    if (quizPillMode === 'vocab') {
      if (quizPhase === 'setup' && !isTyping && e.key === 'Enter') {
        e.preventDefault();
        document.getElementById('startQuizBtn')?.click();
        return;
      }
      if (quizPhase === 'summary' && !isTyping && e.key === 'Enter') {
        e.preventDefault();
        (document.getElementById('quizReviewForgottenBtn') || document.getElementById('quizRestartBtn'))?.click();
        return;
      }
      if (quizPhase === 'playing') {
        const q = quizQuestions[quizIdx];
        if (!q) return;
        if (quizFeedback) {
          if (e.key === 'Enter' || (!isTyping && e.code === 'Space')) {
            e.preventDefault();
            document.getElementById('quizNextBtn')?.click();
          }
          return;
        }
        if (q.question_type === 'typing') {
          if (e.ctrlKey && keyLow === 'h') {
            e.preventDefault();
            document.getElementById('quizGiveUpBtn')?.click();
          } else if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            document.getElementById('quizCheckTypingBtn')?.click();
          }
        } else if (!isTyping) {
          const oi = optIndexFromKey();
          if (oi >= 0 && oi < q.options.length) {
            e.preventDefault();
            document.querySelector<HTMLButtonElement>(`[data-quiz-opt="${oi}"]`)?.click();
          } else if (keyLow === 'f') {
            e.preventDefault();
            document.getElementById('quizGiveUpBtn')?.click();
          }
        }
      }
      return;
    }

    if (quizPillMode === 'fix') {
      if (errorDrillSummary && !isTyping && e.key === 'Enter') {
        e.preventDefault();
        document.getElementById('restartErrDrillBtn')?.click();
        return;
      }
      const q = errorDrillBatch[errorDrillIdx];
      if (!q) {
        if (!isTyping && e.key === 'Enter') {
          e.preventDefault();
          document.getElementById('startErrDrillBtn')?.click();
        }
        return;
      }
      if (errorDrillFeedback) {
        if (e.ctrlKey && keyLow === 'i') {
          e.preventDefault();
          document.getElementById('errDrillAiGradeBtn')?.click();
        } else if (e.ctrlKey && keyLow === 'y') {
          e.preventDefault();
          document.getElementById('errDrillSelfAcceptBtn')?.click();
        } else if (e.key === 'Enter' || (!isTyping && e.code === 'Space')) {
          e.preventDefault();
          document.getElementById('nextErrDrillBtn')?.click();
        }
        return;
      }
      if (q.drill_mode === 'TYPING') {
        if (e.key === 'Enter' && !e.shiftKey) {
          e.preventDefault();
          document.getElementById('checkErrDrillBtn')?.click();
        }
      } else if (!isTyping) {
        const oi = optIndexFromKey();
        if (oi >= 0 && oi < q.options.length) {
          e.preventDefault();
          document.querySelector<HTMLButtonElement>(`[data-edrill-opt="${oi}"]`)?.click();
        }
      }
      return;
    }

    if (quizPillMode === 'micro') {
      if (microSummary && !isTyping && e.key === 'Enter') {
        e.preventDefault();
        document.getElementById('restartMicroBtn')?.click();
        return;
      }
      const q = microQuestions[microIdx];
      if (!q) {
        if (!isTyping && e.key === 'Enter') {
          e.preventDefault();
          document.getElementById('startMicroBtn')?.click();
        }
        return;
      }
      if (microFeedback) {
        if (e.ctrlKey && keyLow === 'i') {
          e.preventDefault();
          document.getElementById('microAiGradeBtn')?.click();
        } else if (e.ctrlKey && keyLow === 'y') {
          e.preventDefault();
          document.getElementById('microSelfAcceptBtn')?.click();
        } else if (e.key === 'Enter' || (!isTyping && e.code === 'Space')) {
          e.preventDefault();
          document.getElementById('microNextBtn')?.click();
        }
        return;
      }
      if (q.options.length > 0 && !isTyping) {
        const oi = optIndexFromKey();
        if (oi >= 0 && oi < q.options.length) {
          e.preventDefault();
          document.querySelector<HTMLButtonElement>(`[data-micro-opt="${oi}"]`)?.click();
        }
      } else if (q.options.length === 0) {
        if (e.ctrlKey && keyLow === 'h') {
          e.preventDefault();
          document.getElementById('microGiveUpBtn')?.click();
        } else if (e.key === 'Enter' && !e.shiftKey) {
          e.preventDefault();
          document.getElementById('microCheckBtn')?.click();
        }
      }
    }
  }
});

async function maybeRunAutoRefillInBackground() {
  if (autoRefillTriggeredThisSession) return;
  const st = dash?.auto_refill_status;
  if (!st || !st.should_auto_refill || !st.recommended_topic) return;
  autoRefillTriggeredThisSession = true;
  try {
    const res = await invoke<RefillPackResult>('refill_content_pack', {
      topic: st.recommended_topic,
      isAuto: true,
    });
    dash = await invoke<DashboardState>('get_dashboard');
    dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
    dictationBatch = [];
    errorDrillBatch = [];
    autoRefillToastMsg = `✨ Đã tự động nạp gói tuần mới (${res.topic}): +${res.vocab_added} từ · +${res.dictations_added} Dictation · +${res.prompts_added} Writing · +${res.paraphrases_added} Paraphrase · +${res.drills_added} Luyện lỗi`;
    renderApp();
  } catch {
    // Silent fallback when offline or rate-limited
  }
}

async function init() {
  initTtsVoices();
  dash = await invoke<DashboardState>('get_dashboard');
  dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: false });
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
  void maybeRunAutoRefillInBackground();

  await listen('start-pomodoro-15', () => {
    pomoSecs = 15 * 60;
    togglePomodoro(true);
  });
}

init();
