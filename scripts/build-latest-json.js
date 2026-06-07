import fs from 'fs';
import path from 'path';

const version = process.env.BUILD_VERSION;
const pubDate = process.env.BUILD_PUB_DATE;
const notes = process.env.BUILD_NOTES || 'Update available';
const baseUrl = process.env.BUILD_BASE_URL;
const assetsDir = process.env.BUILD_ASSETS_DIR || 'assets';

if (!version || !baseUrl) {
  console.error('Missing required env vars: BUILD_VERSION, BUILD_BASE_URL');
  process.exit(1);
}

const files = fs.readdirSync(assetsDir);
console.log('Available files:', files);

let platforms = {};

// macOS: .app.tar.gz
const macTar = files.find(f => f.endsWith('.app.tar.gz'));
const macSig = files.find(f => f.endsWith('.app.tar.gz.sig'));
if (macTar) {
  let sig = '';
  if (macSig) {
    sig = fs.readFileSync(path.join(assetsDir, macSig), 'utf8').trim();
  }
  platforms['darwin-universal'] = {
    signature: sig,
    url: baseUrl + '/' + encodeURIComponent(macTar)
  };
  console.log('Added darwin-universal:', macTar);
}

// Windows: look for any .zip that's an updater artifact (not .sig)
// Patterns: .nsis.zip, -setup.exe.zip, .msi.zip
const winZip = files.find(f => 
  (f.includes('.nsis.zip') || f.includes('-setup.exe.zip') || f.includes('.msi.zip')) 
  && !f.endsWith('.sig')
);
const winZipSig = files.find(f => 
  (f.includes('.nsis.zip.sig') || f.includes('-setup.exe.zip.sig') || f.includes('.msi.zip.sig'))
);

if (winZip) {
  let sig = '';
  if (winZipSig && fs.existsSync(path.join(assetsDir, winZipSig))) {
    sig = fs.readFileSync(path.join(assetsDir, winZipSig), 'utf8').trim();
  }
  platforms['windows-x86_64'] = {
    signature: sig,
    url: baseUrl + '/' + encodeURIComponent(winZip)
  };
  console.log('Added windows-x86_64:', winZip, '(sig:', winZipSig ? 'yes' : 'no', ')');
} else {
  console.log('No Windows updater zip found in:', files);
}

const latest = { version, notes, pub_date: pubDate, platforms };
const outPath = path.join(assetsDir, 'latest.json');
fs.writeFileSync(outPath, JSON.stringify(latest, null, 2));
console.log('Generated latest.json:', JSON.stringify(latest, null, 2));
