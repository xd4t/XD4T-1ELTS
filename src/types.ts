export interface SettingsPublic {
  theme: string;
  has_api_key: boolean;
  api_key_masked: string;
  gemini_model: string;
  exam_date: string;
  reminder_time: string;
  autostart: boolean;
  target_band: number;
  onboarding_done: boolean;
  writing_checklist: string[];
  writing_focus_notes: string[];
  phase_labels: Record<string, string>;
}

export interface BackupResultDto {
  cancelled: boolean;
  path: string | null;
  message: string;
}

export interface RestoreResultDto {
  cancelled: boolean;
  dashboard: DashboardState | null;
  message: string;
}

export interface WidgetRuntimeInfo {
  running: boolean;
  autostart: boolean;
  exe_found: boolean;
}

export interface RoadmapPreview {
  exam_date: string;
  overall_current: number;
  overall_target: number;
  buffer_target: number;
  skill_bands: SkillBand[];
  daily_minutes: number;
  reminder_time: string;
  quests_count: number;
  checklist_count: number;
  focus_notes_count: number;
  error_logs_count: number;
  vocab_count: number;
  phases_count: number;
  weeks_count: number;
  phases_summary: string[];
  warnings: string[];
}

export interface ApiTestResult {
  ok: boolean;
  status_code: number;
  endpoint: string;
  message: string;
}

export interface DailyQuest {
  id: number;
  date: string;
  track: string;
  title: string;
  meta: string;
  minutes: number;
  done: boolean;
}

export interface StudyDay {
  date: string;
  track: string;
  minutes: number;
  quests_done: number;
}

export interface VocabCard {
  id: number;
  track: string;
  topic: string;
  word: string;
  meaning: string;
  example: string;
  ease: number;
  interval_days: number;
  reps: number;
  due_date: string;
}

export interface WritingPrompt {
  id: string;
  mode: string;
  title: string;
  prompt: string;
  outline_hint: string;
  target_words: number;
}

export interface Correction {
  wrong: string;
  right: string;
  category: string;
  reason: string;
}

export interface GradeResult {
  ta: number;
  cc: number;
  lr: number;
  gra: number;
  overall: number;
  corrections: Correction[];
  band65_rewrite: string;
  summary_vi: string;
}

export interface GradeResponseDto {
  result: GradeResult;
  endpoint: string;
  essay_id: number;
}

export interface ErrorLog {
  id: number;
  category: string;
  wrong_text: string;
  right_text: string;
  count: number;
  created_at: string;
}

export interface PracticeScore {
  id: number;
  skill: string;
  source: string;
  score: number;
  note: string;
  created_at: string;
}

export interface RoadmapItem {
  id: number;
  track: string;
  phase: number;
  week_label: string;
  title: string;
  done: boolean;
}

export interface SkillBand {
  skill: string;
  current: number;
  target: number;
}

export interface VocabQuizStatus {
  last_quiz_at: string | null;
  days_since_last_quiz: number | null;
  last_quiz_score: string | null;
  total_quizzes: number;
  quiz_due: boolean;
  quiz_due_reason: string;
}

export interface QuizQuestion {
  card_id: number;
  word: string;
  meaning: string;
  example: string;
  topic: string;
  question_type: 'meaning_mcq' | 'cloze_mcq' | 'typing';
  prompt_text: string;
  cloze_sentence: string | null;
  hint: string;
  options: string[];
}

export interface QuizAnswerInput {
  card_id: number;
  correct: boolean;
  gave_up: boolean;
}

export interface ForgottenVocabCard {
  card_id: number;
  word: string;
  meaning: string;
  example: string;
}

export interface QuizSubmitResult {
  total_questions: number;
  correct_count: number;
  accuracy_pct: number;
  forgotten_cards: ForgottenVocabCard[];
  marked_quest_done: boolean;
}

export interface DashboardState {
  settings: SettingsPublic;
  streak: number;
  days_until_exam: number;
  quests: DailyQuest[];
  heatmap: StudyDay[];
  skill_bands: SkillBand[];
  due_vocab_count: number;
  total_vocab_count: number;
  quiz_status: VocabQuizStatus;
  roadmap: RoadmapItem[];
  error_logs: ErrorLog[];
  practice_scores: PracticeScore[];
  prompts: WritingPrompt[];
}
