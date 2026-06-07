import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

const tag = process.env.RELEASE_TAG || '';
const repo = process.env.RELEASE_REPO || '';

if (!tag || !repo) {
  console.error('Missing RELEASE_TAG or RELEASE_REPO env vars');
  process.exit(1);
}

function findFilesRecursive(dir) {
  let results = [];
  try {
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);
      if (entry.isDirectory()) {
        results = results.concat(findFilesRecursive(fullPath));
      } else {
        results.push(fullPath);
      }
    }
  } catch (e) {
    // ignore
  }
  return results;
}

// Search all bundle directories
const targetDir = path.join('src-tauri', 'target');
let allBundleFiles = [];

// Check both release and universal-apple-darwin/release paths
for (const subPath of ['release', 'universal-apple-darwin/release']) {
  const bundleDir = path.join(targetDir, subPath, 'bundle');
  if (fs.existsSync(bundleDir)) {
    console.log('Scanning:', bundleDir);
    allBundleFiles = allBundleFiles.concat(findFilesRecursive(bundleDir));
  }
}

console.log('All bundle files found:', allBundleFiles.length);
allBundleFiles.forEach(f => console.log('  ', f));

// Filter to updater artifacts only
const updaterFiles = allBundleFiles.filter(f => {
  const name = path.basename(f);
  return name.endsWith('.tar.gz') || 
         name.endsWith('.tar.gz.sig') ||
         name.endsWith('.nsis.zip') || 
         name.endsWith('.nsis.zip.sig') ||
         name.endsWith('.msi.zip') || 
         name.endsWith('.msi.zip.sig');
});

console.log('Updater artifacts to upload:', updaterFiles);

for (const filepath of updaterFiles) {
  const filename = path.basename(filepath);
  try {
    console.log('Uploading:', filename);
    execSync(`gh release upload "${tag}" "${filepath}" --repo "${repo}" --clobber`, {
      stdio: 'inherit'
    });
    console.log('  Success:', filename);
  } catch (e) {
    console.error('  Warning: failed to upload', filename);
  }
}

console.log('Done uploading updater artifacts');
