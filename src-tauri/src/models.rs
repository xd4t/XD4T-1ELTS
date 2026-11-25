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
pub struct DashboardState {
    pub settings: SettingsPublic,
    pub streak: i64,
    pub days_until_exam: i64,
    pub quests: Vec<DailyQuest>,
    pub heatmap: Vec<StudyDay>,
    pub skill_bands: Vec<SkillBand>,
    pub due_vocab_count: i64,
    pub total_vocab_count: i64,
    pub roadmap: Vec<RoadmapItem>,
    pub error_logs: Vec<ErrorLog>,
    pub recent_essays: Vec<Essay>,
    pub practice_scores: Vec<PracticeScore>,
    pub prompts: Vec<WritingPrompt>,
}
