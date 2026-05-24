import fs from 'node:fs';
import path from 'node:path';

function walk(dir, out = []) {
  for (const ent of fs
    ? fs.readdirSync(dir, { withFileTypes: true })
    : []) {
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) walk(p, out);
    else if (ent.name.endsWith('.rs') && ent.name !== 'error.rs') out.push(p);
  }
  return out;
}

function findLastUseEnd(lines, startIdx) {
  let lastEnd = -1;
  let i = startIdx;
  while (i < lines.length) {
    const line = lines[i];
    if (line.startsWith('use ')) {
      let depth = 0;
      let j = i;
      while (j < lines.length) {
        for (const ch of lines[j]) {
          if (ch === '{') depth += 1;
          if (ch === '}') depth -= 1;
        }
        if (depth <= 0 && lines[j].includes(';')) {
          lastEnd = j;
          i = j + 1;
          break;
        }
        j += 1;
      }
      if (j >= lines.length) break;
    } else if (line.trim() === '' || line.startsWith('#')) {
      i += 1;
    } else {
      break;
    }
  }
  return lastEnd;
}

function insertImport(lines) {
  if (lines.some((l) => l.includes('use crate::api::error::CommandError'))) {
    return lines;
  }
  let insertAt = 0;
  while (insertAt < lines.length && lines[insertAt].startsWith('//!')) {
    insertAt += 1;
  }
  while (insertAt < lines.length && lines[insertAt].trim() === '') {
    insertAt += 1;
  }
  const lastUseEnd = findLastUseEnd(lines, insertAt);
  const importLine = 'use crate::api::error::CommandError;';
  if (lastUseEnd >= insertAt) {
    lines.splice(lastUseEnd + 1, 0, importLine);
  } else {
    lines.splice(insertAt, 0, importLine, '');
  }
  return lines;
}

export function migrateApiSource(src) {
  if (!src.includes('to_frontend_error') && !src.includes(', String>')) {
    return src;
  }

  let out = insertImport(src.split('\n')).join('\n');

  out = out.replace(
    /\)\s*->\s*Result<([^,>]+(?:<[^>]+>)?),\s*String>/g,
    ') -> Result<$1, CommandError>',
  );

  out = out.replace(
    /\.map_err\(\|e:\s*crate::error::AppError\|\s*e\.to_frontend_error\(\)\)/g,
    '',
  );
  out = out.replace(/\.map_err\(\|e:\s*AppError\|\s*e\.to_frontend_error\(\)\)/g, '');
  out = out.replace(/\.map_err\(\|e\|\s*e\.to_frontend_error\(\)\)/g, '');
  out = out.replace(
    /\.map_err\(\|e\|\s*AppError::from\(e\)\.to_frontend_error\(\)\)/g,
    '.map_err(AppError::from)',
  );
  out = out.replace(
    /\.map_err\(\|e\|\s*AppError::([^)]+\))\)\.to_frontend_error\(\)\)/g,
    '.map_err(|e| AppError::$1))',
  );
  out = out.replace(/\.map_err\(\|e\|\s*e\.to_string\(\)\)/g, '');

  out = out.replace(/\.to_frontend_error\(\)/g, '');

  out = out.replace(/return Err\(AppError::([^;]+?)\);/gs, (m, inner) => {
    if (/\)\.into\(\)\);$/.test(m.trim())) {
      return m;
    }
    return `return Err(AppError::${inner}.into());`;
  });

  out = out.replace(
    /\.ok_or_else\(\|\|\s*AppError::([^)]+\))\)/g,
  '.ok_or_else(|| AppError::$1.into())',
  );

  out = out.replace(
    /\.map_err\(\|_\|\s*AppError::([^)]+\))\)/g,
    '.map_err(|_| AppError::$1)',
  );

  return out;
}

export function migrateFiles(files) {
  for (const file of files) {
    const src = fs.readFileSync(file, 'utf8');
    const out = migrateApiSource(src);
    if (out !== src) fs.writeFileSync(file, out, 'utf8');
  }
}

if (process.argv[1]?.endsWith('migrate-api-command-error.mjs')) {
  const roots = process.argv.slice(2);
  const files = [];
  for (const root of roots) {
    if (fs.statSync(root).isDirectory()) walk(root, files);
    else files.push(root);
  }
  migrateFiles(files);
}
