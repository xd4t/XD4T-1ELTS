export function esc(s: string): string {
  return s
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

const CONTRACTIONS: Record<string, string[]> = {
  "don't": ['do', 'not'],
  "doesn't": ['does', 'not'],
  "didn't": ['did', 'not'],
  "can't": ['can', 'not'],
  cannot: ['can', 'not'],
  "won't": ['will', 'not'],
  "wouldn't": ['would', 'not'],
  "couldn't": ['could', 'not'],
  "shouldn't": ['should', 'not'],
  "isn't": ['is', 'not'],
  "aren't": ['are', 'not'],
  "wasn't": ['was', 'not'],
  "weren't": ['were', 'not'],
  "hasn't": ['has', 'not'],
  "haven't": ['have', 'not'],
  "hadn't": ['had', 'not'],
  "it's": ['it', 'is'],
  "they're": ['they', 'are'],
  "we're": ['we', 'are'],
  "you're": ['you', 'are'],
  "i'm": ['i', 'am'],
  "that's": ['that', 'is'],
  "there's": ['there', 'is'],
};

export function normalizeDictationTokens(s: string): string[] {
  const rawTokens = s
    .toLowerCase()
    .replace(/[^\w\s'-]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim()
    .split(' ')
    .filter(Boolean);

  const out: string[] = [];
  for (const tok of rawTokens) {
    const clean = tok.replace(/^'+|'+$/g, '');
    if (!clean) continue;
    if (CONTRACTIONS[clean]) {
      out.push(...CONTRACTIONS[clean]);
    } else {
      out.push(clean);
    }
  }
  return out;
}

export type DiffToken = { kind: 'ok' | 'extra' | 'miss'; word: string };

export function computeWordDiff(userText: string, targetText: string): {
  tokens: DiffToken[];
  pct: number;
  wrongWords: string[];
} {
  const a = normalizeDictationTokens(userText);
  const b = normalizeDictationTokens(targetText);
  const m = a.length;
  const n = b.length;

  const L: number[][] = Array.from({ length: m + 1 }, () => Array(n + 1).fill(0));
  for (let i = m - 1; i >= 0; i--) {
    for (let j = n - 1; j >= 0; j--) {
      L[i][j] = a[i] === b[j] ? L[i + 1][j + 1] + 1 : Math.max(L[i + 1][j], L[i][j + 1]);
    }
  }

  const tokens: DiffToken[] = [];
  const wrongWords: string[] = [];
  let i = 0;
  let j = 0;
  while (i < m || j < n) {
    if (i < m && j < n && a[i] === b[j]) {
      tokens.push({ kind: 'ok', word: b[j] });
      i++;
      j++;
    } else if (i < m && (j >= n || L[i + 1][j] >= L[i][j + 1])) {
      tokens.push({ kind: 'extra', word: a[i] });
      i++;
    } else {
      tokens.push({ kind: 'miss', word: b[j] });
      if (!wrongWords.includes(b[j])) {
        wrongWords.push(b[j]);
      }
      j++;
    }
  }

  const pct = n > 0 ? Math.round((100 * L[0][0]) / n) : 0;
  return { tokens, pct, wrongWords };
}

export function normalizeFlexibleText(s: string): string {
  return s
    .toLowerCase()
    .replace(/[^\w\s']/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
}

export function isErrorFixMatch(input: string, target: string): boolean {
  const normIn = normalizeFlexibleText(input);
  if (!normIn) return false;
  const normTarget = normalizeFlexibleText(target);
  if (normIn === normTarget) return true;

  // Accept any primary fix variant before '(' or split by '/' or '|'
  const baseNoParen = target.split('(')[0].trim();
  if (normalizeFlexibleText(baseNoParen) === normIn) return true;

  const variants = baseNoParen
    .split(/[|/]/)
    .map((v) => normalizeFlexibleText(v))
    .filter(Boolean);
  return variants.some((v) => v === normIn);
}

let cachedVoices: SpeechSynthesisVoice[] = [];

export function initTtsVoices(): void {
  if (typeof window === 'undefined' || !('speechSynthesis' in window)) return;
  const load = () => {
    cachedVoices = window.speechSynthesis.getVoices() || [];
  };
  load();
  window.speechSynthesis.onvoiceschanged = load;
}

export function isTtsSupported(): boolean {
  return typeof window !== 'undefined' && 'speechSynthesis' in window;
}

export function speakText(text: string, accent = 'en-GB', rate = 1.0): void {
  if (!isTtsSupported() || !text.trim()) return;
  window.speechSynthesis.cancel();
  const u = new SpeechSynthesisUtterance(text);
  u.lang = accent === 'en-US' ? 'en-US' : 'en-GB';
  u.rate = rate;

  if (!cachedVoices.length) {
    cachedVoices = window.speechSynthesis.getVoices() || [];
  }
  const targetLang = u.lang.toLowerCase();
  const exact = cachedVoices.find((v) => v.lang.toLowerCase() === targetLang);
  const fallback = cachedVoices.find((v) => v.lang.toLowerCase().startsWith('en'));
  if (exact) u.voice = exact;
  else if (fallback) u.voice = fallback;

  window.speechSynthesis.speak(u);
}
