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

const MAX_ERROR_FIX_VARIANTS = 32;

function stripExplanatoryParens(s: string): string {
  return s.replace(/\([^)]*\)/g, ' ').replace(/\[[^\]]*\]/g, ' ');
}

function expandContractionTokens(tokens: string[]): string[] {
  const out: string[] = [];
  for (const tok of tokens) {
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

function normalizeErrorFixTokens(s: string): string[] {
  const cleaned = stripExplanatoryParens(s)
    .toLowerCase()
    .replace(/[|;/,.:!?"]/g, ' ')
    .replace(/[^\w\s'-]/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
  if (!cleaned) return [];
  return expandContractionTokens(cleaned.split(' ').filter(Boolean));
}

function expandSlashTokensInClause(clause: string, cap = MAX_ERROR_FIX_VARIANTS): string[] {
  const rawTokens = stripExplanatoryParens(clause)
    .replace(/[|;]/g, ' ')
    .trim()
    .split(/\s+/)
    .filter(Boolean);

  let combos: string[][] = [[]];
  for (const rawTok of rawTokens) {
    // Standalone "/" is a clause/item separator, not an A/B word choice.
    if (/^[/\\,.:!?]+$/.test(rawTok)) continue;

    // Split A/B (e.g., "Humans/People" or "is/are,") into alternative token lists.
    const slashParts = rawTok.split('/');
    const choices: string[][] = [];
    if (slashParts.length > 1) {
      for (const part of slashParts) {
        const toks = normalizeErrorFixTokens(part);
        if (toks.length > 0) choices.push(toks);
      }
    }

    if (choices.length <= 1) {
      const toks = normalizeErrorFixTokens(rawTok);
      if (toks.length === 0) continue;
      for (const c of combos) {
        c.push(...toks);
      }
    } else {
      const next: string[][] = [];
      for (const prefix of combos) {
        for (const ch of choices) {
          next.push([...prefix, ...ch]);
          if (next.length >= cap) break;
        }
        if (next.length >= cap) break;
      }
      combos = next;
    }
  }

  const seen = new Set<string>();
  const out: string[] = [];
  for (const c of combos) {
    const joined = c.join(' ').trim();
    if (joined && !seen.has(joined)) {
      seen.add(joined);
      out.push(joined);
      if (out.length >= cap) break;
    }
  }
  return out;
}

function buildErrorFixVariants(expectedText: string): string[] {
  const stripped = stripExplanatoryParens(expectedText).trim();
  if (!stripped) return [];

  const seen = new Set<string>();
  const variants: string[] = [];
  const pushVariant = (v: string) => {
    if (v && !seen.has(v)) {
      seen.add(v);
      variants.push(v);
    }
  };

  // 1. Full-text A/B expansion (treats standalone "/" or "|" as space)
  for (const v of expandSlashTokensInClause(stripped, MAX_ERROR_FIX_VARIANTS)) {
    pushVariant(v);
    if (variants.length >= MAX_ERROR_FIX_VARIANTS) return variants;
  }

  // 2. Independent alternatives separated by "|", ";", or " / " (with spaces)
  const clauses = stripped
    .split(/\s*[|;]\s*|\s+\/\s+/)
    .map((c) => c.trim())
    .filter(Boolean);
  if (clauses.length > 1) {
    for (const clause of clauses) {
      for (const v of expandSlashTokensInClause(clause, MAX_ERROR_FIX_VARIANTS)) {
        if (v.split(' ').filter(Boolean).length >= 2) {
          pushVariant(v);
          if (variants.length >= MAX_ERROR_FIX_VARIANTS) return variants;
        }
      }
    }
  }

  return variants;
}

function computeLcsLength(a: string[], b: string[]): number {
  const m = a.length;
  const n = b.length;
  const L: number[][] = Array.from({ length: m + 1 }, () => Array(n + 1).fill(0));
  for (let i = m - 1; i >= 0; i--) {
    for (let j = n - 1; j >= 0; j--) {
      L[i][j] = a[i] === b[j] ? L[i + 1][j + 1] + 1 : Math.max(L[i + 1][j], L[i][j + 1]);
    }
  }
  return L[0][0];
}

export function evaluateErrorFixAnswer(
  userInput: string,
  wrongText: string,
  expectedText: string
): { matched: boolean; matchReason: string; variants: string[] } {
  const userTokens = normalizeErrorFixTokens(userInput);
  const normUser = userTokens.join(' ');
  const variants = buildErrorFixVariants(expectedText);

  if (!normUser) {
    return { matched: false, matchReason: 'Chưa nhập câu sửa', variants };
  }

  const wrongTokens = normalizeErrorFixTokens(wrongText);
  const normWrong = wrongTokens.join(' ');
  if (normWrong && normUser === normWrong) {
    return { matched: false, matchReason: 'Vẫn giống nguyên câu sai', variants };
  }

  if (variants.includes(normUser)) {
    return { matched: true, matchReason: 'Khớp biến thể chuẩn', variants };
  }

  // Fuzzy token-diff match (>= 85%) with core wrong-word removal check
  const userTokenSet = new Set(userTokens);
  for (const variant of variants) {
    const varTokens = variant.split(' ').filter(Boolean);
    if (varTokens.length === 0) continue;

    const varSet = new Set(varTokens);
    const wrongOnlyTokens = wrongTokens.filter((w) => !varSet.has(w));
    const stillHasCoreWrong = wrongOnlyTokens.some((w) => userTokenSet.has(w));
    if (stillHasCoreWrong) continue;

    const lcs = computeLcsLength(userTokens, varTokens);
    const denom = Math.max(userTokens.length, varTokens.length);
    const simPct = denom > 0 ? Math.round((100 * lcs) / denom) : 0;
    if (simPct >= 85) {
      return {
        matched: true,
        matchReason: `Khớp thông minh (${simPct}%, đã loại bỏ từ sai)`,
        variants,
      };
    }
  }

  return { matched: false, matchReason: 'Chưa khớp hoàn toàn', variants };
}

export function isErrorFixMatch(input: string, target: string): boolean {
  return evaluateErrorFixAnswer(input, '', target).matched;
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
