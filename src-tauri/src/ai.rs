use crate::models::GradeResult;
use serde_json::json;

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

fn round_half(v: f64) -> f64 {
    (v * 2.0).round() / 2.0
}

pub async fn grade_writing_with_gemini(
    api_key: &str,
    preferred_model: &str,
    mode: &str,
    prompt: &str,
    content: &str,
) -> Result<(GradeResult, String), String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err("Chưa có Gemini API Key trong Cài đặt.".to_string());
    }

    let system_instruction = r#"You are a strict IELTS Writing Examiner. Grade the student's response and return ONLY valid JSON matching this exact schema (no markdown fences):
{
  "ta": 5.5,
  "cc": 5.5,
  "lr": 5.0,
  "gra": 4.5,
  "overall": 5.0,
  "summary_vi": "Nhận xét ngắn gọn bằng tiếng Việt (1-2 câu) về điểm mạnh/yếu và thống kê nhanh 3 nhóm lỗi nền nếu có.",
  "corrections": [
    {"wrong": "exact wrong phrase from student text", "right": "corrected Band 6.5 phrase", "category": "Nhóm 1 · Chia động từ/Mạo từ/Số nhiều | Nhóm 2 · Nhầm từ/Word family | Nhóm 3 · Comma splice/Câu cụt", "reason": "Giải thích ngắn bằng tiếng Việt"}
  ],
  "band65_rewrite": "A natural Band 6.5-7.0 rewrite of the student's response."
}
Rules:
1. Scores must be in 0.5 increments (e.g. 4.5, 5.0, 5.5, 6.0, 6.5).
2. Pay special attention to and categorize the student's 3 recurring error groups:
   - Nhóm 1: Subject-verb agreement (everybody knows), countable nouns needing a/an/the or -s (humans/people), articles.
   - Nhóm 2: Confused words (change/chance, choose/choice, effect/affect) and word family (harmful vs harmfully, In addition vs In additionally).
   - Nhóm 3: Comma splices (joining 2 independent clauses with a comma) and sentence fragments.
3. Provide up to 6 most impactful corrections."#;

    let user_msg = format!(
        "Mode: {mode}\nPrompt: {prompt}\n\nStudent Essay:\n{content}"
    );

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

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(45))
        .build()
        .map_err(|e| format!("HTTP client error: {e}"))?;

    let mut candidates: Vec<(String, String)> = Vec::new();
    let base_model = if preferred_model.trim().is_empty() {
        "gemini-3-flash-preview"
    } else {
        preferred_model.trim()
    };

    // Primary: generativelanguage v1beta (verified HTTP 200 with AQ. key)
    let models = [
        base_model,
        "gemini-3-flash-preview",
        "gemini-3.1-flash-lite-preview",
        "gemini-2.5-flash",
    ];
    for m in models {
        let gl_url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{m}:generateContent?key={key}"
        );
        if !candidates.iter().any(|(_, u)| u == &gl_url) {
            candidates.push((format!("generativelanguage/{m}"), gl_url));
        }
    }
    // Secondary fallback for AQ. Vertex AI Express Mode keys if Vertex API is enabled
    if key.starts_with("AQ.") {
        for m in [base_model, "gemini-2.5-flash", "gemini-2.0-flash"] {
            let vp_url = format!(
                "https://aiplatform.googleapis.com/v1/publishers/google/models/{m}:generateContent?key={key}"
            );
            if !candidates.iter().any(|(_, u)| u == &vp_url) {
                candidates.push((format!("aiplatform/{m}"), vp_url));
            }
        }
    }

    let mut last_err = String::new();
    for (label, url) in candidates {
        let resp = match client.post(&url).json(&payload).send().await {
            Ok(r) => r,
            Err(e) => {
                last_err = format!("{label}: network error ({e})");
                continue;
            }
        };

        let status = resp.status();
        if status.is_success() {
            let json_val: serde_json::Value = resp
                .json()
                .await
                .map_err(|e| format!("Invalid response JSON: {e}"))?;
            let text = json_val["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .ok_or_else(|| "AI trả về rỗng (không có candidates text).".to_string())?;
            let grade = parse_grade_response(text)?;
            return Ok((grade, label));
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

    Err(format!("Không thể chấm AI ({last_err}). Hãy dùng nút COPY PROMPT CHẤM AI hoặc kiểm tra API Key."))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn parse_grade_invalid_json_returns_err() {
        let res = parse_grade_response("not json at all");
        assert!(res.is_err());
    }
}
