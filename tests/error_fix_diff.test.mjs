import test from 'node:test';
import assert from 'node:assert/strict';
import { evaluateErrorFixAnswer, isErrorFixMatch } from '../src/diff.ts';

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
