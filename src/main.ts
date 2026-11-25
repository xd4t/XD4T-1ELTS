import './styles.css';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { openUrl } from '@tauri-apps/plugin-opener';
import type {
  DashboardState,
  GradeResponseDto,
  GradeResult,
  VocabCard,
  WritingPrompt,
} from './types';

let dash: DashboardState | null = null;
let activeTab: 'today' | 'writing' | 'vocab' | 'roadmap' = 'today';
let writingMode = 'paragraph_90';
let selectedPromptId = 'p90-1';
let hintOpen = false;
let lastGrade: GradeResult | null = null;
let lastEndpoint = '';
let isGrading = false;
let dueCards: VocabCard[] = [];
let cardIdx = 0;
let cardFlipped = false;
let draftEssay = '';

let pomoSecs = 15 * 60;
let pomoTimer: number | null = null;

function esc(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function getCurrentPrompt(): WritingPrompt | undefined {
  if (!dash) return undefined;
  return (
    dash.prompts.find((p) => p.id === selectedPromptId) ||
    dash.prompts.find((p) => p.mode === writingMode) ||
    dash.prompts[0]
  );
}

function renderApp() {
  if (!dash) return;
  const isLight = dash.settings.theme === 'light';
  document.documentElement.classList.toggle('light', isLight);
  const doneQuests = dash.quests.filter((q) => q.done).length;
  const totalQuests = dash.quests.length || 1;
  const questPct = Math.round((doneQuests / totalQuests) * 100);

  const curPrompt = getCurrentPrompt();
  const modePrompts = dash.prompts.filter((p) => p.mode === writingMode);

  const p1 = dash.roadmap.filter((r) => r.phase === 1);
  const p2 = dash.roadmap.filter((r) => r.phase === 2);
  const p3 = dash.roadmap.filter((r) => r.phase === 3);
  const p4 = dash.roadmap.filter((r) => r.phase === 4);
  const pct = (arr: typeof p1) =>
    arr.length ? Math.round((arr.filter((x) => x.done).length / arr.length) * 100) : 0;

  const curCard = dueCards[cardIdx];

  const root = document.getElementById('app')!;
  root.innerHTML = `
  <div class="app-shell ${isLight ? 'light' : ''}" id="shell">
    <div class="titlebar">
      <div class="tb-left">
        <div class="brand-pill">XD4T // IELTS 6.5</div>
        <div class="stat-pill"><span>THI ${esc(dash.settings.exam_date)}:</span> <b>${dash.days_until_exam}d</b></div>
        <div class="stat-pill"><span>STREAK:</span> <b>${dash.streak} ngày</b></div>
        <div class="stat-pill"><span>API:</span> <b>${dash.settings.has_api_key ? 'SẴN SÀNG' : 'CHƯA CÓ KEY'}</b></div>
      </div>
      <div class="tb-right">
        <button class="btn btn-sm" id="pomoBtn" title="Bật/tạm dừng đếm ngược 15 phút">⏱ ${formatPomo(pomoSecs)}</button>
        <button class="btn btn-sm" id="themeBtn">${isLight ? 'ĐỔI NỀN: ĐEN' : 'ĐỔI NỀN: TRẮNG'}</button>
        <button class="btn btn-sm" id="openSettingsBtn">CÀI ĐẶT</button>
      </div>
    </div>

    <div class="nav">
      <button class="nav-tab ${activeTab === 'today' ? 'active' : ''}" data-tab="today">[1] HÔM NAY</button>
      <button class="nav-tab ${activeTab === 'writing' ? 'active' : ''}" data-tab="writing">[2] WRITING + AI</button>
      <button class="nav-tab ${activeTab === 'vocab' ? 'active' : ''}" data-tab="vocab">[3] TỪ VỰNG SRS (${dueCards.length})</button>
      <button class="nav-tab ${activeTab === 'roadmap' ? 'active' : ''}" data-tab="roadmap">[4] LỘ TRÌNH & SỔ LỖI</button>
      <div class="nav-meta">Mục tiêu Band ${dash.settings.target_band.toFixed(1)}</div>
    </div>

    <div class="content">
      <!-- TAB 1: TODAY -->
      <div class="panel ${activeTab === 'today' ? 'active' : ''}" id="tab-today">
        <div class="grid-4" style="margin-bottom:12px;">
          ${dash.skill_bands
            .map((b) => {
              const fill = Math.min(100, Math.max(10, Math.round((b.current / 9.0) * 100)));
              return `
              <div class="band-card">
                <div class="band-name">${esc(b.skill)}${b.skill === 'Writing' ? ' ★' : ''}</div>
                <div class="band-val">${b.current.toFixed(1)} <small>→ ${b.target.toFixed(1)}</small></div>
                <div class="bar-track"><div class="bar-fill" style="width:${fill}%"></div></div>
              </div>`;
            })
            .join('')}
        </div>

        <div class="grid-2">
          <div class="box">
            <div class="box-title">
              <span>NHIỆM VỤ HÔM NAY (30 PHÚT · BẤM ĐỂ TICK)</span>
              <span>${doneQuests}/${totalQuests} XONG</span>
            </div>
            <div class="bar-track" style="margin-bottom:10px;"><div class="bar-fill" style="width:${questPct}%"></div></div>
            ${dash.quests
              .map(
                (q) => `
              <div class="quest ${q.done ? 'done' : ''}" data-quest-id="${q.id}">
                <div class="q-check">${q.done ? '✓' : ''}</div>
                <div>
                  <div class="q-title">${esc(q.title)}</div>
                  <div class="q-meta">${esc(q.meta)}</div>
                </div>
                <div class="q-tag">${String(q.minutes).padStart(2, '0')} PHÚT</div>
              </div>`
              )
              .join('')}
          </div>

          <div class="box">
            <div class="box-title">
              <span>HEATMAP CHUỖI HỌC (10 TUẦN)</span>
              <span>30p/ngày</span>
            </div>
            <div style="font-size:11.5px;color:var(--m-muted);">
              Mỗi ô = 1 ngày. Hoàn thành tối thiểu 1 nhiệm vụ (≥10 phút) để sáng ô & giữ streak.
            </div>
            <div class="heatmap">
              ${dash.heatmap
                .map((d) => {
                  const lvl =
                    d.minutes >= 30 ? 4 : d.minutes >= 20 ? 3 : d.minutes >= 10 ? 2 : d.minutes > 0 ? 1 : 0;
                  return `<div class="hm-cell ${lvl ? 'hm-' + lvl : ''}" title="${d.date}: ${d.minutes} phút"></div>`;
                })
                .join('')}
            </div>
            <div style="display:flex;justify-content:space-between;align-items:center;margin-top:10px;font-size:11px;color:var(--m-muted);">
              <span>Ít</span>
              <div style="display:flex;gap:3px;">
                <span style="width:10px;height:10px;background:var(--m-heat-0);border:1px solid var(--m-line);display:inline-block;"></span>
                <span style="width:10px;height:10px;background:var(--m-heat-1);border:1px solid var(--m-line);display:inline-block;"></span>
                <span style="width:10px;height:10px;background:var(--m-heat-2);border:1px solid var(--m-line);display:inline-block;"></span>
                <span style="width:10px;height:10px;background:var(--m-heat-3);border:1px solid var(--m-line);display:inline-block;"></span>
                <span style="width:10px;height:10px;background:var(--m-heat-4);border:1px solid var(--m-line);display:inline-block;"></span>
              </div>
              <span>Nhiều</span>
            </div>
          </div>
        </div>
      </div>

      <!-- TAB 2: WRITING + AI -->
      <div class="panel ${activeTab === 'writing' ? 'active' : ''}" id="tab-writing">
        <div class="mode-row">
          <button class="btn btn-sm mode-btn ${writingMode === 'paragraph_90' ? 'active' : ''}" data-wmode="paragraph_90">ĐOẠN 90 TỪ</button>
          <button class="btn btn-sm mode-btn ${writingMode === 'task1' ? 'active' : ''}" data-wmode="task1">TASK 1 (150+)</button>
          <button class="btn btn-sm mode-btn ${writingMode === 'task2' ? 'active' : ''}" data-wmode="task2">TASK 2 (250+)</button>
          <select class="select-input" id="promptSelect">
            ${modePrompts
              .map(
                (p) =>
                  `<option value="${esc(p.id)}" ${curPrompt?.id === p.id ? 'selected' : ''}>${esc(p.title)}</option>`
              )
              .join('')}
          </select>
          <button class="btn btn-sm" style="margin-left:auto;" id="toggleHintBtn">+ GỢI Ý KHUNG BÀI 6.5</button>
        </div>

        <div class="grid-2">
          <div class="box">
            <div class="box-title">
              <span>ĐỀ BÀI & KHUNG VIẾT</span>
              <span id="wordCount">0 / ${curPrompt?.target_words || 90} từ</span>
            </div>
            <div class="prompt-box"><b>Đề:</b> ${esc(curPrompt?.prompt || '')}</div>
            <div class="hint-box ${hintOpen ? 'open' : ''}" id="hintBox">${esc(curPrompt?.outline_hint || '')}</div>
            <textarea class="editor" id="essayInput" placeholder="Nhập bài viết tiếng Anh của bạn tại đây..."></textarea>
            <div style="border:1px solid var(--m-line);background:var(--m-bg);padding:8px;margin-top:8px;font-size:11px;">
              <div style="font-weight:700;color:var(--m-muted);margin-bottom:5px;text-transform:uppercase;">CHECKLIST SOÁT LỖI 3 PHÚT TRƯỚC KHI NỘP:</div>
              <div style="display:grid;grid-template-columns:1fr 1fr;gap:4px;">
                <label style="display:flex;gap:5px;align-items:flex-start;cursor:pointer;"><input type="checkbox" class="chk-3m" /> <span>1. Chủ ngữ số ít (-s/-es/has)</span></label>
                <label style="display:flex;gap:5px;align-items:flex-start;cursor:pointer;"><input type="checkbox" class="chk-3m" /> <span>2. Danh từ đếm được (a/an/the/-s)</span></label>
                <label style="display:flex;gap:5px;align-items:flex-start;cursor:pointer;"><input type="checkbox" class="chk-3m" /> <span>3. chance/change, choose/choice, effect/affect</span></label>
                <label style="display:flex;gap:5px;align-items:flex-start;cursor:pointer;"><input type="checkbox" class="chk-3m" /> <span>4. Tính từ/từ loại (harmful, In addition)</span></label>
                <label style="display:flex;gap:5px;align-items:flex-start;cursor:pointer;"><input type="checkbox" class="chk-3m" /> <span>5. Không comma splice / câu cụt</span></label>
                <label style="display:flex;gap:5px;align-items:flex-start;cursor:pointer;"><input type="checkbox" class="chk-3m" /> <span>6. Đúng khung, kết bài không thêm ý mới</span></label>
              </div>
            </div>
            <div style="display:flex;justify-content:space-between;align-items:center;margin-top:8px;gap:6px;flex-wrap:wrap;">
              <button class="btn btn-sm" id="copyPromptBtn">COPY PROMPT CHẤM AI</button>
              <button class="btn btn-primary" id="gradeBtn" ${isGrading ? 'disabled' : ''}>${isGrading ? 'ĐANG CHẤM AI...' : 'CHẤM BAND 6.5 NGAY →'}</button>
            </div>
            <div id="writingStatus" style="font-size:11px;color:var(--m-muted);margin-top:6px;"></div>
          </div>

          <div class="box" id="gradePanel">
            <div class="box-title">
              <span>KẾT QUẢ CHẤM AI (4 TIÊU CHÍ)</span>
              <span>ƯỚC LƯỢNG: <b>${lastGrade ? lastGrade.overall.toFixed(1) : '—'}</b></span>
            </div>
            <div class="score-grid">
              <div class="score-box"><b>${lastGrade ? lastGrade.ta.toFixed(1) : '—'}</b><span>TA</span></div>
              <div class="score-box"><b>${lastGrade ? lastGrade.cc.toFixed(1) : '—'}</b><span>CC</span></div>
              <div class="score-box"><b>${lastGrade ? lastGrade.lr.toFixed(1) : '—'}</b><span>LR</span></div>
              <div class="score-box"><b>${lastGrade ? lastGrade.gra.toFixed(1) : '—'}</b><span>GRA</span></div>
            </div>
            ${
              lastGrade
                ? `
              ${lastGrade.summary_vi ? `<div style="font-size:11.5px;margin-bottom:8px;"><b>Nhận xét:</b> ${esc(lastGrade.summary_vi)}</div>` : ''}
              <div style="font-size:11.5px;margin-bottom:8px;">
                <b>Sửa lỗi trực tiếp:</b>
                <div style="border:1px solid var(--m-line);background:var(--m-bg);padding:8px;margin-top:4px;line-height:1.65;">
                  ${lastGrade.corrections
                    .map(
                      (c) =>
                        `<div style="margin-bottom:4px;"><span class="err-del">${esc(c.wrong)}</span> → <span class="ok-add">${esc(c.right)}</span> <span style="color:var(--m-muted);font-size:10.5px;">[${esc(c.category)}: ${esc(c.reason)}]</span></div>`
                    )
                    .join('') || 'Không có lỗi đáng kể.'}
                </div>
              </div>
              <div style="font-size:11.5px;">
                <b>Bài mẫu nâng Band 6.5:</b>
                <div style="border:1px solid var(--m-line);background:var(--m-bg);padding:8px;margin-top:4px;color:var(--m-muted);">
                  ${esc(lastGrade.band65_rewrite)}
                </div>
              </div>
              <div style="margin-top:8px;display:flex;justify-content:space-between;align-items:center;">
                <span style="font-size:10.5px;color:var(--m-muted);">${esc(lastEndpoint)}</span>
                <button class="btn btn-sm" id="saveErrorsBtn">+ LƯU LỖI VÀO SỔ</button>
              </div>`
                : `<div style="color:var(--m-muted);font-size:11.5px;">Viết bài ở khung bên trái rồi bấm <b>CHẤM BAND 6.5 NGAY →</b> (hoặc dùng <b>COPY PROMPT CHẤM AI</b> miễn phí).</div>`
            }
          </div>
        </div>
      </div>

      <!-- TAB 3: VOCAB SRS -->
      <div class="panel ${activeTab === 'vocab' ? 'active' : ''}" id="tab-vocab">
        <div class="box">
          <div class="box-title">
            <span>FLASHCARD SRS (THUẬT TOÁN SM-2 · OFFLINE · ${dash.total_vocab_count} THẺ)</span>
            <span>${curCard ? `THẺ ${cardIdx + 1} / ${dueCards.length} CẦN ÔN HÔM NAY` : 'ĐÃ ÔN HẾT HÔM NAY ✓'}</span>
          </div>
          ${
            curCard
              ? `
            <div class="fc-stage" id="fcStage">
              <span class="fc-topic">${esc(curCard.topic.toUpperCase())} · BAND 6.5 · REPS ${curCard.reps}</span>
              <div class="fc-word">${esc(curCard.word)}</div>
              ${!cardFlipped ? `<div style="font-size:11px;color:var(--m-muted);">(Bấm vào thẻ để xem nghĩa & ví dụ)</div>` : ''}
              <div class="fc-back ${cardFlipped ? 'open' : ''}">
                <div style="font-weight:700;margin-bottom:4px;">${esc(curCard.meaning)}</div>
                <div style="font-size:11.5px;color:var(--m-muted);">"${esc(curCard.example)}"</div>
              </div>
            </div>
            <div class="srs-row">
              <button class="btn srs-btn" data-q="0">[1] QUÊN · 10p</button>
              <button class="btn srs-btn" data-q="1">[2] KHÓ · 1 ngày</button>
              <button class="btn srs-btn" data-q="2">[3] NHỚ · 3 ngày</button>
              <button class="btn btn-primary srs-btn" data-q="3">[4] DỄ · 7 ngày</button>
            </div>`
              : `<div style="padding:28px;text-align:center;color:var(--m-muted);">Tuyệt vời! Bạn đã hoàn thành toàn bộ thẻ đến hạn hôm nay.</div>`
          }
        </div>
      </div>

      <!-- TAB 4: ROADMAP & ERROR LOG -->
      <div class="panel ${activeTab === 'roadmap' ? 'active' : ''}" id="tab-roadmap">
        <div class="grid-2">
          <div class="box">
            <div class="box-title">
              <span>LỘ TRÌNH 24 TUẦN V2 (4 GIAI ĐOẠN · 3 CHECKPOINT)</span>
              <span>${dash.roadmap.filter((r) => r.done).length} / ${dash.roadmap.length}</span>
            </div>
            <div style="margin-bottom:6px;">
              <div style="display:flex;justify-content:space-between;font-size:11px;margin-bottom:2px;"><b>GĐ1 · Diệt lỗi nền & 5 khung T2 + 3 biểu đồ T1 (T1–7)</b><span>${pct(p1)}%</span></div>
              <div class="bar-track"><div class="bar-fill" style="width:${pct(p1)}%"></div></div>
            </div>
            <div style="margin-bottom:6px;">
              <div style="display:flex;justify-content:space-between;font-size:11px;margin-bottom:2px;"><b>GĐ2 · Tăng tốc & 6 cấu trúc nâng cao (T8–15)</b><span>${pct(p2)}%</span></div>
              <div class="bar-track"><div class="bar-fill" style="width:${pct(p2)}%"></div></div>
            </div>
            <div style="margin-bottom:6px;">
              <div style="display:flex;justify-content:space-between;font-size:11px;margin-bottom:2px;"><b>GĐ3 · Vùng đệm 7.0 & Forecast thực chiến (T16–21)</b><span>${pct(p3)}%</span></div>
              <div class="bar-track"><div class="bar-fill" style="width:${pct(p3)}%"></div></div>
            </div>
            <div style="margin-bottom:10px;">
              <div style="display:flex;justify-content:space-between;font-size:11px;margin-bottom:2px;"><b>GĐ4 · Nước rút phòng thi (T22–24)</b><span>${pct(p4)}%</span></div>
              <div class="bar-track"><div class="bar-fill" style="width:${pct(p4)}%"></div></div>
            </div>
            <div class="roadmap-list">
              ${dash.roadmap
                .map(
                  (r) => `
                <div class="rm-item ${r.done ? 'done' : ''}" data-rm-id="${r.id}">
                  <div class="rm-check">${r.done ? '✓' : ''}</div>
                  <b class="rm-week">${esc(r.week_label)}:</b>
                  <span class="rm-title">${esc(r.title)}</span>
                </div>`
                )
                .join('')}
            </div>
            <div style="display:flex;gap:6px;flex-wrap:wrap;">
              <button class="btn btn-sm" id="openStudy4Btn">↗ MỞ STUDY4</button>
              <button class="btn btn-sm" id="openIotBtn">↗ MỞ IELTS ONLINE TESTS</button>
              <button class="btn btn-sm btn-primary" id="openScoreModalBtn">+ NHẬP ĐIỂM ĐỀ VỪA LÀM</button>
            </div>
          </div>

          <div class="box">
            <div class="box-title">
              <span>SỔ LỖI HAY MẮC (TỰ GOM TỪ WRITING)</span>
              <span>${dash.error_logs.length} MỤC</span>
            </div>
            <div style="display:flex;gap:4px;margin-bottom:8px;flex-wrap:wrap;">
              <input class="text-input" id="errCat" placeholder="Loại lỗi" style="width:95px;" value="Ngữ pháp" />
              <input class="text-input" id="errWrong" placeholder="Viết sai..." style="flex:1;min-width:90px;" />
              <input class="text-input" id="errRight" placeholder="Sửa chuẩn..." style="flex:1;min-width:90px;" />
              <button class="btn btn-sm" id="addErrBtn">+ THÊM</button>
            </div>
            <div class="log-table-wrap">
              <table class="log-table">
                <thead>
                  <tr><th>Loại lỗi</th><th>Hay viết sai</th><th>Sửa chuẩn</th><th>Lần</th></tr>
                </thead>
                <tbody>
                  ${dash.error_logs
                    .map(
                      (e) =>
                        `<tr><td>${esc(e.category)}</td><td class="err-del">${esc(e.wrong_text)}</td><td class="ok-add">${esc(e.right_text)}</td><td>${e.count}</td></tr>`
                    )
                    .join('')}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="tray-strip">
      <div class="tray-badge">
        <span class="dot-live"></span>
        <span><b>SYSTEM TRAY:</b> Chạy ngầm cùng Windows · Nhắc lúc ${esc(dash.settings.reminder_time)} nếu chưa học</span>
      </div>
      <div style="color:var(--m-muted);">Rust + Tauri v2 + SQLite · Module track: ielts</div>
    </div>

    <!-- SETTINGS MODAL -->
    <div class="modal-backdrop" id="settingsModal">
      <div class="modal-box">
        <div class="box-title"><span>CÀI ĐẶT HỆ THỐNG</span><button class="btn btn-sm" id="closeSettingsBtn">✕</button></div>
        <div class="form-row">
          <label>Gemini API Key ${dash.settings.has_api_key ? `(Đang lưu: ${esc(dash.settings.api_key_masked)})` : '(Chưa có)'}</label>
          <input type="password" id="setKey" placeholder="Để trống nếu giữ nguyên key hiện tại" />
        </div>
        <div class="form-row">
          <label>Model Gemini</label>
          <select id="setModel">
            <option value="gemini-3-flash-preview" ${dash.settings.gemini_model === 'gemini-3-flash-preview' ? 'selected' : ''}>gemini-3-flash-preview (Khuyên dùng)</option>
            <option value="gemini-3.1-flash-lite-preview" ${dash.settings.gemini_model === 'gemini-3.1-flash-lite-preview' ? 'selected' : ''}>gemini-3.1-flash-lite-preview (Nhanh)</option>
            <option value="gemini-2.5-flash" ${dash.settings.gemini_model === 'gemini-2.5-flash' ? 'selected' : ''}>gemini-2.5-flash</option>
          </select>
        </div>
        <div class="form-row">
          <label>Ngày thi dự kiến (YYYY-MM-DD)</label>
          <input type="text" id="setExamDate" value="${esc(dash.settings.exam_date)}" />
        </div>
        <div class="form-row">
          <label>Giờ nhắc học mỗi ngày (HH:MM)</label>
          <input type="text" id="setReminder" value="${esc(dash.settings.reminder_time)}" />
        </div>
        <div class="form-row" style="display:flex;align-items:center;gap:8px;">
          <input type="checkbox" id="setAutostart" style="width:auto;" ${dash.settings.autostart ? 'checked' : ''} />
          <label for="setAutostart" style="margin:0;cursor:pointer;">Tự khởi động cùng Windows</label>
        </div>
        <div style="display:flex;justify-content:flex-end;gap:6px;margin-top:14px;">
          <button class="btn" id="cancelSettingsBtn">HỦY</button>
          <button class="btn btn-primary" id="saveSettingsBtn">LƯU CÀI ĐẶT</button>
        </div>
      </div>
    </div>

    <!-- SCORE MODAL -->
    <div class="modal-backdrop" id="scoreModal">
      <div class="modal-box">
        <div class="box-title"><span>NHẬP ĐIỂM ĐỀ VỪA LÀM</span><button class="btn btn-sm" id="closeScoreBtn">✕</button></div>
        <div class="form-row">
          <label>Kỹ năng</label>
          <select id="scSkill">
            <option value="Listening">Listening</option>
            <option value="Reading">Reading</option>
            <option value="Writing">Writing</option>
            <option value="Speaking">Speaking</option>
          </select>
        </div>
        <div class="form-row">
          <label>Nguồn đề (VD: Study4 Cam 15 Test 1)</label>
          <input type="text" id="scSource" value="Study4 Cambridge" />
        </div>
        <div class="form-row">
          <label>Band điểm (0.0 – 9.0)</label>
          <input type="number" step="0.5" min="0" max="9" id="scVal" value="5.5" />
        </div>
        <div class="form-row">
          <label>Ghi chú</label>
          <input type="text" id="scNote" placeholder="VD: Đúng 23/40 câu" />
        </div>
        <div style="display:flex;justify-content:flex-end;gap:6px;margin-top:14px;">
          <button class="btn" id="cancelScoreBtn">HỦY</button>
          <button class="btn btn-primary" id="saveScoreBtn">LƯU ĐIỂM</button>
        </div>
      </div>
    </div>
  </div>`;

  bindEvents();
}

function formatPomo(s: number): string {
  const m = String(Math.floor(s / 60)).padStart(2, '0');
  const sec = String(s % 60).padStart(2, '0');
  return `${m}:${sec}`;
}

function togglePomodoro(forceStart = false) {
  if (pomoTimer !== null && !forceStart) {
    window.clearInterval(pomoTimer);
    pomoTimer = null;
    const btn = document.getElementById('pomoBtn');
    if (btn) btn.textContent = `⏱ ${formatPomo(pomoSecs)}`;
    return;
  }
  if (pomoTimer !== null) return;
  pomoTimer = window.setInterval(async () => {
    if (pomoSecs <= 1) {
      window.clearInterval(pomoTimer!);
      pomoTimer = null;
      pomoSecs = 15 * 60;
      dash = await invoke<DashboardState>('add_pomodoro_minutes', { minutes: 15 });
      renderApp();
      return;
    }
    pomoSecs -= 1;
    const btn = document.getElementById('pomoBtn');
    if (btn) btn.textContent = `⏱ ${formatPomo(pomoSecs)}`;
  }, 1000);
}

function bindEvents() {
  document.querySelectorAll<HTMLButtonElement>('.nav-tab').forEach((btn) => {
    btn.onclick = () => {
      activeTab = btn.dataset.tab as typeof activeTab;
      renderApp();
    };
  });

  document.getElementById('pomoBtn')!.onclick = () => togglePomodoro(false);

  document.getElementById('themeBtn')!.onclick = async () => {
    if (!dash) return;
    const next = dash.settings.theme === 'light' ? 'dark' : 'light';
    dash.settings = await invoke('update_settings', { theme: next });
    renderApp();
  };

  document.querySelectorAll<HTMLElement>('[data-quest-id]').forEach((el) => {
    el.onclick = async () => {
      const id = Number(el.dataset.questId);
      dash = await invoke<DashboardState>('toggle_quest', { id });
      renderApp();
    };
  });

  // Writing tab
  document.querySelectorAll<HTMLButtonElement>('[data-wmode]').forEach((btn) => {
    btn.onclick = () => {
      writingMode = btn.dataset.wmode!;
      const first = dash?.prompts.find((p) => p.mode === writingMode);
      if (first) selectedPromptId = first.id;
      renderApp();
    };
  });

  const promptSel = document.getElementById('promptSelect') as HTMLSelectElement | null;
  if (promptSel) {
    promptSel.onchange = () => {
      selectedPromptId = promptSel.value;
      renderApp();
    };
  }

  const toggleHintBtn = document.getElementById('toggleHintBtn');
  if (toggleHintBtn) {
    toggleHintBtn.onclick = () => {
      hintOpen = !hintOpen;
      document.getElementById('hintBox')?.classList.toggle('open', hintOpen);
    };
  }

  const essayInput = document.getElementById('essayInput') as HTMLTextAreaElement | null;
  if (essayInput) {
    essayInput.value = draftEssay;
    const updateWC = () => {
      draftEssay = essayInput.value;
      const txt = essayInput.value.trim();
      const n = txt ? txt.split(/\s+/).length : 0;
      const target = getCurrentPrompt()?.target_words || 90;
      const wc = document.getElementById('wordCount');
      if (wc) wc.textContent = `${n} / ${target} từ`;
    };
    essayInput.oninput = updateWC;
    updateWC();
  }

  const copyPromptBtn = document.getElementById('copyPromptBtn');
  if (copyPromptBtn) {
    copyPromptBtn.onclick = async () => {
      const p = getCurrentPrompt();
      const content = (document.getElementById('essayInput') as HTMLTextAreaElement)?.value || '';
      const rubric = `Hãy đóng vai Giám khảo IELTS Writing chấm bài dưới đây theo 4 tiêu chí (TA, CC, LR, GRA) và trả kết quả rõ ràng bằng tiếng Việt + bản viết lại Band 6.5:\n\n[CHẾ ĐỘ]: ${writingMode}\n[ĐỀ BÀI]: ${p?.prompt || ''}\n\n[BÀI VIẾT]:\n${content}`;
      await navigator.clipboard.writeText(rubric);
      const st = document.getElementById('writingStatus');
      if (st) st.textContent = '✓ Đã copy prompt + bài viết vào Clipboard! Dán vào ChatGPT/Gemini web để chấm miễn phí.';
    };
  }

  const gradeBtn = document.getElementById('gradeBtn');
  if (gradeBtn) {
    gradeBtn.onclick = async () => {
      const content = (document.getElementById('essayInput') as HTMLTextAreaElement)?.value.trim() || '';
      if (content.length < 20) {
        const st = document.getElementById('writingStatus');
        if (st) st.textContent = 'Hãy viết ít nhất vài câu trước khi chấm.';
        return;
      }
      isGrading = true;
      renderApp();
      // Preserve textarea content across re-render
      const restored = document.getElementById('essayInput') as HTMLTextAreaElement | null;
      if (restored) restored.value = content;
      try {
        const p = getCurrentPrompt();
        const res = await invoke<GradeResponseDto>('grade_writing', {
          mode: writingMode,
          prompt: p?.prompt || '',
          content,
        });
        lastGrade = res.result;
        lastEndpoint = res.endpoint;
        dash = await invoke<DashboardState>('get_dashboard');
      } catch (err) {
        const st = document.getElementById('writingStatus');
        if (st) st.textContent = `Lỗi: ${String(err)}`;
      } finally {
        isGrading = false;
        renderApp();
        const after = document.getElementById('essayInput') as HTMLTextAreaElement | null;
        if (after) after.value = content;
      }
    };
  }

  const saveErrorsBtn = document.getElementById('saveErrorsBtn');
  if (saveErrorsBtn && lastGrade) {
    saveErrorsBtn.onclick = async () => {
      if (!lastGrade) return;
      dash = await invoke<DashboardState>('save_corrections_to_errors', {
        corrections: lastGrade.corrections,
      });
      saveErrorsBtn.textContent = `✓ ĐÃ LƯU ${lastGrade.corrections.length} LỖI VÀO SỔ`;
    };
  }

  // Vocab SRS
  const fcStage = document.getElementById('fcStage');
  if (fcStage) {
    fcStage.onclick = () => {
      cardFlipped = !cardFlipped;
      renderApp();
    };
  }

  document.querySelectorAll<HTMLButtonElement>('.srs-btn').forEach((btn) => {
    btn.onclick = async () => {
      const card = dueCards[cardIdx];
      if (!card) return;
      const quality = Number(btn.dataset.q);
      await invoke('review_vocab', { cardId: card.id, quality });
      if (quality === 0) {
        dueCards.push(dueCards.splice(cardIdx, 1)[0]);
      } else {
        dueCards.splice(cardIdx, 1);
      }
      if (cardIdx >= dueCards.length) cardIdx = 0;
      cardFlipped = false;
      dash = await invoke<DashboardState>('get_dashboard');
      renderApp();
    };
  });

  // Roadmap & Error Log
  document.querySelectorAll<HTMLElement>('[data-rm-id]').forEach((el) => {
    el.onclick = async () => {
      const id = Number(el.dataset.rmId);
      dash = await invoke<DashboardState>('toggle_roadmap_item', { id });
      renderApp();
    };
  });

  document.getElementById('openStudy4Btn')!.onclick = () => openUrl('https://study4.com/tests/ielts/');
  document.getElementById('openIotBtn')!.onclick = () => openUrl('https://ieltsonlinetests.com/');

  document.getElementById('addErrBtn')!.onclick = async () => {
    const category = (document.getElementById('errCat') as HTMLInputElement).value.trim() || 'Ngữ pháp';
    const wrongText = (document.getElementById('errWrong') as HTMLInputElement).value.trim();
    const rightText = (document.getElementById('errRight') as HTMLInputElement).value.trim();
    if (!wrongText || !rightText) return;
    dash = await invoke<DashboardState>('add_error_log', { category, wrongText, rightText });
    renderApp();
  };

  // Modals
  const settingsModal = document.getElementById('settingsModal')!;
  document.getElementById('openSettingsBtn')!.onclick = () => settingsModal.classList.add('open');
  document.getElementById('closeSettingsBtn')!.onclick = () => settingsModal.classList.remove('open');
  document.getElementById('cancelSettingsBtn')!.onclick = () => settingsModal.classList.remove('open');
  settingsModal.onclick = (e) => {
    if (e.target === settingsModal) settingsModal.classList.remove('open');
  };
  document.getElementById('saveSettingsBtn')!.onclick = async () => {
    const keyVal = (document.getElementById('setKey') as HTMLInputElement).value.trim();
    const modelVal = (document.getElementById('setModel') as HTMLSelectElement).value;
    const examVal = (document.getElementById('setExamDate') as HTMLInputElement).value.trim();
    const remVal = (document.getElementById('setReminder') as HTMLInputElement).value.trim();
    const autoVal = (document.getElementById('setAutostart') as HTMLInputElement).checked;
    await invoke('update_settings', {
      geminiApiKey: keyVal ? keyVal : null,
      geminiModel: modelVal,
      examDate: examVal,
      reminderTime: remVal,
      autostart: autoVal,
    });
    dash = await invoke<DashboardState>('get_dashboard');
    settingsModal.classList.remove('open');
    renderApp();
  };

  const scoreModal = document.getElementById('scoreModal')!;
  document.getElementById('openScoreModalBtn')!.onclick = () => scoreModal.classList.add('open');
  document.getElementById('closeScoreBtn')!.onclick = () => scoreModal.classList.remove('open');
  document.getElementById('cancelScoreBtn')!.onclick = () => scoreModal.classList.remove('open');
  scoreModal.onclick = (e) => {
    if (e.target === scoreModal) scoreModal.classList.remove('open');
  };
  document.getElementById('saveScoreBtn')!.onclick = async () => {
    const skill = (document.getElementById('scSkill') as HTMLSelectElement).value;
    const source = (document.getElementById('scSource') as HTMLInputElement).value.trim() || 'Practice';
    const score = parseFloat((document.getElementById('scVal') as HTMLInputElement).value) || 5.0;
    const note = (document.getElementById('scNote') as HTMLInputElement).value.trim();
    dash = await invoke<DashboardState>('add_practice_score', { skill, source, score, note });
    scoreModal.classList.remove('open');
    renderApp();
  };
}

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') {
    document.querySelectorAll('.modal-backdrop.open').forEach((m) => m.classList.remove('open'));
  }
});

async function init() {
  dash = await invoke<DashboardState>('get_dashboard');
  dueCards = await invoke<VocabCard[]>('get_due_vocab');
  renderApp();

  await listen('start-pomodoro-15', () => {
    pomoSecs = 15 * 60;
    togglePomodoro(true);
  });
}

init();
