use crate::models::{ErrorLog, GradeResult, SpeakingAiPayload};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiTestResult {
    pub ok: bool,
    pub status_code: u16,
    pub endpoint: String,
    pub message: String,
}

pub fn scrub_secret(text: &str, secret: &str) -> String {
    let trimmed = secret.trim();
    if trimmed.is_empty() {
        return text.to_string();
    }
    text.replace(trimmed, "[REDACTED]")
}

pub fn build_focus_rules(focus_notes: &[String]) -> String {
    if focus_notes.is_empty() {
        return "   - Nhóm 1: Subject-verb agreement (everybody knows), countable nouns needing a/an/the or -s (humans/people), articles.\n   - Nhóm 2: Confused words (change/chance, choose/choice, effect/affect) and word family (harmful vs harmfully, In addition vs In additionally).\n   - Nhóm 3: Comma splices (joining 2 independent clauses with a comma) and sentence fragments.".to_string();
    }
    focus_notes
        .iter()
        .enumerate()
        .map(|(i, note)| format!("   - Điểm yếu {}: {}", i + 1, note))
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn build_top3_rules(top_errors: &[ErrorLog]) -> String {
    if top_errors.is_empty() {
        return String::new();
    }
    let items: Vec<String> = top_errors
        .iter()
        .take(3)
        .enumerate()
        .map(|(i, e)| {
            format!(
                "   - Top #{}: [{}] \"{}\" -> \"{}\" (đã mắc {} lần)",
                i + 1,
                e.category,
                e.wrong_text,
                e.right_text,
                e.count
            )
        })
        .collect();
    format!("\n4. Student's Top 3 most frequent past errors from Error Log (flag if repeated):\n{}", items.join("\n"))
}

fn tokenize_keywords(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .map(str::trim)
        .filter(|s| s.len() >= 3)
        .filter(|s| !matches!(*s, "the" | "and" | "for" | "with" | "from" | "that" | "this" | "nhóm" | "lỗi" | "writing" | "speaking" | "listening" | "reading"))
        .map(|s| s.to_string())
        .collect()
}

pub fn detect_repeated_top_errors(result: &GradeResult, top_errors: &[ErrorLog]) -> Vec<String> {
    let mut repeated = Vec::new();
    for (idx, top) in top_errors.iter().take(3).enumerate() {
        let top_cat_tokens = tokenize_keywords(&top.category);
        let top_wrong_tokens = tokenize_keywords(&top.wrong_text);
        for c in &result.corrections {
            let c_cat_low = c.category.to_lowercase();
            let c_reason_low = c.reason.to_lowercase();
            let c_wrong_low = c.wrong.to_lowercase();

            let cat_match = top_cat_tokens
                .iter()
                .any(|tok| c_cat_low.contains(tok) || c_reason_low.contains(tok));
            let phrase_match = !top.wrong_text.trim().is_empty()
                && (c_wrong_low.contains(&top.wrong_text.to_lowercase())
                    || top_wrong_tokens.iter().filter(|t| c_wrong_low.contains(t.as_str())).count() >= 2);

            if cat_match || phrase_match {
                let msg = format!(
                    "Lỗi #{} ({}) — “{}” → “{}” (đã mắc {} lần trước đó)",
                    idx + 1,
                    top.category,
                    c.wrong,
                    c.right,
                    top.count
                );
                if !repeated.contains(&msg) {
                    repeated.push(msg);
                }
                break;
            }
        }
    }
    repeated
}

pub fn parse_grade_response(raw_text: &str) -> Result<GradeResult, String> {
    let cleaned = raw_text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let start = cleaned.find('{').unwrap_or(0);
    let end = cleaned.rfind('}').map(|i| i + 1).unwrap_or(cleaned.len());
    let slice = &cleaned[start..end];

    let mut parsed: GradeResult = serde_json::from_str(slice)
        .map_err(|e| format!("Không parse được JSON từ AI: {e}"))?;

    parsed.ta = round_half(parsed.ta.clamp(0.0, 9.0));
    parsed.cc = round_half(parsed.cc.clamp(0.0, 9.0));
    parsed.lr = round_half(parsed.lr.clamp(0.0, 9.0));
    parsed.gra = round_half(parsed.gra.clamp(0.0, 9.0));
    parsed.overall = round_half(((parsed.ta + parsed.cc + parsed.lr + parsed.gra) / 4.0).clamp(0.0, 9.0));
    Ok(parsed)
}

pub fn parse_speaking_grade_response(raw_text: &str) -> Result<SpeakingAiPayload, String> {
    let cleaned = raw_text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let start = cleaned.find('{').unwrap_or(0);
    let end = cleaned.rfind('}').map(|i| i + 1).unwrap_or(cleaned.len());
    let slice = &cleaned[start..end];

    let mut parsed: SpeakingAiPayload = serde_json::from_str(slice)
        .map_err(|e| format!("Không parse được JSON chấm Speaking từ AI: {e}"))?;

    parsed.fc_band = round_half(parsed.fc_band.clamp(0.0, 9.0));
    parsed.lr_band = round_half(parsed.lr_band.clamp(0.0, 9.0));
    parsed.gra_band = round_half(parsed.gra_band.clamp(0.0, 9.0));
    parsed.p_band = round_half(parsed.p_band.clamp(0.0, 9.0));
    parsed.overall_band = round_half(
        ((parsed.fc_band + parsed.lr_band + parsed.gra_band + parsed.p_band) / 4.0).clamp(0.0, 9.0),
    );
    parsed.top_errors.truncate(3);
    Ok(parsed)
}

fn round_half(v: f64) -> f64 {
    (v * 2.0).round() / 2.0
}

fn build_candidate_endpoints(key: &str, preferred_model: &str) -> Vec<(String, String)> {
    let mut candidates: Vec<(String, String)> = Vec::new();
    let base_model = if preferred_model.trim().is_empty() {
        "gemini-3-flash-preview"
    } else {
        preferred_model.trim()
    };

    let models = [
        base_model,
        "gemini-3-flash-preview",
        "gemini-3.1-flash-lite-preview",
        "gemini-2.5-flash",
    ];
    for m in models {
        let gl_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{m}:generateContent"
        );
        if !candidates.iter().any(|(_, u)| u == &gl_url) {
            candidates.push((format!("generativelanguage/{m}"), gl_url));
        }
    }
    if key.starts_with("AQ.") {
        for m in [base_model, "gemini-2.5-flash", "gemini-2.0-flash"] {
            let vp_url = format!(
                "https://aiplatform.googleapis.com/v1/publishers/google/models/{m}:generateContent"
            );
            if !candidates.iter().any(|(_, u)| u == &vp_url) {
                candidates.push((format!("aiplatform/{m}"), vp_url));
            }
        }
    }
    candidates
}

