import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { validateDebtLedger } from './debt-ledger.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const fixture = `# Inventory
## Current
| ID | 项 | 优先级 | 条件 | 状态 |
| --- | --- | --- | --- | --- |
| <a id="debt-d-test-01"></a> **D-TEST-01** | why \\| both | P1 | keep scope | **Partial · Minimal Done / Full OPEN** |
| O-1 | ordinary | P1 | unchanged | **Done** |
| K-PLATFORM-01a | child | P1 | unchanged | Observe |

| ID | 项 | 优先级 | 状态 |
| --- | --- | --- | --- |
| [D-TEST-01](#debt-d-test-01)（引用） | pointer | — | 当前状态只见权威行；历史 Full 已完成是旧快照 |

## §5 历史归档
| ID | 项 | 说明 |
| --- | --- | --- |
| [D-TEST-01](#debt-d-test-01)（历史引用） | past | old Done wording remains |
| OLD-01 | old-only | old Done |
| OLD-01 | another past observation | Deferred at another time |

## 速查坐标
`;

function change(source, from, to) {
  assert.ok(source.includes(from), `mutation target absent: ${from}`);
  return source.replace(from, to);
}

test('separates current records, references and repeated historical-only observations', () => {
  assert.deepEqual(validateDebtLedger(fixture), {
    currentRows: 3,
    historicalOnlyRows: 2,
    references: 2,
    historicalReferences: 1,
    anchors: 1,
    legacyRows: 0,
  });
});

test('accepts the actual inventory without reinterpreting its debt states', () => {
  validateDebtLedger(fs.readFileSync(path.join(root, 'handoff/TECHNICAL_DEBT_INVENTORY.md'), 'utf8'));
});

const mutations = [
  ['duplicate current ID even with a different state',
    '| O-1 | ordinary', '| O-1 | second | P0 | new | OPEN |\n| O-1 | ordinary', /duplicate current row/],
  ['duplicate explicit anchor',
    '| O-1 | ordinary', '| <a id="debt-d-test-01"></a> **D-TEST-01** | another | P0 | new | Done |\n| O-1 | ordinary', /duplicate anchor/],
  ['owner anchor has the wrong ID',
    '<a id="debt-d-test-01">', '<a id="debt-o-1">', /wrong anchor/],
  ['reference points to a different ID',
    '[D-TEST-01](#debt-d-test-01)（引用）', '[D-TEST-01](#debt-o-1)（引用）', /wrong anchor/],
  ['reference has no anchored current owner',
    '<a id="debt-d-test-01"></a> **D-TEST-01**', '**D-TEST-01**', /no matching current anchor owner/],
  ['dangling reference even when its ID and anchor agree',
    '[D-TEST-01](#debt-d-test-01)（引用）', '[UNKNOWN-01](#debt-unknown-01)（引用）', /no matching current anchor owner/],
  ['reference lacks the explicit marker',
    '[D-TEST-01](#debt-d-test-01)（引用）', '[D-TEST-01](#debt-d-test-01)', /malformed ID cell/],
  ['historical marker used in a current table',
    '[D-TEST-01](#debt-d-test-01)（引用）', '[D-TEST-01](#debt-d-test-01)（历史引用）', /marker disagrees/],
  ['current marker used in history',
    '[D-TEST-01](#debt-d-test-01)（历史引用）', '[D-TEST-01](#debt-d-test-01)（引用）', /marker disagrees/],
  ['historical duplicate of a current ID is not a second state row',
    '[D-TEST-01](#debt-d-test-01)（历史引用）', '**D-TEST-01**', /historical repetition/],
  ['historical-only row cannot own a current anchor',
    '| OLD-01 | old-only', '| <a id="debt-old-01"></a> **OLD-01** | old-only', /historical-only row/],
  ['reference status replaced by Done',
    '当前状态只见权威行；历史 Full 已完成是旧快照', '**Done**', /another current declaration/],
  ['reference status adds another declaration',
    '当前状态只见权威行；历史 Full 已完成是旧快照', '当前状态只见权威行；**OPEN**', /another current declaration/],
  ['current Chinese declaration appended to a reference',
    '当前状态只见权威行；历史 Full 已完成是旧快照', '当前状态只见权威行；当前状态：Done', /another current declaration/],
  ['missing reference status column',
    '| pointer | — | 当前状态', '| pointer | 当前状态', /column count/],
  ['ID cell gets unrecognized suffix text',
    '| O-1 | ordinary', '| O-1 unexpected | ordinary', /malformed ID cell/],
  ['ID header changed to hide its rows',
    '| ID | 项 | 优先级 | 条件 | 状态 |', '| other | 项 | 优先级 | 条件 | 状态 |', /outside an ID table/],
  ['case-colliding child IDs',
    '| K-PLATFORM-01a | child', '| K-PLATFORM-01A | alias | P0 | new | OPEN |\n| K-PLATFORM-01a | child', /case-colliding current ID/],
  ['anchor hidden in a table header',
    '| ID | 项 | 优先级 | 条件 | 状态 |', '| ID | <a id="debt-d-test-01"></a> | 优先级 | 条件 | 状态 |', /anchor cannot be in a table header/],
  ['ID table delimiter malformed',
    '| --- | --- | --- | --- | --- |', '| --- | nope | --- | --- | --- |', /matching table separator/],
  ['missing history boundary',
    '## §5 历史归档', '## Old records', /marker disagrees|need one history section/],
  ['misplaced history end',
    '## 速查坐标', '## Current elsewhere', /history must end/],
  ['debt anchor outside an owner cell',
    '# Inventory', '# Inventory\n<a id="debt-o-1"></a>', /anchor must belong/],
  ['extra anchor in a description',
    'why \\| both', 'why <a id="debt-o-1"></a>', /anchor outside/],
  ['missing trailing pipe',
    '| O-1 | ordinary | P1 | unchanged | **Done** |', '| O-1 | ordinary | P1 | unchanged | **Done**', /trailing unescaped pipe/],
  ['missing leading pipe cannot silently drop an ID row',
    '| O-1 | ordinary', 'O-1 | ordinary', /leading pipe/],
  ['unterminated fenced example',
    '## 速查坐标', '## 速查坐标\n```md\nexample', /unterminated code fence/],
];
for (const [name, from, to, error] of mutations) {
  test(`rejects ${name}`, () => assert.throws(() => validateDebtLedger(change(fixture, from, to)), error));
}

test('fenced examples do not become current owners or history boundaries', () => {
  const code = ['~~~md', '## §5 历史归档', '| O-1 | duplicate |', '~~~', ''].join('\n');
  assert.deepEqual(validateDebtLedger(code + fixture), validateDebtLedger(fixture));
});

test('renaming a reference status column cannot hide its direct declaration', () => {
  const declaration = change(fixture, '当前状态只见权威行；历史 Full 已完成是旧快照', '**OPEN**');
  const renamed = change(declaration, '| ID | 项 | 优先级 | 状态 |', '| ID | 项 | 优先级 | 记载 |');
  assert.throws(() => validateDebtLedger(renamed), /another current declaration/);
});

test('legacy non-ID labels remain outside the status and ID counts', () => {
  const legacy = change(fixture, '| O-1 | ordinary', '| **D-OPUS-05 Phase 2** | historical alias | P1 | legacy | Observe |\n| O-1 | ordinary');
  assert.equal(validateDebtLedger(legacy).legacyRows, 1);
  assert.equal(validateDebtLedger(legacy).currentRows, 3);
});

test('an empty inventory cannot pass by skipping every row', () => {
  assert.throws(() => validateDebtLedger('## §5 历史归档\n## 速查坐标\n'), /no current ID rows/);
});

test('CLI returns native 0 for valid input and native 1 for the same duplicate-ID counterexample', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'oclive-debt-ledger-'));
  const file = path.join(directory, 'fixture.md');
  try {
    const invoke = () => spawnSync(process.execPath, [path.join(root, 'scripts/check-debt-ledger.mjs'), '--file', file], { encoding: 'utf8' });
    fs.writeFileSync(file, fixture, 'utf8');
    const good = invoke();
    assert.equal(good.error, undefined);
    assert.equal(good.status, 0, good.stderr);
    assert.match(good.stdout, /structure: PASS/);
    fs.writeFileSync(file, change(fixture, '| O-1 | ordinary', '| O-1 | second | P0 | new | OPEN |\n| O-1 | ordinary'), 'utf8');
    const bad = invoke();
    assert.equal(bad.error, undefined);
    assert.equal(bad.status, 1);
    assert.match(bad.stderr, /duplicate current row/);
    assert.doesNotMatch(bad.stdout, /PASS/);
  } finally {
    if (fs.existsSync(file)) fs.unlinkSync(file);
    fs.rmdirSync(directory);
  }
});
