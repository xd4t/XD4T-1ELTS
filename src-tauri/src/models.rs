use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub theme: String,
    pub gemini_api_key: String,
    pub gemini_model: String,
    pub exam_date: String,
    pub reminder_time: String,
    pub autostart: bool,
    pub target_band: f64,
    pub onboarding_done: bool,
    pub tts_accent: String,
    pub writing_checklist: Vec<String>,
    pub writing_focus_notes: Vec<String>,
    pub skill_targets: BTreeMap<String, f64>,
    pub phase_labels: BTreeMap<i64, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsPublic {
    pub theme: String,
    pub has_api_key: bool,
    pub api_key_masked: String,
    pub gemini_model: String,
    pub exam_date: String,
    pub reminder_time: String,
    pub autostart: bool,
    pub target_band: f64,
    pub onboarding_done: bool,
    pub tts_accent: String,
    pub writing_checklist: Vec<String>,
    pub writing_focus_notes: Vec<String>,
    pub phase_labels: BTreeMap<i64, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyQuest {
    pub id: i64,
    pub date: String,
    pub track: String,
    pub title: String,
    pub meta: String,
    pub minutes: i64,
    pub done: bool,
    #[serde(default = "default_tier_core")]
    pub tier: String,
    #[serde(default)]
    pub action_hint: String,
}

fn default_tier_core() -> String {
    "core".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyDay {
    pub date: String,
    pub track: String,
    pub minutes: i64,
    pub quests_done: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabCard {
    pub id: i64,
    pub track: String,
    pub topic: String,
    pub word: String,
    pub meaning: String,
    pub example: String,
    pub ease: f64,
    pub interval_days: i64,
    pub reps: i64,
    pub due_date: String,
    #[serde(default)]
    pub lapses: i64,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub is_user_card: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingPrompt {
    pub id: String,
    pub mode: String,
    pub title: String,
    pub prompt: String,
    pub outline_hint: String,
    pub target_words: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Correction {
    pub wrong: String,
    pub right: String,
    pub category: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradeResult {
    pub ta: f64,
    pub cc: f64,
    pub lr: f64,
    pub gra: f64,
    pub overall: f64,
    pub corrections: Vec<Correction>,
    pub band65_rewrite: String,
    #[serde(default)]
    pub summary_vi: String,
    #[serde(default)]
    pub repeated_top_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Essay {
    pub id: i64,
    pub mode: String,
    pub prompt: String,
    pub content: String,
    pub word_count: i64,
    pub ta: f64,
    pub cc: f64,
    pub lr: f64,
    pub gra: f64,
    pub overall: f64,
    pub feedback_json: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLog {
    pub id: i64,
    pub category: String,
    pub wrong_text: String,
    pub right_text: String,
    pub count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PracticeScore {
    pub id: i64,
    pub skill: String,
    pub source: String,
    pub score: f64,
    pub note: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapItem {
    pub id: i64,
    pub track: String,
    pub phase: i64,
    pub week_label: String,
    pub title: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillBand {
    pub skill: String,
    pub current: f64,
    pub target: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VocabQuizStatus {
    pub last_quiz_at: Option<String>,
    pub days_since_last_quiz: Option<i64>,
    pub last_quiz_score: Option<String>,
    pub total_quizzes: i64,
    pub quiz_due: bool,
    pub quiz_due_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub card_id: i64,
    pub word: String,
    pub meaning: String,
    pub example: String,
    pub topic: String,
    pub question_type: String,
    pub prompt_text: String,
    pub cloze_sentence: Option<String>,
    pub hint: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizAnswerInput {
    pub card_id: i64,
    pub correct: bool,
    pub gave_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgottenVocabCard {
    pub card_id: i64,
    pub word: String,
    pub meaning: String,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizSubmitResult {
    pub total_questions: i64,
    pub correct_count: i64,
    pub accuracy_pct: i64,
    pub forgotten_cards: Vec<ForgottenVocabCard>,
    pub marked_quest_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParaphrasePair {
    pub id: i64,
    pub question_term: String,
    pub passage_paraphrase: String,
    pub source_type: String,
    pub trap_note: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationSentence {
    pub id: i64,
    pub level: String,
    pub section: String,
    pub trap_focus: String,
    pub sentence: String,
    pub meaning_vi: String,
    pub practiced_count: i64,
    pub last_accuracy: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationResultInput {
    pub sentence_id: i64,
    pub accuracy_pct: i64,
    pub wrong_words: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationSubmitResult {
    pub total_sentences: i64,
    pub avg_accuracy: i64,
    pub marked_quest_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorFixQuestion {
    pub error_id: i64,
    pub category: String,
    pub wrong_text: String,
    pub right_text: String,
    pub count: i64,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroCheckItem {
    pub item_type: String,
    pub ref_id: i64,
    pub source_label: String,
    pub prompt_text: String,
    pub answer_text: String,
    pub hint: String,
    pub meaning_or_note: String,
    pub options: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroCheckAnswerInput {
    pub item_type: String,
    pub ref_id: i64,
    pub correct: bool,
    pub gave_up: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroCheckSubmitResult {
    pub total_questions: i64,
    pub correct_count: i64,
    pub accuracy_pct: i64,
    pub marked_quest_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakingCue {
    pub id: i64,
    pub week_num: i64,
    pub level: String,
    pub topic: String,
    pub cue_title: String,
    pub cue_bullets: Vec<String>,
    pub part3_questions: Vec<String>,
    pub target_phrases: Vec<String>,
    pub used_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakingErrorItem {
    pub wrong: String,
    pub fix: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakingAiPayload {
    pub transcript: String,
    pub fc_band: f64,
    pub lr_band: f64,
    pub gra_band: f64,
    pub p_band: f64,
    pub overall_band: f64,
    pub wpm_start: i64,
    pub wpm_end: i64,
    pub fillers_start: i64,
    pub fillers_end: i64,
    pub progress_note: String,
    #[serde(default)]
    pub top_errors: Vec<SpeakingErrorItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakingSession {
    pub id: i64,
    pub created_at: String,
    pub cue_id: Option<i64>,
    pub cue_title: String,
    pub track: String,
    pub mode: String,
    pub wpm_start: i64,
    pub wpm_end: i64,
    pub fillers_start: i64,
    pub fillers_end: i64,
    pub fc_band: f64,
    pub lr_band: f64,
    pub gra_band: f64,
    pub p_band: f64,
    pub overall_band: f64,
    pub progress_note: String,
    pub transcript: String,
    pub feedback_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeakingGradeResponse {
    pub session: SpeakingSession,
    pub top_errors: Vec<SpeakingErrorItem>,
    pub endpoint: String,
    pub marked_quest_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    pub settings: SettingsPublic,
    pub streak: i64,
    pub days_until_exam: i64,
    pub quests: Vec<DailyQuest>,
    pub heatmap: Vec<StudyDay>,
    pub skill_bands: Vec<SkillBand>,
    pub due_vocab_count: i64,
    pub total_vocab_count: i64,
    pub leech_count: i64,
    pub current_week: i64,
    pub quiz_status: VocabQuizStatus,
    pub roadmap: Vec<RoadmapItem>,
    pub error_logs: Vec<ErrorLog>,
    pub paraphrase_pairs: Vec<ParaphrasePair>,
    pub speaking_sessions: Vec<SpeakingSession>,
    pub recent_essays: Vec<Essay>,
    pub practice_scores: Vec<PracticeScore>,
    pub prompts: Vec<WritingPrompt>,
}
