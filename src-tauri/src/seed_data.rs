use crate::models::WritingPrompt;

pub struct SeedVocab {
    pub topic: &'static str,
    pub word: &'static str,
    pub meaning: &'static str,
    pub example: &'static str,
}

// 40 cards: 9 Reading 03/11 + 5 Confused/Word-family pairs + 26 Band 6.5-7.0 Collocations
pub const VOCAB_SEEDS: &[SeedVocab] = &[
    // 9 từ buổi Reading 03/11
    SeedVocab { topic: "Reading 03/11", word: "inferior", meaning: "kém hơn, thấp kém hơn (về chất lượng)", example: "Online lectures are not necessarily inferior to in-person classes." },
    SeedVocab { topic: "Reading 03/11", word: "closure", meaning: "sự đóng cửa, ngừng hoạt động", example: "The sudden closure of rural schools forced students to commute farther." },
    SeedVocab { topic: "Reading 03/11", word: "campus", meaning: "khuôn viên trường đại học", example: "Living on campus helps freshmen build stronger social networks." },
    SeedVocab { topic: "Reading 03/11", word: "attitude", meaning: "thái độ, quan điểm", example: "Public attitudes toward remote work have shifted dramatically since 2020." },
    SeedVocab { topic: "Reading 03/11", word: "unengaging", meaning: "nhàm chán, không thu hút", example: "Students often lose focus when reading unengaging textbook passages." },
    SeedVocab { topic: "Reading 03/11", word: "merely", meaning: "chỉ là, đơn thuần là (= only)", example: "Technology is merely a tool; teachers still guide critical thinking." },
    SeedVocab { topic: "Reading 03/11", word: "hybrid", meaning: "kết hợp (trực tiếp + trực tuyến)", example: "Many universities now adopt a hybrid learning model." },
    SeedVocab { topic: "Reading 03/11", word: "permanent", meaning: "vĩnh viễn, lâu dài (trái với temporary)", example: "Air pollution can cause permanent damage to children's lungs." },
    SeedVocab { topic: "Reading 03/11", word: "debated", meaning: "được tranh luận, bàn cãi", example: "Whether AI will replace teachers remains a hotly debated issue." },

    // 5 cặp từ dễ nhầm & word family (Nhóm lỗi 2)
    SeedVocab { topic: "Word Family / Nhầm từ", word: "change (thay đổi) vs chance (cơ hội)", meaning: "change = sự thay đổi | chance = cơ hội (a good chance to develop)", example: "Studying abroad gives young people a good chance to develop independence." },
    SeedVocab { topic: "Word Family / Nhầm từ", word: "choose (v) vs choice (n)", meaning: "choose = lựa chọn (động từ) | choice = sự lựa chọn (danh từ)", example: "Students should have the freedom to choose their own career path." },
    SeedVocab { topic: "Word Family / Nhầm từ", word: "effect (n) vs affect (v)", meaning: "have an effect on (n) | affect something (v: tác động đến)", example: "Excessive screen time can adversely affect teenagers' sleep quality." },
    SeedVocab { topic: "Word Family / Nhầm từ", word: "harm (n/v) / harmful (adj) / harmfully (adv)", meaning: "Dùng tính từ trước danh từ: harmful effects (KHÔNG dùng harmfully effects)", example: "Plastic waste has harmful effects on marine ecosystems." },
    SeedVocab { topic: "Word Family / Nhầm từ", word: "talent (n) / talented (adj)", meaning: "a talented artist (KHÔNG dùng an talent artist)", example: "A talented artist can inspire millions through visual storytelling." },

    // Technology (5)
    SeedVocab { topic: "Technology", word: "play a pivotal role in", meaning: "đóng vai trò then chốt trong...", example: "Automation plays a pivotal role in enhancing industrial efficiency." },
    SeedVocab { topic: "Technology", word: "bridge the digital divide", meaning: "thu hẹp khoảng cách số", example: "Affordable internet access helps bridge the digital divide in rural areas." },
    SeedVocab { topic: "Technology", word: "render manual labor obsolete", meaning: "làm cho lao động chân tay trở nên lỗi thời", example: "Advanced robotics may render certain forms of manual labor obsolete." },
    SeedVocab { topic: "Technology", word: "pose a threat to privacy", meaning: "gây ra mối đe dọa đối với quyền riêng tư", example: "Unregulated data collection can pose a serious threat to personal privacy." },
    SeedVocab { topic: "Technology", word: "foster technological innovation", meaning: "thúc đẩy đổi mới công nghệ", example: "Public funding for research helps foster technological innovation." },

    // Education (5)
    SeedVocab { topic: "Education", word: "bridge the gap between", meaning: "thu hẹp khoảng cách giữa...", example: "Internships bridge the gap between academic theory and workplace skills." },
    SeedVocab { topic: "Education", word: "acquire practical skills", meaning: "tiếp thu các kỹ năng thực tế", example: "Vocational courses help students acquire practical skills for employment." },
    SeedVocab { topic: "Education", word: "cultivate critical thinking", meaning: "rèn luyện tư duy phản biện", example: "Open classroom discussions cultivate critical thinking among learners." },
    SeedVocab { topic: "Education", word: "keep pace with", meaning: "theo kịp với", example: "Curricula must be updated regularly to keep pace with industry demands." },
    SeedVocab { topic: "Education", word: "alleviate academic pressure", meaning: "giảm bớt áp lực học tập", example: "Continuous assessment can alleviate academic pressure before final exams." },

    // Work (4)
    SeedVocab { topic: "Work", word: "alleviate the burden of", meaning: "giảm bớt gánh nặng của...", example: "Smart sensors help alleviate the burden of manual inspection." },
    SeedVocab { topic: "Work", word: "strike a work-life balance", meaning: "cân bằng giữa công việc và cuộc sống", example: "Flexible hours help employees strike a healthier work-life balance." },
    SeedVocab { topic: "Work", word: "boost workplace productivity", meaning: "nâng cao năng suất nơi làm việc", example: "Clear task management systems can boost workplace productivity." },
    SeedVocab { topic: "Work", word: "gain a competitive edge", meaning: "giành được lợi thế cạnh tranh", example: "Bilingual graduates often gain a competitive edge in the job market." },

    // Environment (4)
    SeedVocab { topic: "Environment", word: "mitigate the impact of", meaning: "giảm thiểu tác động của...", example: "Reforestation helps mitigate the impact of extreme weather events." },
    SeedVocab { topic: "Environment", word: "curb carbon emissions", meaning: "kiềm chế lượng khí thải carbon", example: "Stricter transport laws are essential to curb carbon emissions." },
    SeedVocab { topic: "Environment", word: "transition to renewable energy", meaning: "chuyển đổi sang năng lượng tái tạo", example: "Governments should subsidize solar power to transition to renewable energy." },
    SeedVocab { topic: "Environment", word: "deplete natural resources", meaning: "làm cạn kiệt tài nguyên thiên nhiên", example: "Overconsumption continues to deplete finite natural resources." },

    // Health (4)
    SeedVocab { topic: "Health", word: "adopt a sedentary lifestyle", meaning: "sống lối sống ít vận động", example: "Many office workers adopt a sedentary lifestyle due to long desk hours." },
    SeedVocab { topic: "Health", word: "prevent chronic illnesses", meaning: "ngăn ngừa các bệnh mãn tính", example: "Regular aerobic exercise helps prevent chronic illnesses later in life." },
    SeedVocab { topic: "Health", word: "take a toll on mental health", meaning: "gây ảnh hưởng xấu đến sức khỏe tinh thần", example: "Constant social media comparison can take a toll on mental health." },
    SeedVocab { topic: "Health", word: "strain public healthcare systems", meaning: "gây áp lực lên hệ thống y tế công", example: "An aging population can strain public healthcare systems." },

    // Society (4)
    SeedVocab { topic: "Society", word: "widen the wealth gap", meaning: "nới rộng khoảng cách giàu nghèo", example: "Unequal access to quality education can widen the wealth gap." },
    SeedVocab { topic: "Society", word: "foster social cohesion", meaning: "thúc đẩy sự gắn kết xã hội", example: "Community events foster social cohesion in diverse neighborhoods." },
    SeedVocab { topic: "Society", word: "allocate public funds to", meaning: "phân bổ ngân sách công cho...", example: "Authorities should allocate public funds to affordable housing." },
    SeedVocab { topic: "Society", word: "impose stricter regulations on", meaning: "áp đặt quy định nghiêm ngặt hơn lên...", example: "Cities should impose stricter regulations on single-use plastics." },
];

