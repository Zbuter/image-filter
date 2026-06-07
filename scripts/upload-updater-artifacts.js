const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const tag = process.env.RELEASE_TAG || '';
const repo = process.env.RELEASE_REPO || '';

if (!tag || !repo) {
  console.error('Missing RELEASE_TAG or RELEASE_REPO env vars');
  process.exit(1);
}

function findFiles(dir, patterns) {
  let results = [];
  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        results = results.concat(findFiles(fullPath, patterns));
      } else if (patterns.some(p => entry.name.endsWith(p))) {
        results.push(fullPath);
      }
    }
  } catch (e) {
    // directory doesn't exist or not readable
  }
  return results;
}

const updaterPatterns = [
  '.tar.gz', '.tar.gz.sig',
  '.nsis.zip', '.nsis.zip.sig',
  '.msi.zip', '.msi.zip.sig'
];

const searchDirs = [
  path.join('src-tauri', 'target', 'release', 'bundle'),
  path.join('src-tauri', 'target', 'universal-apple-darwin', 'release', 'bundle')
];

let files = [];
for (const dir of searchDirs) {
  if (fs.existsSync(dir)) {
    files = files.concat(findFiles(dir, updaterPatterns));
  }
}

console.log('Found updater artifacts:', files);

for (const filepath of files) {
  const filename = path.basename(filepath);
  try {
    console.log('Uploading:', filename);
    execSync(`gh release upload "${tag}" "${filepath}" --repo "${repo}" --clobber`, {
      stdio: 'inherit'
    });
  } catch (e) {
    console.error('Warning: failed to upload', filename);
  }
}

console.log('Done uploading updater artifacts');