async fn post_gemini_json(
    key: &str,
    preferred_model: &str,
    payload: &serde_json::Value,
    timeout_secs: u64,
    fallback_hint: &str,
) -> Result<(String, String), String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(timeout_secs))
        .build()
        .map_err(|e| scrub_secret(&format!("HTTP client error: {e}"), key))?;

    let candidates = build_candidate_endpoints(key, preferred_model);
    let mut last_err = String::new();

    for (label, url) in candidates {
        let resp = match client
            .post(&url)
            .header("x-goog-api-key", key)
            .json(payload)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let safe_err = scrub_secret(&e.to_string(), key);
                last_err = format!("{label}: network error ({safe_err})");
                continue;
            }
        };

        let status = resp.status();
        if status.is_success() {
            let json_val: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| scrub_secret(&format!("Invalid response JSON: {e}"), key))?;
            let text = json_val["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .ok_or_else(|| "AI trả về rỗng (không có candidates text).".to_string())?;
            return Ok((text.to_string(), label));
        } else {
            let code = status.as_u16();
            last_err = format!("{label}: HTTP {code}");
            if code == 401 || code == 403 || code == 404 || code == 503 || code == 429 {
                continue;
            } else {
                break;
            }
        }
    }

    Err(scrub_secret(&format!("{fallback_hint} ({last_err})"), key))
}

