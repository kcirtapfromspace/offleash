import fs from 'node:fs';
import path from 'node:path';
import url from 'node:url';

const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');
const testsDir = path.resolve(root, 'tests');
const requirementsPath = path.resolve(root, 'coverage', 'requirements.json');

const requirements = JSON.parse(fs.readFileSync(requirementsPath, 'utf8'));
const requiredIds = new Set(requirements.ids);

const foundIds = new Set();

const idRegex = /[A-Z]{3,4}-\d{3}/g;

function walk(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walk(fullPath);
    } else if (entry.isFile() && entry.name.endsWith('.spec.ts')) {
      const content = fs.readFileSync(fullPath, 'utf8');
      const matches = content.match(idRegex) || [];
      for (const match of matches) {
        foundIds.add(match);
      }
    }
  }
}

walk(testsDir);

const missing = [...requiredIds].filter((id) => !foundIds.has(id));
const extra = [...foundIds].filter((id) => !requiredIds.has(id));

if (missing.length) {
  console.error('Missing required test IDs:');
  for (const id of missing) {
    console.error(`- ${id}`);
  }
  process.exit(1);
}

console.log(`Coverage OK: ${foundIds.size}/${requiredIds.size} IDs found.`);

if (extra.length) {
  console.warn('Extra test IDs not in requirements:');
  for (const id of extra) {
    console.warn(`- ${id}`);
  }
}
