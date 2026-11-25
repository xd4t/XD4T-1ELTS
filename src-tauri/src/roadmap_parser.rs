use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::models::SkillBand;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedQuest {
    pub title: String,
    pub meta: String,
    pub minutes: i64,
    #[serde(default = "default_core_tier")]
    pub tier: String,
    #[serde(default)]
    pub action_hint: String,
}

fn default_core_tier() -> String {
    "core".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedErrorLog {
    pub category: String,
    pub wrong_text: String,
    pub right_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedVocabCard {
    pub topic: String,
    pub word: String,
    pub meaning: String,
    pub example: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedParaphrase {
    pub question_term: String,
    pub passage_paraphrase: String,
    pub source_type: String,
    pub trap_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRoadmapWeek {
    pub phase: i64,
    pub phase_label: String,
    pub week_label: String,
    pub title: String,
    #[serde(default)]
    pub checked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedRoadmap {
    pub exam_date: Option<String>,
    pub overall_current: Option<f64>,
    pub overall_target: Option<f64>,
    pub buffer_target: Option<f64>,
    pub skill_bands: Vec<SkillBand>,
    pub daily_minutes: Option<i64>,
    pub reminder_time: Option<String>,
    pub daily_quests: Vec<ParsedQuest>,
    pub writing_checklist: Vec<String>,
    pub writing_focus_notes: Vec<String>,
    pub error_logs: Vec<ParsedErrorLog>,
    pub vocab_cards: Vec<ParsedVocabCard>,
    pub paraphrase_pairs: Vec<ParsedParaphrase>,
    pub roadmap_weeks: Vec<ParsedRoadmapWeek>,
    pub phase_labels: BTreeMap<i64, String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapPreview {
    pub exam_date: String,
    pub overall_current: f64,
    pub overall_target: f64,
    pub buffer_target: f64,
    pub skill_bands: Vec<SkillBand>,
    pub daily_minutes: i64,
    pub reminder_time: String,
    pub quests_count: usize,
    pub checklist_count: usize,
    pub focus_notes_count: usize,
    pub error_logs_count: usize,
    pub vocab_count: usize,
    pub paraphrase_count: usize,
    pub phases_count: usize,
    pub weeks_count: usize,
    pub phases_summary: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    None,
    DailyQuests,
    WritingChecklist,
    WritingFocus,
    ErrorLogs,
    Vocab,
    Paraphrase,
    Roadmap,
}

fn strip_outer_fences(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with("```") {
        let mut lines: Vec<&str> = trimmed.lines().collect();
        if !lines.is_empty() && lines[0].trim_start().starts_with("```") {
            lines.remove(0);
        }
        if !lines.is_empty() && lines.last().map(|l| l.trim() == "```").unwrap_or(false) {
            lines.pop();
        }
        return lines.join("\n");
    }
    trimmed.to_string()
}

fn strip_backticks(s: &str) -> String {
    s.replace('`', "").trim().to_string()
}

fn strip_list_marker_with_check(line: &str) -> Option<(&str, bool)> {
    let t = line.trim();
    let rest = if let Some(r) = t.strip_prefix("- ") {
        Some(r.trim())
    } else if let Some(r) = t.strip_prefix("* ") {
        Some(r.trim())
    } else if let Some(dot_idx) = t.find(". ") {
        if dot_idx > 0 && dot_idx <= 3 && t[..dot_idx].chars().all(|c| c.is_ascii_digit()) {
            Some(t[dot_idx + 2..].trim())
        } else {
            None
        }
    } else {
        None
    }?;

    if let Some(after) = rest.strip_prefix("[ ] ") {
        return Some((after.trim(), false));
    }
    if let Some(after) = rest.strip_prefix("[x] ").or_else(|| rest.strip_prefix("[X] ")) {
        return Some((after.trim(), true));
    }
    Some((rest, false))
}

fn normalize_date(raw: &str) -> Option<String> {
    let clean = strip_backticks(raw);
    if NaiveDate::parse_from_str(&clean, "%Y-%m-%d").is_ok() {
        return Some(clean);
    }
    if let Ok(d) = NaiveDate::parse_from_str(&clean, "%d/%m/%Y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }
    if let Ok(d) = NaiveDate::parse_from_str(&clean, "%d-%m-%Y") {
        return Some(d.format("%Y-%m-%d").to_string());
    }
    None
}

fn normalize_time(raw: &str) -> Option<String> {
    let clean = strip_backticks(raw);
    let parts: Vec<&str> = clean.split(':').collect();
    if parts.len() == 2 {
        if let (Ok(h), Ok(m)) = (parts[0].trim().parse::<u32>(), parts[1].trim().parse::<u32>()) {
            if h < 24 && m < 60 {
                return Some(format!("{h:02}:{m:02}"));
            }
        }
    }
    None
}

fn parse_band_pair(raw: &str) -> Option<(f64, f64)> {
    let clean = strip_backticks(raw).replace('→', "->");
    let parts: Vec<&str> = clean.split("->").collect();
    if parts.len() == 2 {
        let cur = parts[0].trim().parse::<f64>().ok()?;
        let tgt = parts[1].trim().parse::<f64>().ok()?;
        return Some((cur.clamp(0.0, 9.0), tgt.clamp(0.0, 9.0)));
    }
    None
}

fn detect_section(heading: &str) -> Section {
    let h = heading.trim().to_lowercase();
    if h.contains("nhiệm vụ")
        || h.contains("nhiem vu")
        || h.contains("lịch ngày")
        || h.contains("lich ngay")
        || h.contains("daily")
        || h.contains("quest")
    {
        Section::DailyQuests
    } else if h.contains("checklist") {
        Section::WritingChecklist
    } else if h.contains("điểm yếu") || h.contains("diem yeu") || h.contains("soi kỹ") || h.contains("soi ky") || h.contains("focus") {
        Section::WritingFocus
    } else if h.contains("sổ lỗi") || h.contains("so loi") || h.contains("error") {
        Section::ErrorLogs
    } else if h.contains("paraphrase") {
        Section::Paraphrase
    } else if h.contains("từ vựng")
        || h.contains("tu vung")
        || h.contains("sentence mining")
        || h.contains("srs")
        || h.contains("vocab")
    {
        Section::Vocab
    } else if h.contains("lộ trình")
        || h.contains("lo trinh")
        || h.contains("giai đoạn")
        || h.contains("giai doan")
        || h.contains("roadmap")
        || h.contains("theo tuần")
    {
        Section::Roadmap
    } else {
        Section::None
    }
}

pub fn infer_quest_action_hint(title: &str) -> String {
    let low = title.to_lowercase();
    if low.contains("micro-check") || low.contains("micro check") {
        "micro_check".to_string()
    } else if low.contains("dictation") || low.contains("nghe chép") || low.contains("chép chính tả") {
        "dictation".to_string()
    } else if low.contains("luyện nói") || low.contains("cue card") || low.contains("speaking") {
        "speaking".to_string()
    } else if low.contains("quiz") || low.contains("cứng đầu") || low.contains("leech") {
        "quiz".to_string()
    } else if low.contains("paraphrase") && !low.contains("study4") {
        "paraphrase".to_string()
    } else if low.contains("study4") {
        "study4".to_string()
    } else if low.contains("writing") || low.contains("viết") {
        "writing".to_string()
    } else if low.contains("srs") || low.contains("flashcard") || low.contains("từ vựng") {
        "srs".to_string()
    } else {
        String::new()
    }
}

fn parse_quest_line(item: &str) -> ParsedQuest {
    let clean = strip_backticks(item);
    let mut minutes = 15_i64;
    let mut tier = "core".to_string();
    let mut rest = clean.as_str();

    loop {
        if rest.starts_with('[') {
            if let Some(end_idx) = rest.find(']') {
                let tag = rest[1..end_idx].trim();
                let tag_low = tag.to_lowercase();
                if tag_low == "bonus" || tag_low == "tùy chọn" {
                    tier = "bonus".to_string();
                } else if tag_low == "core" {
                    tier = "core".to_string();
                } else {
                    let digits: String = tag.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if let Ok(m) = digits.parse::<i64>() {
                        if m > 0 {
                            minutes = m;
                        }
                    }
                }
                rest = rest[end_idx + 1..].trim();
                continue;
            }
        }
        break;
    }

    let (title, meta) = if let Some((t, m)) = rest.split_once('|') {
        (t.trim().to_string(), m.trim().to_string())
    } else {
        (rest.trim().to_string(), "Nhiệm vụ cá nhân".to_string())
    };
    let action_hint = infer_quest_action_hint(&title);

    ParsedQuest {
        title,
        meta,
        minutes,
        tier,
        action_hint,
    }
}

fn parse_error_line(item: &str) -> Option<ParsedErrorLog> {
    let clean = strip_backticks(item);
    let mut category = "Ngữ pháp".to_string();
    let mut rest = clean.as_str();

    if rest.starts_with('[') {
        if let Some(end_idx) = rest.find(']') {
            let cat = rest[1..end_idx].trim();
            if !cat.is_empty() {
                category = cat.to_string();
            }
            rest = rest[end_idx + 1..].trim();
        }
    }

    let normalized = rest.replace('→', "->");
    let (wrong, right) = normalized.split_once("->")?;
    let wrong_text = wrong.trim().to_string();
    let right_text = right.trim().to_string();
    if wrong_text.is_empty() || right_text.is_empty() {
        return None;
    }
    Some(ParsedErrorLog {
        category,
        wrong_text,
        right_text,
    })
}

fn parse_vocab_line(item: &str) -> Option<ParsedVocabCard> {
    let clean = strip_backticks(item);
    let mut topic = "Sentence Mining".to_string();
    let mut rest = clean.as_str();

    if rest.starts_with('[') {
        if let Some(end_idx) = rest.find(']') {
            let t = rest[1..end_idx].trim();
            if !t.is_empty() {
                topic = t.to_string();
            }
            rest = rest[end_idx + 1..].trim();
        }
    }

    let parts: Vec<&str> = rest.split('|').map(|s| s.trim()).collect();
    if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
        return None;
    }
    let example = if parts.len() >= 3 && !parts[2].is_empty() {
        parts[2].to_string()
    } else {
        format!("Example with {}.", parts[0])
    };
    Some(ParsedVocabCard {
        topic,
        word: parts[0].to_string(),
        meaning: parts[1].to_string(),
        example,
    })
}

fn parse_paraphrase_line(item: &str) -> Option<ParsedParaphrase> {
    let clean = strip_backticks(item);
    let normalized = clean.replace('→', "->");
    let (left, right_all) = normalized.split_once("->")?;
    let question_term = left.trim().to_string();
    if question_term.is_empty() {
        return None;
    }
    let parts: Vec<&str> = right_all.split('|').map(|s| s.trim()).collect();
    let passage_paraphrase = parts.first().copied().unwrap_or("").trim().to_string();
    if passage_paraphrase.is_empty() {
        return None;
    }
    let source_type = parts.get(1).copied().unwrap_or("Study4 Reading").trim().to_string();
    let trap_note = parts.get(2).copied().unwrap_or("").trim().to_string();
    Some(ParsedParaphrase {
        question_term,
        passage_paraphrase,
        source_type,
        trap_note,
    })
}

fn parse_roadmap_table_row(line: &str) -> Option<(String, String, bool)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
        return None;
    }
    let cells: Vec<&str> = trimmed
        .trim_matches('|')
        .split('|')
        .map(|s| s.trim())
        .collect();
    if cells.len() < 2 {
        return None;
    }
    // Skip separator rows like |---|---|
    if cells.iter().all(|c| !c.is_empty() && c.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')) {
        return None;
    }
    let c0_low = cells[0].to_lowercase();
    if c0_low == "tuần" || c0_low == "week" || c0_low == "mốc" {
        return None;
    }
    let week_label = strip_backticks(cells[0]);
    let mut title_parts: Vec<String> = cells[1..]
        .iter()
        .map(|c| strip_backticks(c))
        .filter(|s| !s.is_empty())
        .collect();
    if week_label.is_empty() || title_parts.is_empty() {
        return None;
    }
    let checked = title_parts.last().map(|s| s == "✓" || s.eq_ignore_ascii_case("done")).unwrap_or(false);
    if checked && title_parts.len() > 1 {
        title_parts.pop();
    }
    Some((week_label, title_parts.join(" · "), checked))
}

fn extract_phase_number(heading: &str, fallback_phase: i64) -> i64 {
    let lower = heading.to_lowercase();
    for prefix in ["gđ", "gd", "giai đoạn", "giai doan", "phase", "p"] {
        if let Some(idx) = lower.find(prefix) {
            let after = &lower[idx + prefix.len()..];
            let digits: String = after
                .trim_start()
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(n) = digits.parse::<i64>() {
                if n > 0 {
                    return n;
                }
            }
        }
    }
    fallback_phase
}

pub fn parse_roadmap_markdown(raw: &str) -> Result<ParsedRoadmap, String> {
    let content = strip_outer_fences(raw);
    if content.trim().is_empty() {
        return Err("Nội dung Markdown đang trống. Hãy dán lộ trình hoặc chọn file .md trước.".to_string());
    }

    let mut out = ParsedRoadmap {
        exam_date: None,
        overall_current: None,
        overall_target: None,
        buffer_target: None,
        skill_bands: Vec::new(),
        daily_minutes: None,
        reminder_time: None,
        daily_quests: Vec::new(),
        writing_checklist: Vec::new(),
        writing_focus_notes: Vec::new(),
        error_logs: Vec::new(),
        vocab_cards: Vec::new(),
        paraphrase_pairs: Vec::new(),
        roadmap_weeks: Vec::new(),
        phase_labels: BTreeMap::new(),
        warnings: Vec::new(),
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut idx = 0;

    if lines.first().map(|l| l.trim() == "---").unwrap_or(false) {
        idx = 1;
        while idx < lines.len() {
            let line = lines[idx].trim();
            idx += 1;
            if line == "---" {
                break;
            }
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim().to_lowercase();
                let val = strip_backticks(v.trim());
                match key.as_str() {
                    "exam_date" => {
                        if let Some(d) = normalize_date(&val) {
                            out.exam_date = Some(d);
                        } else {
                            out.warnings.push(format!("Ngày thi '{val}' chưa đúng chuẩn YYYY-MM-DD, giữ nguyên mặc định."));
                        }
                    }
                    "overall_current" => out.overall_current = val.parse::<f64>().ok(),
                    "overall_target" => out.overall_target = val.parse::<f64>().ok(),
                    "buffer_target" => out.buffer_target = val.parse::<f64>().ok(),
                    "daily_minutes" => out.daily_minutes = val.parse::<i64>().ok(),
                    "reminder_time" => {
                        if let Some(t) = normalize_time(&val) {
                            out.reminder_time = Some(t);
                        } else {
                            out.warnings.push(format!("Giờ nhắc '{val}' chưa đúng chuẩn HH:MM, giữ nguyên mặc định."));
                        }
                    }
                    "listening" | "reading" | "speaking" | "writing" => {
                        if let Some((cur, tgt)) = parse_band_pair(&val) {
                            let skill_name = match key.as_str() {
                                "listening" => "Listening",
                                "reading" => "Reading",
                                "speaking" => "Speaking",
                                _ => "Writing",
                            };
                            out.skill_bands.push(SkillBand {
                                skill: skill_name.to_string(),
                                current: cur,
                                target: tgt,
                            });
                        } else {
                            out.warnings.push(format!("Band '{key}: {val}' chưa đúng dạng 'hiện_tại -> mục_tiêu'."));
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    let mut section = Section::None;
    let mut current_phase: i64 = 1;
    let mut current_phase_label = "GĐ1: Lộ trình nền tảng".to_string();
    let mut week_counter = 0_usize;

    while idx < lines.len() {
        let line = lines[idx].trim();
        idx += 1;
        if line.is_empty() {
            continue;
        }
        if let Some(h2) = line.strip_prefix("## ") {
            let detected = detect_section(h2);
            if detected == Section::Roadmap {
                section = Section::Roadmap;
                let h2_low = h2.to_lowercase();
                if h2_low.contains("giai đoạn") || h2_low.contains("giai doan") || h2_low.contains("gđ") || h2_low.contains("phase") {
                    let next_fallback = if out.phase_labels.is_empty() { 1 } else { current_phase + 1 };
                    current_phase = extract_phase_number(h2, next_fallback);
                    current_phase_label = strip_backticks(h2);
                    out.phase_labels
                        .entry(current_phase)
                        .or_insert_with(|| current_phase_label.clone());
                }
            } else {
                section = detected;
            }
            continue;
        }
        if let Some(h3) = line.strip_prefix("### ") {
            if section == Section::Roadmap {
                let next_fallback = if out.phase_labels.is_empty() { 1 } else { current_phase + 1 };
                current_phase = extract_phase_number(h3, next_fallback);
                current_phase_label = strip_backticks(h3);
                out.phase_labels
                    .entry(current_phase)
                    .or_insert_with(|| current_phase_label.clone());
            }
            continue;
        }

        if section == Section::Roadmap && line.starts_with('|') {
            if let Some((week_label, title, checked)) = parse_roadmap_table_row(line) {
                week_counter += 1;
                out.phase_labels
                    .entry(current_phase)
                    .or_insert_with(|| current_phase_label.clone());
                out.roadmap_weeks.push(ParsedRoadmapWeek {
                    phase: current_phase,
                    phase_label: current_phase_label.clone(),
                    week_label,
                    title,
                    checked,
                });
            }
            continue;
        }

        if let Some((item, checked)) = strip_list_marker_with_check(line) {
            if item.is_empty() {
                continue;
            }
            match section {
                Section::DailyQuests => {
                    let q = parse_quest_line(item);
                    if !q.title.is_empty() {
                        out.daily_quests.push(q);
                    }
                }
                Section::WritingChecklist => {
                    let clean = strip_backticks(item);
                    if !clean.is_empty() {
                        out.writing_checklist.push(clean);
                    }
                }
                Section::WritingFocus => {
                    let clean = strip_backticks(item);
                    if !clean.is_empty() {
                        out.writing_focus_notes.push(clean);
                    }
                }
                Section::ErrorLogs => {
                    if let Some(err) = parse_error_line(item) {
                        out.error_logs.push(err);
                    } else {
                        out.warnings.push(format!("Bỏ qua dòng sổ lỗi chưa đúng dạng 'sai -> đúng': {item}"));
                    }
                }
                Section::Vocab => {
                    if let Some(card) = parse_vocab_line(item) {
                        out.vocab_cards.push(card);
                    } else {
                        out.warnings.push(format!("Bỏ qua dòng từ vựng thiếu dấu '|': {item}"));
                    }
                }
                Section::Paraphrase => {
                    if let Some(pair) = parse_paraphrase_line(item) {
                        out.paraphrase_pairs.push(pair);
                    } else {
                        out.warnings.push(format!("Bỏ qua dòng Paraphrase chưa đúng dạng 'từ -> đồng nghĩa | nguồn': {item}"));
                    }
                }
                Section::Roadmap => {
                    week_counter += 1;
                    let clean = strip_backticks(item);
                    let (week_label, title) = if let Some((w, t)) = clean.split_once(':') {
                        (w.trim().to_string(), t.trim().to_string())
                    } else {
                        (format!("Tuần {week_counter}"), clean)
                    };
                    out.phase_labels
                        .entry(current_phase)
                        .or_insert_with(|| current_phase_label.clone());
                    out.roadmap_weeks.push(ParsedRoadmapWeek {
                        phase: current_phase,
                        phase_label: current_phase_label.clone(),
                        week_label,
                        title,
                        checked,
                    });
                }
                Section::None => {}
            }
        }
    }

    let has_any_data = out.exam_date.is_some()
        || !out.skill_bands.is_empty()
        || !out.daily_quests.is_empty()
        || !out.writing_checklist.is_empty()
        || !out.writing_focus_notes.is_empty()
        || !out.error_logs.is_empty()
        || !out.vocab_cards.is_empty()
        || !out.paraphrase_pairs.is_empty()
        || !out.roadmap_weeks.is_empty();

    if !has_any_data {
        return Err("Không nhận diện được cấu trúc lộ trình XD4T-1ELTS. Hãy bấm [XEM MẪU CHUẨN] hoặc [COPY PROMPT TẠO LỘ TRÌNH CHO AI] để tạo đúng định dạng.".to_string());
    }

    if out.daily_quests.is_empty() {
        out.warnings.push("Chưa có mục '## Nhiệm vụ hàng ngày' — sẽ giữ lịch xoay vòng Core/Bonus mặc định.".to_string());
    }
    if out.roadmap_weeks.is_empty() {
        out.warnings.push("Chưa có mục '## Lộ trình theo tuần' — sẽ giữ lộ trình tuần hiện tại.".to_string());
    }
    if out.writing_checklist.is_empty() {
        out.warnings.push("Chưa có mục '## Checklist Writing cá nhân' — sẽ dùng checklist mặc định.".to_string());
    }

    Ok(out)
}

pub fn build_preview(parsed: &ParsedRoadmap) -> RoadmapPreview {
    let phases_summary: Vec<String> = parsed.phase_labels.values().cloned().collect();
    RoadmapPreview {
        exam_date: parsed.exam_date.clone().unwrap_or_else(|| "Giữ nguyên".to_string()),
        overall_current: parsed.overall_current.unwrap_or(5.0),
        overall_target: parsed.overall_target.unwrap_or(6.5),
        buffer_target: parsed.buffer_target.or(parsed.overall_target).unwrap_or(7.0),
        skill_bands: parsed.skill_bands.clone(),
        daily_minutes: parsed.daily_minutes.unwrap_or(30),
        reminder_time: parsed.reminder_time.clone().unwrap_or_else(|| "21:00".to_string()),
        quests_count: parsed.daily_quests.len(),
        checklist_count: parsed.writing_checklist.len(),
        focus_notes_count: parsed.writing_focus_notes.len(),
        error_logs_count: parsed.error_logs.len(),
        vocab_count: parsed.vocab_cards.len(),
        paraphrase_count: parsed.paraphrase_pairs.len(),
        phases_count: parsed.phase_labels.len(),
        weeks_count: parsed.roadmap_weeks.len(),
        phases_summary,
        warnings: parsed.warnings.clone(),
    }
}

pub fn get_roadmap_ai_prompt() -> String {
    r#"Bạn là chuyên gia lập lộ trình tự học IELTS cá nhân hóa cho ứng dụng XD4T-1ELTS v1.6.0.

Hãy phỏng vấn nhanh tôi đúng 5 câu ngắn gọn trước (hỏi gộp trong 1 tin nhắn để tôi trả lời 1 lần):
1. Điểm đầu vào hiện tại của bạn (Overall + Listening / Reading / Speaking / Writing)?
2. Mục tiêu Band bạn muốn đạt (6.5 hay 8.0) và vùng đệm an toàn?
3. Ngày thi dự kiến (YYYY-MM-DD) và mỗi ngày bạn học được khoảng bao nhiêu phút (Core ≤ 30p)?
4. Bạn hay mắc những lỗi Writing/Speaking/Listening nào nhất?
5. Bạn muốn nhắc giờ học lúc mấy giờ mỗi tối (HH:MM)?

Sau khi tôi trả lời 5 câu trên, HÃY XUẤT RA DUY NHẤT 1 BLOCK CODE MARKDOWN (`lo-trinh.md`) đúng 100% khuôn mẫu `XD4T-1ELTS v1.6.0` dưới đây (không giải thích lan man bên ngoài block):

```markdown
---
exam_date: 2027-05-15
overall_current: 5.0
overall_target: 6.5
buffer_target: 7.0
listening: 5.5 -> 7.5
reading: 5.0 -> 7.0
speaking: 5.5 -> 6.5
writing: 4.5 -> 6.0
daily_minutes: 30
reminder_time: 21:00
---

## Nhiệm vụ Core
- [core] [8p] Ôn SRS 8p (Audio-First / Cloze) | Giữ trí nhớ dài hạn
- [core] [2p] Micro-Check 2p (5 câu rà từ/lỗi/paraphrase) | Chống quên hàng ngày
- [core] [20p] Study4 Reading: 1 Passage bấm giờ 16p -> nạp 2 cặp Bảng Paraphrase | Luyện thực chiến
- [bonus] [15p] Nghe chép câu Dictation 5 câu bẫy -s/-ed | Tăng độ tinh tai

## Checklist Writing cá nhân
- Chủ ngữ số ít (-s/-es/has)
- Danh từ đếm được có mạo từ hoặc số nhiều
- Tránh nhầm cặp từ / sai từ loại (effect/affect, harmful)
- Không nối 2 câu bằng dấu phẩy (comma splice)
- Đúng khung dạng đề, kết bài không thêm ý mới

## Điểm yếu Writing cần AI soi kỹ
- Lỗi chia động từ, số ít/số nhiều và mạo từ
- Nhầm từ loại và cặp từ gần nghĩa
- Câu cụt và nối câu bằng dấu phẩy

## Sổ lỗi ban đầu
- [Writing · Ngữ pháp] Everybody know -> Everybody knows
- [Writing · Từ loại] harmfully effects -> harmful effects

## Bảng Paraphrase
- decline -> fall off, dwindle | Cam 18 T2 · T/F/NG | Phân biệt giá vs nhu cầu
- harmful -> detrimental to | Study4 #12 · MCQ | Hay gặp ở câu Not Given

## Sentence Mining
- [Technology] play a pivotal role in | đóng vai trò then chốt trong | Automation plays a pivotal role in modern industry.
- [Environment] mitigate the adverse impact of | giảm thiểu tác động tiêu cực của | Strict laws can mitigate the adverse impact of pollution.

## Lộ trình theo tuần
### GĐ1: Diệt lỗi nền & Khung bài (T1-T6)
- [ ] Tuần 1: Study4 Mini-Mock 15p + 4 cặp Paraphrase | W: rà Top 3 lỗi nhóm 1 | Nói 4-3-2 | Đích: Lỗi ≤ 3/bài
- [ ] Tuần 2: Study4 P1 + Sentence Mining 4 từ | W: T2 Discussion | Dictation -s/-ed | Đích: W ≥ 5.0
### GĐ2: Tăng tốc & Luyện đề (T7-T14)
- [ ] Tuần 7: Study4 Cam 13 + Bảng Paraphrase | W: Task 1 Line | Nói Part 2+3 | Đích: R ≥ 25/40
```

Yêu cầu bắt buộc khi xuất file:
- Giữ nguyên tên các key trong frontmatter (`exam_date`, `overall_current`, `overall_target`, `buffer_target`, `listening`, `reading`, `speaking`, `writing`, `daily_minutes`, `reminder_time`).
- Có thể dùng `- [ ] Tuần X: ...` hoặc bảng `| Tuần X | ... |`.
- Lộ trình theo tuần phải gắn KPI định lượng phù hợp với số ngày còn lại đến ngày thi của tôi."#.to_string()
}

pub fn get_sample_roadmap_md() -> String {
    include_str!("../../docs/LO-TRINH-MAU-V1.6.md").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v160_roadmap_with_paraphrase_table_and_checkbox_weeks() {
        let md = r#"```markdown
---
exam_date: 2027-06-20
overall_current: 5.5
overall_target: 7.0
buffer_target: 7.5
listening: 6.0 → 8.0
reading: 5.5 -> 7.5
speaking: 5.5 -> 7.0
writing: `5.0 -> 6.5`
daily_minutes: 30
reminder_time: 20:30
---

## Nhiệm vụ Core
- [core] [8p] Ôn SRS 8p (Audio-First / Cloze) | Hàng ngày
- [core] [2p] Micro-Check 2p | 5 câu tổng hợp
- [bonus] [15p] Nghe chép câu Dictation 5 câu | Tùy chọn

## Bảng Paraphrase
- decline -> fall off, dwindle | Cam 18 T2 · T/F/NG | Phân biệt giá vs nhu cầu
- children -> youngsters, minors | Cam 17 T1 · Matching

## Sentence Mining
- [Society] bridge the gap | thu hẹp khoảng cách | Education helps bridge the gap between rich and poor.

## Lộ trình theo tuần
### GĐ1: Nền tảng (T1-T2)
- [x] Tuần 1: Ôn ngữ pháp cốt lõi + Study4 Mini-Mock
- [ ] Tuần 2: Khung Task 2 Opinion + Dictation -s/-ed
### GĐ2: Luyện đề (T3-T4)
| Tuần | Trọng tâm | Đích tuần |
|---|---|---|
| Tuần 3 | Cam 17 + Bảng Paraphrase | R ≥ 26/40 |
```"#;

        let parsed = parse_roadmap_markdown(md).unwrap();
        assert_eq!(parsed.exam_date.as_deref(), Some("2027-06-20"));
        assert_eq!(parsed.daily_quests.len(), 3);
        assert_eq!(parsed.daily_quests[0].tier, "core");
        assert_eq!(parsed.daily_quests[2].tier, "bonus");
        assert_eq!(parsed.daily_quests[1].action_hint, "micro_check");
        assert_eq!(parsed.paraphrase_pairs.len(), 2);
        assert_eq!(parsed.paraphrase_pairs[0].question_term, "decline");
        assert_eq!(parsed.paraphrase_pairs[0].trap_note, "Phân biệt giá vs nhu cầu");
        assert_eq!(parsed.vocab_cards.len(), 1);
        assert_eq!(parsed.roadmap_weeks.len(), 3);
        assert!(parsed.roadmap_weeks[0].checked);
        assert!(!parsed.roadmap_weeks[1].checked);
        assert_eq!(parsed.roadmap_weeks[2].week_label, "Tuần 3");
        assert!(parsed.roadmap_weeks[2].title.contains("Cam 17"));
    }

    #[test]
    fn parses_official_sample_v16_md() {
        let md = get_sample_roadmap_md();
        let parsed = parse_roadmap_markdown(&md).unwrap();
        assert!(parsed.roadmap_weeks.len() >= 12);
        assert!(!parsed.paraphrase_pairs.is_empty());
        assert!(!parsed.vocab_cards.is_empty());
        assert!(!parsed.daily_quests.is_empty());
    }

    #[test]
    fn rejects_empty_or_unrecognized_markdown_with_friendly_error() {
        let err1 = parse_roadmap_markdown("   ").unwrap_err();
        assert!(err1.contains("trống"));

        let err2 = parse_roadmap_markdown("# Hello world\nJust some random notes.").unwrap_err();
        assert!(err2.contains("Không nhận diện được"));
    }

    #[test]
    fn normalizes_dd_mm_yyyy_date_and_missing_sections_give_warnings() {
        let md = r#"---
exam_date: 15/05/2027
writing: 4.5 -> 6.0
---
## Nhiệm vụ hàng ngày
- [20p] Viết 1 bài ngắn | Luyện câu
"#;
        let parsed = parse_roadmap_markdown(md).unwrap();
        assert_eq!(parsed.exam_date.as_deref(), Some("2027-05-15"));
        assert!(!parsed.warnings.is_empty());
        let preview = build_preview(&parsed);
        assert_eq!(preview.quests_count, 1);
    }
}
