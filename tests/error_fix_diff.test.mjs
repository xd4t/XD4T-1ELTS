import test from 'node:test';
import assert from 'node:assert/strict';
import {
  evaluateErrorFixAnswer,
  extractDictationErrorPairs,
  findClozeMatchRange,
  isErrorFixMatch,
  makeCardClozeHtml,
} from '../src/diff.ts';

test('exact reported bug: Humans have / Students need vs Humans/People have / Students need', () => {
  const res = evaluateErrorFixAnswer(
    'Humans have / Students need',
    'human have / Student need',
    'Humans/People have / Students need'
  );
  assert.equal(res.matched, true);
  assert.ok(res.variants.includes('humans have students need'));
  assert.ok(res.variants.includes('people have students need'));
});

test('separators and punctuation variations pass (comma, semicolon, no slash)', () => {
  for (const input of [
    'People have, Students need',
    'Humans have; Students need',
    'People have Students need',
  ]) {
    const res = evaluateErrorFixAnswer(
      input,
      'human have / Student need',
      'Humans/People have / Students need'
    );
    assert.equal(res.matched, true, `Expected "${input}" to match`);
  }
});

test('parenthetical explanations and contractions are normalized', () => {
  const res = evaluateErrorFixAnswer(
    'They do not agree (chủ ngữ số nhiều)',
    'They does not agree',
    "They don't/do not agree (chia động từ số nhiều)"
  );
  assert.equal(res.matched, true);
});

test('pipe and slash-separated independent alternatives (>= 2 words) pass', () => {
  const resPipe = evaluateErrorFixAnswer(
    'Much of the populace tends to',
    'Many people tend to',
    'Much of the populace tends to | may arguably precipitate'
  );
  assert.equal(resPipe.matched, true);

  const resClause = evaluateErrorFixAnswer(
    'Students need',
    'human have / Student need',
    'Humans/People have / Students need'
  );
  assert.equal(resClause.matched, true);
});

test('repeating exact wrongText is rejected', () => {
  const res = evaluateErrorFixAnswer(
    'human have / Student need',
    'human have / Student need',
    'Humans/People have / Students need'
  );
  assert.equal(res.matched, false);
});

test('fuzzy >= 85% match passes when core wrong word is fixed, fails when core wrong word remains', () => {
  const ok = evaluateErrorFixAnswer(
    'The government should invest heavily in public transportations',
    'The government should invests heavily in public transport',
    'The government should invest heavily in public transport'
  );
  assert.equal(ok.matched, true);

  const stillWrong = evaluateErrorFixAnswer(
    'The government should invests heavily in public transport',
    'The government should invests heavily in public transport',
    'The government should invest heavily in public transport'
  );
  assert.equal(stillWrong.matched, false);
});

test('isErrorFixMatch delegates cleanly', () => {
  assert.equal(isErrorFixMatch('People have / Students need', 'Humans/People have / Students need'), true);
  assert.equal(isErrorFixMatch('', 'Humans/People have / Students need'), false);
});

test('makeCardClozeHtml handles inflected verbs, multi-word modifiers, and never leaks target phrase', () => {
  const html1 = makeCardClozeHtml({
    word: 'play a pivotal role in',
    example: 'Automation plays a pivotal role in enhancing industrial efficiency.',
    meaning: 'đóng vai trò then chốt',
  });
  assert.ok(html1.includes('<span class="fc-blank">'));
  assert.ok(!html1.toLowerCase().includes('plays a pivotal role in'));

  const html2 = makeCardClozeHtml({
    word: 'deplete',
    example: 'Overconsumption depleted finite natural resources.',
    meaning: 'làm cạn kiệt',
  });
  assert.ok(html2.includes('<span class="fc-blank">'));
  assert.ok(!html2.toLowerCase().includes('depleted'));

  const html3 = makeCardClozeHtml({
    word: 'pose a threat to',
    example: 'Unregulated data collection can pose a serious threat to privacy.',
    meaning: 'gây mối đe dọa',
  });
  assert.ok(html3.includes('<span class="fc-blank">'));
  assert.ok(findClozeMatchRange('Automation plays a pivotal role in work.', 'play a pivotal role in') !== null);
});

test('extractDictationErrorPairs extracts up to 3 clean wrong->right pairs', () => {
  const pairs = extractDictationErrorPairs(
    'The library open at nine on wednesday except during august',
    'The community library opens at nine on Wednesdays except during August.',
    3
  );
  assert.ok(pairs.length >= 2 && pairs.length <= 3);
  assert.ok(pairs.some((p) => p.right === 'community'));
  assert.ok(pairs.some((p) => p.wrong === 'open' && p.right === 'opens'));
});
