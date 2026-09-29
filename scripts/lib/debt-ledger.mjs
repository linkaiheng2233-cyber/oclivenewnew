/** Structural contract for the inventory, not a debt-status adjudicator. */
const ID = '[A-Z][A-Z0-9]*(?:-[A-Za-z0-9]+)+';
const plainId = new RegExp(`^(?:\\*\\*(${ID})\\*\\*|(${ID}))$`);
const bareRow = new RegExp(`^(?:\\*\\*)?${ID}(?:\\*\\*)?\\s*\\|`);
const ownerId = new RegExp(`^<a id=(["'])(debt-[a-z0-9-]+)\\1></a> \\*\\*(${ID})\\*\\*$`);
const referenceId = new RegExp(`^\\[(${ID})\\]\\(#(debt-[a-z0-9-]+)\\)（(历史引用|引用)）$`);
const declaration = /^(?:\*\*|`)?(?:Done|OPEN|Observe|Deferred|Partial|Minimal|Full|Implemented|Locally verified|Blocked)(?=$|[\s:：·*`（(])|^(?:\*\*|`)?(?:冻结|当前状态|当前裁定)/i;
const forward = /^(?:当前|Minimal\/Full 状态|解冻).*只见(?:\s*§\d+(?:\.\d+)?\s*)?权威行$/;

// Existing non-ID labels remain out of scope; new unknown labels fail closed.
const legacyLabels = new Set([
  '**Deep / deep_capsule**',
  '**dual_core** / **expert_routing** / **blueprint v3**',
  '**§3.1**',
  '**模式 3**',
  '~~**模式 2**~~',
  '**F4 / V2-remote**',
  '**§3.5–3.7**',
  '**§5.3**',
  '**D-OPUS-05 Phase 2**',
]);

function fail(line, message) {
  throw new Error(`inventory:${line}: ${message}`);
}

function cellsOf(line, number) {
  if (!line.trim().startsWith('|')) return null;
  const cells = [];
  let cell = '';
  let escaped = false;
  for (const char of line.trim().slice(1)) {
    if (char === '|' && !escaped) {
      cells.push(cell.trim());
      cell = '';
    } else {
      cell += char;
    }
    escaped = char === '\\' && !escaped;
  }
  if (cell !== '') fail(number, 'table row needs a trailing unescaped pipe');
  return cells;
}

function isSeparator(cells) {
  return cells?.length > 0 && cells.every(cell => /^:?-{3,}:?$/.test(cell));
}

function referenceStatus(cells, header, line, historical) {
  if (!historical) {
    for (const cell of cells.slice(2)) {
      if (cell.split(/[；;]/).some(part => declaration.test(part.trim()) && !forward.test(part.trim()))) {
        fail(line, 'reference contains another current declaration');
      }
    }
  }
  const statusColumn = header.indexOf('状态');
  if (statusColumn < 0) return;
  const [pointer, ...notes] = cells[statusColumn].split(/[；;]/).map(part => part.trim());
  if (!forward.test(pointer)) {
    fail(line, 'reference status must forward to the authoritative row');
  }
  // Historical notes may mention old outcomes; direct new declarations may not.
  if (notes.some(note => declaration.test(note))) {
    fail(line, 'reference status contains another current declaration');
  }
}

export function validateDebtLedger(markdown) {
  const records = [];
  const owners = new Map();
  const normalizedOwners = new Map();
  const anchors = new Map();
  let header = null;
  let fence = null;
  let historical = false;
  let historySections = 0;
  let historyEnds = 0;
  let legacyRows = 0;
  const lines = markdown.split(/\r?\n/);

  for (let index = 0; index < lines.length; index++) {
    const line = lines[index];
    const number = index + 1;
    const fenceMark = line.match(/^ {0,3}(`{3,}|~{3,})(.*)$/);
    if (fence) {
      if (fenceMark && fenceMark[1][0] === fence.char
        && fenceMark[1].length >= fence.length && !fenceMark[2].trim()) fence = null;
      continue;
    }
    if (fenceMark) {
      fence = { char: fenceMark[1][0], length: fenceMark[1].length, line: number };
      header = null;
      continue;
    }
    if (/^## /.test(line)) {
      if (historical) {
        if (line !== '## 速查坐标') fail(number, 'history must end at the coordinates section');
        historical = false;
        historyEnds++;
      }
      if (line === '## §5 历史归档') {
        historical = true;
        historySections++;
      }
    }
    const debtAnchors = [...line.matchAll(/\bid\s*=\s*["']?debt-/g)];
    const cells = cellsOf(line, number);
    if (!cells) {
      if (bareRow.test(line.trim())) fail(number, 'ID row needs a leading pipe');
      header = null;
      if (debtAnchors.length) fail(number, 'debt anchor must belong to an ID row');
      continue;
    }
    if (cells[0] === 'ID' || cells[0] === '子 ID') {
      if (debtAnchors.length) fail(number, 'debt anchor cannot be in a table header');
      const separator = cellsOf(lines[index + 1] ?? '', number + 1);
      if (!isSeparator(separator) || separator.length !== cells.length) {
        fail(number, 'ID header needs a matching table separator');
      }
      header = cells;
      continue;
    }
    if (isSeparator(cells)) continue;
    const plain = cells[0]?.match(plainId);
    const owner = cells[0]?.match(ownerId);
    const reference = cells[0]?.match(referenceId);
    if (!header) {
      if (plain || owner || reference || debtAnchors.length) {
        fail(number, 'ID row is outside an ID table');
      }
      continue;
    }
    if (cells.length !== header.length) fail(number, 'ID row column count differs from its header');
    if (legacyLabels.has(cells[0])) {
      if (debtAnchors.length) fail(number, 'legacy label cannot own a debt anchor');
      legacyRows++;
      continue;
    }
    if (!plain && !owner && !reference) fail(number, 'unsupported or malformed ID cell');
    const id = plain ? (plain[1] ?? plain[2]) : owner ? owner[3] : reference[1];
    const anchor = owner ? owner[2] : reference ? reference[2] : null;
    const expectedAnchor = `debt-${id.toLowerCase()}`;
    if (anchor && anchor !== expectedAnchor) fail(number, `${id}: wrong anchor ${anchor}`);
    if (debtAnchors.length !== (owner ? 1 : 0)) fail(number, 'debt anchor outside its owner cell');
    if (owner) {
      if (historical) fail(number, 'historical-only row cannot be a current anchor owner');
      if (anchors.has(anchor)) fail(number, `duplicate anchor ${anchor}`);
      anchors.set(anchor, number);
    }
    if (reference) {
      if ((reference[3] === '历史引用') !== historical) fail(number, 'reference marker disagrees with its section');
      referenceStatus(cells, header, number, historical);
    } else if (!historical) {
      if (owners.has(id)) fail(number, `${id}: duplicate current row (first at ${owners.get(id).line})`);
      if (normalizedOwners.has(id.toLowerCase())) fail(number, `${id}: case-colliding current ID`);
      owners.set(id, { anchor, line: number });
      normalizedOwners.set(id.toLowerCase(), id);
    }
    records.push({ id, anchor, reference: Boolean(reference), historical, line: number });
  }

  if (fence) fail(fence.line, 'unterminated code fence');
  if (historySections !== 1 || historyEnds !== 1 || historical) {
    fail(1, 'need one history section followed by the coordinates section');
  }
  if (!owners.size) fail(1, 'no current ID rows');
  for (const row of records) {
    if (row.reference) {
      const owner = owners.get(row.id);
      if (!owner || owner.anchor !== row.anchor) fail(row.line, `${row.id}: reference has no matching current anchor owner`);
    } else if (row.historical && owners.has(row.id)) {
      fail(row.line, `${row.id}: historical repetition of a current ID must be an explicit reference`);
    }
  }
  return {
    currentRows: owners.size,
    historicalOnlyRows: records.filter(row => row.historical && !row.reference).length,
    references: records.filter(row => row.reference).length,
    historicalReferences: records.filter(row => row.reference && row.historical).length,
    anchors: anchors.size,
    legacyRows,
  };
}
