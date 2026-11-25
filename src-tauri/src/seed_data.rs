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
        // 4 Task 2
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
    SeedVocab { topic: "Hedging & Stance", word: "to a considerable extent", meaning: "ở mức độ đáng kể (tránh khẳng định tuyệt đối)", example: "Urban livability depends, to a considerable extent, on accessible public transit." },
    SeedVocab { topic: "Hedging & Stance", word: "it is widely contended that", meaning: "nhiều ý kiến lập luận rằng", example: "It is widely contended that early bilingualism enhances cognitive flexibility." },
    SeedVocab { topic: "Hedging & Stance", word: "arguably", meaning: "có thể lập luận rằng (giảm sắc thái cực đoan)", example: "Preventive healthcare is arguably more cost-effective than emergency treatment." },
    SeedVocab { topic: "Hedging & Stance", word: "be inextricably linked to", meaning: "gắn bó mật thiết, không thể tách rời với", example: "Economic stability is inextricably linked to equitable access to education." },
    SeedVocab { topic: "Hedging & Stance", word: "strike a delicate balance between", meaning: "giữ thế cân bằng tinh tế giữa...", example: "Regulators must strike a delicate balance between data privacy and innovation." },
    SeedVocab { topic: "Hedging & Stance", word: "warrant immediate intervention", meaning: "đòi hỏi sự can thiệp kịp thời", example: "Escalating youth anxiety rates warrant immediate intervention from schools." },
    SeedVocab { topic: "Hedging & Stance", word: "yield tangible dividends", meaning: "mang lại lợi ích/kết quả rõ rệt", example: "Long-term investment in teacher training yields tangible dividends in literacy." },
    SeedVocab { topic: "Hedging & Stance", word: "by no means a panacea", meaning: "hoàn toàn không phải liều thuốc vạn năng", example: "Remote learning is by no means a panacea for unequal educational resources." },

    SeedVocab { topic: "Society (C1–C2)", word: "exacerbate socioeconomic disparities", meaning: "làm trầm trọng thêm bất bình đẳng kinh tế–xã hội", example: "Unequal broadband access can exacerbate socioeconomic disparities across regions." },
    SeedVocab { topic: "Society (C1–C2)", word: "bolster social cohesion", meaning: "thúc đẩy sự gắn kết cộng đồng", example: "Shared civic spaces bolster social cohesion in rapidly urbanizing cities." },
    SeedVocab { topic: "Society (C1–C2)", word: "allocate public funds judiciously", meaning: "phân bổ ngân sách công một cách sáng suốt", example: "Municipalities must allocate public funds judiciously to essential infrastructure." },
    SeedVocab { topic: "Society (C1–C2)", word: "erode interpersonal trust", meaning: "làm xói mòn niềm tin giữa người với người", example: "Algorithmic echo chambers can gradually erode interpersonal trust." },
    SeedVocab { topic: "Society (C1–C2)", word: "bridge the generational divide", meaning: "thu hẹp khoảng cách thế hệ", example: "Intergenerational volunteering programs help bridge the generational divide." },
    SeedVocab { topic: "Society (C1–C2)", word: "impose stringent regulations on", meaning: "áp đặt quy định nghiêm ngặt lên...", example: "Authorities should impose stringent regulations on predatory digital advertising." },

    SeedVocab { topic: "Technology (C1–C2)", word: "render manual roles obsolete", meaning: "khiến các công việc thủ công trở nên lỗi thời", example: "Generative automation may render certain routine administrative roles obsolete." },
    SeedVocab { topic: "Technology (C1–C2)", word: "mitigate adverse repercussions", meaning: "giảm thiểu hệ lụy tiêu cực", example: "Reskilling subsidies can mitigate the adverse repercussions of rapid automation." },
    SeedVocab { topic: "Technology (C1–C2)", word: "ubiquitous digital connectivity", meaning: "kết nối số hiện diện khắp nơi", example: "Ubiquitous digital connectivity has reshaped how knowledge is disseminated." },
    SeedVocab { topic: "Technology (C1–C2)", word: "pose an unprecedented threat to", meaning: "gây ra mối đe dọa chưa từng có đối với...", example: "Unchecked deepfakes pose an unprecedented threat to electoral integrity." },
    SeedVocab { topic: "Technology (C1–C2)", word: "catalyze technological breakthroughs", meaning: "thúc đẩy các đột phá công nghệ", example: "Cross-border academic collaboration helps catalyze technological breakthroughs." },
    SeedVocab { topic: "Technology (C1–C2)", word: "blur the boundaries between", meaning: "làm mờ ranh giới giữa...", example: "Always-on messaging tools often blur the boundaries between work and leisure." },

    SeedVocab { topic: "Environment (C1–C2)", word: "decarbonize heavy industry", meaning: "giảm phát thải carbon trong công nghiệp nặng", example: "Electrifying public fleets and decarbonizing heavy industry are vital in megacities." },
    SeedVocab { topic: "Environment (C1–C2)", word: "stem the tide of", meaning: "ngăn chặn đà gia tăng của...", example: "Circular packaging laws can help stem the tide of marine plastic pollution." },
    SeedVocab { topic: "Environment (C1–C2)", word: "accelerate the transition toward", meaning: "đẩy nhanh quá trình chuyển dịch sang...", example: "Targeted tax incentives accelerate the transition toward renewable energy." },
    SeedVocab { topic: "Environment (C1–C2)", word: "deplete finite natural resources", meaning: "làm cạn kiệt tài nguyên thiên nhiên hữu hạn", example: "Unrestrained industrial expansion continues to deplete finite natural resources." },
    SeedVocab { topic: "Environment (C1–C2)", word: "alleviate chronic congestion", meaning: "giảm bớt ùn tắc kéo dài", example: "Congestion pricing has helped several capitals alleviate chronic congestion." },
    SeedVocab { topic: "Environment (C1–C2)", word: "inflict irreversible damage on", meaning: "gây tổn hại không thể đảo ngược lên...", example: "Deforestation can inflict irreversible damage on fragile biodiversity." },

    SeedVocab { topic: "Education (C1–C2)", word: "cultivate independent inquiry", meaning: "nuôi dưỡng tư duy tự chủ và tìm tòi", example: "Seminar-style classes cultivate independent inquiry far better than rote memorization." },
    SeedVocab { topic: "Education (C1–C2)", word: "narrow the attainment gap", meaning: "thu hẹp khoảng cách thành tích học tập", example: "Early literacy support can narrow the attainment gap for disadvantaged pupils." },
    SeedVocab { topic: "Education (C1–C2)", word: "equip graduates with transferable skills", meaning: "trang bị kỹ năng chuyển đổi linh hoạt", example: "Interdisciplinary degrees equip graduates with transferable skills for shifting markets." },
    SeedVocab { topic: "Education (C1–C2)", word: "overemphasis on standardized testing", meaning: "sự quá chú trọng vào thi cử chuẩn hóa", example: "An overemphasis on standardized testing can stifle genuine creativity." },
    SeedVocab { topic: "Education (C1–C2)", word: "pedagogical innovation", meaning: "đổi mới phương pháp sư phạm", example: "Digital simulations encourage pedagogical innovation in science classrooms." },
    SeedVocab { topic: "Education (C1–C2)", word: "keep abreast of", meaning: "bắt kịp, cập nhật sát sao với", example: "Higher education curricula must keep abreast of emerging industry standards." },

    SeedVocab { topic: "Work & Economy", word: "bolster economic resilience", meaning: "củng cố sức chống chịu kinh tế", example: "Diversifying export sectors helps bolster economic resilience against shocks." },
    SeedVocab { topic: "Work & Economy", word: "alleviate occupational burnout", meaning: "giảm thiểu kiệt sức nghề nghiệp", example: "Predictable rest policies significantly alleviate occupational burnout." },
    SeedVocab { topic: "Work & Economy", word: "gain a decisive competitive edge", meaning: "giành lợi thế cạnh tranh quyết định", example: "Firms that invest in R&D gain a decisive competitive edge." },
    SeedVocab { topic: "Work & Economy", word: "spur sustainable productivity", meaning: "thúc đẩy năng suất bền vững", example: "Autonomy at work tends to spur sustainable productivity rather than micromanagement." },

    SeedVocab { topic: "Health (C1–C2)", word: "predispose individuals to", meaning: "khiến cá nhân dễ mắc phải...", example: "Prolonged sedentary habits predispose individuals to cardiovascular disease." },
    SeedVocab { topic: "Health (C1–C2)", word: "take a heavy toll on", meaning: "gây tổn hại nặng nề lên...", example: "Chronic sleep deprivation can take a heavy toll on cognitive performance." },
    SeedVocab { topic: "Health (C1–C2)", word: "strain public healthcare capacity", meaning: "gây quá tải năng lực y tế công", example: "Demographic aging will inevitably strain public healthcare capacity." },
    SeedVocab { topic: "Health (C1–C2)", word: "promote preventive well-being", meaning: "thúc đẩy chăm sóc sức khỏe dự phòng", example: "Walkable neighborhoods promote preventive well-being across all age groups." },
];