pub async fn grade_writing_with_gemini(
    api_key: &str,
    preferred_model: &str,
    mode: &str,
    prompt: &str,
    content: &str,
    focus_notes: &[String],
    top_errors: &[ErrorLog],
    target_band: f64,
) -> Result<(GradeResult, String), String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err("Chưa có Gemini API Key trong Cài đặt.".to_string());
    }

    let focus_rules = build_focus_rules(focus_notes);
    let top3_rules = build_top3_rules(top_errors);
    let is_80 = target_band >= 7.5;
    let (examiner_role, right_desc, rewrite_desc, score_examples) = if is_80 {
        (
            "You are a strict IELTS Writing Band 8.0 Examiner.",
            "corrected Band 7.5-8.0 phrase",
            "A natural Band 7.5-8.0+ rewrite demonstrating C1-C2 collocations, academic hedging, and error-free complex grammar.",
            "6.0, 6.5, 7.0, 7.5, 8.0",
        )
    } else {
        (
            "You are a strict IELTS Writing Examiner.",
            "corrected Band 6.5 phrase",
            "A natural Band 6.5-7.0 rewrite of the student's response.",
            "4.5, 5.0, 5.5, 6.0, 6.5",
        )
    };
    let system_instruction = format!(
        r#"{examiner_role} Grade the student's response and return ONLY valid JSON matching this exact schema (no markdown fences):
{{
  "ta": 5.5,
  "cc": 5.5,
  "lr": 5.0,
  "gra": 4.5,
  "overall": 5.0,
  "summary_vi": "Nhận xét ngắn gọn bằng tiếng Việt (1-2 câu) về điểm mạnh/yếu và thống kê nhanh các lỗi nền nếu có.",
  "corrections": [
    {{"wrong": "exact wrong phrase from student text", "right": "{right_desc}", "category": "Tên nhóm lỗi ngắn gọn", "reason": "Giải thích ngắn bằng tiếng Việt"}}
  ],
  "band65_rewrite": "{rewrite_desc}"
}}
Rules:
1. Scores must be in 0.5 increments (e.g. {score_examples}).
2. Pay special attention to the student's recurring error areas:
{focus_rules}
3. Provide up to 6 most impactful corrections.{top3_rules}"#
    );

    let user_msg = format!("Mode: {mode}\nPrompt: {prompt}\n\nStudent Essay:\n{content}");

    let payload = json!({
        "contents": [{
            "role": "user",
            "parts": [{ "text": format!("{system_instruction}\n\n{user_msg}") }]
        }],
        "generationConfig": {
            "temperature": 0.2,
            "responseMimeType": "application/json"
        }
    });

    let (text, label) = post_gemini_json(
        key,
        preferred_model,
        &payload,
        45,
        "Không thể chấm AI. Hãy dùng nút COPY PROMPT CHẤM AI hoặc kiểm tra API Key",
    )
    .await?;

    let mut grade = parse_grade_response(&text)?;
    grade.repeated_top_errors = detect_repeated_top_errors(&grade, top_errors);
    Ok((grade, label))
}

pub(crate) fn validate_audio_mime_type(mime_type: &str) -> Result<&str, String> {
    let trimmed = mime_type.trim();
    if trimmed.is_empty() {
        return Ok("audio/webm");
    }
    let base = trimmed
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    match base.as_str() {
        "audio/webm" | "audio/ogg" | "audio/mpeg" | "audio/mp4" | "audio/wav" => Ok(trimmed),
        _ => Err(format!(
            "Định dạng âm thanh không được hỗ trợ: {}. Chỉ chấp nhận audio/webm, audio/ogg, audio/mpeg, audio/mp4, audio/wav.",
            trimmed
        )),
    }
}

