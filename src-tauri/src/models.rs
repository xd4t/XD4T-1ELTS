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
    #[serde(default = "default_true")]
    pub auto_refill_enabled: bool,
    #[serde(default)]
    pub last_ai_refill_at: String,
    #[serde(default)]
    pub last_ai_refill_topic: String,
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
    #[serde(default = "default_true")]
    pub auto_refill_enabled: bool,
    #[serde(default)]
    pub last_ai_refill_at: String,
    #[serde(default)]
    pub last_ai_refill_topic: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRefillStatus {
    pub enabled: bool,
    pub should_auto_refill: bool,
    pub trigger_reason: String,
    pub recommended_topic: String,
    pub last_refill_at: String,
    pub last_refill_topic: String,
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

fn default_seed_source() -> String {
    "seed".to_string()
}

fn default_level_65() -> String {
    "6.5".to_string()
}

fn default_drill_mode() -> String {
    "GRAMMAR_MCQ".to_string()
}

fn default_error_status() -> String {
    "active".to_string()
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
    #[serde(default = "default_seed_source")]
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WritingPrompt {
    pub id: String,
    pub mode: String,
    pub title: String,
    pub prompt: String,
    pub outline_hint: String,
    pub target_words: i64,
    #[serde(default)]
    pub topic: String,
    #[serde(default)]
    pub target_vocab: Vec<String>,
    #[serde(default = "default_seed_source")]
    pub source: String,
    #[serde(default)]
    pub practiced_count: i64,
    #[serde(default)]
    pub chart_spec_json: String,
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
    #[serde(default = "default_drill_mode")]
    pub drill_mode: String,
    #[serde(default = "default_error_status")]
    pub status: String,
    #[serde(default)]
    pub streak_distinct: i64,
    #[serde(default)]
    pub last_drill_id: i64,
    #[serde(default)]
    pub last_context_tag: String,
    #[serde(default)]
    pub drill_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDrillQuestion {
    pub drill_id: i64,
    pub error_log_id: i64,
    pub category: String,
    pub drill_mode: String,
    pub context_tag: String,
    pub stem: String,
    pub options: Vec<String>,
    pub answer: String,
    pub rationale_vi: String,
    pub wrong_text: String,
    pub right_text: String,
    pub streak_distinct: i64,
    pub status: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDrillResultInput {
    pub error_log_id: i64,
    pub drill_id: i64,
    pub context_tag: String,
    pub correct: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDrillSingleResult {
    pub error_log_id: i64,
    pub streak_distinct: i64,
    pub status: String,
    pub newly_mastered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDrillSubmitResult {
    pub total_questions: i64,
    pub correct_count: i64,
    pub accuracy_pct: i64,
    pub newly_mastered: Vec<String>,
    pub marked_quest_done: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DictationErrorPair {
    pub wrong: String,
    pub right: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicPackVocab {
    pub word: String,
    pub meaning: String,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicPackDictation {
    #[serde(default)]
    pub section: String,
    #[serde(default)]
    pub trap_focus: String,
    pub sentence: String,
    pub meaning_vi: String,
}

fn deserialize_string_or_json_val<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val = serde_json::Value::deserialize(deserializer)?;
    match val {
        serde_json::Value::Null => Ok(String::new()),
        serde_json::Value::String(s) => Ok(s),
        other => Ok(other.to_string()),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicPackPrompt {
    pub task_type: String,
    pub title: String,
    pub prompt_text: String,
    #[serde(default)]
    pub outline_hint: String,
    #[serde(default)]
    pub target_vocab: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_or_json_val")]
    pub chart_spec_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicPackParaphrase {
    pub question_term: String,
    pub passage_paraphrase: String,
    #[serde(default)]
    pub source_type: String,
    #[serde(default)]
    pub trap_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicPackDrill {
    #[serde(default)]
    pub error_index: usize,
    pub drill_mode: String,
    #[serde(default)]
    pub context_tag: String,
    pub stem: String,
    #[serde(default)]
    pub options: Vec<String>,
    pub answer: String,
    #[serde(default)]
    pub rationale_vi: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicPackPayload {
    #[serde(default)]
    pub vocab: Vec<TopicPackVocab>,
    #[serde(default)]
    pub dictations: Vec<TopicPackDictation>,
    #[serde(default)]
    pub writing_prompts: Vec<TopicPackPrompt>,
    #[serde(default)]
    pub paraphrases: Vec<TopicPackParaphrase>,
    #[serde(default)]
    pub error_drills: Vec<TopicPackDrill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefillPackResult {
    pub topic: String,
    pub level: String,
    pub vocab_added: i64,
    pub dictations_added: i64,
    pub prompts_added: i64,
    pub paraphrases_added: i64,
    pub drills_added: i64,
    pub endpoint: String,
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
    #[serde(default = "default_level_65")]
    pub level: String,
    #[serde(default = "default_seed_source")]
    pub source: String,
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
    #[serde(default = "default_seed_source")]
    pub source: String,
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
pub struct ErrorFixAiGrade {
    pub correct: bool,
    pub feedback: String,
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
    pub auto_refill_status: AutoRefillStatus,
}
