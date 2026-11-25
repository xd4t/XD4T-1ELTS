import './styles.css';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { openUrl } from '@tauri-apps/plugin-opener';
import donateQrUrl from './assets/donate-qr.png';
import {
  computeWordDiff,
  esc,
  initTtsVoices,
  isErrorFixMatch,
  isTtsSupported,
  speakText,
} from './diff';
import type {
  ApiTestResult,
  BackupResultDto,
  DashboardState,
  DictationResultInput,
  DictationSentence,
  DictationSubmitResult,
  ErrorFixQuestion,
  GradeResponseDto,
  GradeResult,
  MicroCheckAnswerInput,
  MicroCheckItem,
  MicroCheckSubmitResult,
  QuizAnswerInput,
  QuizQuestion,
  QuizSubmitResult,
  RestoreResultDto,
  RoadmapPreview,
  SpeakingCue,
  SpeakingGradeResponse,
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

// Error Fix Quiz State
let errorFixQuestions: ErrorFixQuestion[] = [];
let errorFixIdx = 0;
let errorFixInput = '';
let errorFixFeedback: { correct: boolean; target: string } | null = null;
let errorFixCorrectCount = 0;
let errorFixDone = false;

// Micro-Check State
let microQuestions: MicroCheckItem[] = [];
let microIdx = 0;
let microAnswers: MicroCheckAnswerInput[] = [];
let microInput = '';
let microFeedback: { correct: boolean; gaveUp: boolean; target: string; note: string } | null = null;
let microSummary: MicroCheckSubmitResult | null = null;
let microSecsLeft = 120;
let microTimer: number | null = null;

// Dictation State
let dictationBatch: DictationSentence[] = [];
let dictationIdx = 0;
let dictationRate = 0.85;
let dictationInput = '';
let dictationHintWords = 0;
let dictationChecked: { pct: number; tokensHtml: string; wrongWords: string[]; pushed: boolean } | null = null;
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

function makeCardClozeHtml(card: VocabCard): string {
  const ex = card.example;
  const w = card.word.split('(')[0].split('/')[0].trim();
  if (w && ex.toLowerCase().includes(w.toLowerCase())) {
    const idx = ex.toLowerCase().indexOf(w.toLowerCase());
    const before = esc(ex.slice(0, idx));
    const after = esc(ex.slice(idx + w.length));
    return `${before}<span class="fc-blank">&nbsp;</span>${after}`;
  }
  return `${esc(ex)} <div class="small muted" style="margin-top:4px;">(Nghĩa: ${esc(card.meaning)})</div>`;
}

function highlightWordInExample(example: string, word: string): string {
  const w = word.split('(')[0].split('/')[0].trim();
  if (w && example.toLowerCase().includes(w.toLowerCase())) {
    const idx = example.toLowerCase().indexOf(w.toLowerCase());
    return `${esc(example.slice(0, idx))}<mark>${esc(example.slice(idx, idx + w.length))}</mark>${esc(example.slice(idx + w.length))}`;
  }
  return esc(example);
}

function questTargetFromHint(hint: string, title: string): string {
  switch (hint) {
    case 'srs':
      return 'vocab:flash';
    case 'micro_check':
      return 'vocab:quiz:micro';
    case 'quiz':
      return 'vocab:quiz:fix';
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
      if (low.includes('quiz')) return 'vocab:quiz:fix';
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
      }
      currentChunks = [];
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
  studioOpen = true;
  speakingGradeOut = null;
  speakingStatusMsg = '';
  speakingStatusErr = false;
  manualCheckOpen = false;
  if (!studioCue) {
    await loadSpeakingCue(false);
  }
  renderApp();
}

async function goToTarget(target: string): Promise<void> {
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
        await loadErrorFixQuiz();
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

async function loadErrorFixQuiz(): Promise<void> {
  errorFixQuestions = await invoke<ErrorFixQuestion[]>('generate_error_fix_quiz', { count: 5 });
  errorFixIdx = 0;
  errorFixInput = '';
  errorFixFeedback = null;
  errorFixCorrectCount = 0;
  errorFixDone = false;
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
  microTimer = window.setInterval(() => {
    if (microSecsLeft > 0) {
      microSecsLeft -= 1;
      const el = document.getElementById('microTimerDisplay');
      if (el) el.textContent = `⏱ ${fmtMMSS(microSecsLeft)}`;
    } else {
      stopMicroTimer();
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
        curCard
          ? `
        <div class="fc-card" data-theme-ref="${curTheme}">
          <div class="fc-corner">
            ${
              isLeech
                ? `<span class="tag leech" title="Quên ≥ 3 lần">🔥 Từ cứng đầu · quên ${curCard.lapses} lần</span>`
                : `<span class="tag soft">${esc(curCard.topic.toUpperCase())} · ${isAim80 ? 'BAND 8.0' : 'BAND 6.5'} · REPS ${curCard.reps}</span>`
            }
            <span class="row">
              ${
                isLeech
                  ? `<button type="button" class="btn btn-sm" id="openLeechFixBtn" style="border-color:var(--v16-leech);color:var(--v16-leech);font-weight:700;">✍️ Chữa từ này</button>`
                  : ''
              }
              <button type="button" class="btn ghost ibtn" id="hideVocabBtn" title="Ẩn thẻ khỏi hàng đợi">🙈</button>
              <button type="button" class="btn ghost ibtn" id="deleteVocabBtn" title="Xóa thẻ">🗑</button>
            </span>
          </div>

          ${
            cardFaceMode === 'cloze'
              ? `
                <div class="fc-ex">${makeCardClozeHtml(curCard)}</div>
                ${
                  cardFlipped
                    ? `<div class="fc-word-lg" style="margin-top:6px;">${esc(curCard.word)} ${ttsOk ? `<button type="button" class="btn ibtn" data-say="${esc(curCard.word)}" title="Đọc từ">🔊</button>` : ''}</div>
                       <div class="small muted">${esc(curCard.meaning)}</div>`
                    : `<div class="small muted">Nhớ lại từ còn thiếu rồi bấm Lật thẻ <kbd>Space</kbd></div>`
                }
              `
              : cardFaceMode === 'listen'
                ? `
                  ${
                    ttsOk
                      ? `<button type="button" class="btn btn-primary lg" id="listenReplayBtn" data-say="${esc(curCard.word)}">🔊 Nghe lại <kbd style="color:inherit;border-color:currentColor;">V</kbd></button>`
                      : `<div class="small muted">Thiết bị chưa bật TTS — lật thẻ để xem chữ</div>`
                  }
                  <div class="${cardFlipped ? '' : 'masked'}">
                    <div class="fc-word-lg">${esc(curCard.word)}</div>
                    <div class="small muted" style="margin-top:4px;">${esc(curCard.meaning)}</div>
                  </div>
                  <div class="fc-ex ${cardFlipped ? '' : 'masked'}">
                    ${highlightWordInExample(curCard.example, curCard.word)}
                    ${ttsOk && cardFlipped ? `<button type="button" class="btn ghost ibtn" data-say="${esc(curCard.example)}" title="Đọc câu">🔉</button>` : ''}
                  </div>
                  ${!cardFlipped ? `<div class="small muted">Mặt chữ ẩn tới khi lật thẻ</div>` : ''}
                `
                : `
                  <div class="row" style="justify-content:center;">
                    <span class="fc-word-lg">${esc(curCard.word)}</span>
                    ${ttsOk ? `<button type="button" class="btn ibtn" data-say="${esc(curCard.word)}" title="Đọc từ">🔊</button>` : ''}
                  </div>
                  ${
                    cardFlipped
                      ? `<div style="font-weight:700;font-size:15px;">${esc(curCard.meaning)}</div>
                         <div class="fc-ex">
                           ${highlightWordInExample(curCard.example, curCard.word)}
                           ${ttsOk ? `<button type="button" class="btn ghost ibtn" data-say="${esc(curCard.example)}" title="Đọc câu">🔉</button>` : ''}
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

function renderErrorFixQuiz(): string {
  if (errorFixDone) {
    const tot = errorFixQuestions.length || 1;
    const pct = Math.round((errorFixCorrectCount / tot) * 100);
    return `
      <div class="stage-wrap box">
        <div class="quiz-summary-hero">
          <div class="quiz-summary-score mono-num">${errorFixCorrectCount} / ${tot} · ${pct}%</div>
          <div class="small muted">Đã hoàn thành luyện gõ sửa lỗi từ Sổ lỗi!</div>
        </div>
        <div style="text-align:center;">
          <button type="button" class="btn btn-primary" id="restartErrFixBtn">LÀM LƯỢT SỬA LỖI MỚI →</button>
        </div>
      </div>`;
  }
  const q = errorFixQuestions[errorFixIdx];
  if (!q) {
    return `
      <div class="stage-wrap box" style="text-align:center;padding:24px;">
        <div class="small muted" style="margin-bottom:10px;">Sẵn sàng luyện gõ sửa lại các lỗi hay mắc nhất trong Sổ lỗi.</div>
        <button type="button" class="btn btn-primary" id="startErrFixBtn">BẮT ĐẦU QUIZ SỬA LỖI (5 CÂU) →</button>
      </div>`;
  }
  const tot = errorFixQuestions.length || 1;
  const pct = Math.round(((errorFixIdx + (errorFixFeedback ? 1 : 0)) / tot) * 100);
  return `
    <div class="stage-wrap box">
      <div class="row small muted" style="justify-content:space-between;">
        <span>Câu ${errorFixIdx + 1}/${tot} · <span class="src-badge">${esc(q.category)} · ${q.count} lần · ${esc(q.created_at)}</span></span>
      </div>
      <div class="bar-track" style="margin:8px 0 12px;"><div class="bar-fill" style="width:${pct}%"></div></div>
      <div class="label-uc">Cụm / câu bạn đã viết sai</div>
      <div style="font-size:16px;margin:4px 0 12px;"><s style="color:var(--m-err);">${esc(q.wrong_text)}</s></div>
      <div class="label-uc" style="margin-bottom:4px;">Gõ lại cụm / câu sửa chuẩn</div>
      <textarea class="text-input" id="errFixInput" rows="2" style="width:100%;font-size:14px;" placeholder="Nhập cách sửa đúng..." ${errorFixFeedback ? 'disabled' : ''}>${esc(errorFixInput)}</textarea>
      <div class="row" style="justify-content:space-between;margin-top:8px;flex-wrap:wrap;">
        <span class="small muted">So khớp linh hoạt (bỏ hoa/thường, dấu câu, khoảng trắng thừa)</span>
        ${
          !errorFixFeedback
            ? `<button type="button" class="btn btn-primary" id="checkErrFixBtn">Kiểm tra <kbd style="color:inherit;border-color:currentColor;">Enter</kbd></button>`
            : `<button type="button" class="btn btn-primary" id="nextErrFixBtn">${errorFixIdx + 1 >= tot ? 'Xem tổng kết →' : 'Câu tiếp →'}</button>`
        }
      </div>
      ${
        errorFixFeedback
          ? `<div class="quiz-feedback ${errorFixFeedback.correct ? 'ok' : 'err'}">
              <b>${errorFixFeedback.correct ? '✓ Chính xác!' : '✗ Chưa khớp hoàn toàn'}</b> — Đáp án chuẩn: <b>${esc(errorFixFeedback.target)}</b>
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
          <button type="button" class="btn btn-primary" id="restartMicroBtn">LÀM LẠI MICRO-CHECK MỚI →</button>
        </div>
      </div>`;
  }
  const q = microQuestions[microIdx];
  if (!q) {
    return `
      <div class="stage-wrap box" style="text-align:center;padding:24px;">
        <div class="small muted" style="margin-bottom:10px;">Trộn nhanh 5 câu (2 từ dễ quên + 2 câu Sổ lỗi + 1 cặp Paraphrase) để rà trí nhớ trong 2 phút.</div>
        <button type="button" class="btn btn-primary" id="startMicroBtn">⚡ BẮT ĐẦU MICRO-CHECK 5 CÂU (2P) →</button>
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
                  ${String.fromCharCode(65 + i)}. ${esc(opt)}
                </button>`
                )
                .join('')}
            </div>`
          : `<div class="small muted" style="margin-bottom:4px;">Gợi ý: <b class="mono-num">${esc(q.hint)}</b></div>
             <textarea class="text-input" id="microInput" rows="2" style="width:100%;font-size:14px;" placeholder="Gõ đáp án tiếng Anh..." ${microFeedback ? 'disabled' : ''}>${esc(microInput)}</textarea>
             <div class="row" style="justify-content:flex-end;margin-top:8px;">
               ${
                 !microFeedback
                   ? `<button type="button" class="btn btn-sm" id="microGiveUpBtn">❓ Quên / Xem đáp án</button>
                      <button type="button" class="btn btn-primary" id="microCheckBtn">Kiểm tra <kbd style="color:inherit;border-color:currentColor;">Enter</kbd></button>`
                   : ''
               }
             </div>`
      }
      ${
        microFeedback
          ? `<div class="quiz-feedback ${microFeedback.correct ? 'ok' : 'err'}">
              <div><b>${microFeedback.correct ? '✓ Chính xác!' : microFeedback.gaveUp ? '↺ Đã đẩy về SRS' : '✗ Chưa khớp'}</b> — Đáp án: <b>${esc(microFeedback.target)}</b></div>
              ${microFeedback.note ? `<div class="small muted" style="margin-top:3px;">${esc(microFeedback.note)}</div>` : ''}
              <div style="margin-top:8px;"><button type="button" class="btn btn-primary btn-sm" id="microNextBtn">${microIdx + 1 >= tot ? 'Hoàn thành Micro-Check →' : 'Câu tiếp →'}</button></div>
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
      <button type="button" class="pill" aria-pressed="${quizPillMode === 'fix'}" data-qpill="fix">🔧 Quiz Gõ Sửa Lỗi</button>
      <button type="button" class="pill" aria-pressed="${quizPillMode === 'micro'}" data-qpill="micro">⚡ Micro-Check 5 câu (2p)</button>
    </div>`;

  if (quizPillMode === 'fix') {
    return `${pillsHtml}${renderErrorFixQuiz()}`;
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
            ${quizLoading ? 'ĐANG TẠO BÀI QUIZ...' : 'BẮT ĐẦU QUIZ 10 CÂU →'}
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
                    ? `<div style="margin-top:10px;"><button class="btn btn-sm" id="quizGiveUpBtn" type="button">❓ Quên từ này (đẩy về SRS)</button></div>`
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
      <span class="sp"></span>
      <span class="small">Câu ${dictationIdx + 1} / ${tot} · ${esc(cur.section)} · <span class="tag soft">${esc(cur.trap_focus)}</span></span>
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
        <button type="button" class="btn" id="dictHintBtn">💡 Xem gợi ý 1 từ</button>
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
              <button type="button" class="btn btn-sm" id="pushDictErrBtn" ${dictationChecked.pushed || !dictationChecked.wrongWords.length ? 'disabled' : ''}>
                ${dictationChecked.pushed ? `✓ Đã đẩy ${dictationChecked.wrongWords.length} từ vào Sổ lỗi` : '+ Đẩy vào Sổ lỗi Listening'}
              </button>
              <button type="button" class="btn btn-sm btn-primary" id="dictNextBtn">
                ${dictationIdx + 1 >= tot ? 'Hoàn thành & Lưu buổi nghe →' : 'Câu tiếp →'}
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
    <div class="studio-overlay ${studioOpen ? 'on' : ''}" id="speakStudioModal" role="dialog" aria-label="Phòng Luyện Nói">
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
            <button type="button" class="btn btn-primary" id="studioGoBtn">${studioRunning ? '⏸ Dừng' : '● Bắt đầu'}</button>
            <button type="button" class="btn" id="studioNextStageBtn">⏭ Chặng sau</button>
            <button type="button" class="btn" id="studioReplayBtn" ${recordedBlobs.length === 0 ? 'disabled' : ''}>▶ Nghe lại</button>
            <button type="button" class="btn" id="studioSaveAudioBtn" ${recordedBlobs.length === 0 ? 'disabled' : ''}>💾 Lưu file</button>
          </div>

          <div class="speak-score-box">
            <div class="row" style="justify-content:space-between;flex-wrap:wrap;">
              <span class="label-uc">Chấm điểm Speaking</span>
              <button type="button" class="btn btn-primary btn-sm" id="gradeSpeakingAiBtn" ${isGradingSpeaking ? 'disabled' : ''}>
                ${isGradingSpeaking ? 'ĐANG CHẤM AUDIO...' : '✨ Chấm AI (Gemini Audio)'}
              </button>
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
              <span class="small muted" style="width:100%;">Chưa có API key? Dùng quy trình miễn phí:</span>
              <button type="button" class="btn btn-sm" id="fallbackDownloadAudioBtn" ${recordedBlobs.length === 0 ? 'disabled' : ''}>⬇ Tải file ghi âm</button>
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
                <div style="display:grid;grid-template-columns:repeat(4,1fr);gap:6px;margin-top:6px;">
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
          </div>
        </div>
      </div>
    </div>

    <!-- Drawer + Thêm từ (Sentence Mining) -->
    <aside class="v-drawer ${addVocabOpen ? 'on' : ''}" id="addVocabDrawer" aria-label="Thêm từ">
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
  const top3Errors = dash.error_logs.slice(0, 3);

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
      <button class="nav-tab ${activeTab === 'vocab' ? 'active' : ''}" data-tab="vocab">[3] TỪ VỰNG &amp; NGHE (${onlyLeechFilter ? dash.due_vocab_count : dueCards.length})</button>
      <button class="nav-tab ${activeTab === 'roadmap' ? 'active' : ''}" data-tab="roadmap">[4] LỘ TRÌNH &amp; SỔ LỖI</button>
      <div class="nav-meta">Tuần ${dash.current_week} · Track ${isAim80 ? '8.0' : '6.5'} · Mục tiêu Band ${dash.settings.target_band.toFixed(1)}</div>
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
                <button type="button" class="btn" data-go="speak">🎙️ Phòng Luyện Nói</button>
                <button type="button" class="btn" data-go="vocab:dict">🎧 Nghe chép câu</button>
                <button type="button" class="btn" data-go="vocab:quiz:micro">⚡ Micro-Check 2p</button>
                <button type="button" class="btn" data-go="writing">📝 Viết bài</button>
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
          top3Errors.length >= 3
            ? `
          <div class="top3-bar ${top3Collapsed ? 'collapsed' : ''}" id="top3Bar">
            <span>⚠️</span>
            <div class="top3-body">
              <div class="row">
                <b class="small">Rà nhanh Top 3 lỗi bạn hay mắc trước khi viết</b>
                <span class="small muted">· đếm từ Sổ lỗi</span>
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
            <button type="button" class="btn ghost ibtn" id="toggleTop3Btn" aria-expanded="${!top3Collapsed}" title="Thu gọn">${top3Collapsed ? '▾' : '▴'}</button>
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
                  `<option value="${esc(p.id)}" ${curPrompt?.id === p.id ? 'selected' : ''}>${esc(p.title)}</option>`
              )
              .join('')}
          </select>
          <button class="btn btn-sm" style="margin-left:auto;" id="toggleHintBtn">+ GỢI Ý KHUNG BÀI ${isAim80 ? '8.0' : '6.5'}</button>
        </div>

        <div class="grid-2">
          <div class="box">
            <div class="box-title">
              <span>ĐỀ BÀI &amp; KHUNG VIẾT</span>
              <span id="wordCount">0 / ${curPrompt?.target_words || 90} từ</span>
            </div>
            <div class="prompt-box"><b>Đề:</b> ${esc(curPrompt?.prompt || '')}</div>
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

      <!-- TAB 3: TỪ VỰNG & NGHE -->
      <div class="panel ${activeTab === 'vocab' ? 'active' : ''}" id="tab-vocab">
        <div class="row" style="margin-bottom:12px;flex-wrap:wrap;">
          <div class="seg" role="group" aria-label="Sub-mode Từ vựng và Nghe">
            <button type="button" aria-pressed="${vocabSubMode === 'flashcard'}" data-vsub="flashcard">📇 Flashcard</button>
            <button type="button" aria-pressed="${vocabSubMode === 'quiz'}" data-vsub="quiz">📝 Quiz &amp; Sửa lỗi ${dash.quiz_status.quiz_due ? '· ●' : ''}</button>
            <button type="button" aria-pressed="${vocabSubMode === 'dictation'}" data-vsub="dictation">🎧 Nghe chép câu</button>
          </div>
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
                    <tr><th>Nguồn / Loại</th><th>Hay mắc</th><th>Sửa chuẩn</th><th>Lần</th></tr>
                  </thead>
                  <tbody>
                    ${dash.error_logs
                      .map(
                        (e) =>
                          `<tr><td><span class="tag soft">${esc(e.category)}</span></td><td class="err-del">${esc(e.wrong_text)}</td><td class="ok-add">${esc(e.right_text)}</td><td>${e.count}</td></tr>`
                      )
                      .join('')}
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
              ${paraFormMsg ? `<div class="status-msg ${paraFormErr ? 'err' : 'ok'}" style="margin-bottom:6px;">${esc(paraFormMsg)}</div>` : ''}
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
                    ${dash.paraphrase_pairs
                      .map(
                        (p) => `
                      <tr>
                        <td><b>${esc(p.question_term)}</b></td>
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
                      .join('')}
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
      <div style="color:var(--m-muted);">XD4T-1ELTS v1.6.0 · Rust + Tauri v2 + SQLite</div>
    </div>

    ${renderSpeakingStudio()}

    <!-- SETTINGS MODAL -->
    <div class="modal-backdrop" id="settingsModal">
      <div class="modal-box modal-wide">
        <div class="box-title"><span>CÀI ĐẶT HỆ THỐNG &amp; DỮ LIỆU</span><button class="btn btn-sm" id="closeSettingsBtn">✕</button></div>

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

    <!-- ROADMAP IMPORT MODAL -->
    <div class="modal-backdrop" id="roadmapModal">
      <div class="modal-box modal-wide">
        <div class="box-title"><span>NẠP LỘ TRÌNH CÁ NHÂN (.MD)</span><button class="btn btn-sm" id="closeRoadmapBtn">✕</button></div>
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
      <div class="modal-box">
        <div class="box-title"><span>NHẬP ĐIỂM ĐỀ VỪA LÀM</span><button class="btn btn-sm" id="closeScoreBtn">✕</button></div>
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
      const focusNotes = (dash?.settings.writing_focus_notes || []).map((n, i) => `${i + 1}. ${n}`).join('\n');
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

  // Tab 3: Sub-modes & Flashcard
  document.querySelectorAll<HTMLButtonElement>('[data-vsub]').forEach((btn) => {
    btn.onclick = async () => {
      window.speechSynthesis?.cancel();
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
      dash = await invoke<DashboardState>('toggle_hide_vocab_card', { cardId: card.id, hidden: true });
      dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
      if (cardIdx >= dueCards.length) cardIdx = 0;
      cardFlipped = false;
      renderApp();
    };
  }

  const deleteVocabBtn = document.getElementById('deleteVocabBtn');
  if (deleteVocabBtn) {
    deleteVocabBtn.onclick = async () => {
      const card = dueCards[cardIdx];
      if (!card) return;
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
      addVocabOpen = true;
      addVocabMsg = '';
      renderApp();
      document.getElementById('addWordIn')?.focus();
    };
  }
  const closeDrawer = () => {
    addVocabOpen = false;
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
      quizPillMode = btn.dataset.qpill as typeof quizPillMode;
      if (quizPillMode === 'fix' && !errorFixQuestions.length) {
        await loadErrorFixQuiz();
      } else if (quizPillMode === 'micro' && !microQuestions.length && !microSummary) {
        await startMicroCheck();
      }
      renderApp();
    };
  });

  // Error Fix Quiz
  document.getElementById('startErrFixBtn')?.addEventListener('click', async () => {
    await loadErrorFixQuiz();
    renderApp();
  });
  document.getElementById('restartErrFixBtn')?.addEventListener('click', async () => {
    await loadErrorFixQuiz();
    renderApp();
  });
  const errFixIn = document.getElementById('errFixInput') as HTMLTextAreaElement | null;
  const submitErrFix = () => {
    const q = errorFixQuestions[errorFixIdx];
    if (!q || errorFixFeedback || !errFixIn) return;
    errorFixInput = errFixIn.value;
    const ok = isErrorFixMatch(errorFixInput, q.right_text);
    if (ok) errorFixCorrectCount += 1;
    errorFixFeedback = { correct: ok, target: q.right_text };
    renderApp();
  };
  if (errFixIn) {
    errFixIn.oninput = () => {
      errorFixInput = errFixIn.value;
    };
    errFixIn.onkeydown = (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        submitErrFix();
      }
    };
  }
  document.getElementById('checkErrFixBtn')?.addEventListener('click', submitErrFix);
  document.getElementById('nextErrFixBtn')?.addEventListener('click', () => {
    if (errorFixIdx + 1 >= errorFixQuestions.length) {
      errorFixDone = true;
    } else {
      errorFixIdx += 1;
      errorFixInput = '';
      errorFixFeedback = null;
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
    const ok = !gaveUp && (q.item_type === 'error_fix' ? isErrorFixMatch(val, q.answer_text) : isTypingAnswerMatch(val, q.answer_text));
    microAnswers.push({ item_type: q.item_type, ref_id: q.ref_id, correct: ok, gave_up: gaveUp });
    microFeedback = { correct: ok, gaveUp, target: q.answer_text, note: q.meaning_or_note };
    renderApp();
  };
  if (microInEl) {
    microInEl.oninput = () => {
      microInput = microInEl.value;
    };
    microInEl.onkeydown = (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        checkMicro(false);
      }
    };
  }
  document.getElementById('microCheckBtn')?.addEventListener('click', () => checkMicro(false));
  document.getElementById('microGiveUpBtn')?.addEventListener('click', () => checkMicro(true));
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
  });

  document.querySelectorAll<HTMLButtonElement>('[data-rate]').forEach((btn) => {
    btn.onclick = () => {
      dictationRate = Number(btn.dataset.rate);
      renderApp();
    };
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
    const tokensHtml = tokens
      .map((t) =>
        t.kind === 'ok'
          ? `<span class="w-ok">${esc(t.word)}</span>`
          : t.kind === 'miss'
            ? `<span class="w-fix">${esc(t.word)}</span>`
            : `<span class="w-bad">${esc(t.word)}</span>`
      )
      .join(' ');
    dictationChecked = { pct, tokensHtml, wrongWords, pushed: false };
    renderApp();
  };

  if (dictInputEl) {
    dictInputEl.oninput = () => {
      dictationInput = dictInputEl.value;
    };
    dictInputEl.onkeydown = (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        runDictCheck();
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
    if (!cur || !dictationChecked || dictationChecked.pushed || !dictationChecked.wrongWords.length) return;
    const wrongSnippet = dictationInput.trim() || '(nghe sót)';
    const rightSnippet = dictationChecked.wrongWords.join(', ');
    dash = await invoke<DashboardState>('add_error_log', {
      category: `Listening · ${cur.trap_focus}`,
      wrongText: wrongSnippet.slice(0, 80),
      rightText: rightSnippet,
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
    } else {
      dictationIdx += 1;
      dictationInput = '';
      dictationHintWords = 0;
      dictationChecked = null;
    }
    renderApp();
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
      renderApp();
      return;
    }
    errFormMsg = '✓ Đã thêm vào Sổ lỗi';
    errFormErr = false;
    scrollToNewErr = true;
    dash = await invoke<DashboardState>('add_error_log', { category, wrongText, rightText });
    renderApp();
  });

  document.getElementById('addParaBtn')?.addEventListener('click', async () => {
    const qTerm = (document.getElementById('paraQTerm') as HTMLInputElement).value.trim();
    const pPass = (document.getElementById('paraPassage') as HTMLInputElement).value.trim();
    const pSrc = (document.getElementById('paraSource') as HTMLInputElement).value.trim() || 'Study4 Reading';
    const pTrap = (document.getElementById('paraTrap') as HTMLInputElement).value.trim();
    if (!qTerm || !pPass) {
      paraFormMsg = '✕ Nhập Từ câu hỏi và Trong bài đọc.';
      paraFormErr = true;
      renderApp();
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

  if (scrollToNewErr) {
    scrollToNewErr = false;
    document.querySelector('.log-table-wrap')?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }

  // Speaking Studio Overlay
  const closeStudio = () => {
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
    renderApp();
  };
  document.getElementById('closeStudioBtn')?.addEventListener('click', closeStudio);
  document.getElementById('vBackdrop')?.addEventListener('click', () => {
    if (addVocabOpen) addVocabOpen = false;
    if (studioOpen) closeStudio();
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

  document.getElementById('studioReplayBtn')?.addEventListener('click', () => {
    const last = recordedBlobs[recordedBlobs.length - 1];
    if (!last) return;
    if (studioPlaybackAudio) {
      studioPlaybackAudio.pause();
    }
    studioPlaybackAudio = new Audio(last.url);
    void studioPlaybackAudio.play();
  });

  const downloadLastAudio = () => {
    const last = recordedBlobs[recordedBlobs.length - 1];
    if (!last) return;
    const a = document.createElement('a');
    a.href = last.url;
    a.download = `xd4t-speaking-${Date.now()}.webm`;
    a.click();
  };
  document.getElementById('studioSaveAudioBtn')?.addEventListener('click', downloadLastAudio);
  document.getElementById('fallbackDownloadAudioBtn')?.addEventListener('click', downloadLastAudio);

  document.getElementById('gradeSpeakingAiBtn')?.addEventListener('click', async () => {
    if (!dash) return;
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
  const openSettings = () => settingsModal.classList.add('open');
  document.getElementById('openSettingsBtn')!.onclick = openSettings;
  document.getElementById('bannerOpenSettingsBtn')?.addEventListener('click', openSettings);
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
      dash.settings = await invoke('update_settings', { theme: btn.dataset.pickTheme! });
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
        dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
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
        widgetInfo = await invoke<WidgetRuntimeInfo>('set_widget_enabled', { enabled: widgetChk.checked });
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

  document.querySelectorAll<HTMLButtonElement>('.settings-level-card').forEach((btn) => {
    btn.onclick = async () => {
      dash = await invoke<DashboardState>('set_study_level', { level: btn.dataset.pickLevel! });
      dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
      cardIdx = 0;
      cardFlipped = false;
      studioCue = null;
      dictationBatch = [];
      renderApp();
      document.getElementById('settingsModal')?.classList.add('open');
    };
  });

  document.getElementById('saveSettingsBtn')!.onclick = async () => {
    const keyVal = (document.getElementById('setKey') as HTMLInputElement).value.trim();
    const modelVal = (document.getElementById('setModel') as HTMLSelectElement).value;
    const examVal = (document.getElementById('setExamDate') as HTMLInputElement).value.trim();
    const remVal = (document.getElementById('setReminder') as HTMLInputElement).value.trim();
    const autoVal = (document.getElementById('setAutostart') as HTMLInputElement).checked;
    const ttsVal = (document.getElementById('setTtsAccent') as HTMLSelectElement).value;
    await invoke('update_settings', {
      geminiApiKey: keyVal ? keyVal : null,
      geminiModel: modelVal,
      examDate: examVal,
      reminderTime: remVal,
      autostart: autoVal,
      ttsAccent: ttsVal,
    });
    dash = await invoke<DashboardState>('get_dashboard');
    settingsModal.classList.remove('open');
    renderApp();
  };

  // Welcome Wizard Events
  document.getElementById('skipWelcomeBtn')?.addEventListener('click', () => finishOnboarding());
  document.querySelectorAll<HTMLButtonElement>('.welcome-theme-card').forEach((btn) => {
    btn.onclick = async () => {
      if (!dash) return;
      dash.settings = await invoke('update_settings', { theme: btn.dataset.pickTheme! });
      renderApp();
    };
  });
  document.querySelectorAll<HTMLButtonElement>('.welcome-level-card').forEach((btn) => {
    btn.onclick = async () => {
      const level = btn.dataset.pickLevel!;
      dash = await invoke<DashboardState>('set_study_level', { level });
      dueCards = await invoke<VocabCard[]>('get_due_vocab', { onlyLeech: onlyLeechFilter });
      cardIdx = 0;
      cardFlipped = false;
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
  document.getElementById('openRoadmapModalBtn')?.addEventListener('click', () => roadmapModal.classList.add('open'));
  document.getElementById('openSampleMdBtn')?.addEventListener('click', async () => {
    roadmapDraftMd = await invoke<string>('get_sample_roadmap_md');
    roadmapPreview = await invoke<RoadmapPreview>('preview_roadmap_md', { markdown: roadmapDraftMd });
    roadmapStatusMsg = '✓ Đã nạp sẵn mẫu chuẩn XD4T-1ELTS v1.6.0 để bạn xem / chỉnh sửa.';
    roadmapStatusOk = true;
    renderApp();
    document.getElementById('roadmapModal')!.classList.add('open');
  });
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
    scoreModal.classList.add('open');
  };
  document.getElementById('openScoreModalBtn')!.onclick = () => openScoreForSkill();
  document.getElementById('openScoreTab1Btn')?.addEventListener('click', () => openScoreForSkill());
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

// Global keyboard shortcuts
document.addEventListener('keydown', (e) => {
  const activeTag = document.activeElement?.tagName;
  const isTyping = activeTag === 'INPUT' || activeTag === 'TEXTAREA' || activeTag === 'SELECT';

  if (e.key === 'Escape') {
    const welcomeEl = document.getElementById('welcomeModal');
    if (welcomeEl?.classList.contains('open')) {
      welcomeOpen = false;
      void finishOnboarding();
    }
    if (addVocabOpen) {
      addVocabOpen = false;
      renderApp();
    }
    if (studioOpen) {
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
      renderApp();
    }
    document.querySelectorAll('.modal-backdrop.open').forEach((m) => m.classList.remove('open'));
    return;
  }

  if (e.ctrlKey && e.code === 'Space' && activeTab === 'vocab' && vocabSubMode === 'dictation') {
    e.preventDefault();
    const cur = dictationBatch[dictationIdx];
    if (cur) speakText(cur.sentence, dash?.settings.tts_accent || 'en-GB', dictationRate);
    return;
  }

  if (!isTyping && activeTab === 'vocab' && vocabSubMode === 'flashcard' && !studioOpen && !addVocabOpen) {
    if (e.code === 'Space') {
      e.preventDefault();
      cardFlipped = !cardFlipped;
      renderApp();
    } else if (e.key.toLowerCase() === 'v') {
      const cur = dueCards[cardIdx];
      if (cur) speakText(cur.word, dash?.settings.tts_accent || 'en-GB', 1.0);
    }
  }
});

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

  await listen('start-pomodoro-15', () => {
    pomoSecs = 15 * 60;
    togglePomodoro(true);
  });
}

init();
