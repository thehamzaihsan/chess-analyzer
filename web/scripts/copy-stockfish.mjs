import { copyFileSync, mkdirSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, '..');
const srcDir = join(root, 'node_modules', 'stockfish', 'src');
const destDir = join(root, 'public', 'stockfish');

mkdirSync(destDir, { recursive: true });

const files = ['stockfish-nnue-16-single.js', 'stockfish-nnue-16-single.wasm'];

for (const file of files) {
  const from = join(srcDir, file);
  const to = join(destDir, file);
  if (!existsSync(from)) {
    console.warn(`[copy-stockfish] missing ${from}, skipping`);
    continue;
  }
  copyFileSync(from, to);
}

console.log('[copy-stockfish] copied engine files to public/stockfish');