pub fn default_writing_prompts() -> Vec<WritingPrompt> {
    vec![
        // 4 Paragraph 90 words
        WritingPrompt {
            id: "p90-1".into(),
            mode: "paragraph_90".into(),
            title: "AI & Tự động hóa (Body 90 từ)".into(),
            prompt: "Automation and AI are replacing many manual jobs. Write ONE body paragraph (80–100 words) explaining one major benefit of this trend.".into(),
            outline_hint: "KHUNG 4 CÂU (Topic -> Explain -> Example -> Result):\n1. Topic: One primary advantage of automation is higher productivity.\n2. Explain: Machines perform repetitive tasks continuously without fatigue.\n3. Example: In modern factories, robotic arms assemble components faster.\n4. Result: Workers can shift to higher-value creative or supervisory roles.".into(),
            target_words: 90,
        },
        WritingPrompt {
            id: "p90-2".into(),
            mode: "paragraph_90".into(),
            title: "Học trực tuyến (Body 90 từ)".into(),
            prompt: "Online courses are becoming increasingly popular. Write ONE body paragraph (80–100 words) explaining one key advantage for working adults.".into(),
            outline_hint: "KHUNG 4 CÂU (Topic -> Explain -> Example -> Result):\n1. Topic: Online learning offers unmatched flexibility for full-time workers.\n2. Explain: Learners can study after shifts without commuting.\n3. Example: Recorded lectures allow employees to review complex topics at their own pace.\n4. Result: More adults can upgrade their skills and gain a competitive edge.".into(),
            target_words: 90,
        },
        WritingPrompt {
            id: "p90-3".into(),
            mode: "paragraph_90".into(),
            title: "Giao thông công cộng (Body 90 từ)".into(),
            prompt: "Traffic congestion is worsening in major cities. Write ONE body paragraph (80–100 words) proposing one effective solution.".into(),
            outline_hint: "KHUNG 4 CÂU:\n1. Topic: Investing in reliable public transit is a practical way to ease congestion.\n2. Explain: Fast metro and bus networks persuade commuters to leave private cars at home.\n3. Example: Dedicated bus lanes ensure predictable travel times during rush hour.\n4. Result: Urban roads become less crowded and carbon emissions drop.".into(),
            target_words: 90,
        },
        WritingPrompt {
            id: "p90-4".into(),
            mode: "paragraph_90".into(),
            title: "Làm việc từ xa (Body 90 từ)".into(),
            prompt: "Remote work has become common in many industries. Write ONE body paragraph (80–100 words) discussing one drawback of working from home.".into(),
            outline_hint: "KHUNG 4 CÂU:\n1. Topic: A major drawback of remote work is professional isolation.\n2. Explain: Without face-to-face contact, employees struggle to build team trust.\n3. Example: New hires often hesitate to ask quick questions over chat.\n4. Result: Collaboration slows down and job satisfaction can decline.".into(),
            target_words: 90,
        },
        // 4 Task 1
        WritingPrompt {
            id: "t1-1".into(),
            mode: "task1".into(),
            title: "Task 1 · Line Graph (Internet Usage)".into(),
            prompt: "The line graph below shows the percentage of households with high-speed internet in three countries (Vietnam, Thailand, Malaysia) between 2015 and 2025. Summarize the information by selecting and reporting the main features, and make comparisons where relevant. (150+ words)".into(),
            outline_hint: "KHUNG TASK 1 OVERVIEW:\n• Intro: Paraphrase đề + timeframe (2015–2025).\n• Overview: Nêu 2 ý lớn (xu hướng chung tăng/giảm + đối tượng cao nhất/thay đổi mạnh nhất). KHÔNG đưa số liệu vào Overview.\n• Body 1: Số liệu đầu kỳ (2015) & so sánh giữa kỳ.\n• Body 2: Số liệu cuối kỳ (2025) & khoảng cách thu hẹp.".into(),
            target_words: 150,
        },
        WritingPrompt {
            id: "t1-2".into(),
            mode: "task1".into(),
            title: "Task 1 · Bar Chart (Energy Sources)".into(),
            prompt: "The bar chart compares electricity generation by source (coal, hydro, solar, wind) in a Southeast Asian country in 2010 and 2025. Summarize the main features and make comparisons. (150+ words)".into(),
            outline_hint: "KHUNG TASK 1 BAR CHART:\n• Intro: The bar chart compares...\n• Overview: Fossil fuels declined slightly while solar and wind rose sharply.\n• Body 1: Coal and hydro figures & comparison.\n• Body 2: Renewable surge (solar & wind) and remaining gap.".into(),
            target_words: 150,
        },
        WritingPrompt {
            id: "t1-3".into(),
            mode: "task1".into(),
            title: "Task 1 · Pie Charts (Household Spending)".into(),
            prompt: "The two pie charts illustrate average monthly household expenditure across five categories (housing, food, transport, education, leisure) in 2005 and 2025. Summarize the main features. (150+ words)".into(),
            outline_hint: "KHUNG TASK 1 PIE CHART:\n• Intro: Paraphrase the two pie charts.\n• Overview: Housing remained the largest share; education grew while food fell proportionally.\n• Body 1: Largest categories (housing, food).\n• Body 2: Smaller shifting categories (education, transport, leisure).".into(),
            target_words: 150,
        },
        WritingPrompt {
            id: "t1-4".into(),
            mode: "task1".into(),
            title: "Task 1 · Process (Plastic Recycling)".into(),
            prompt: "The diagram illustrates the stages involved in recycling plastic bottles into new consumer products. Summarize the process by describing the main stages. (150+ words)".into(),
            outline_hint: "KHUNG TASK 1 PROCESS:\n• Intro: The diagram depicts the process of...\n• Overview: Nêu tổng số bước + điểm bắt đầu & kết thúc.\n• Body 1: Collection, sorting, compressing, crushing into flakes.\n• Body 2: Washing, melting into pellets, molding into new items.".into(),
            target_words: 150,
        },
        // 4 Task 2 (covering Opinion, Discussion+Opinion, Adv-Disadv, Problem-Solution)
        WritingPrompt {
            id: "t2-1".into(),
            mode: "task2".into(),
            title: "Task 2 · Opinion (Tuần 1 · Study What They Love)".into(),
            prompt: "Some people think that university students should only study subjects that will be useful in the future, such as science and technology, while others believe they should study whatever they like. To what extent do you agree or disagree? (250+ words)".into(),
            outline_hint: "KHUNG 5 DẠNG TASK 2 (CHUẨN SIMON):\n1. Opinion (Agree/Disagree): Intro (Paraphrase + Thesis rõ lập trường) -> Body 1 (Lý do 1) -> Body 2 (Lý do 2) -> Conclusion (Nhắc lại lập trường, KHÔNG thêm ý mới).\n2. Discussion + Opinion: Intro -> Body 1 (Quan điểm A) -> Body 2 (Quan điểm B + nghiêng về B) -> Conclusion.\n3. Adv–Disadv: Intro -> Body 1 (Mặt yếu hơn) -> Body 2 (Mặt trội hơn) -> Conclusion.\n4. Problem–Solution: Intro -> Body 1 (Nguyên nhân/Vấn đề) -> Body 2 (Giải pháp) -> Conclusion.\n5. Two-part: Intro -> Body 1 (Trả lời câu hỏi 1) -> Body 2 (Trả lời câu hỏi 2) -> Conclusion.".into(),
            target_words: 250,
        },
        WritingPrompt {
            id: "t2-2".into(),
            mode: "task2".into(),
            title: "Task 2 · Discussion + Opinion (University vs Vocational)".into(),
            prompt: "Some people think school leavers should go to university, while others believe learning a practical trade is better. Discuss both views and give your own opinion. (250+ words)".into(),
            outline_hint: "KHUNG DISCUSSION + OPINION:\n• Intro: Both academic degrees and vocational training have merit; however, I believe...\n• Body 1 (View A): Benefits of university (theoretical depth, research/management paths).\n• Body 2 (View B + Opinion): Benefits of vocational trades (practical skills, faster employment).\n• Conclusion: Restate balanced view + personal stance (no new ideas).".into(),
            target_words: 250,
        },
        WritingPrompt {
            id: "t2-3".into(),
            mode: "task2".into(),
            title: "Task 2 · Problem/Solution (Urban Pollution)".into(),
            prompt: "Air pollution in large cities is becoming a serious threat to public health. What are the main causes of this problem, and what measures can be taken to tackle it? (250+ words)".into(),
            outline_hint: "KHUNG PROBLEM–SOLUTION:\n• Intro: Acknowledge urban air pollution + state essay examines causes and solutions.\n• Body 1 (Causes): Private vehicles + industrial emissions.\n• Body 2 (Solutions): Subsidize public transit/EVs + enforce factory emission standards.\n• Conclusion: Combined government action and public adoption can curb emissions.".into(),
            target_words: 250,
        },
        WritingPrompt {
            id: "t2-4".into(),
            mode: "task2".into(),
            title: "Task 2 · Adv/Disadv (Remote Work)".into(),
            prompt: "More companies are allowing employees to work from home several days a week. Do the advantages of this trend outweigh the disadvantages? (250+ words)".into(),
            outline_hint: "KHUNG ADVANTAGES–DISADVANTAGES:\n• Intro: Hybrid/remote work is widespread; advantages outweigh drawbacks.\n• Body 1 (Disadvantages): Isolation, blurred work-life boundaries.\n• Body 2 (Advantages): Zero commute, higher focus, better work-life balance.\n• Conclusion: Net positive when managed with clear communication.".into(),
            target_words: 250,
        },
    ]
}

