const fs = require('fs');
let c = fs.readFileSync('src-tauri/Cargo.toml', 'utf8');
c = c.replace(
  'ort = { version = "2.0.0-rc.12", features = ["load-dynamic"] }',
  'ort = "2.0.0-rc.12"'
);
fs.writeFileSync('src-tauri/Cargo.toml', c, 'utf8');
console.log('Removed load-dynamic feature');