// Lộ trình 24 tuần Aim 6.5 (Vùng đệm 7.0) — Gắn nhịp công cụ v1.6.0 + KPI định lượng + 3 Checkpoint (Tuần 6, 14, 20)
pub const ROADMAP_SEEDS_65: &[SeedRoadmap] = &[
    SeedRoadmap { phase: 1, week_label: "Tuần 1", title: "Study4 Mini-Mock 15p + 4 cặp Paraphrase | W: rà Top 3 lỗi nhóm 1 + T2 Opinion | Dictation bẫy -s/-ed | Nói 4-3-2 | Đích: Lỗi nhóm 1 ≤ 3/bài" },
    SeedRoadmap { phase: 1, week_label: "Tuần 2", title: "Study4 Reading P1 (16p) + Sentence Mining 4 từ SRS | W: T2 Discussion (rà Top 3) | Nói Cue Card #3–4 | Quiz & Chữa Leech | Đích: W ≥ 5.0" },
    SeedRoadmap { phase: 1, week_label: "Tuần 3", title: "Study4 T/F/NG + Bảng Paraphrase Reading (6 dòng) | W: Lỗi nhầm từ + T2 Adv–Disadv | Dictation 10 câu nối âm | Nói 4-3-2 | Đích: R/L ≥ 20/40" },
    SeedRoadmap { phase: 1, week_label: "Tuần 4", title: "Study4 Sec 1–2 (14p) + Dictation 5 câu con số | W: T2 Problem–Solution (rà Top 3) | Nói Part 2+3 A.R.E.A | Quiz T5+CN | Đích: Đủ 4/5 khung T2" },
    SeedRoadmap { phase: 1, week_label: "Tuần 5", title: "Study4 Summary/MCQ + Bảng Paraphrase | W: Diệt comma splice + T2 Two-part | Dictation bẫy đổi ý | Chữa sạch 🔥 Leech | Đích: 0 câu cụt" },
    SeedRoadmap { phase: 1, week_label: "Tuần 6", title: "✅ CHECKPOINT MỐC 1 (Cam 12 Full + T2 + Nói 2p): Đích L ≥ 23/40 (6.0), R ≥ 23/40 (6.0), W ≥ 5.5, S ≥ 5.5, 0 Leech tồn đọng" },

    SeedRoadmap { phase: 2, week_label: "Tuần 7", title: "W: Task 1 Line Graph (rà Top 3 lỗi mạo từ) | Study4 Reading P2 (16p) + Mining | Dictation Part 3 | Nói 4-3-2 tốc độ 130+ WPM | Đích: Overview 0 số" },
    SeedRoadmap { phase: 2, week_label: "Tuần 8", title: "W: MĐQH & rút gọn + chấm AI | Study4 Heading + Bảng Paraphrase | Dictation 5 câu -ed | Nói Part 2 (2:00) | Quiz giữa tuần | Đích: ≥ 2 MĐQH/bài" },
    SeedRoadmap { phase: 2, week_label: "Tuần 9", title: "W: Task 1 Bar/Pie + viết lại đoạn cũ ≥ 6.0 | Study4 Cam 13 (16p) | Dictation bẫy phủ định | Nói Cue Card + Part 3 | Đích: R ≥ 25/40" },
    SeedRoadmap { phase: 2, week_label: "Tuần 10", title: "W: Điều kiện 1–2 (rà Top 3) + chấm AI | Study4 Listening Sec 3 (14p) + Dictation 6 câu | Nói A.R.E.A | Quiz & Chữa Leech | Đích: L ≥ 26/40" },
    SeedRoadmap { phase: 2, week_label: "Tuần 11", title: "W: Bị động học thuật + Task 1 Table | Study4 P2–3 + Bảng Paraphrase | Dictation Sec 4 | Nói 4-3-2 giảm từ đệm < 5 | Đích: W ≥ 5.5–6.0" },
    SeedRoadmap { phase: 2, week_label: "Tuần 12", title: "W: Danh từ hóa (Nominalization) | Study4 Cam 15 Mini-Mock + Mining | Dictation học thuật | Nói Part 2+3 | Quiz tổng kết | Đích: R/L ≥ 27/40" },
    SeedRoadmap { phase: 2, week_label: "Tuần 13", title: "W: Nhượng bộ Although/Despite + rà Top 3 | Study4 T/F/NG tốc độ + Bảng Paraphrase | Dictation 1.15x | Chữa 🔥 Leech | Đích: W ≥ 6.0" },
    SeedRoadmap { phase: 2, week_label: "Tuần 14", title: "✅ CHECKPOINT MỐC 2 (Cam 15 Test 4 Full 4 kỹ năng): Đích L ≥ 28/40 (6.5), R ≥ 27/40 (6.5), W ≥ 5.5–6.0, S ≥ 6.0 (Dictation ≥ 85%)" },

    SeedRoadmap { phase: 3, week_label: "Tuần 15", title: "W: Task 1 Process/Map + Not only...but also | Study4 Cam 16 (16p) + Bảng Paraphrase | Nói Part 2+3 chấm AI | Quiz sửa lỗi | Đích: Đủ dạng T1" },
    SeedRoadmap { phase: 3, week_label: "Tuần 16", title: "Study4 Cam 16 Full + Mini-Mock 16p | W: 4 chủ đề tủ (Edu, Tech, Work, Env) rà Top 3 | Dictation 1.15x | Nói Forecast #1–4 | Đích: R/L 30–32/40" },
    SeedRoadmap { phase: 3, week_label: "Tuần 17", title: "W: Full 60p + Mandatory Rewrite đoạn yếu | Study4 P3 + Mining 6 cụm | Nói 4-3-2 Forecast #5–8 | Quiz định kỳ & Chữa Leech | Đích: W ≥ 6.0" },
    SeedRoadmap { phase: 3, week_label: "Tuần 18", title: "Study4 Cam 17 bấm giờ máy + Bảng Paraphrase | W: 4 chủ đề (Health, Urban, Gov, Media) | Dictation Part 4 | Nói Thi thật 2:00+Part 3 | Đích: R/L 31–33/40" },
    SeedRoadmap { phase: 3, week_label: "Tuần 19", title: "W: 4 chủ đề (Crime, Family, Culture, Transport) + rà Top 3 | Study4 Cam 18 Mini-Mock | Nói phản xạ Part 3 45s | Quiz sửa lỗi | Đích: W 6.0–6.5" },
    SeedRoadmap { phase: 3, week_label: "Tuần 20", title: "✅ CHECKPOINT MỐC 3 (Cam 18 Full 4 kỹ năng): 2 đề liên tiếp ≥ 6.5–7.0 (L ≥ 32/40, R ≥ 30/40, S ≥ 6.0, W ≥ 6.0) → Chốt ngày thi!" },

    SeedRoadmap { phase: 4, week_label: "Tuần 21", title: "Study4 Cam 19 Full Mock (L→R→W liền mạch) | Nói Thi thật chấm Gemini Audio | Rà Bảng Paraphrase + Chữa 🔥 Leech | Đích: Không hụt giờ" },
    SeedRoadmap { phase: 4, week_label: "Tuần 22", title: "Study4 Cam 19 Test 2 Full + Dictation 1.15x giữ tai | W: Sạch Top 3 lỗi (0 lặp lỗi) | Nói 12 trục Forecast | Đích: Vùng đệm 6.75–7.0" },
    SeedRoadmap { phase: 4, week_label: "Tuần 23", title: "Actual Tests Study4 + Quiz tổng hợp từ vựng & Sổ lỗi | W: Soát checklist 3p | Nói giữ nhịp 140 WPM | Đích: L/R ổn định ≥ 32/40" },
    SeedRoadmap { phase: 4, week_label: "Tuần 24", title: "Nước rút phòng thi: Micro-Check 2p + SRS nhẹ 8p + nghe Dictation 3 câu khởi động tai; ngủ đủ giấc, giữ tâm lý vững vàng" },
];

#[allow(dead_code)]
pub const ROADMAP_SEEDS: &[SeedRoadmap] = ROADMAP_SEEDS_65;

