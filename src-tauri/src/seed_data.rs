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

#[allow(dead_code)]
pub fn default_writing_prompts() -> Vec<WritingPrompt> {
    default_writing_prompts_for_band(7.0)
}

pub fn default_writing_prompts_for_band(target_band: f64) -> Vec<WritingPrompt> {
    let is_80 = target_band >= 7.5;
    vec![
        // 4 Paragraph 90 words
        WritingPrompt {
            id: "p90-1".into(),
            mode: "paragraph_90".into(),
            title: "AI & Tự động hóa (Body 90 từ)".into(),
            prompt: "Automation and AI are replacing many manual jobs. Write ONE body paragraph (80–100 words) explaining one major benefit of this trend.".into(),
            outline_hint: if is_80 {
                "KHUNG BAND 7.5–8.0 (Topic -> Mechanism -> Nuanced Example/Concession -> Result):\n1. Topic: One compelling advantage of automation lies in its capacity to spur sustainable productivity.\n2. Mechanism: By executing repetitive procedures with near-zero variance, automated systems mitigate human error.\n3. Concession/Nuance: While initial capital outlays can be substantial, long-term efficiency gains arguably outweigh them.\n4. Result: Consequently, human talent can be reallocated toward strategic and creative innovation.".into()
            } else {
                "KHUNG 4 CÂU (Topic -> Explain -> Example -> Result):\n1. Topic: One primary advantage of automation is higher productivity.\n2. Explain: Machines perform repetitive tasks continuously without fatigue.\n3. Example: In modern factories, robotic arms assemble components faster.\n4. Result: Workers can shift to higher-value creative or supervisory roles.".into()
            },
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
            outline_hint: if is_80 {
                "KHUNG TASK 1 OVERVIEW 7.5+ (Gom nhóm thông minh):\n• Intro: Paraphrase đề + timeframe (2015–2025) tự nhiên.\n• Overview: Nêu 2 đặc điểm nổi bật nhất (xu hướng chủ đạo + đối tượng dẫn đầu/bứt phá mạnh nhất), KHÔNG chèn số liệu.\n• Body 1: Gom nhóm các đối tượng có quỹ đạo tương đồng, nêu mốc khởi điểm & điểm chuyển biến.\n• Body 2: So sánh tương quan cuối kỳ (bội số/chênh lệch) thay vì liệt kê tuần tự.".into()
            } else {
                "KHUNG TASK 1 OVERVIEW:\n• Intro: Paraphrase đề + timeframe (2015–2025).\n• Overview: Nêu 2 ý lớn (xu hướng chung tăng/giảm + đối tượng cao nhất/thay đổi mạnh nhất). KHÔNG đưa số liệu vào Overview.\n• Body 1: Số liệu đầu kỳ (2015) & so sánh giữa kỳ.\n• Body 2: Số liệu cuối kỳ (2025) & khoảng cách thu hẹp.".into()
            },
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
            outline_hint: if is_80 {
                "KHUNG TASK 2 BAND 7.5–8.0 (Lập luận sâu + Concession + Hedging):\n• Intro: Contextualize nhẹ + Thesis nêu rõ mức độ đồng tình (to a considerable extent / arguably).\n• Body 1 (Counter-argument + Rebuttal): Thừa nhận mặt hợp lý của quan điểm đối lập (Admittedly...), sau đó phản biện sắc bén (However, this overlooks...).\n• Body 2 (Core Argument): Phân tích cơ chế nhân–quả sâu + ví dụ cụ thể, tránh over-generalization.\n• Conclusion: Khẳng định lại lập trường bằng paraphrase mới, không thêm ý mới.".into()
            } else {
                "KHUNG 5 DẠNG TASK 2 (CHUẨN SIMON):\n1. Opinion (Agree/Disagree): Intro (Paraphrase + Thesis rõ lập trường) -> Body 1 (Lý do 1) -> Body 2 (Lý do 2) -> Conclusion (Nhắc lại lập trường, KHÔNG thêm ý mới).\n2. Discussion + Opinion: Intro -> Body 1 (Quan điểm A) -> Body 2 (Quan điểm B + nghiêng về B) -> Conclusion.\n3. Adv–Disadv: Intro -> Body 1 (Mặt yếu hơn) -> Body 2 (Mặt trội hơn) -> Conclusion.\n4. Problem–Solution: Intro -> Body 1 (Nguyên nhân/Vấn đề) -> Body 2 (Giải pháp) -> Conclusion.\n5. Two-part: Intro -> Body 1 (Trả lời câu hỏi 1) -> Body 2 (Trả lời câu hỏi 2) -> Conclusion.".into()
            },
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

// 40 cards: C1–C2 Academic Collocations & Hedging for Aim 8.0
pub const VOCAB_SEEDS_80: &[SeedVocab] = &[
    // Academic Hedging & Argumentation (8)
    SeedVocab { topic: "Hedging & Stance", word: "to a considerable extent", meaning: "ở mức độ đáng kể (tránh khẳng định tuyệt đối)", example: "Urban livability depends, to a considerable extent, on accessible public transit." },
    SeedVocab { topic: "Hedging & Stance", word: "it is widely contended that", meaning: "nhiều ý kiến lập luận rằng", example: "It is widely contended that early bilingualism enhances cognitive flexibility." },
    SeedVocab { topic: "Hedging & Stance", word: "arguably", meaning: "có thể lập luận rằng (giảm sắc thái cực đoan)", example: "Preventive healthcare is arguably more cost-effective than emergency treatment." },
    SeedVocab { topic: "Hedging & Stance", word: "be inextricably linked to", meaning: "gắn bó mật thiết, không thể tách rời với", example: "Economic stability is inextricably linked to equitable access to education." },
    SeedVocab { topic: "Hedging & Stance", word: "strike a delicate balance between", meaning: "giữ thế cân bằng tinh tế giữa...", example: "Regulators must strike a delicate balance between data privacy and innovation." },
    SeedVocab { topic: "Hedging & Stance", word: "warrant immediate intervention", meaning: "đòi hỏi sự can thiệp kịp thời", example: "Escalating youth anxiety rates warrant immediate intervention from schools." },
    SeedVocab { topic: "Hedging & Stance", word: "yield tangible dividends", meaning: "mang lại lợi ích/kết quả rõ rệt", example: "Long-term investment in teacher training yields tangible dividends in literacy." },
    SeedVocab { topic: "Hedging & Stance", word: "by no means a panacea", meaning: "hoàn toàn không phải liều thuốc vạn năng", example: "Remote learning is by no means a panacea for unequal educational resources." },

    // Society & Inequality (6)
    SeedVocab { topic: "Society (C1–C2)", word: "exacerbate socioeconomic disparities", meaning: "làm trầm trọng thêm bất bình đẳng kinh tế–xã hội", example: "Unequal broadband access can exacerbate socioeconomic disparities across regions." },
    SeedVocab { topic: "Society (C1–C2)", word: "foster social cohesion (C1)", meaning: "thúc đẩy sự gắn kết cộng đồng", example: "Shared civic spaces foster social cohesion in rapidly urbanizing cities." },
    SeedVocab { topic: "Society (C1–C2)", word: "allocate public funds judiciously", meaning: "phân bổ ngân sách công một cách sáng suốt", example: "Municipalities must allocate public funds judiciously to essential infrastructure." },
    SeedVocab { topic: "Society (C1–C2)", word: "erode interpersonal trust", meaning: "làm xói mòn niềm tin giữa người với người", example: "Algorithmic echo chambers can gradually erode interpersonal trust." },
    SeedVocab { topic: "Society (C1–C2)", word: "bridge the generational divide", meaning: "thu hẹp khoảng cách thế hệ", example: "Intergenerational volunteering programs help bridge the generational divide." },
    SeedVocab { topic: "Society (C1–C2)", word: "impose stringent regulations on", meaning: "áp đặt quy định nghiêm ngặt lên...", example: "Authorities should impose stringent regulations on predatory digital advertising." },

    // Technology & AI (6)
    SeedVocab { topic: "Technology (C1–C2)", word: "render manual roles obsolete", meaning: "khiến các công việc thủ công trở nên lỗi thời", example: "Generative automation may render certain routine administrative roles obsolete." },
    SeedVocab { topic: "Technology (C1–C2)", word: "mitigate adverse repercussions", meaning: "giảm thiểu hệ lụy tiêu cực", example: "Reskilling subsidies can mitigate the adverse repercussions of rapid automation." },
    SeedVocab { topic: "Technology (C1–C2)", word: "ubiquitous digital connectivity", meaning: "kết nối số hiện diện khắp nơi", example: "Ubiquitous digital connectivity has reshaped how knowledge is disseminated." },
    SeedVocab { topic: "Technology (C1–C2)", word: "pose an unprecedented threat to", meaning: "gây ra mối đe dọa chưa từng có đối với...", example: "Unchecked deepfakes pose an unprecedented threat to electoral integrity." },
    SeedVocab { topic: "Technology (C1–C2)", word: "catalyze technological breakthroughs", meaning: "thúc đẩy các đột phá công nghệ", example: "Cross-border academic collaboration helps catalyze technological breakthroughs." },
    SeedVocab { topic: "Technology (C1–C2)", word: "blur the boundaries between", meaning: "làm mờ ranh giới giữa...", example: "Always-on messaging tools often blur the boundaries between work and leisure." },

    // Environment & Urbanization (6)
    SeedVocab { topic: "Environment (C1–C2)", word: "curb carbon emissions (C1)", meaning: "kiềm chế lượng khí thải carbon", example: "Electrifying public fleets is vital to curb carbon emissions in megacities." },
    SeedVocab { topic: "Environment (C1–C2)", word: "stem the tide of", meaning: "ngăn chặn đà gia tăng của...", example: "Circular packaging laws can help stem the tide of marine plastic pollution." },
    SeedVocab { topic: "Environment (C1–C2)", word: "accelerate the transition toward", meaning: "đẩy nhanh quá trình chuyển dịch sang...", example: "Targeted tax incentives accelerate the transition toward renewable energy." },
    SeedVocab { topic: "Environment (C1–C2)", word: "deplete finite natural resources", meaning: "làm cạn kiệt tài nguyên thiên nhiên hữu hạn", example: "Unrestrained industrial expansion continues to deplete finite natural resources." },
    SeedVocab { topic: "Environment (C1–C2)", word: "alleviate chronic congestion", meaning: "giảm bớt ùn tắc kéo dài", example: "Congestion pricing has helped several capitals alleviate chronic congestion." },
    SeedVocab { topic: "Environment (C1–C2)", word: "inflict irreversible damage on", meaning: "gây tổn hại không thể đảo ngược lên...", example: "Deforestation can inflict irreversible damage on fragile biodiversity." },

    // Education & Cognition (6)
    SeedVocab { topic: "Education (C1–C2)", word: "cultivate independent inquiry", meaning: "nuôi dưỡng tư duy tự chủ và tìm tòi", example: "Seminar-style classes cultivate independent inquiry far better than rote memorization." },
    SeedVocab { topic: "Education (C1–C2)", word: "narrow the attainment gap", meaning: "thu hẹp khoảng cách thành tích học tập", example: "Early literacy support can narrow the attainment gap for disadvantaged pupils." },
    SeedVocab { topic: "Education (C1–C2)", word: "equip graduates with transferable skills", meaning: "trang bị kỹ năng chuyển đổi linh hoạt", example: "Interdisciplinary degrees equip graduates with transferable skills for shifting markets." },
    SeedVocab { topic: "Education (C1–C2)", word: "overemphasis on standardized testing", meaning: "sự quá chú trọng vào thi cử chuẩn hóa", example: "An overemphasis on standardized testing can stifle genuine creativity." },
    SeedVocab { topic: "Education (C1–C2)", word: "pedagogical innovation", meaning: "đổi mới phương pháp sư phạm", example: "Digital simulations encourage pedagogical innovation in science classrooms." },
    SeedVocab { topic: "Education (C1–C2)", word: "keep abreast of", meaning: "bắt kịp, cập nhật sát sao với", example: "Higher education curricula must keep abreast of emerging industry standards." },

    // Work & Economy (4)
    SeedVocab { topic: "Work & Economy", word: "bolster economic resilience", meaning: "củng cố sức chống chịu kinh tế", example: "Diversifying export sectors helps bolster economic resilience against shocks." },
    SeedVocab { topic: "Work & Economy", word: "alleviate occupational burnout", meaning: "giảm thiểu kiệt sức nghề nghiệp", example: "Predictable rest policies significantly alleviate occupational burnout." },
    SeedVocab { topic: "Work & Economy", word: "gain a decisive competitive edge", meaning: "giành lợi thế cạnh tranh quyết định", example: "Firms that invest in R&D gain a decisive competitive edge." },
    SeedVocab { topic: "Work & Economy", word: "spur sustainable productivity", meaning: "thúc đẩy năng suất bền vững", example: "Autonomy at work tends to spur sustainable productivity rather than micromanagement." },

    // Health & Lifestyle (4)
    SeedVocab { topic: "Health (C1–C2)", word: "predispose individuals to", meaning: "khiến cá nhân dễ mắc phải...", example: "Prolonged sedentary habits predispose individuals to cardiovascular disease." },
    SeedVocab { topic: "Health (C1–C2)", word: "take a heavy toll on", meaning: "gây tổn hại nặng nề lên...", example: "Chronic sleep deprivation can take a heavy toll on cognitive performance." },
    SeedVocab { topic: "Health (C1–C2)", word: "strain public healthcare capacity", meaning: "gây quá tải năng lực y tế công", example: "Demographic aging will inevitably strain public healthcare capacity." },
    SeedVocab { topic: "Health (C1–C2)", word: "promote preventive well-being", meaning: "thúc đẩy chăm sóc sức khỏe dự phòng", example: "Walkable neighborhoods promote preventive well-being across all age groups." },
];

// Lộ trình 24 tuần Aim 6.5 (Vùng đệm 7.0) — Gắn KPI định lượng từng tuần + 3 Cổng Checkpoint
pub const ROADMAP_SEEDS_65: &[SeedRoadmap] = &[
    // Giai đoạn 1: Tuần 1-7 (25/11/2026 -> 12/01/2027) — Diệt lỗi nền & 5 khung T2 + 3 biểu đồ T1
    SeedRoadmap { phase: 1, week_label: "Tuần 1", title: "[W: Lỗi nhóm 1 (số ít/nhiều, chia ĐT, mạo từ) + Task 2 Opinion | R/L: Official Guide T/F/NG + Dictation | S: A.R.E.A | Đích tuần: Lỗi nhóm 1 ≤ 3 lỗi/bài]" },
    SeedRoadmap { phase: 1, week_label: "Tuần 2", title: "[W: Lỗi nhóm 1 + Task 2 Discussion & Opinion (viết lại bài test cũ) | R/L: Matching Headings + DailyDictation | S: Part 1 lưu loát | Đích tuần: W ≥ 5.0]" },
    SeedRoadmap { phase: 1, week_label: "Tuần 3", title: "[W: Lỗi nhóm 2 (change/chance, effect/affect, word family) + Task 2 Adv–Disadv | R/L: Summary Completion + 5 cặp Paraphrase | S: Sửa âm cuối s/ed | Đích tuần: R/L ≥ 20/40]" },
    SeedRoadmap { phase: 1, week_label: "Tuần 4", title: "[W: Lỗi nhóm 2 + Task 2 Problem/Cause–Solution | R/L: Linearthinking Passage 1–2 | S: Part 1 + lý do A.R.E.A | Đích tuần: Đủ 4/5 khung Task 2]" },
    SeedRoadmap { phase: 1, week_label: "Tuần 5", title: "[W: Lỗi nhóm 3 (câu cụt, comma splice) + Task 2 Two-part question | R/L: Multiple Choice + Distractors | S: Mở rộng ví dụ | Đích tuần: 0 câu cụt/comma splice]" },
    SeedRoadmap { phase: 1, week_label: "Tuần 6", title: "[W: Task 1 Line Graph (Overview + xu hướng) + viết lại T2 yếu nhất | R/L: Cam 12 Passage bấm giờ 20p | S: Ghi âm 2p | Đích tuần: Overview 0 chèn số liệu]" },
    SeedRoadmap { phase: 1, week_label: "Tuần 7", title: "✅ CHECKPOINT MỐC 1 (Cam 12 Test 1 + T2 Discussion): Đích L ≥ 23/40 (6.0), R ≥ 23/40 (6.0), W ≥ 5.5 (lỗi nhóm 1 ≤ 2/đoạn), S ≥ 5.5" },

    // Giai đoạn 2: Tuần 8-15 (13/01/2027 -> 09/03/2027) — Tăng tốc & 6 cấu trúc nâng cao
    SeedRoadmap { phase: 2, week_label: "Tuần 8", title: "[W: Mệnh đề quan hệ & rút gọn (Students studying...) | R/L: Cam 13 T1–2 + Linearthinking P2 | S: Bắt đầu Part 2 (2p) | Đích tuần: Dùng ≥ 2 MĐQH/bài]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 9", title: "[W: Thực hành MĐQH viết lại bài cũ đạt ≥ 6.0 | R/L: Cam 13 T3–4 + bảng Paraphrase | S: Part 2 trôi chảy 2p | Đích tuần: R ≥ 25/40]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 10", title: "[W: Câu điều kiện loại 1 & 2 (If governments invested...) | R/L: Cam 14 T1–2 + L Section 3 | S: Part 3 theo A.R.E.A | Đích tuần: L ≥ 26/40]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 11", title: "[W: Câu bị động học thuật (More funding should be allocated...) | R/L: Cam 14 T3–4 + Distractor traps | S: Nối ý mượt | Đích tuần: W ≥ 5.5–6.0]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 12", title: "[W: Danh từ hóa (Nominalization: The surge in...) | R/L: Cam 15 T1–2 + Dictation Section 4 | S: Part 2 + Part 3 | Đích tuần: R/L ≥ 27/40]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 13", title: "[W: Nhượng bộ & tương phản (Although/While/Despite) | R/L: Cam 15 T3 + T/F/NG tốc độ | S: Sửa lỗi thì & chia ĐT | Đích tuần: W ≥ 6.0]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 14", title: "[W: Nhấn mạnh (Not only... but also...) + Task 1 Process/Map | R/L: Luyện sâu Section 3–4 | S: 6 câu chuyện Part 2 | Đích tuần: Đủ mọi dạng Task 1]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 15", title: "✅ CHECKPOINT MỐC 2 (Cam 15 Test 4 Full 4 kỹ năng): Đích L ≥ 28/40 (6.5–7.0), R ≥ 27/40 (6.5), W ≥ 5.5–6.0, S ≥ 6.0 (Hụt W<5.5: dồn T6 sang Writing)" },

    // Giai đoạn 3: Tuần 16-21 (10/03/2027 -> 20/04/2027) — Xây vùng đệm 7.0 & Forecast thực chiến
    SeedRoadmap { phase: 3, week_label: "Tuần 16", title: "[W: Key to IELTS Writing + Full Test 60p | R/L: Cam 16 Full bấm giờ Study4 | S: Forecast Q1–2/2027 (4 chủ đề) | Đích tuần: R/L 30–32/40]" },
    SeedRoadmap { phase: 3, week_label: "Tuần 17", title: "[W: Full Test 60p + 4 chủ đề tủ (Education, Tech, Work, Env) | R/L: Cam 16–17 Full | S: Gom 4 câu chuyện Part 2 | Đích tuần: W ≥ 6.0]" },
    SeedRoadmap { phase: 3, week_label: "Tuần 18", title: "[W: Mandatory Rewrite đoạn yếu lên ≥ 6.5 | R/L: Cam 17 Full bấm giờ máy | S: Gom đủ 8 câu chuyện Forecast | Đích tuần: R/L 31–33/40]" },
    SeedRoadmap { phase: 3, week_label: "Tuần 19", title: "[W: Full Test 60p + 4 chủ đề tủ (Health, Urban, Gov, Media) | R/L: Cam 18 T1–2 Full | S: Part 3 phản xạ 30s | Đích tuần: W 6.0–6.5]" },
    SeedRoadmap { phase: 3, week_label: "Tuần 20", title: "[W: 4 chủ đề tủ (Crime, Family, Culture, Transport) + Checklist 3p | R/L: Cam 18 T3–4 Full | S: Đủ 12 câu chuyện khung | Đích tuần: L 32–34/40, R 30–32/40]" },
    SeedRoadmap { phase: 3, week_label: "Tuần 21", title: "✅ CHECKPOINT MỐC 3 (Cam 18): 2 Full Test liên tiếp ≥ 6.5–7.0 (L≥7.0, R≥6.5, S≥6.0, W≥5.5) -> Chốt lịch đăng ký thi 05/2027!" },

    // Giai đoạn 4: Tuần 22-24 (21/04/2027 -> giữa 05/2027) — Nước rút phòng thi
    SeedRoadmap { phase: 4, week_label: "Tuần 22", title: "[W/R/L: Cam 19 Full Mock Test (L→R→W 1 mạch đúng giờ thi) x2 lần | S: Mock 15p với AI/bạn | Đích tuần: Không hụt giờ bất kỳ bài nào]" },
    SeedRoadmap { phase: 4, week_label: "Tuần 23", title: "[W: Ôn Sổ lỗi + Checklist 3p + 5 khung T2 | R/L: Actual Tests mới nhất | S: 12 câu chuyện Forecast | Đích tuần: Ổn định vùng đệm 6.75–7.0]" },
    SeedRoadmap { phase: 4, week_label: "Tuần 24", title: "[Nước rút: Không nạp kiến thức mới; 3 ngày cuối ngủ đủ giấc, nghe nhẹ 15p + lướt Collocations giữ phản xạ phòng thi]" },
];

#[allow(dead_code)]
pub const ROADMAP_SEEDS: &[SeedRoadmap] = ROADMAP_SEEDS_65;

// Lộ trình 24 tuần Aim 8.0 (Dành cho nền đã vững 6.5+) — Kèm KPI định lượng từng tuần + 3 Checkpoints
pub const ROADMAP_SEEDS_80: &[SeedRoadmap] = &[
    // Giai đoạn 1 · Tuần 1–6: Phá trần 6.5 lên 7.0+ (Precision, Nuance & Distractor Mastery)
    SeedRoadmap { phase: 1, week_label: "Tuần 1", title: "[W: Diệt Over-generalization & tập Hedging (tend to/arguably) | R: T/F/NG suy luận P3 | Đích tuần: R ≥ 11/13]" },
    SeedRoadmap { phase: 1, week_label: "Tuần 2", title: "[W: Thay linker cơ học bằng Semantic Referencing | L: Part 3 Multiple Choice dài | Đích tuần: L Part 3 ≥ 8/10]" },
    SeedRoadmap { phase: 1, week_label: "Tuần 3", title: "[W: Task 2 Opinion & Discussion có Counter-argument + Rebuttal | R: Matching Headings | Đích tuần: W ≥ 7.0]" },
    SeedRoadmap { phase: 1, week_label: "Tuần 4", title: "[W: Task 1 Grouping by Pattern & Key Comparisons | L: Part 4 Note Completion C1 | Đích tuần: L Part 4 ≥ 9/10]" },
    SeedRoadmap { phase: 1, week_label: "Tuần 5", title: "[W: Task 2 Problem–Solution & Two-part (nguyên nhân gốc & hệ quả tầng 2) | S: Part 3 Speculating]" },
    SeedRoadmap { phase: 1, week_label: "Tuần 6", title: "✅ CHECKPOINT MỐC 1 (Cam 15 Full): Đích L ≥ 33/40 (7.5), R ≥ 33/40 (7.5), W ≥ 7.0, S ≥ 7.0" },

    // Giai đoạn 2 · Tuần 7–14: Nâng tầm C1–C2 & Tư duy phản biện Band 7.5–8.0
    SeedRoadmap { phase: 2, week_label: "Tuần 7", title: "[W: Nominalization & Non-defining Relative Clauses | R: Cam 16 T1–2 bấm giờ 54p | Đích tuần: R ≥ 34/40]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 8", title: "[W: Participle Clauses & Inversion tiết chế | L: Cam 16 T1–2 tốc độ 1.1x | Đích tuần: L ≥ 34/40]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 9", title: "[W: Chủ đề AI Ethics & Automation (C1 collocations + T2 38p) | S: Part 2 Storytelling có chiều sâu]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 10", title: "[W: Economic Inequality & Public Spending + T1 Mixed Charts | R: Cam 16 T3–4 (Multiple Choice P3)]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 11", title: "[W: Climate Policy & Urban Planning + T1 Process/Map | L: Cam 16 T3–4 (Attitude & Agreement traps)]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 12", title: "[W: Higher Education & Work Future (sạch 100% mạo từ/giới từ) | S: Part 3 Hedging & Idiomatic]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 13", title: "[W: Media Censorship, Crime & Culture | R+L: Cam 17 T1–3 bấm giờ khắt khe | Đích tuần: 35+/40]" },
    SeedRoadmap { phase: 2, week_label: "Tuần 14", title: "✅ CHECKPOINT MỐC 2 (Cam 17 T4 Full 4 kỹ năng): Đích L ≥ 35/40 (8.0), R ≥ 35/40 (8.0), W ≥ 7.0–7.5, S ≥ 7.0–7.5" },

    // Giai đoạn 3 · Tuần 15–20: Chốt chuẩn Band 8.0 & Mastery 12 chủ đề trừu tượng
    SeedRoadmap { phase: 3, week_label: "Tuần 15", title: "[Cam 18 T1–2 (R 52p, L 1.1x) + 2 bài Full Writing 60p + Mandatory Rewrite đoạn < 7.5]" },
    SeedRoadmap { phase: 3, week_label: "Tuần 16", title: "[Cam 18 T3–4 + Chủ đề Bioethics, Space Funding & Globalization + Sổ bẫy Paraphrase C1–C2]" },
    SeedRoadmap { phase: 3, week_label: "Tuần 17", title: "[Speaking Forecast Part 2–3 nâng cao (gom 12 trục câu chuyện C1) + 2 bài Full Writing 60p]" },
    SeedRoadmap { phase: 3, week_label: "Tuần 18", title: "[Cam 19 T1–2 bấm giờ phòng thi + Rà soát độ chính xác GRA (≥ 80% câu hoàn toàn không lỗi)]" },
    SeedRoadmap { phase: 3, week_label: "Tuần 19", title: "[Cam 19 T3–4 + 2 bài Full Writing 60p (T1 18p + T2 38p + 4p soát lỗi tinh vi)]" },
    SeedRoadmap { phase: 3, week_label: "Tuần 20", title: "✅ CHECKPOINT MỐC 3: 2 Full Test liên tiếp ≥ 7.5–8.0 (L ≥ 8.0, R ≥ 8.0, W ≥ 7.0–7.5, S ≥ 7.0–7.5)" },

    // Giai đoạn 4 · Tuần 21–24: Nước rút Peak Performance phòng thi
    SeedRoadmap { phase: 4, week_label: "Tuần 21", title: "[Full Mock Test 3 kỹ năng liền mạch (L→R→W 2h40p không nghỉ) x2 lần + Chữa sâu từng câu sai]" },
    SeedRoadmap { phase: 4, week_label: "Tuần 22", title: "[Actual Tests mới nhất + Tối ưu tốc độ đọc lướt & kiểm soát nhịp viết không hụt giờ]" },
    SeedRoadmap { phase: 4, week_label: "Tuần 23", title: "[Ôn tổng Sổ lỗi cá nhân, 40+ Collocations C1–C2 tủ & 12 trục Speaking Forecast Part 2–3]" },
    SeedRoadmap { phase: 4, week_label: "Tuần 24", title: "[Giữ phản xạ nhẹ 30p/ngày, cân chỉnh đồng hồ sinh học đúng giờ thi, tâm lý vững vàng]" },
];
