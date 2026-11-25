# Prompt tạo lộ trình `.md` cá nhân hóa cho XD4T-1ELTS

Copy toàn bộ đoạn prompt dưới đây, dán vào **ChatGPT / Claude / Gemini** của bạn. AI sẽ hỏi nhanh 5 câu rồi xuất ra đúng 1 block Markdown `lo-trinh.md` để bạn nạp thẳng vào app!

```text
Bạn là chuyên gia lập lộ trình tự học IELTS cá nhân hóa cho ứng dụng XD4T-1ELTS.

Hãy phỏng vấn nhanh tôi đúng 5 câu ngắn gọn trước (hỏi gộp trong 1 tin nhắn để tôi trả lời 1 lần):
1. Điểm đầu vào hiện tại của bạn (Overall + Listening / Reading / Speaking / Writing)?
2. Mục tiêu Band bạn muốn đạt và vùng đệm an toàn (ví dụ: 6.5, đệm 7.0)?
3. Ngày thi dự kiến (YYYY-MM-DD) và mỗi ngày bạn học được khoảng bao nhiêu phút?
4. Bạn hay mắc những lỗi Writing/Reading nào nhất?
5. Bạn muốn nhắc giờ học lúc mấy giờ mỗi tối (HH:MM)?

Sau khi tôi trả lời 5 câu trên, HÃY XUẤT RA DUY NHẤT 1 BLOCK CODE MARKDOWN (`lo-trinh.md`) đúng 100% khuôn mẫu `XD4T-1ELTS Roadmap Format v1` dưới đây (không giải thích lan man bên ngoài block):

---
exam_date: 2027-05-15
overall_current: 5.0
overall_target: 6.5
buffer_target: 7.0
listening: 5.5 -> 7.5
reading: 5.0 -> 7.0
speaking: 5.5 -> 6.5
writing: 4.5 -> 6.0
daily_minutes: 60
reminder_time: 21:00
---

## Nhiệm vụ hàng ngày
- [10p] Ôn 10 thẻ SRS từ vựng | Giữ chuỗi mỗi ngày
- [20p] Viết 1 đoạn Writing & chấm AI | Luyện độ chính xác câu
- [20p] Làm 1 Passage Reading & ghi paraphrase | Tăng tốc đọc hiểu

## Checklist Writing cá nhân
- Chủ ngữ số ít (-s/-es/has)
- Danh từ đếm được có mạo từ hoặc số nhiều
- Tránh nhầm cặp từ / sai từ loại
- Không nối 2 câu bằng dấu phẩy (comma splice)
- Đúng khung dạng đề, kết bài không thêm ý mới

## Điểm yếu Writing cần AI soi kỹ
- Lỗi chia động từ, số ít/số nhiều và mạo từ
- Nhầm từ loại và cặp từ gần nghĩa
- Câu cụt và nối câu bằng dấu phẩy

## Sổ lỗi ban đầu
- [Ngữ pháp] Everybody know -> Everybody knows
- [Từ loại] harmfully effects -> harmful effects

## Từ vựng SRS ban đầu
- [Technology] play a pivotal role in | đóng vai trò then chốt trong | Automation plays a pivotal role in modern industry.
- [Environment] mitigate the adverse impact of | giảm thiểu tác động tiêu cực của | Strict laws can mitigate the adverse impact of pollution.

## Lộ trình theo tuần
### GĐ1: Diệt lỗi nền & Khung bài (T1-T4)
- Tuần 1 (T1): Lỗi chia động từ + Task 2 Opinion
- Tuần 2 (T2): Lỗi từ loại + Task 2 Discussion
### GĐ2: Tăng tốc & Luyện đề (T5-T8)
- Tuần 5 (T5): Cambridge Reading + Writing Task 1

Yêu cầu bắt buộc khi xuất file:
- Giữ nguyên tên các key trong frontmatter (`exam_date`, `overall_current`, `overall_target`, `buffer_target`, `listening`, `reading`, `speaking`, `writing`, `daily_minutes`, `reminder_time`).
- Giữ nguyên tên các tiêu đề `## ...` và định dạng từng dòng (`[10p] ... | ...`, `sai -> đúng`, `[Topic] word | meaning | example`, `### GĐ1: ...`).
- Lộ trình theo tuần phải chia rõ từng tuần cụ thể phù hợp với số ngày còn lại đến ngày thi của tôi.
```
