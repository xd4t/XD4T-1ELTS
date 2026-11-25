using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Text;
using System.Globalization;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Web.Script.Serialization;
using System.Windows;
using System.Windows.Automation;
using System.Windows.Forms;
using Microsoft.Win32;

namespace XD4TIeltsBar
{
    internal sealed class WidgetStatusData
    {
        public string Theme = "pastel-male";
        public int QuestsDone = 0;
        public int QuestsTotal = 3;
        public int Streak = 0;
        public double OverallBand = 5.0;
        public double TargetBand = 7.0;
        public int DaysLeft = 171;
        public int TodayMinutes = 0;
    }

    internal sealed class IeltsWidgetForm : Form
    {
        private const int BaseWidth = 180;
        private const int BaseHeight = 38;

        private const int WS_EX_TOOLWINDOW = 0x00000080;
        private const int WS_EX_TOPMOST = 0x00000008;
        private const int WS_EX_NOACTIVATE = 0x08000000;
        private const int WM_MOUSEACTIVATE = 0x0021;
        private const int MA_NOACTIVATE = 3;
        private const int WM_SETTINGCHANGE = 0x001A;
        private const uint SWP_NOACTIVATE = 0x0010;
        private const uint SWP_SHOWWINDOW = 0x0040;
        private const uint GW_HWNDPREV = 3;
        private const int DWMWA_WINDOW_CORNER_PREFERENCE = 33;
        private const int DWMWCP_DONOTROUND = 1;
        private const int DWMWCP_ROUNDSMALL = 3;
        private bool? _lastAppliedMono = null;

        private static readonly IntPtr HWND_TOPMOST = new IntPtr(-1);
        private static readonly int WM_TASKBARCREATED = RegisterWindowMessageW("TaskbarCreated");
        private static readonly int WM_WIDGET_EXIT = RegisterWindowMessageW("XD4TIeltsBar_Exit");

        [StructLayout(LayoutKind.Sequential)]
        private struct RECT
        {
            public int Left, Top, Right, Bottom;
            public int Width { get { return Right - Left; } }
            public int Height { get { return Bottom - Top; } }
        }

        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        private static extern IntPtr FindWindowW(string lpClassName, string lpWindowName);
        [DllImport("user32.dll")]
        private static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);
        [DllImport("user32.dll")]
        private static extern bool IsWindowVisible(IntPtr hWnd);
        [DllImport("user32.dll")]
        private static extern bool SetWindowPos(IntPtr hWnd, IntPtr hWndInsertAfter, int X, int Y, int cx, int cy, uint uFlags);
        [DllImport("user32.dll")]
        private static extern IntPtr GetWindow(IntPtr hWnd, uint uCmd);
        [DllImport("user32.dll")]
        private static extern IntPtr GetForegroundWindow();
        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        private static extern int GetClassNameW(IntPtr hWnd, StringBuilder lpClassName, int nMaxCount);
        [DllImport("user32.dll", CharSet = CharSet.Unicode)]
        private static extern int RegisterWindowMessageW(string lpString);
        [DllImport("user32.dll")]
        private static extern uint GetDpiForWindow(IntPtr hwnd);
        [DllImport("dwmapi.dll")]
        private static extern int DwmSetWindowAttribute(IntPtr hwnd, int attr, ref int attrValue, int attrSize);

        private readonly string _statusFilePath;
        private readonly System.Windows.Forms.Timer _pollTimer;
        private readonly ToolTip _toolTip;
        private readonly ContextMenuStrip _menu;
        private FileSystemWatcher _watcher;

        private WidgetStatusData _data = new WidgetStatusData();
        private bool _isDark = true;
        private bool _hovered = false;
        private double _dpiScale = 1.0;
        private string _fontName = "Consolas";

        // UIA cached bounds for Windows 11 Weather/Widgets & Start button
        private int _lastKnownWidgetsRight = -1;
        private int _lastKnownStartLeft = -1;
        private DateTime _lastUiaProbeUtc = DateTime.MinValue;
        private int _uiaScanning = 0;

        public IeltsWidgetForm()
        {
            FormBorderStyle = FormBorderStyle.None;
            ShowInTaskbar = false;
            StartPosition = FormStartPosition.Manual;
            TopMost = true;
            DoubleBuffered = true;
            SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.UserPaint | ControlStyles.OptimizedDoubleBuffer | ControlStyles.ResizeRedraw, true);