// Lộ trình 24 tuần Aim 8.0 (Dành cho nền 6.5+) — Nhịp công cụ v1.6.0 + KPI C1–C2 + 3 Checkpoint (Tuần 6, 14, 20)
pub const ROADMAP_SEEDS_80: &[SeedRoadmap] = &[
    SeedRoadmap { phase: 1, week_label: "Tuần 1", title: "Study4 P3 (16p) + Bảng Paraphrase C1 | W: Diệt Over-generalization + Hedging (rà Top 3) | Dictation 8.0 Part 4 | Nói 5 cụm C1 | Đích: R P3 ≥ 11/13" },
    SeedRoadmap { phase: 1, week_label: "Tuần 2", title: "Study4 L Part 3 (14p) + Dictation 5 câu bẫy đồng thuận | W: Semantic Referencing thay linker cứng | Nói Part 3 trừu tượng | Đích: L P3 ≥ 8/10" },
    SeedRoadmap { phase: 1, week_label: "Tuần 3", title: "Study4 Matching Headings (16p) + Mining C1 | W: T2 Opinion/Discussion có Counter-argument | Nói 4-3-2 tốc độ 150 WPM | Quiz Leech | Đích: W ≥ 7.0" },
    SeedRoadmap { phase: 1, week_label: "Tuần 4", title: "Study4 L Part 4 (14p) + Dictation 1.15x | W: Task 1 Pattern Grouping + rà Top 3 lỗi | Nói Cue Card #7–8 + Part 3 | Đích: L P4 ≥ 9/10" },
    SeedRoadmap { phase: 1, week_label: "Tuần 5", title: "Study4 T/F/NG suy luận + Bảng Paraphrase C1–C2 | W: T2 Problem–Solution tầng sâu | Nói Speculating | Quiz sửa lỗi C1 | Đích: R/L ≥ 32/40" },
    SeedRoadmap { phase: 1, week_label: "Tuần 6", title: "✅ CHECKPOINT MỐC 1 (Cam 15 Full + Speaking AI): Đích L ≥ 33/40 (7.5), R ≥ 33/40 (7.5), W ≥ 7.0, S ≥ 7.0, Dictation ≥ 90%" },

    SeedRoadmap { phase: 2, week_label: "Tuần 7", title: "Study4 Cam 16 T1–2 Mini-Mock (16p) + Mining | W: Nominalization & Non-defining MĐQH | Nói Part 2 Storytelling C1 | Đích: R ≥ 34/40" },
    SeedRoadmap { phase: 2, week_label: "Tuần 8", title: "Study4 L Cam 16 tốc độ 1.1x + Dictation 5 câu | W: Participle Clauses & Inversion tiết chế | Nói Thi thật Part 2+3 | Quiz & Chữa Leech | Đích: L ≥ 34/40" },
    SeedRoadmap { phase: 2, week_label: "Tuần 9", title: "W: AI Ethics & Automation (38p + rà Top 3) | Study4 P3 + Bảng Paraphrase | Nói Part 3 đa chiều (A.R.E.A nâng cao) | Đích: W ≥ 7.5" },
    SeedRoadmap { phase: 2, week_label: "Tuần 10", title: "Study4 Cam 16 T3–4 (MCQ P3) + Mining C2 | W: Economic Inequality & Mixed Charts | Dictation bẫy học thuật | Nói 4-3-2 | Đích: P3 ≥ 12/13" },
    SeedRoadmap { phase: 2, week_label: "Tuần 11", title: "Study4 L Cam 16 T3–4 (Attitude traps) + Dictation | W: Climate Policy & Process/Map | Nói chấm Gemini Audio (< 3 fillers) | Đích: L ≥ 35/40" },
    SeedRoadmap { phase: 2, week_label: "Tuần 12", title: "W: Higher Education & Work Future (0 lỗi mạo từ/giới từ) | Study4 P3 + Paraphrase | Nói Part 3 Hedging & Idiomatic | Quiz C1–C2 | Đích: GRA ≥ 7.5" },
    SeedRoadmap { phase: 2, week_label: "Tuần 13", title: "Study4 Cam 17 T1–3 bấm giờ khắt khe (52p) | W: Media Censorship & Crime | Dictation 1.15x | Chữa sạch 🔥 Leech | Đích: R/L 35+/40" },
    SeedRoadmap { phase: 2, week_label: "Tuần 14", title: "✅ CHECKPOINT MỐC 2 (Cam 17 T4 Full 4 kỹ năng): Đích L ≥ 35/40 (8.0), R ≥ 35/40 (8.0), W ≥ 7.0–7.5, S ≥ 7.0–7.5" },

    SeedRoadmap { phase: 3, week_label: "Tuần 15", title: "Study4 Cam 18 T1–2 (R 52p) + Bảng Paraphrase C2 | W: 2 bài Full 60p + Rewrite đoạn < 7.5 | Nói Part 2+3 chấm AI | Đích: 0 lặp Top 3" },
    SeedRoadmap { phase: 3, week_label: "Tuần 16", title: "Study4 Cam 18 T3–4 + Mining C2 | W: Bioethics, Space Funding & Globalization | Dictation 1.15x | Quiz tổng kết tuần | Đích: R/L 36/40" },
    SeedRoadmap { phase: 3, week_label: "Tuần 17", title: "Nói Forecast Part 2–3 nâng cao (12 trục C1) + Thi thật 2:00 | W: 2 bài Full 60p rà Top 3 | Study4 Mini-Mock | Đích: FC/LR ≥ 7.5" },
    SeedRoadmap { phase: 3, week_label: "Tuần 18", title: "Study4 Cam 19 T1–2 + Bảng Paraphrase | W: Soát GRA (≥ 85% câu hoàn toàn sạch lỗi) | Dictation Part 4 | Quiz sửa lỗi C1 | Đích: L/R 36–38/40" },
    SeedRoadmap { phase: 3, week_label: "Tuần 19", title: "Study4 Cam 19 T3–4 + Mining | W: T1 18p + T2 38p + 4p soát lỗi tinh vi | Nói Part 3 phản biện 45s/câu | Đích: W ≥ 7.5" },
    SeedRoadmap { phase: 3, week_label: "Tuần 20", title: "✅ CHECKPOINT MỐC 3 (2 Full Test liên tiếp ≥ 7.5–8.0): Đích L ≥ 36/40 (8.0), R ≥ 36/40 (8.0), W ≥ 7.5, S ≥ 7.5" },

    SeedRoadmap { phase: 4, week_label: "Tuần 21", title: "Study4 Full Mock 3 kỹ năng liền mạch (L→R→W 2h40p) x2 | Chữa sâu câu sai vào Sổ lỗi & Bảng Paraphrase | Nói Thi thật AI" },
    SeedRoadmap { phase: 4, week_label: "Tuần 22", title: "Actual Tests Study4 mới nhất + Dictation 1.15x | Tối ưu tốc độ đọc lướt P3 & kiểm soát nhịp viết không hụt giờ | Chữa 🔥 Leech" },
    SeedRoadmap { phase: 4, week_label: "Tuần 23", title: "Quiz tổng hợp 40+ Collocations C1–C2 & Sổ lỗi cá nhân | Nói ôn 12 trục Forecast Part 2–3 | Rà toàn bộ Bảng Paraphrase" },
    SeedRoadmap { phase: 4, week_label: "Tuần 24", title: "Peak Performance: Micro-Check 2p + SRS C1 8p + Dictation nhẹ 3 câu; giữ đồng hồ sinh học đúng giờ thi, tâm lý sắc bén" },
];

pub struct SeedErrorLog {
    pub category: &'static str,
    pub wrong_text: &'static str,
    pub right_text: &'static str,
    pub count: i64,
}

pub const ERROR_LOG_SEEDS_65: &[SeedErrorLog] = &[
    SeedErrorLog { category: "Writing · Chia động từ", wrong_text: "Everybody know", right_text: "Everybody knows", count: 3 },
    SeedErrorLog { category: "Writing · Danh từ đếm được", wrong_text: "human have / Student need", right_text: "Humans need", count: 3 },
    SeedErrorLog { category: "Writing · Mạo từ a/an", wrong_text: "an new adult / an talent artist", right_text: "a new adult", count: 2 },
    SeedErrorLog { category: "Writing · Nhầm từ", wrong_text: "a good change to develop", right_text: "a good chance to develop", count: 2 },
    SeedErrorLog { category: "Writing · Effect/Affect", wrong_text: "can effect our lives", right_text: "can affect our lives", count: 2 },
    SeedErrorLog { category: "Writing · Từ loại", wrong_text: "harmfully effects", right_text: "harmful effects", count: 2 },
    SeedErrorLog { category: "Writing · Comma splice", wrong_text: "...students, they are...", right_text: "...students, as they are...", count: 2 },
    SeedErrorLog { category: "Writing · Câu cụt", wrong_text: "Firstly, positive point about students...", right_text: "One advantage is that students...", count: 2 },
    SeedErrorLog { category: "Reading · T/F/NG", wrong_text: "Bỏ qua từ tuyệt đối every/all/only", right_text: "Soi kỹ từ tuyệt đối", count: 2 },
    SeedErrorLog { category: "Reading · Paraphrase", wrong_text: "Nhầm were delivering ↔ introduce", right_text: "Soi nghĩa đối lập thật", count: 2 },
];

