const { execSync } = require('child_process');
const os = require('os');

const platform = os.platform();
const manifestDir = `${__dirname}/../src-tauri`;

const ortDir = platform === 'darwin'
  ? `${manifestDir}/onnxruntime/osx-universal2`
  : platform === 'win32'
    ? `${manifestDir}/onnxruntime/win-x64`
    : `${manifestDir}/onnxruntime/linux-x64`;

process.env.ORT_LIB_LOCATION = ortDir;
if (platform === 'darwin') {
  process.env.ORT_PREFER_DYNAMIC_LINK = '1';
}

const args = process.argv.slice(2).join(' ');
try {
  execSync(`npx tauri ${args}`, { stdio: 'inherit', env: process.env });
} catch (e) {
  process.exit(e.status || 1);
}