pub struct SeedRoadmap {
    pub phase: i64,
    pub week_label: &'static str,
    pub title: &'static str,
}

// Lộ trình 24 tuần V2 (4 Giai đoạn + 3 Checkpoint Mốc 1/2/3)
pub const ROADMAP_SEEDS: &[SeedRoadmap] = &[
    // Giai đoạn 1: Tuần 1-7 (25/11/2026 -> 12/01/2027)
    SeedRoadmap { phase: 1, week_label: "Tuần 1 (25/11)", title: "Lỗi nhóm 1 (số ít/nhiều, chia động từ, mạo từ) + Task 2 Opinion" },
    SeedRoadmap { phase: 1, week_label: "Tuần 2 (02/12)", title: "Lỗi nhóm 1 + Task 2 Discussion + Opinion (viết lại đề test 02/11)" },
    SeedRoadmap { phase: 1, week_label: "Tuần 3 (09/12)", title: "Lỗi nhóm 2 (change/chance, effect/affect, word family) + Task 2 Adv–Disadv" },
    SeedRoadmap { phase: 1, week_label: "Tuần 4 (16/12)", title: "Lỗi nhóm 2 + Task 2 Problem/Cause–Solution + Reading Linearthinking" },
    SeedRoadmap { phase: 1, week_label: "Tuần 5 (23/12)", title: "Lỗi nhóm 3 (câu cụt, comma splice) + Task 2 Two-part question" },
    SeedRoadmap { phase: 1, week_label: "Tuần 6 (30/12)", title: "Task 1 Line Graph (Overview + xu hướng) + viết lại Task 2 yếu nhất" },
    SeedRoadmap { phase: 1, week_label: "Tuần 7 (06/01)", title: "Task 1 Bar/Table/Pie + ✅ CHECKPOINT MỐC 1 (Cam 12 T1: L≥6.0, R≥6.0, W≥5.5)" },
    // Giai đoạn 2: Tuần 8-15 (13/01/2027 -> 09/03/2027)
    SeedRoadmap { phase: 2, week_label: "Tuần 8 (13/01)", title: "Mệnh đề quan hệ & rút gọn + Cam 13 Listening/Reading" },
    SeedRoadmap { phase: 2, week_label: "Tuần 9 (20/01)", title: "Thực hành MĐQH viết lại bài cũ + Speaking Part 2 (2 phút)" },
    SeedRoadmap { phase: 2, week_label: "Tuần 10 (27/01)", title: "Câu điều kiện loại 1 & 2 + Reading Passage 2–3 Linearthinking" },
    SeedRoadmap { phase: 2, week_label: "Tuần 11 (03/02)", title: "Câu bị động học thuật (More funding should be allocated...)" },
    SeedRoadmap { phase: 2, week_label: "Tuần 12 (10/02)", title: "Danh từ hóa (Nominalization) + Listening Section 3–4 Distractors" },
    SeedRoadmap { phase: 2, week_label: "Tuần 13 (17/02)", title: "Nhượng bộ & tương phản (Although/While/Despite) + Speaking A.R.E.A" },
    SeedRoadmap { phase: 2, week_label: "Tuần 14 (24/02)", title: "Cấu trúc nhấn mạnh (Not only... but also...) + Task 1 Process/Map" },
    SeedRoadmap { phase: 2, week_label: "Tuần 15 (03/03)", title: "Task 1 Mixed Charts + ✅ CHECKPOINT MỐC 2 (Cam 15 T4: L≥6.5-7.0, R≥6.5, W≥5.5-6.0)" },
    // Giai đoạn 3: Tuần 16-21 (10/03/2027 -> 20/04/2027)
    SeedRoadmap { phase: 3, week_label: "Tuần 16 (10/03)", title: "Cam 16 Full L+R (mục tiêu 30–34/40) + The Key to IELTS Writing" },
    SeedRoadmap { phase: 3, week_label: "Tuần 17 (17/03)", title: "Full Test Writing 60p + 4 chủ đề tủ (Education, Tech, Work, Env)" },
    SeedRoadmap { phase: 3, week_label: "Tuần 18 (24/03)", title: "Cam 17 Full L+R + Speaking Forecast Q1–2/2027 (gom 12 câu chuyện)" },
    SeedRoadmap { phase: 3, week_label: "Tuần 19 (31/03)", title: "Full Test Writing 60p + 4 chủ đề tủ (Health, Urban, Gov, Media)" },
    SeedRoadmap { phase: 3, week_label: "Tuần 20 (07/04)", title: "Cam 18 Full L+R + 4 chủ đề tủ (Crime, Family, Culture, Transport)" },
    SeedRoadmap { phase: 3, week_label: "Tuần 21 (14/04)", title: "✅ CHECKPOINT MỐC 3: 2 Full Test liên tiếp ≥ 6.5–7.0 -> Chốt lịch thi 05/2027" },
    // Giai đoạn 4: Tuần 22-24 (21/04/2027 -> giữa 05/2027)
    SeedRoadmap { phase: 4, week_label: "Tuần 22 (21/04)", title: "Nước rút: Cam 19 Full Test (L->R->W 1 mạch) x2 lần/tuần" },
    SeedRoadmap { phase: 4, week_label: "Tuần 23 (28/04)", title: "Actual Tests + Ôn Sổ lỗi cá nhân & Checklist 3 phút & 5 khung Task 2" },
    SeedRoadmap { phase: 4, week_label: "Tuần 24 (05/05)", title: "12 câu chuyện Speaking Forecast + 3 ngày cuối giữ nhịp nhẹ, ngủ đủ" },
];