pub const ERROR_LOG_SEEDS_80: &[SeedErrorLog] = &[
    SeedErrorLog { category: "Writing · Thiếu Hedging", wrong_text: "All people will definitely cause", right_text: "Many people tend to cause", count: 3 },
    SeedErrorLog { category: "Writing · Linker máy móc", wrong_text: "Firstly, Moreover, In conclusion", right_text: "This disparity shows", count: 3 },
    SeedErrorLog { category: "Writing · Collocation C1", wrong_text: "make a research", right_text: "conduct empirical research", count: 2 },
    SeedErrorLog { category: "Writing · Thiếu Concession", wrong_text: "Technology only brings benefits", right_text: "Admittedly, technology has drawbacks; however, benefits prevail", count: 2 },
    SeedErrorLog { category: "Writing · Dangling Participle", wrong_text: "Walking into the city, the pollution was severe.", right_text: "Walking into the city, commuters saw severe pollution.", count: 2 },
    SeedErrorLog { category: "Writing · Nominalization", wrong_text: "Because people consume too much plastic", right_text: "Excessive plastic consumption", count: 2 },
    SeedErrorLog { category: "Reading · T/F/NG P3", wrong_text: "Suy luận quá đà giữa False và Not Given", right_text: "Chỉ chọn False khi bài đọc phủ định trực tiếp", count: 2 },
    SeedErrorLog { category: "Reading · MC Attitude", wrong_text: "Nhầm lẫn thái độ tác giả", right_text: "Bám sát từ chỉ thái độ", count: 2 },
    SeedErrorLog { category: "Listening · Part 3 Trap", wrong_text: "Đồng thuận giả trước yet/however", right_text: "Nghe hết câu phản hồi sau yet", count: 2 },
    SeedErrorLog { category: "Speaking · Part 3", wrong_text: "Sa đà kể chuyện cá nhân", right_text: "Phân tích vĩ mô theo A.R.E.A", count: 2 },
];

pub struct SeedParaphrase {
    pub question_term: &'static str,
    pub passage_paraphrase: &'static str,
    pub source_type: &'static str,
    pub trap_note: &'static str,
}

pub const PARAPHRASE_SEEDS: &[SeedParaphrase] = &[
    SeedParaphrase { question_term: "decline", passage_paraphrase: "fall off, dwindle, shrink", source_type: "Cam 18 T2 · T/F/NG", trap_note: "Phân biệt giảm giá (price) ≠ giảm nhu cầu (demand)" },
    SeedParaphrase { question_term: "children", passage_paraphrase: "youngsters, minors, juveniles", source_type: "Cam 17 T1 · Matching", trap_note: "Cẩn thận bẫy teenager ≠ primary pupil" },
    SeedParaphrase { question_term: "harmful", passage_paraphrase: "detrimental to, adverse effects", source_type: "Study4 #12 · MCQ", trap_note: "Hay gặp trong câu hỏi Not Given / Yes-No" },
    SeedParaphrase { question_term: "urban areas", passage_paraphrase: "metropolitan centers, densely populated cities", source_type: "Cam 16 T3 · Heading", trap_note: "Tránh nhầm với suburban / rural outskirts" },
    SeedParaphrase { question_term: "unexpectedly", passage_paraphrase: "took researchers by surprise, unforeseen", source_type: "Cam 18 T1 · Summary", trap_note: "Kiểm tra xem có từ phủ định ở câu trước không" },
    SeedParaphrase { question_term: "solve the problem", passage_paraphrase: "alleviate the issue, remedy the situation", source_type: "Study4 P3 · Completion", trap_note: "alleviate = giảm bớt ≠ loại bỏ hoàn toàn" },
];

pub struct SeedDictation {
    pub level: &'static str,
    pub section: &'static str,
    pub trap_focus: &'static str,
    pub sentence: &'static str,
    pub meaning_vi: &'static str,
}