pub async fn grade_speaking_audio_with_gemini(
    api_key: &str,
    preferred_model: &str,
    cue_title: &str,
    part3_questions: &[String],
    mode: &str,
    track: &str,
    audio_b64_list: &[String],
    mime_type: &str,
) -> Result<(SpeakingAiPayload, String), String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err("Chưa có Gemini API Key trong Cài đặt.".to_string());
    }
    if audio_b64_list.is_empty() {
        return Err("Chưa có file ghi âm nào để gửi chấm AI.".to_string());
    }
    if audio_b64_list.len() > 2 {
        return Err("Tối đa chỉ gửi 2 đoạn ghi âm (<= 1.5MB mỗi đoạn) để tối ưu tốc độ.".to_string());
    }
    for (idx, b64) in audio_b64_list.iter().enumerate() {
        // Base64 of 1.5MB binary is ~2,097,152 chars
        if b64.len() > 2_100_000 {
            return Err(format!(
                "Đoạn ghi âm #{} vượt quá giới hạn 1.5MB. Hãy ghi âm ở định dạng nén hoặc dùng nút Copy Prompt.",
                idx + 1
            ));
        }
    }

    let safe_mime = validate_audio_mime_type(mime_type)?;
    let p3_text = part3_questions
        .iter()
        .enumerate()
        .map(|(i, q)| format!("{}. {}", i + 1, q))
        .collect::<Vec<_>>()
        .join("\n");

    let target_note = if track.trim() == "8.0" {
        "Band 8.0 target: assess natural C1-C2 collocations, abstract Part 3 reasoning, minimal hesitation, and grammatical accuracy."
    } else {
        "Band 6.5 target: assess fluency, clear A.R.E.A structure, natural linking, and common -s/-ed/tense accuracy."
    };

    let prompt_text = format!(
        r#"You are a certified IELTS Speaking Examiner. Listen carefully to the student's audio recording(s) (if 2 clips are attached in 4-3-2 or Part 2+3 mode, clip 1 is start/Part 2 and clip 2 is final/Part 3).
{target_note}
Mode: {mode} | Track: {track}
Cue Card: {cue_title}
Part 3 Questions:
{p3_text}

Return ONLY valid JSON (no markdown fences) matching this schema:
{{
  "transcript": "Accurate verbatim transcript of what the student said in the audio",
  "fc_band": 6.0,
  "lr_band": 6.0,
  "gra_band": 5.5,
  "p_band": 6.0,
  "overall_band": 6.0,
  "wpm_start": 112,
  "wpm_end": 135,
  "fillers_start": 12,
  "fillers_end": 5,
  "progress_note": "Nhận xét ngắn bằng tiếng Việt về độ trôi chảy, phát âm ước lượng và tiến bộ giữa các chặng",
  "top_errors": [
    {{"wrong": "exact phrase spoken incorrectly", "fix": "natural corrected phrase", "category": "Ngữ pháp / Từ vựng / Phát âm"}}
  ]
}}
Rules:
1. Band scores must be 0.5 increments. Note that p_band is estimated from audio.
2. Provide up to 3 most impactful items in top_errors so the student can log them into Sổ lỗi."#
    );

    let mut parts = vec![json!({ "text": prompt_text })];
    for b64 in audio_b64_list {
        parts.push(json!({
            "inline_data": {
                "mime_type": safe_mime,
                "data": b64
            }
        }));
    }

    let payload = json!({
        "contents": [{
            "role": "user",
            "parts": parts
        }],
        "generationConfig": {
            "temperature": 0.2,
            "responseMimeType": "application/json"
        }
    });

    let (text, label) = post_gemini_json(
        key,
        preferred_model,
        &payload,
        60,
        "Không thể chấm Speaking Audio AI. Hãy dùng nút Copy Prompt + Rubric hoặc Tự chấm 5 mục",
    )
    .await?;

    let parsed = parse_speaking_grade_response(&text)?;
    Ok((parsed, label))
}

