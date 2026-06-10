const fs = require('fs');
let c = fs.readFileSync('src/stores/app.ts', 'utf8');
c = c.replace(
  "invoke<number>('mark_image_feedback', { path, is_waste: isWaste })",
  "invoke<number>('mark_image_feedback', { path, isWaste })"
);
fs.writeFileSync('src/stores/app.ts', c, 'utf8');
console.log('Reverted to camelCase isWaste');