// 24 IELTS Listening Dictation sentences (12 for 6.5 + 12 for 8.0)
pub const DICTATION_SEEDS: &[SeedDictation] = &[
    // Track 6.5 (12 câu)
    SeedDictation { level: "6.5", section: "Listening Part 1", trap_focus: "Con số & Ngày tháng", sentence: "The community library opens at quarter to nine on Wednesdays except during August.", meaning_vi: "Thư viện cộng đồng mở cửa lúc 8 giờ 45 vào các ngày thứ Tư trừ tháng Tám." },
    SeedDictation { level: "6.5", section: "Listening Part 1", trap_focus: "Từ đổi ý (actually)", sentence: "I originally booked the seventeenth of October, but I actually need the twenty-seventh.", meaning_vi: "Ban đầu tôi đặt ngày 17 tháng 10, nhưng thực ra tôi cần ngày 27." },
    SeedDictation { level: "6.5", section: "Listening Part 2", trap_focus: "Bẫy số nhiều -s", sentence: "Visitors must leave large backpacks and umbrellas in the lockers near the entrance.", meaning_vi: "Khách tham quan phải gửi ba lô lớn và ô ở tủ khóa gần lối vào." },
    SeedDictation { level: "6.5", section: "Listening Part 2", trap_focus: "Bẫy đuôi -ed", sentence: "The historic clock tower was completely restored after funding was approved last spring.", meaning_vi: "Tháp đồng hồ lịch sử đã được trùng tu hoàn toàn sau khi kinh phí được duyệt mùa xuân năm ngoái." },
    SeedDictation { level: "6.5", section: "Listening Part 2", trap_focus: "Bẫy phủ định (rather than)", sentence: "Most commuters chose the express ferry rather than waiting for the delayed suburban train.", meaning_vi: "Phần lớn người đi làm chọn phà cao tốc thay vì chờ chuyến tàu ngoại ô bị trễ." },
    SeedDictation { level: "6.5", section: "Listening Part 3", trap_focus: "Bẫy đổi ý (however)", sentence: "The survey looked promising at first; however, several participants failed to return their forms.", meaning_vi: "Khảo sát ban đầu có vẻ khả quan; tuy nhiên một vài người tham gia đã không nộp lại phiếu." },
    SeedDictation { level: "6.5", section: "Listening Part 3", trap_focus: "Bẫy số nhiều -s", sentence: "Budget constraints can seriously affect the outcomes of the student research projects.", meaning_vi: "Hạn chế ngân sách có thể ảnh hưởng nghiêm trọng đến kết quả các dự án nghiên cứu sinh viên." },
    SeedDictation { level: "6.5", section: "Listening Part 3", trap_focus: "Bẫy đuôi -ed", sentence: "Both tutors agreed that the presentation improved once unnecessary charts were removed.", meaning_vi: "Cả hai giảng viên đều đồng ý bài thuyết trình đã tốt hơn sau khi bỏ các biểu đồ thừa." },
    SeedDictation { level: "6.5", section: "Listening Part 4", trap_focus: "Bẫy số nhiều -s", sentence: "Coastal wetlands protect nearby towns by absorbing storm surges and filtering pollutants.", meaning_vi: "Đất ngập nước ven biển bảo vệ thị trấn lân cận bằng cách hấp thụ nước dâng bão và lọc chất ô nhiễm." },
    SeedDictation { level: "6.5", section: "Listening Part 4", trap_focus: "Con số & Tỷ lệ", sentence: "Nearly sixty-five percent of urban households sorted plastic bottles before collection.", meaning_vi: "Gần 65% hộ gia đình đô thị đã phân loại chai nhựa trước khi thu gom." },
    SeedDictation { level: "6.5", section: "Listening Part 4", trap_focus: "Bẫy đuôi -ed", sentence: "Early wooden bridges collapsed frequently because engineers underestimated seasonal floods.", meaning_vi: "Những cây cầu gỗ đầu tiên thường sập vì kỹ sư đánh giá thấp lũ theo mùa." },
    SeedDictation { level: "6.5", section: "Listening Part 4", trap_focus: "Từ chuyển ý (instead of)", sentence: "Architects recommended natural ventilation instead of installing expensive air conditioning units.", meaning_vi: "Kiến trúc sư đề xuất thông gió tự nhiên thay vì lắp đặt các máy điều hòa đắt đỏ." },

    // Track 8.0 (12 câu)
    SeedDictation { level: "8.0", section: "Listening Part 3", trap_focus: "Bẫy phủ định ngầm", sentence: "Although the pilot study seemed inconclusive, its methodology nonetheless revealed subtle biases.", meaning_vi: "Dù nghiên cứu thử nghiệm có vẻ chưa ngã ngũ, phương pháp của nó vẫn bộc lộ những thiên lệch tinh vi." },
    SeedDictation { level: "8.0", section: "Listening Part 3", trap_focus: "Bẫy đổi ý (yet)", sentence: "I appreciate why the committee endorsed the proposal, yet the long-term maintenance costs remain prohibitive.", meaning_vi: "Tôi hiểu vì sao ủy ban ủng hộ đề xuất, nhưng chi phí bảo trì dài hạn vẫn quá đắt đỏ." },
    SeedDictation { level: "8.0", section: "Listening Part 3", trap_focus: "Bẫy số nhiều -s", sentence: "Discrepancies between satellite measurements and ground sensors complicated the researchers' assumptions.", meaning_vi: "Sự chênh lệch giữa số đo vệ tinh và cảm biến mặt đất đã làm phức tạp hóa các giả định của nhóm nghiên cứu." },
    SeedDictation { level: "8.0", section: "Listening Part 3", trap_focus: "Bẫy đuôi -ed", sentence: "Several manuscripts previously dismissed as forgeries were later authenticated through pigment analysis.", meaning_vi: "Nhiều bản thảo từng bị bác bỏ là giả mạo sau đó đã được xác thực qua phân tích sắc tố." },
    SeedDictation { level: "8.0", section: "Listening Part 4", trap_focus: "Học thuật Part 4", sentence: "Rapid urbanization has inadvertently fragmented wildlife corridors across temperate grasslands.", meaning_vi: "Đô thị hóa nhanh đã vô tình chia cắt các hành lang động vật hoang dã trên thảo nguyên ôn đới." },
    SeedDictation { level: "8.0", section: "Listening Part 4", trap_focus: "Bẫy số nhiều -s", sentence: "Marine biologists observed substantial shifts in migration patterns following warmer ocean currents.", meaning_vi: "Các nhà sinh vật học biển quan sát thấy sự dịch chuyển đáng kể trong tuyến di cư theo dòng hải lưu ấm hơn." },
    SeedDictation { level: "8.0", section: "Listening Part 4", trap_focus: "Con số & Niên đại", sentence: "Excavations conducted between nineteen eighty-four and nineteen ninety-two uncovered three distinct settlements.", meaning_vi: "Các cuộc khai quật tiến hành từ 1984 đến 1992 đã phát lộ ba khu định cư riêng biệt." },
    SeedDictation { level: "8.0", section: "Listening Part 4", trap_focus: "Bẫy phủ định (scarcely)", sentence: "Without subsidized insulation grants, low-income tenants could scarcely afford energy-efficient upgrades.", meaning_vi: "Nếu thiếu trợ cấp cách nhiệt, người thuê thu nhập thấp khó có thể chi trả nâng cấp tiết kiệm năng lượng." },
    SeedDictation { level: "8.0", section: "Listening Part 4", trap_focus: "Bẫy đuôi -ed", sentence: "Once industrial runoff was diverted from the estuary, native shellfish populations gradually rebounded.", meaning_vi: "Khi dòng thải công nghiệp được chuyển hướng khỏi cửa sông, quần thể sò bản địa dần phục hồi." },
    SeedDictation { level: "8.0", section: "Listening Part 4", trap_focus: "Từ chuyển ý (conversely)", sentence: "Monoculture plantations maximize short-term timber yields; conversely, mixed forests bolster ecological resilience.", meaning_vi: "Rừng trồng độc canh tối đa hóa sản lượng gỗ ngắn hạn; ngược lại, rừng hỗn giao tăng sức chống chịu sinh thái." },
    SeedDictation { level: "8.0", section: "Listening Part 4", trap_focus: "Bẫy số nhiều -s", sentence: "Archaeological artifacts preserved in peat bogs provide invaluable insights into prehistoric diets.", meaning_vi: "Hiện vật khảo cổ bảo quản trong đầm lầy than bùn cung cấp hiểu biết vô giá về chế độ ăn tiền sử." },
    SeedDictation { level: "8.0", section: "Listening Part 4", trap_focus: "Học thuật Part 4", sentence: "Regulatory oversight failed to anticipate how algorithmic trading amplifies market volatility during downturns.", meaning_vi: "Cơ quan giám sát đã không lường trước cách giao dịch thuật toán khuếch đại biến động thị trường khi suy thoái." },
];

