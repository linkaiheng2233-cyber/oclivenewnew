// Author helper for this example only. It does not validate or generate projects.
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

if (process.argv.length !== 2) throw new Error('This example helper takes no arguments');
const root = fs.realpathSync(path.join(path.dirname(fileURLToPath(import.meta.url)), 'package'));
const instructionPath = path.join(root, 'instructions.json');
const manifestPath = path.join(root, 'oclive.scaffold.json');
for (const file of [instructionPath, manifestPath]) {
  if (fs.realpathSync(file) !== file) throw new Error(`Example metadata must not be a link: ${file}`);
}
const instruction = JSON.parse(fs.readFileSync(instructionPath, 'utf8'));
const manifest = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
const inside = (file) => {
  const relative = path.relative(root, file);
  return relative !== '' && !path.isAbsolute(relative) && relative !== '..' && !relative.startsWith(`..${path.sep}`);
};
for (const file of instruction.files) {
  const source = fs.realpathSync(path.resolve(root, file.source));
  if (!inside(source)) throw new Error(`Example source outside package: ${file.source}`);
  file.sha256 = sha256(fs.readFileSync(source));
}
const generator = manifest.generators.find((entry) => entry.id === 'project');
if (!generator || generator.driver.kind !== 'instruction' || generator.driver.path !== 'instructions.json') {
  throw new Error('This helper only updates the example project generator');
}
const bytes = Buffer.from(`${JSON.stringify(instruction, null, 2)}\n`, 'utf8');
generator.driver.sha256 = sha256(bytes);
fs.writeFileSync(instructionPath, bytes);
fs.writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8');
console.log('Example digests updated. Validate, inspect and explicitly refresh the source lock before generating.');
