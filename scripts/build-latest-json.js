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

// Windows NSIS: .nsis.zip
const winNsis = files.find(f => f.includes('.nsis.zip') && !f.endsWith('.sig'));
const winNsisSig = files.find(f => f.includes('.nsis.zip.sig'));
if (winNsis) {
  let sig = '';
  if (winNsisSig) {
    sig = fs.readFileSync(path.join(assetsDir, winNsisSig), 'utf8').trim();
  }
  platforms['windows-x86_64'] = {
    signature: sig,
    url: baseUrl + '/' + encodeURIComponent(winNsis)
  };
  console.log('Added windows-x86_64:', winNsis);
}

// Windows MSI fallback: .msi.zip
if (!platforms['windows-x86_64']) {
  const winMsi = files.find(f => f.includes('.msi.zip') && !f.endsWith('.sig'));
  const winMsiSig = files.find(f => f.includes('.msi.zip.sig'));
  if (winMsi) {
    let sig = '';
    if (winMsiSig) {
      sig = fs.readFileSync(path.join(assetsDir, winMsiSig), 'utf8').trim();
    }
    platforms['windows-x86_64'] = {
      signature: sig,
      url: baseUrl + '/' + encodeURIComponent(winMsi)
    };
    console.log('Added windows-x86_64 (msi):', winMsi);
  }
}

const latest = { version, notes, pub_date: pubDate, platforms };
const outPath = path.join(assetsDir, 'latest.json');
fs.writeFileSync(outPath, JSON.stringify(latest, null, 2));
console.log('Generated latest.json:', JSON.stringify(latest, null, 2));