pub async fn test_gemini_connection_live(
    api_key: &str,
    preferred_model: &str,
) -> Result<ApiTestResult, String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err("Chưa có Gemini API Key để kiểm tra. Hãy dán key hoặc nạp từ file trước.".to_string());
    }

    let payload = json!({
        "contents": [{
            "role": "user",
            "parts": [{ "text": "Reply with exactly: OK" }]
        }],
        "generationConfig": {
            "temperature": 0.0,
            "maxOutputTokens": 8
        }
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| scrub_secret(&format!("HTTP client error: {e}"), key))?;

    let candidates = build_candidate_endpoints(key, preferred_model);
    let mut last_status = 0_u16;
    let mut last_err = String::new();

    for (label, url) in candidates {
        let resp = match client
            .post(&url)
            .header("x-goog-api-key", key)
            .json(&payload)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let safe_err = scrub_secret(&e.to_string(), key);
                last_err = format!("{label}: lỗi mạng ({safe_err})");
                continue;
            }
        };

        let code = resp.status().as_u16();
        last_status = code;
        if resp.status().is_success() {
            return Ok(ApiTestResult {
                ok: true,
                status_code: code,
                endpoint: label.clone(),
                message: format!("✓ Kết nối thành công · HTTP {code} · {label}"),
            });
        } else {
            last_err = format!("{label}: HTTP {code}");
            if code == 401 || code == 403 || code == 404 || code == 503 || code == 429 {
                continue;
            } else {
                break;
            }
        }
    }

    Ok(ApiTestResult {
        ok: false,
        status_code: last_status,
        endpoint: String::new(),
        message: scrub_secret(
            &format!("✕ Kết nối thất bại ({last_err}). Kiểm tra lại key hoặc mạng."),
            key,
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Correction;

    #[test]
    fn parse_grade_json_with_fences() {
        let raw = r#"```json
        {
          "ta": 5.5,
          "cc": 5.5,
          "lr": 5.0,
          "gra": 4.5,
          "overall": 5.0,
          "summary_vi": "Ý rõ nhưng còn lỗi chia động từ.",
          "corrections": [
            {"wrong": "it make", "right": "it makes", "category": "Chia động từ", "reason": "Chủ ngữ số ít"}
          ],
          "band65_rewrite": "One primary advantage of automation is increased efficiency."
        }
        ```"#;
        let res = parse_grade_response(raw).unwrap();
        assert_eq!(res.ta, 5.5);
        assert_eq!(res.gra, 4.5);
        assert_eq!(res.overall, 5.0);
        assert_eq!(res.corrections.len(), 1);
        assert_eq!(res.corrections[0].right, "it makes");
    }

    #[test]
    fn detect_repeated_top_errors_matches_category_and_phrase() {
        let top = vec![ErrorLog {
            id: 1,
            category: "Writing · Mạo từ a/an".to_string(),
            wrong_text: "an new adult".to_string(),
            right_text: "a new adult".to_string(),
            count: 5,
            created_at: "2026-11-25".to_string(),
        }];
        let grade = GradeResult {
            ta: 6.0,
            cc: 6.0,
            lr: 6.0,
            gra: 5.5,
            overall: 6.0,
            corrections: vec![Correction {
                wrong: "important part".to_string(),
                right: "an important part".to_string(),
                category: "Mạo từ".to_string(),
                reason: "Thiếu mạo từ trước danh từ đếm được số ít".to_string(),
            }],
            band65_rewrite: String::new(),
            summary_vi: String::new(),
            repeated_top_errors: Vec::new(),
        };
        let repeated = detect_repeated_top_errors(&grade, &top);
        assert_eq!(repeated.len(), 1);
        assert!(repeated[0].contains("Lỗi #1"));
    }

    #[test]
    fn parse_speaking_grade_json_works() {
        let raw = r#"{"transcript":"I like relaxing by the lake.","fc_band":6.2,"lr_band":6.0,"gra_band":5.5,"p_band":6.0,"overall_band":6.0,"wpm_start":110,"wpm_end":132,"fillers_start":10,"fillers_end":4,"progress_note":"Trôi chảy hơn rõ ở chặng 3","top_errors":[{"wrong":"it make me","fix":"it makes me","category":"Chia động từ"}]}"#;
        let parsed = parse_speaking_grade_response(raw).unwrap();
        assert_eq!(parsed.fc_band, 6.0);
        assert_eq!(parsed.wpm_end, 132);
        assert_eq!(parsed.top_errors.len(), 1);
    }

    #[test]
    fn parse_grade_invalid_json_returns_err() {
        let res = parse_grade_response("not json at all");
        assert!(res.is_err());
    }

    #[test]
    fn scrub_secret_removes_api_key_from_error_strings() {
        let secret = "AIzaSyDummySecretKey123456";
        let raw_err = format!("error sending request for url (https://example.com?key={secret})");
        let safe = scrub_secret(&raw_err, secret);
        assert!(!safe.contains(secret));
        assert!(safe.contains("[REDACTED]"));
    }

    #[test]
    fn focus_rules_formats_custom_notes() {
        let notes = vec!["Lỗi mạo từ".to_string(), "Comma splice".to_string()];
        let rules = build_focus_rules(&notes);
        assert!(rules.contains("Điểm yếu 1: Lỗi mạo từ"));
        assert!(rules.contains("Điểm yếu 2: Comma splice"));
    }

    #[test]
    fn validate_audio_mime_type_enforces_whitelist() {
        assert_eq!(validate_audio_mime_type("").unwrap(), "audio/webm");
        assert_eq!(
            validate_audio_mime_type("audio/webm;codecs=opus").unwrap(),
            "audio/webm;codecs=opus"
        );
        assert_eq!(validate_audio_mime_type(" AUDIO/OGG ").unwrap(), "AUDIO/OGG");
        assert_eq!(validate_audio_mime_type("audio/mpeg").unwrap(), "audio/mpeg");
        assert_eq!(validate_audio_mime_type("audio/mp4").unwrap(), "audio/mp4");
        assert_eq!(validate_audio_mime_type("audio/wav").unwrap(), "audio/wav");

        assert!(validate_audio_mime_type("text/plain").is_err());
        assert!(validate_audio_mime_type("image/png").is_err());
        assert!(validate_audio_mime_type("audio/x-evil").is_err());
        assert!(validate_audio_mime_type("application/octet-stream").is_err());
    }
}
