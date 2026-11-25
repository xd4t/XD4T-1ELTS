using System;
using System.Threading;
using System.Windows.Forms;

namespace XD4TIeltsBar
{
    internal static class Program
    {
        [STAThread]
        private static void Main()
        {
            bool createdNew;
            using (Mutex mutex = new Mutex(true, @"Local\XD4TIeltsBar_SingleInstance", out createdNew))
            {
                if (!createdNew)
                {
                    return;
                }

                Application.EnableVisualStyles();
                Application.SetCompatibleTextRenderingDefault(false);
                using (IeltsWidgetForm form = new IeltsWidgetForm())
                {
                    Application.Run(form);
                }
            }
        }
    }
}
