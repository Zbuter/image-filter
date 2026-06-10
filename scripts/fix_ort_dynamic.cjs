const fs = require('fs');
let c = fs.readFileSync('src-tauri/Cargo.toml', 'utf8');
c = c.replace(
  'ort = "2.0.0-rc.12"',
  'ort = { version = "2.0.0-rc.12", features = ["load-dynamic"] }'
);
fs.writeFileSync('src-tauri/Cargo.toml', c, 'utf8');
console.log('Restored load-dynamic feature');