pub struct SeedSpeakingCue {
    pub week_num: i64,
    pub level: &'static str,
    pub topic: &'static str,
    pub cue_title: &'static str,
    pub bullets: [&'static str; 4],
    pub part3: [&'static str; 4],
    pub phrases: [&'static str; 5],
}

// 48 Speaking Cue Cards: 24 for Track 6.5 + 24 for Track 8.0 (aligned to Weeks 1-24)
pub const SPEAKING_SEEDS: &[SeedSpeakingCue] = &[
    // --- TRACK 6.5 (24 cues, Week 1..24) ---
    SeedSpeakingCue { week_num: 1, level: "6.5", topic: "Education & Habits", cue_title: "Describe a skill you learned when you were a child.", bullets: ["what the skill was", "who taught you this skill", "how long it took to learn", "and explain why it is still useful today."], part3: ["What practical skills should schools teach children today?", "Is it easier for children or adults to learn new skills?", "How has technology changed the way people acquire skills?", "Do parents put too much pressure on children to learn early?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "pick up a practical habit", "step-by-step guidance", "build confidence early", "remain relevant in adulthood"] },
    SeedSpeakingCue { week_num: 2, level: "6.5", topic: "People & Work", cue_title: "Describe a person you admire for their hard work.", bullets: ["who this person is", "how you know them", "what kind of work they do", "and explain why you admire their dedication."], part3: ["What qualities make someone a reliable worker?", "Why do some people work overtime even without extra pay?", "Should companies reward effort or only final results?", "How does work-life balance differ across generations?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "stay committed despite setbacks", "a strong work ethic", "lead by example", "strike a healthy balance"] },
    SeedSpeakingCue { week_num: 3, level: "6.5", topic: "Places & Lifestyle", cue_title: "Describe a quiet place you like to go to relax.", bullets: ["where this place is", "how often you go there", "what you usually do there", "and explain why it helps you relax."], part3: ["Why do people in big cities find it hard to relax?", "Should employers provide relaxation spaces at work?", "Are public parks well maintained in your country?", "How do young and older people relax differently?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "unwind after a hectic week", "away from constant notifications", "recharge my mental batteries", "green open spaces"] },
    SeedSpeakingCue { week_num: 4, level: "6.5", topic: "Technology", cue_title: "Describe a piece of technology you find very useful.", bullets: ["what the device or app is", "when you started using it", "how you use it in daily life", "and explain why you could hardly do without it."], part3: ["What are the drawbacks of relying too much on smartphones?", "Has technology made communication more superficial?", "Should children have limits on daily screen time?", "Which invention has changed home life the most?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "streamline daily routines", "keep track of deadlines", "rely heavily on", "face-to-face interaction"] },
    SeedSpeakingCue { week_num: 5, level: "6.5", topic: "Events & Decisions", cue_title: "Describe an important decision you made with someone else.", bullets: ["what the decision was", "who you made it with", "how you reached the agreement", "and explain how you felt about the result."], part3: ["Why is it difficult for groups to agree on major decisions?", "Should parents let teenagers make their own career choices?", "Do leaders make better decisions alone or with a team?", "What makes people regret a decision later?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "weigh the pros and cons", "reach a sensible compromise", "see things from another angle", "turn out better than expected"] },
    SeedSpeakingCue { week_num: 6, level: "6.5", topic: "Environment", cue_title: "Describe something you do to help protect the environment.", bullets: ["what action you take", "how long you have been doing it", "whether it is easy to keep up", "and explain what difference it makes."], part3: ["Are individuals or governments more responsible for pollution?", "Why do some people ignore recycling rules?", "How can schools encourage eco-friendly habits?", "Will electric vehicles solve urban air pollution?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "cut down on single-use plastic", "make a small but steady impact", "raise public awareness", "long-term environmental benefit"] },
    SeedSpeakingCue { week_num: 7, level: "6.5", topic: "Health & Sport", cue_title: "Describe an outdoor activity you enjoy doing.", bullets: ["what the activity is", "where and when you do it", "who you usually do it with", "and explain why you enjoy it so much."], part3: ["Why do many office workers live a sedentary lifestyle?", "How can cities encourage residents to exercise more?", "Are team sports better than individual workouts?", "Should physical education be compulsory at university?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "relieve built-up stress", "stay physically active", "fresh air and scenery", "boost overall well-being"] },
    SeedSpeakingCue { week_num: 8, level: "6.5", topic: "Books & Learning", cue_title: "Describe a book or article that taught you something useful.", bullets: ["what the book or article was about", "when you read it", "what key lesson you learned", "and explain why you would recommend it."], part3: ["Do people read less now because of short videos?", "Are e-books likely to replace printed books completely?", "What kinds of books are most popular in your country?", "How can parents foster a reading habit in children?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "broaden my perspective", "easy to digest", "apply the advice immediately", "lasting impression"] },
    SeedSpeakingCue { week_num: 9, level: "6.5", topic: "Travel & Transport", cue_title: "Describe a memorable journey you took by public transport.", bullets: ["where you went", "what transport you used", "what happened during the trip", "and explain why it was memorable."], part3: ["What makes public transport unattractive in some cities?", "Should public buses and metros be free for students?", "How can traffic congestion be reduced during rush hour?", "Will high-speed trains replace domestic flights?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "avoid heavy traffic jams", "surprisingly punctual and clean", "scenic route along the way", "practical alternative to driving"] },
    SeedSpeakingCue { week_num: 10, level: "6.5", topic: "Culture & Food", cue_title: "Describe a traditional meal you enjoy eating with family or friends.", bullets: ["what the dish or meal is", "when people usually eat it", "how it is prepared", "and explain why it is special to you."], part3: ["Why is fast food becoming more popular among young people?", "Is it important to preserve traditional cooking methods?", "How do shared meals strengthen family bonds?", "Should schools teach basic cooking skills?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "passed down through generations", "fresh local ingredients", "bring everyone around the table", "a sense of cultural identity"] },
    SeedSpeakingCue { week_num: 11, level: "6.5", topic: "Communication", cue_title: "Describe a conversation that changed your mind about something.", bullets: ["who you talked with", "what topic you discussed", "what argument convinced you", "and explain how your view changed afterwards."], part3: ["Why do people sometimes refuse to admit they are wrong?", "Is online debate helpful or harmful for understanding others?", "What makes someone a good listener?", "How do cultural differences affect communication styles?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "challenge my initial assumption", "put forward a clear point", "listen with an open mind", "shift my viewpoint"] },
    SeedSpeakingCue { week_num: 12, level: "6.5", topic: "Money & Shopping", cue_title: "Describe something you bought that you were very happy with.", bullets: ["what you bought", "where you bought it", "why you needed it", "and explain why it turned out to be good value."], part3: ["Why do people often buy things they do not really need?", "How has online shopping affected small local shops?", "Do advertisements influence adults as much as children?", "Is it better to save money or spend it on experiences?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "worth every penny", "compare reviews carefully", "avoid impulse buying", "last for years"] },
    SeedSpeakingCue { week_num: 13, level: "6.5", topic: "Time & Organization", cue_title: "Describe a time when you had to finish a task within a tight deadline.", bullets: ["what the task was", "why the deadline was tight", "how you managed your time", "and explain how you felt when it finished."], part3: ["Why do some people procrastinate even on important tasks?", "Can strict deadlines improve or harm creativity?", "How can students learn better time management?", "Do modern workplaces expect people to work too fast?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "break the task into small chunks", "stay focused under pressure", "meet the deadline on time", "a huge sense of relief"] },
    SeedSpeakingCue { week_num: 14, level: "6.5", topic: "Cities & Architecture", cue_title: "Describe an interesting building you have seen or visited.", bullets: ["where the building is located", "what it looks like", "what it is used for", "and explain why you found it impressive."], part3: ["Should old historic buildings be preserved or replaced?", "What makes a modern city pleasant to live in?", "Why do many skyscrapers look almost identical?", "How does housing design affect community spirit?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "blend traditional and modern styles", "eye-catching landmark", "serve the local community", "preserve architectural heritage"] },
    SeedSpeakingCue { week_num: 15, level: "6.5", topic: "Media & News", cue_title: "Describe a piece of good news you heard recently.", bullets: ["what the news was", "how you heard about it", "who was involved", "and explain why it made you happy."], part3: ["Why do news outlets focus more on bad news than good news?", "How do people verify whether online news is true?", "Are traditional newspapers still relevant today?", "How does constant news consumption affect mental health?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "lift my mood immediately", "spread positive energy", "fact-check reliable sources", "refreshing change from negative headlines"] },
    SeedSpeakingCue { week_num: 16, level: "6.5", topic: "Education Forecast", cue_title: "Describe a course or subject you would like to study in the future.", bullets: ["what subject it is", "where you could study it", "what skills it involves", "and explain why you want to learn it."], part3: ["Should university education be free for everyone?", "Why do many graduates work in fields unrelated to their major?", "How important is lifelong learning in modern careers?", "Are online certificates respected by employers?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "gain a competitive edge", "bridge theory and practice", "adapt to a changing job market", "invest in self-improvement"] },
    SeedSpeakingCue { week_num: 17, level: "6.5", topic: "Work Forecast", cue_title: "Describe an ideal job you would enjoy doing.", bullets: ["what the job is", "what responsibilities it involves", "what skills are required", "and explain why it suits your personality."], part3: ["Is job satisfaction more important than a high salary?", "How will AI change office jobs over the next decade?", "Why do young people change jobs more often today?", "Should governments regulate remote working hours?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "meaningful daily challenges", "collaborative team culture", "room for professional growth", "feel motivated every morning"] },
    SeedSpeakingCue { week_num: 18, level: "6.5", topic: "Society & Volunteering", cue_title: "Describe a time you helped someone in your community.", bullets: ["who you helped", "what problem they had", "how you helped them", "and explain how the experience affected you."], part3: ["Why should young people take part in volunteer work?", "Are people in modern cities less willing to help strangers?", "How can local governments support neighborhood initiatives?", "Should community service be part of school education?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "lend a helping hand", "foster community spirit", "make a tangible difference", "rewarding personal experience"] },
    SeedSpeakingCue { week_num: 19, level: "6.5", topic: "Nature & Weather", cue_title: "Describe a natural place in your country you would recommend to visitors.", bullets: ["where the place is", "what scenery visitors can see", "what activities are available there", "and explain why it is worth visiting."], part3: ["How does tourism harm fragile natural areas?", "Should national parks limit the number of daily visitors?", "Why do people enjoy spending holidays in nature?", "How can eco-tourism benefit local residents?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "breathtaking landscape", "unspoiled natural beauty", "promote responsible tourism", "escape urban noise"] },
    SeedSpeakingCue { week_num: 20, level: "6.5", topic: "Science & Discovery", cue_title: "Describe an interesting scientific fact or invention you learned about.", bullets: ["what the fact or invention is", "where you learned about it", "how it works or helps people", "and explain why you found it fascinating."], part3: ["Should governments spend more money on space or healthcare?", "How can teachers make science more engaging for students?", "Do people trust scientific experts enough today?", "What ethical issues arise from rapid medical advances?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "spark my curiosity", "solve real-world problems", "improve quality of life", "make complex ideas accessible"] },
    SeedSpeakingCue { week_num: 21, level: "6.5", topic: "Goals & Achievement", cue_title: "Describe a goal you set and worked hard to achieve.", bullets: ["what the goal was", "why you chose it", "what obstacles you faced", "and explain how you felt after achieving it."], part3: ["Why do some people give up on their goals too early?", "Is it better to set ambitious goals or realistic ones?", "How can friends or mentors help someone stay motivated?", "Do social media comparisons affect young people's ambitions?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "stick to a consistent routine", "overcome initial frustration", "measure progress week by week", "boost long-term confidence"] },
    SeedSpeakingCue { week_num: 22, level: "6.5", topic: "Arts & Music", cue_title: "Describe a song or piece of music you often listen to.", bullets: ["what kind of music it is", "when you first heard it", "when you usually listen to it", "and explain how it makes you feel."], part3: ["Why do music tastes change as people grow older?", "Should music and art remain compulsory in high schools?", "How has streaming changed the music industry?", "Does background music help or distract when studying?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "create a calm atmosphere", "uplifting melody", "express shared emotions", "help me concentrate"] },
    SeedSpeakingCue { week_num: 23, level: "6.5", topic: "Law & Rules", cue_title: "Describe a rule at school or work that you think is important.", bullets: ["what the rule is", "who has to follow it", "what happens if someone breaks it", "and explain why this rule is necessary."], part3: ["Why do teenagers sometimes dislike strict rules?", "Are workplace dress codes still necessary today?", "What happens when laws are not fairly enforced?", "Should schools involve students when making rules?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "ensure fairness for everyone", "maintain a respectful environment", "prevent unnecessary conflicts", "clear boundaries"] },
    SeedSpeakingCue { week_num: 24, level: "6.5", topic: "Exam Readiness", cue_title: "Describe an experience that taught you patience.", bullets: ["what the situation was", "why it required patience", "how you handled waiting or difficulty", "and explain what lesson you took away."], part3: ["Are people less patient today because of instant technology?", "In which professions is patience most essential?", "How can parents teach patience to young children?", "Does rushing usually lead to more mistakes?"], phrases: ["A.R.E.A: Answer -> Reason -> Example -> Alternative", "stay calm under pressure", "trust gradual progress", "avoid hasty mistakes", "pay off in the long run"] },

    // --- TRACK 8.0 (24 cues, Week 1..24) ---
    SeedSpeakingCue { week_num: 1, level: "8.0", topic: "Hedging & Urbanization", cue_title: "Describe a major change in your neighborhood or city over recent years.", bullets: ["what the change has been", "why it happened", "how local residents reacted", "and evaluate whether the benefits outweigh the drawbacks."], part3: ["To what extent does rapid urbanization erode community cohesion?", "Should municipal planners prioritize heritage conservation over affordable high-rise housing?", "Why do infrastructure upgrades often lag behind population growth?", "How might remote work reshape urban density over the coming decades?"], phrases: ["to a considerable extent", "exacerbate traffic congestion", "erode neighborhood cohesion", "strike a delicate balance", "yield tangible dividends"] },
    SeedSpeakingCue { week_num: 2, level: "8.0", topic: "Media & Attention", cue_title: "Describe a digital habit you deliberately tried to change.", bullets: ["what the habit was", "what prompted you to change it", "what obstacles you encountered", "and explain how changing it affected your focus."], part3: ["Are algorithmic feeds undermining citizens' capacity for sustained critical reading?", "Should tech platforms be legally accountable for addictive interface design?", "Why do awareness campaigns rarely curb excessive screen use on their own?", "How does fragmented attention affect academic performance?"], phrases: ["algorithmic echo chambers", "ubiquitous digital connectivity", "warrant regulatory intervention", "foster sustained concentration", "by no means straightforward"] },
    SeedSpeakingCue { week_num: 3, level: "8.0", topic: "Education & Equity", cue_title: "Describe an educational experience outside a traditional classroom.", bullets: ["what the experience was", "where it took place", "how it differed from conventional schooling", "and explain what lasting insight it gave you."], part3: ["Does an overemphasis on standardized testing stifle genuine intellectual curiosity?", "How can governments narrow the attainment gap between urban and rural schools?", "Should higher education prioritize employability or foundational inquiry?", "Why do vocational tracks still suffer from social stigma in many societies?"], phrases: ["cultivate independent inquiry", "narrow the attainment gap", "overemphasis on standardized testing", "equip learners with transferable skills", "it is widely contended that"] },
    SeedSpeakingCue { week_num: 4, level: "8.0", topic: "Work & Automation", cue_title: "Describe an industry or profession that is changing rapidly.", bullets: ["which industry it is", "what forces are driving the change", "how workers are adapting", "and assess what the future holds for newcomers."], part3: ["Will generative automation widen socioeconomic disparities or create new prosperity?", "Who bears greater responsibility for workforce reskilling: corporations or the state?", "How can companies bolster productivity without triggering occupational burnout?", "Why might human judgment remain indispensable despite smarter algorithms?"], phrases: ["render routine roles obsolete", "mitigate adverse repercussions", "alleviate occupational burnout", "spur sustainable productivity", "inextricably linked to"] },
    SeedSpeakingCue { week_num: 5, level: "8.0", topic: "Environment & Policy", cue_title: "Describe an environmental initiative that impressed you.", bullets: ["what the initiative aimed to solve", "who organized or funded it", "how it was implemented", "and explain why you consider it effective."], part3: ["Why do international climate agreements often fall short of binding enforcement?", "Are carbon taxes fairer than subsidies for green technology?", "How can cities decarbonize transit without penalizing low-income commuters?", "Does consumer boycotting meaningfully alter corporate environmental behavior?"], phrases: ["decarbonize heavy industry", "accelerate the transition toward", "stem the tide of", "inflict irreversible damage on", "allocate public funds judiciously"] },
    SeedSpeakingCue { week_num: 6, level: "8.0", topic: "Public Health", cue_title: "Describe a public health campaign or policy you know about.", bullets: ["what issue it targeted", "how it reached the public", "how effective it appeared to be", "and explain what could make similar policies stronger."], part3: ["Where should governments draw the line between personal liberty and public health mandates?", "Why is preventive well-being consistently underfunded compared with acute care?", "How does socioeconomic status predispose individuals to chronic illnesses?", "Will aging demographics inevitably strain public healthcare capacity?"], phrases: ["promote preventive well-being", "strain public healthcare capacity", "predispose individuals to", "take a heavy toll on", "arguably more cost-effective"] },
    SeedSpeakingCue { week_num: 7, level: "8.0", topic: "Culture & Globalization", cue_title: "Describe a cultural tradition that has adapted to modern times.", bullets: ["what the tradition is", "how it used to be practiced", "how younger generations have modified it", "and explain whether adaptation helps preserve it."], part3: ["Does globalization inevitably homogenize local cultures or spur creative fusion?", "Should governments subsidize traditional arts that cannot survive commercially?", "Why do minority languages disappear faster in urbanized economies?", "How does tourism alter the authenticity of local ceremonies?"], phrases: ["preserve cultural heritage", "gradual cultural homogenization", "strike a delicate balance", "bolster social cohesion", "by no means a panacea"] },
    SeedSpeakingCue { week_num: 8, level: "8.0", topic: "Trust & Journalism", cue_title: "Describe a documentary or investigative report that made you think deeply.", bullets: ["what subject it investigated", "what evidence it presented", "why it stood out from ordinary media", "and explain how it shaped your thinking."], part3: ["What factors have eroded public trust in mainstream journalism?", "How can societies combat misinformation without enabling state censorship?", "Why do sensational narratives spread faster than nuanced analysis?", "Should media literacy be a mandatory school subject?"], phrases: ["erode interpersonal trust", "scrutinize underlying assumptions", "nuanced empirical evidence", "pose an unprecedented threat to", "warrant immediate intervention"] },
    SeedSpeakingCue { week_num: 9, level: "8.0", topic: "AI Ethics", cue_title: "Describe a situation where AI or automated systems made a noticeable difference.", bullets: ["where you encountered the system", "what problem it solved", "what limitations you noticed", "and evaluate how reliable such systems are."], part3: ["Should autonomous algorithms ever make binding decisions in healthcare or justice?", "How can regulators ensure algorithmic transparency without exposing trade secrets?", "Do privacy trade-offs become normalized when convenience increases?", "Who is accountable when an automated system causes harm?"], phrases: ["catalyze technological breakthroughs", "pose an unprecedented threat to", "impose stringent regulations on", "ethical accountability", "to a considerable extent"] },
    SeedSpeakingCue { week_num: 10, level: "8.0", topic: "Economy & Inequality", cue_title: "Describe a public service that has improved significantly in your area.", bullets: ["what the service is", "what shortcomings it previously had", "how it was upgraded", "and explain who benefited most from the improvement."], part3: ["Why does uneven infrastructure perpetuate regional wealth gaps?", "Is privatization of essential utilities ever justified?", "How should municipalities prioritize spending between transit, housing, and parks?", "Does economic growth automatically translate into higher civic well-being?"], phrases: ["allocate public funds judiciously", "exacerbate socioeconomic disparities", "yield tangible dividends", "bridge the generational divide", "sustainable civic infrastructure"] },
    SeedSpeakingCue { week_num: 11, level: "8.0", topic: "Urban Design", cue_title: "Describe a well-designed public space you have visited.", bullets: ["where it is", "what design features stand out", "how diverse groups use the space", "and explain why good urban design matters."], part3: ["How does access to green space affect mental health in megacities?", "Why do car-centric suburbs persist despite environmental warnings?", "Should city centers ban private vehicles completely?", "How can urban planning accommodate aging populations?"], phrases: ["alleviate chronic congestion", "bolster social cohesion", "accessible civic spaces", "walkable urban design", "arguably indispensable"] },
    SeedSpeakingCue { week_num: 12, level: "8.0", topic: "Future of Work", cue_title: "Describe a workplace policy that improved morale or efficiency.", bullets: ["what the policy was", "why it was introduced", "how staff responded", "and evaluate whether it would work in other sectors."], part3: ["Does remote work blur the boundaries between professional and personal life?", "Why do some executives resist flexible schedules despite productivity data?", "How can organizations mentor junior staff effectively in hybrid teams?", "Will the four-day workweek become mainstream?"], phrases: ["blur the boundaries between", "alleviate occupational burnout", "spur sustainable productivity", "gain a decisive competitive edge", "foster intrinsic motivation"] },
    SeedSpeakingCue { week_num: 13, level: "8.0", topic: "Law & Privacy", cue_title: "Describe a rule or law you think needs updating for the digital age.", bullets: ["what the rule or law is", "why it worked in the past", "how technology made it outdated", "and explain how it should be reformed."], part3: ["Why does legislation consistently lag behind technological innovation?", "Should teenagers have stronger digital privacy protections than adults?", "Can heavy fines alone deter tech giants from misusing personal data?", "How do cross-border data flows complicate national laws?"], phrases: ["impose stringent regulations on", "lag behind rapid innovation", "safeguard digital autonomy", "unintended legal loopholes", "warrant structural reform"] },
    SeedSpeakingCue { week_num: 14, level: "8.0", topic: "Critical Thinking", cue_title: "Describe a complex problem you solved by changing your approach.", bullets: ["what the problem was", "why your first approach failed", "how you reframed the issue", "and explain what this taught you about problem-solving."], part3: ["Why do experts sometimes overlook obvious solutions outside their discipline?", "How can education cultivate independent inquiry rather than rote compliance?", "Is failure undervalued as a learning mechanism in competitive societies?", "When is collaborative brainstorming less effective than solitary thinking?"], phrases: ["reframe the underlying issue", "cultivate independent inquiry", "pedagogical innovation", "keep abreast of", "tangible analytical clarity"] },
    SeedSpeakingCue { week_num: 15, level: "8.0", topic: "Housing & Demographics", cue_title: "Describe a challenge young adults face when moving to a large city.", bullets: ["what the challenge is", "why it has intensified recently", "how young people cope with it", "and explain what policy changes could help."], part3: ["Why has housing affordability become a global crisis in major capitals?", "Does delayed homeownership alter family formation and birth rates?", "Should governments cap short-term tourist rentals in residential zones?", "How can secondary cities attract skilled graduates away from overcrowded capitals?"], phrases: ["exacerbate socioeconomic disparities", "strain urban affordability", "alleviate housing pressure", "targeted policy intervention", "by no means simple"] },
    SeedSpeakingCue { week_num: 16, level: "8.0", topic: "Science & Bioethics", cue_title: "Describe a scientific breakthrough that raises both hope and concern.", bullets: ["what the breakthrough is", "what problem it promises to solve", "what ethical risks critics highlight", "and explain where you stand on its use."], part3: ["Should private corporations hold patents on life-saving genetic therapies?", "How should society weigh space exploration budgets against poverty relief?", "Why is public skepticism toward new technologies sometimes justified?", "Who should set ethical boundaries for emerging biotechnology?"], phrases: ["ethical accountability", "catalyze medical breakthroughs", "unintended long-term consequences", "strike a delicate balance", "stringent ethical oversight"] },
    SeedSpeakingCue { week_num: 17, level: "8.0", topic: "Leadership & Influence", cue_title: "Describe a leader or mentor whose decision-making style you respect.", bullets: ["who they are", "what challenging situation they handled", "how they communicated with others", "and explain why their approach was effective."], part3: ["Are charismatic speakers necessarily effective long-term leaders?", "Why do organizations sometimes promote technical experts into poor managers?", "How does transparent communication build institutional resilience?", "Should leaders prioritize consensus or decisive speed during crises?"], phrases: ["bolster institutional resilience", "transparent accountability", "weigh competing priorities", "lead with measured restraint", "yield lasting credibility"] },
    SeedSpeakingCue { week_num: 18, level: "8.0", topic: "Consumerism & Sustainability", cue_title: "Describe an industry that creates excessive waste and how it could reform.", bullets: ["which industry it is", "why waste occurs in its supply chain", "what alternatives already exist", "and explain what prevents faster change."], part3: ["Why does fast fashion thrive despite widespread awareness of its footprint?", "Should manufacturers be legally responsible for post-consumer recycling?", "Can circular economy models remain profitable at scale?", "How do marketing strategies normalize planned obsolescence?"], phrases: ["deplete finite natural resources", "stem the tide of", "accelerate the transition toward", "planned obsolescence", "inflict irreversible damage on"] },
    SeedSpeakingCue { week_num: 19, level: "8.0", topic: "Arts & Public Funding", cue_title: "Describe an artistic project or museum exhibit that left a strong impression.", bullets: ["what the exhibit or project was", "where you experienced it", "what message it conveyed", "and explain why public access to art matters."], part3: ["Should cash-strapped cities spend tax revenue on public art instead of roads?", "Can digital replicas ever match the cultural value of physical museums?", "Why do governments sometimes censor controversial artworks?", "How do arts programs contribute to economic innovation?"], phrases: ["foster civic identity", "allocate public funds judiciously", "transcend linguistic barriers", "stimulate creative inquiry", "arguably undervalued"] },
    SeedSpeakingCue { week_num: 20, level: "8.0", topic: "Intergenerational Ties", cue_title: "Describe something valuable an older person taught you about life or work.", bullets: ["who the person was", "how you got to know them", "what perspective they shared", "and explain how that insight applies today."], part3: ["How can societies bridge the generational divide in rapidly digitizing workplaces?", "Are nursing homes replacing traditional family care, and at what social cost?", "Why do younger and older voters often disagree on public spending priorities?", "What skills do older workers bring that automation cannot replicate?"], phrases: ["bridge the generational divide", "erode interpersonal trust", "accumulated institutional wisdom", "inextricably linked to", "promote mutual empathy"] },
    SeedSpeakingCue { week_num: 21, level: "8.0", topic: "Global Challenges", cue_title: "Describe a global issue that requires cross-border cooperation.", bullets: ["what the issue is", "why individual countries cannot solve it alone", "what progress has been made", "and explain what still blocks faster action."], part3: ["Why do national interests often derail global scientific or climate cooperation?", "Have international institutions lost credibility among younger generations?", "How can wealthier nations support developing economies equitably?", "Does economic interdependence reduce or heighten geopolitical risk?"], phrases: ["cross-border collaboration", "mitigate adverse repercussions", "short-sighted national self-interest", "equitable burden-sharing", "warrant coordinated action"] },
    SeedSpeakingCue { week_num: 22, level: "8.0", topic: "Psychology & Success", cue_title: "Describe a setback that unexpectedly led to long-term improvement.", bullets: ["what the setback was", "how you initially reacted", "what changes you made afterwards", "and explain why it proved valuable."], part3: ["Does modern culture glorify hustle at the expense of sustainable mental health?", "How can schools teach resilience without normalizing excessive stress?", "Why do people often overestimate short-term failures?", "What role does luck play relative to deliberate practice?"], phrases: ["take a heavy toll on", "bolster personal resilience", "deliberate iterative practice", "recalibrate expectations", "yield tangible dividends"] },
    SeedSpeakingCue { week_num: 23, level: "8.0", topic: "Language & Thought", cue_title: "Describe an idiom or expression in another language that fascinated you.", bullets: ["what the expression is", "where you encountered it", "what cultural idea it reflects", "and explain why learning nuances matters."], part3: ["Does learning a second language genuinely alter how people perceive the world?", "Will real-time AI translation reduce the motivation to learn foreign languages?", "Should minority dialects be preserved in formal education?", "Why is pragmatic nuance harder to master than grammar?"], phrases: ["nuanced cultural expression", "transcend literal translation", "ubiquitous translation tools", "cultivate cross-cultural empathy", "by no means redundant"] },
    SeedSpeakingCue { week_num: 24, level: "8.0", topic: "Synthesis & Future", cue_title: "Describe a long-term trend you think will define the next twenty years.", bullets: ["what the trend is", "what early signs are visible today", "which groups will adapt best", "and evaluate both its promise and its risks."], part3: ["How can education prepare students for careers that do not yet exist?", "Will demographic aging slow global innovation or shift its focus?", "What makes predictions about technology notoriously unreliable?", "How can individuals maintain agency amid rapid structural change?"], phrases: ["inextricably linked to", "strike a delicate balance", "equip citizens with transferable skills", "mitigate adverse repercussions", "to a considerable extent"] },
];
