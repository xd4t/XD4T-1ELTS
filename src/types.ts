export interface SettingsPublic {
  theme: string;
  has_api_key: boolean;
  api_key_masked: string;
  gemini_model: string;
  exam_date: string;
  reminder_time: string;
  autostart: boolean;
  target_band: number;
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

export interface DashboardState {
  settings: SettingsPublic;
  streak: number;
  days_until_exam: number;
  quests: DailyQuest[];
  heatmap: StudyDay[];
  skill_bands: SkillBand[];
  due_vocab_count: number;
  total_vocab_count: number;
  roadmap: RoadmapItem[];
  error_logs: ErrorLog[];
  practice_scores: PracticeScore[];
  prompts: WritingPrompt[];
}