            Size = new System.Drawing.Size(BaseWidth, BaseHeight);
            Location = new System.Drawing.Point(-32000, -32000);

            string appData = Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData);
            string dir = Path.Combine(appData, "xd4t-ielts65");
            _statusFilePath = Path.Combine(dir, "widget-status.json");

            _fontName = ResolveFontName();

            _toolTip = new ToolTip
            {
                InitialDelay = 250,
                ReshowDelay = 200,
                AutoPopDelay = 8000,
                ShowAlways = true
            };

            _menu = new ContextMenuStrip();
            _menu.Items.Add("Mở XD4T IELTS 6.5", null, (s, e) => LaunchMainApp());
            _menu.Items.Add("Làm mới dữ liệu", null, (s, e) => { LoadStatus(); Invalidate(); });
            _menu.Items.Add(new ToolStripSeparator());
            _menu.Items.Add("Thoát widget", null, (s, e) => Close());

            MouseEnter += (s, e) => { _hovered = true; Invalidate(); };
            MouseLeave += (s, e) => { _hovered = false; Invalidate(); };
            MouseClick += OnPillMouseClick;

            ReadWindowsTheme();
            LoadStatus();
            StartWatcher(dir);

            _pollTimer = new System.Windows.Forms.Timer { Interval = 2000 };
            _pollTimer.Tick += (s, e) =>
            {
                bool prevDark = _isDark;
                ReadWindowsTheme();
                LoadStatus();
                RepositionOnTaskbar();
                if (prevDark != _isDark) Invalidate();
            };
            _pollTimer.Start();
        }

        private static string ResolveFontName()
        {
            using (InstalledFontCollection ifc = new InstalledFontCollection())
            {
                foreach (FontFamily ff in ifc.Families)
                {
                    if (string.Equals(ff.Name, "JetBrains Mono", StringComparison.OrdinalIgnoreCase))
                        return "JetBrains Mono";
                }
            }
            return "Consolas";
        }

        protected override bool ShowWithoutActivation { get { return true; } }

        protected override CreateParams CreateParams
        {
            get
            {
                CreateParams cp = base.CreateParams;
                cp.ExStyle |= WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST;
                return cp;
            }
        }

        protected override void OnHandleCreated(EventArgs e)
        {
            base.OnHandleCreated(e);
            ApplyRoundedCorners();
            IntPtr tb = FindWindowW("Shell_TrayWnd", null);
            if (tb != IntPtr.Zero)
            {
                ProbeTaskbarUiaSync(tb);
            }
            RepositionOnTaskbar();
        }

        protected override void WndProc(ref Message m)
        {
            if (m.Msg == WM_MOUSEACTIVATE)
            {
                m.Result = new IntPtr(MA_NOACTIVATE);
                return;
            }
            if (m.Msg == WM_SETTINGCHANGE)
            {
                ReadWindowsTheme();
                RepositionOnTaskbar();
                Invalidate();
            }
            if (m.Msg == WM_TASKBARCREATED)
            {
                RepositionOnTaskbar();
            }
            if (m.Msg == WM_WIDGET_EXIT)
            {
                Close();
                return;
            }
            base.WndProc(ref m);
        }

        private void ApplyRoundedCorners()
        {
            try
            {
                string theme = (_data.Theme ?? "pastel-male").Trim().ToLowerInvariant();
                bool isMono = theme != "pastel-male" && theme != "pastel-female";
                if (_lastAppliedMono.HasValue && _lastAppliedMono.Value == isMono) return;
                _lastAppliedMono = isMono;
                int pref = isMono ? DWMWCP_DONOTROUND : DWMWCP_ROUNDSMALL;
                DwmSetWindowAttribute(Handle, DWMWA_WINDOW_CORNER_PREFERENCE, ref pref, Marshal.SizeOf(typeof(int)));
            }
            catch { }
        }

        private void StartWatcher(string dir)
        {
            try
            {
                if (!Directory.Exists(dir)) Directory.CreateDirectory(dir);
                _watcher = new FileSystemWatcher(dir, "widget-status.json")
                {
                    NotifyFilter = NotifyFilters.LastWrite | NotifyFilters.CreationTime | NotifyFilters.FileName
                };
                FileSystemEventHandler onChange = (s, e) =>
                {
                    if (IsHandleCreated && !IsDisposed)
                    {
                        BeginInvoke((MethodInvoker)(() => { LoadStatus(); Invalidate(); }));
                    }
                };
                _watcher.Changed += onChange;
                _watcher.Created += onChange;
                _watcher.Renamed += (s, e) =>
                {
                    if (IsHandleCreated && !IsDisposed)
                    {
                        BeginInvoke((MethodInvoker)(() => { LoadStatus(); Invalidate(); }));
                    }
                };
                _watcher.EnableRaisingEvents = true;
            }
            catch { }
        }

        private void ReadWindowsTheme()
        {
            try
            {
                using (RegistryKey key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize"))
                {
                    if (key != null)
                    {
                        object val = key.GetValue("AppsUseLightTheme");
                        if (val is int)
                        {
                            _isDark = ((int)val) == 0;
                        }
                    }
                }
            }
            catch { }
        }

        private static bool IsWidgetsEnabledInRegistry()
        {
            try
            {
                using (RegistryKey key = Registry.CurrentUser.OpenSubKey(@"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced"))
                {
                    if (key != null)
                    {
                        object val = key.GetValue("TaskbarDa");
                        if (val is int)
                        {
                            return ((int)val) != 0;
                        }
                    }
                }
            }
            catch { }
            return true;
        }

        private void TriggerAsyncUiaProbe(IntPtr taskbarHwnd)
        {
            if ((DateTime.UtcNow - _lastUiaProbeUtc).TotalSeconds < 2.0) return;
            if (Interlocked.CompareExchange(ref _uiaScanning, 1, 0) != 0) return;
            _lastUiaProbeUtc = DateTime.UtcNow;

            ThreadPool.QueueUserWorkItem(_ =>
            {
                try
                {
                    ProbeTaskbarUiaSync(taskbarHwnd);
                }
                finally
                {
                    Interlocked.Exchange(ref _uiaScanning, 0);
                }
            });
        }

        private void ProbeTaskbarUiaSync(IntPtr taskbarHwnd)
        {
            try
            {
                RECT tbRect;
                if (!GetWindowRect(taskbarHwnd, out tbRect)) return;

                AutomationElement root = AutomationElement.FromHandle(taskbarHwnd);
                if (root == null) return;

                AutomationElementCollection children = root.FindAll(TreeScope.Descendants, Condition.TrueCondition);
                int foundWidgetsRight = -1;
                int foundStartLeft = -1;

                foreach (AutomationElement el in children)
                {
                    try
                    {
                        Rect br = el.Current.BoundingRectangle;
                        if (br.IsEmpty || br.Width <= 0 || br.Height <= 0) continue;

                        string aid = el.Current.AutomationId ?? string.Empty;
                        string cls = el.Current.ClassName ?? string.Empty;
                        string name = el.Current.Name ?? string.Empty;

                        if (aid.IndexOf("Widgets", StringComparison.OrdinalIgnoreCase) >= 0 ||
                            cls.IndexOf("Widgets", StringComparison.OrdinalIgnoreCase) >= 0 ||
                            (br.Left >= tbRect.Left && br.Left < tbRect.Left + S(40) && br.Right < tbRect.Left + S(350) && br.Width >= S(30)))
                        {
                            int r = (int)Math.Ceiling(br.Right);
                            if (r > foundWidgetsRight && r < tbRect.Left + S(360))
                            {
                                foundWidgetsRight = r;
                            }
                        }

                        if (string.Equals(aid, "StartButton", StringComparison.OrdinalIgnoreCase) ||
                            (aid.IndexOf("Start", StringComparison.OrdinalIgnoreCase) >= 0 && br.Left > tbRect.Left + S(120)))
                        {
                            int l = (int)Math.Floor(br.Left);
                            if (foundStartLeft < 0 || l < foundStartLeft)
                            {
                                foundStartLeft = l;
                            }
                        }
                    }
                    catch { }
                }

                if (foundWidgetsRight > 0)
                {
                    Interlocked.Exchange(ref _lastKnownWidgetsRight, foundWidgetsRight);
                }
                if (foundStartLeft > 0)
                {
                    Interlocked.Exchange(ref _lastKnownStartLeft, foundStartLeft);
                }
            }
            catch { }
        }

        private void LoadStatus()
        {
            try
            {
                if (!File.Exists(_statusFilePath)) return;
                string json = File.ReadAllText(_statusFilePath, Encoding.UTF8);
                JavaScriptSerializer ser = new JavaScriptSerializer();
                var dict = ser.Deserialize<Dictionary<string, object>>(json);
                if (dict == null) return;

                WidgetStatusData d = new WidgetStatusData();
                if (dict.ContainsKey("theme") && dict["theme"] != null) d.Theme = dict["theme"].ToString();
                if (dict.ContainsKey("quests_done")) d.QuestsDone = Convert.ToInt32(dict["quests_done"], CultureInfo.InvariantCulture);
                if (dict.ContainsKey("quests_total")) d.QuestsTotal = Math.Max(1, Convert.ToInt32(dict["quests_total"], CultureInfo.InvariantCulture));
                if (dict.ContainsKey("streak")) d.Streak = Convert.ToInt32(dict["streak"], CultureInfo.InvariantCulture);
                if (dict.ContainsKey("overall_band")) d.OverallBand = Convert.ToDouble(dict["overall_band"], CultureInfo.InvariantCulture);
                if (dict.ContainsKey("target_band")) d.TargetBand = Convert.ToDouble(dict["target_band"], CultureInfo.InvariantCulture);
                if (dict.ContainsKey("days_left")) d.DaysLeft = Convert.ToInt32(dict["days_left"], CultureInfo.InvariantCulture);
                if (dict.ContainsKey("today_minutes")) d.TodayMinutes = Convert.ToInt32(dict["today_minutes"], CultureInfo.InvariantCulture);
                _data = d;

                string tip = string.Format(
                    CultureInfo.InvariantCulture,
                    "XD4T IELTS 6.5 · Hôm nay: {0}/{1} nhiệm vụ · Streak: {2} ngày · Còn {3} ngày thi (05/2027) · Click để mở App",
                    _data.QuestsDone, _data.QuestsTotal, _data.Streak, _data.DaysLeft);
                if (_toolTip.GetToolTip(this) != tip)
                {
                    _toolTip.SetToolTip(this, tip);
                }
            }
            catch { }
        }

        private int S(int px) { return (int)Math.Round(px * _dpiScale); }
        private float Sf(float px) { return (float)(px * _dpiScale); }

        public void RepositionOnTaskbar()
        {
            if (IsDisposed || !IsHandleCreated) return;
            IntPtr tb = FindWindowW("Shell_TrayWnd", null);
            if (tb == IntPtr.Zero || !IsWindowVisible(tb))
            {
                if (Visible) Hide();
                return;
            }
            RECT tbRect;
            if (!GetWindowRect(tb, out tbRect) || tbRect.Width <= 0 || tbRect.Height <= 0)
            {
                if (Visible) Hide();
                return;
            }

            try
            {
                uint dpi = GetDpiForWindow(tb);
                _dpiScale = dpi > 0 ? dpi / 96.0 : 1.0;
            }
            catch { _dpiScale = 1.0; }

            Screen tbScreen = Screen.FromRectangle(new Rectangle(tbRect.Left, tbRect.Top, tbRect.Width, tbRect.Height));
            if (tbRect.Top >= tbScreen.Bounds.Bottom - 4 || IsFullscreenBlocking(tb, tbScreen))
            {
                if (Visible) Hide();
                return;
            }

            TriggerAsyncUiaProbe(tb);

            int pillW = S(BaseWidth);
            int pillH = Math.Min(S(BaseHeight), Math.Max(S(32), tbRect.Height - S(8)));
            int pillY = tbRect.Top + (tbRect.Height - pillH) / 2;

            int pillX;
            bool widgetsOn = IsWidgetsEnabledInRegistry();
            int knownWRight = Interlocked.CompareExchange(ref _lastKnownWidgetsRight, 0, 0);
            int knownStartLeft = Interlocked.CompareExchange(ref _lastKnownStartLeft, 0, 0);

            if (!widgetsOn)
            {
                pillX = tbRect.Left + S(12);
            }
            else if (knownWRight > 0)
            {
                pillX = knownWRight + S(10);
            }
            else
            {
                // Safe floor when Widgets is enabled but UIA hasn't returned yet
                pillX = tbRect.Left + S(180);
            }

            if (knownStartLeft > 0 && pillX + pillW > knownStartLeft - S(10))
            {
                pillX = Math.Max(tbRect.Left + S(12), knownStartLeft - S(10) - pillW);
            }

            bool needMove = !Visible || Math.Abs(Left - pillX) >= 2 || Math.Abs(Top - pillY) >= 2 || Width != pillW || Height != pillH;
            bool needTopmost = needMove || !IsAboveWindow(Handle, tb);
            if (needMove || needTopmost)
            {
                if (!Visible)
                {
                    SetBounds(pillX, pillY, pillW, pillH);
                    Show();
                }
                SetWindowPos(Handle, HWND_TOPMOST, pillX, pillY, pillW, pillH, SWP_NOACTIVATE | SWP_SHOWWINDOW);
                ApplyRoundedCorners();
                if (needMove) Invalidate();
            }
        }

        private static bool IsAboveWindow(IntPtr self, IntPtr target)
        {
            IntPtr cur = target;
            for (int i = 0; i < 40 && cur != IntPtr.Zero; i++)
            {
                cur = GetWindow(cur, GW_HWNDPREV);
                if (cur == self) return true;
            }
            return false;
        }

        private bool IsFullscreenBlocking(IntPtr taskbarHwnd, Screen tbScreen)
        {
            try
            {
                IntPtr fg = GetForegroundWindow();
                if (fg == IntPtr.Zero || fg == taskbarHwnd || fg == Handle) return false;
                StringBuilder sb = new StringBuilder(64);
                GetClassNameW(fg, sb, sb.Capacity);
                string cls = sb.ToString();
                if (cls == "Shell_TrayWnd" || cls == "Progman" || cls == "WorkerW" || cls == "Shell_SecondaryTrayWnd") return false;
                RECT fr;
                if (IsWindowVisible(fg) && GetWindowRect(fg, out fr))
                {
                    Rectangle b = tbScreen.Bounds;
                    if (fr.Left <= b.Left + 2 && fr.Top <= b.Top + 2 && fr.Right >= b.Right - 2 && fr.Bottom >= b.Bottom - 2)
                        return true;
                }
            }
            catch { }
            return false;
        }

        private void OnPillMouseClick(object sender, MouseEventArgs e)
        {
            if (e.Button == MouseButtons.Right)
            {
                _menu.Show(this, e.Location);
            }
            else if (e.Button == MouseButtons.Left)
            {
                LaunchMainApp();
            }
        }

        private static string ResolveAppExePath()
        {
            string baseDir = AppDomain.CurrentDomain.BaseDirectory;
            string localAppData = Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData);
            string programFiles = Environment.GetFolderPath(Environment.SpecialFolder.ProgramFiles);

            var candidates = new List<string>
            {
                Path.Combine(baseDir, "xd4t-ielts65.exe"),
                Path.GetFullPath(Path.Combine(baseDir, "..", "xd4t-ielts65.exe")),
                Path.Combine(localAppData, "XD4T-1ELTS", "xd4t-ielts65.exe"),
                Path.Combine(localAppData, "Programs", "XD4T IELTS 6.5", "xd4t-ielts65.exe"),
                Path.Combine(programFiles, "XD4T IELTS 6.5", "xd4t-ielts65.exe"),
                Path.GetFullPath(Path.Combine(baseDir, "..", "src-tauri", "target", "release", "xd4t-ielts65.exe")),
            };

            string cfgPath = Path.Combine(baseDir, "app-path.txt");
            if (File.Exists(cfgPath))
            {
                try
                {
                    string custom = File.ReadAllText(cfgPath, Encoding.UTF8).Trim().Trim('"');
                    if (!string.IsNullOrEmpty(custom))
                    {
                        candidates.Add(custom);
                    }
                }
                catch { }
            }

            foreach (string path in candidates)
            {
                if (File.Exists(path)) return path;
            }
            return null;
        }

        private void LaunchMainApp()
        {
            try
            {
                string exePath = ResolveAppExePath();
                if (!string.IsNullOrEmpty(exePath) && File.Exists(exePath))
                {
                    Process.Start(new ProcessStartInfo
                    {
                        FileName = exePath,
                        WorkingDirectory = Path.GetDirectoryName(exePath),
                        UseShellExecute = true
                    });
                }
            }
            catch { }
        }

        protected override void OnPaint(PaintEventArgs e)
        {
            Graphics g = e.Graphics;
            g.SmoothingMode = SmoothingMode.AntiAlias;
            g.TextRenderingHint = TextRenderingHint.ClearTypeGridFit;

            string theme = (_data.Theme ?? "pastel-male").Trim().ToLowerInvariant();
            bool isPastelMale = theme == "pastel-male";
            bool isPastelFemale = theme == "pastel-female";
            bool isMono = !isPastelMale && !isPastelFemale;
            ApplyRoundedCorners();

            Color bg;
            Color border;
            Color fg;
            Color badgeBg;
            Color badgeFg;
            Color track;
            Color okGreen;
            Color alertRed;
            Color progressStart;
            Color progressEnd;

            if (isPastelMale)
            {
                bg = _hovered ? ColorTranslator.FromHtml("#E8F2F0") : ColorTranslator.FromHtml("#F2F7F6");
                border = ColorTranslator.FromHtml("#C8DDD9");
                fg = ColorTranslator.FromHtml("#1E2F36");
                badgeBg = ColorTranslator.FromHtml("#2F6E89");
                badgeFg = ColorTranslator.FromHtml("#FFFFFF");
                track = ColorTranslator.FromHtml("#DCE8E6");
                okGreen = ColorTranslator.FromHtml("#1E7E5C");
                alertRed = ColorTranslator.FromHtml("#C2414B");
                progressStart = ColorTranslator.FromHtml("#2F6E89");
                progressEnd = ColorTranslator.FromHtml("#52A68C");
            }
            else if (isPastelFemale)
            {
                bg = _hovered ? ColorTranslator.FromHtml("#FBECEB") : ColorTranslator.FromHtml("#FDF6F5");
                border = ColorTranslator.FromHtml("#EED2D4");
                fg = ColorTranslator.FromHtml("#3A272B");
                badgeBg = ColorTranslator.FromHtml("#B54E61");
                badgeFg = ColorTranslator.FromHtml("#FFFFFF");
                track = ColorTranslator.FromHtml("#F4E1E2");
                okGreen = ColorTranslator.FromHtml("#2D7A54");
                alertRed = ColorTranslator.FromHtml("#B93846");
                progressStart = ColorTranslator.FromHtml("#B54E61");
                progressEnd = ColorTranslator.FromHtml("#E59876");
            }
            else
            {
                bool useDark = theme == "mono-dark" ? true : (theme == "mono-light" ? false : _isDark);
                bg = useDark
                    ? (_hovered ? ColorTranslator.FromHtml("#1F1F25") : ColorTranslator.FromHtml("#0C0C0E"))
                    : (_hovered ? ColorTranslator.FromHtml("#E7E7EE") : ColorTranslator.FromHtml("#FFFFFF"));
                border = useDark ? ColorTranslator.FromHtml("#242429") : ColorTranslator.FromHtml("#DCDCE2");
                fg = useDark ? ColorTranslator.FromHtml("#F2F2F5") : ColorTranslator.FromHtml("#0C0C0E");
                badgeBg = useDark ? ColorTranslator.FromHtml("#F2F2F5") : ColorTranslator.FromHtml("#18181C");
                badgeFg = useDark ? ColorTranslator.FromHtml("#0C0C0E") : ColorTranslator.FromHtml("#FFFFFF");
                track = useDark ? ColorTranslator.FromHtml("#222228") : ColorTranslator.FromHtml("#E3E3EA");
                okGreen = useDark ? ColorTranslator.FromHtml("#22C55E") : ColorTranslator.FromHtml("#15803D");
                alertRed = useDark ? ColorTranslator.FromHtml("#EF4444") : ColorTranslator.FromHtml("#D70015");
                progressStart = badgeBg;
                progressEnd = badgeBg;
            }

            g.Clear(bg);

            float outerR = isMono ? 0f : Sf(6f);
            float badgeR = isMono ? 0f : Sf(4f);
            float barR = isMono ? 0f : Sf(1.5f);

            RectangleF outer = new RectangleF(0.5f, 0.5f, Width - 1f, Height - 1f);
            using (GraphicsPath p = CreateRoundedRect(outer, outerR))
            using (SolidBrush br = new SolidBrush(bg))
            using (Pen pen = new Pen(border, 1f))
            {
                g.FillPath(br, p);
                g.DrawPath(pen, p);
            }

            float padX = Sf(8f);
            float badgeW = Sf(44f);
            float badgeH = Sf(18f);
            float badgeY = Sf(6f);
            RectangleF badgeRect = new RectangleF(padX, badgeY, badgeW, badgeH);
            using (GraphicsPath bp = CreateRoundedRect(badgeRect, badgeR))
            using (SolidBrush bbr = new SolidBrush(badgeBg))
            {
                g.FillPath(bbr, bp);
            }
            using (Font bf = new Font(_fontName, Sf(9f), System.Drawing.FontStyle.Bold, GraphicsUnit.Point))
            using (SolidBrush btf = new SolidBrush(badgeFg))
            using (StringFormat sf = new StringFormat { Alignment = StringAlignment.Center, LineAlignment = StringAlignment.Center })
            {
                g.DrawString("IELTS", bf, btf, badgeRect, sf);
            }

            bool allDone = _data.QuestsTotal > 0 && _data.QuestsDone >= _data.QuestsTotal;
            bool lateAlert = DateTime.Now.Hour >= 20 && _data.QuestsDone == 0;
            Color textColor = allDone ? okGreen : (lateAlert ? alertRed : fg);
            string statText = string.Format(
                CultureInfo.InvariantCulture,
                "{0}/{1} · {2}d · {3:0.0}",
                _data.QuestsDone, _data.QuestsTotal, _data.Streak, _data.OverallBand);

            float textX = padX + badgeW + Sf(7f);
            RectangleF textRect = new RectangleF(textX, badgeY - Sf(1f), Width - textX - padX, badgeH + Sf(2f));
            using (Font tf = new Font(_fontName, Sf(9.2f), System.Drawing.FontStyle.Bold, GraphicsUnit.Point))
            using (SolidBrush tbr = new SolidBrush(textColor))
            using (StringFormat sf = new StringFormat { Alignment = StringAlignment.Far, LineAlignment = StringAlignment.Center })
            {
                g.DrawString(statText, tf, tbr, textRect, sf);
            }

            float barH = Sf(3f);
            float barY = Height - Sf(6.5f);
            float barW = Width - padX * 2f;
            RectangleF trackRect = new RectangleF(padX, barY, barW, barH);
            using (GraphicsPath tp = CreateRoundedRect(trackRect, barR))
            using (SolidBrush tbr = new SolidBrush(track))
            {
                g.FillPath(tbr, tp);
            }

            double ratio = _data.QuestsTotal > 0 ? Math.Min(1.0, Math.Max(0.0, (double)_data.QuestsDone / _data.QuestsTotal)) : 0.0;
            if (ratio > 0.0)
            {
                float fillW = Math.Max(Sf(4f), (float)(barW * ratio));
                RectangleF fillRect = new RectangleF(padX, barY, fillW, barH);
                using (GraphicsPath fp = CreateRoundedRect(fillRect, barR))
                {
                    if (allDone)
                    {
                        using (SolidBrush fbr = new SolidBrush(okGreen))
                        {
                            g.FillPath(fbr, fp);
                        }
                    }
                    else if (progressStart != progressEnd)
                    {
                        using (LinearGradientBrush lbr = new LinearGradientBrush(fillRect, progressStart, progressEnd, LinearGradientMode.Horizontal))
                        {
                            g.FillPath(lbr, fp);
                        }
                    }
                    else
                    {
                        using (SolidBrush fbr = new SolidBrush(progressStart))
                        {
                            g.FillPath(fbr, fp);
                        }
                    }
                }
            }
        }

        private static GraphicsPath CreateRoundedRect(RectangleF bounds, float radius)
        {
            GraphicsPath path = new GraphicsPath();
            float r = Math.Max(0f, Math.Min(radius, Math.Min(bounds.Width, bounds.Height) / 2f));
            if (r <= 0.2f)
            {
                path.AddRectangle(bounds);
                path.CloseFigure();
                return path;
            }
            float d = r * 2f;
            path.AddArc(bounds.X, bounds.Y, d, d, 180f, 90f);
            path.AddArc(bounds.Right - d, bounds.Y, d, d, 270f, 90f);
            path.AddArc(bounds.Right - d, bounds.Bottom - d, d, d, 0f, 90f);
            path.AddArc(bounds.X, bounds.Bottom - d, d, d, 90f, 90f);
            path.CloseFigure();
            return path;
        }

        protected override void Dispose(bool disposing)
        {
            if (disposing)
            {
                if (_pollTimer != null) _pollTimer.Dispose();
                if (_watcher != null) _watcher.Dispose();
                if (_toolTip != null) _toolTip.Dispose();
                if (_menu != null) _menu.Dispose();
            }
            base.Dispose(disposing);
        }
    }
}
