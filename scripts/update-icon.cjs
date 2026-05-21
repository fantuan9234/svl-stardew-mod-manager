const fs = require('fs');
const path = require('path');

const ICONS_DIR = path.join(__dirname, '..', 'src-tauri', 'icons');

// Copy new-icon.png to be the main source icons
fs.copyFileSync(
    path.join(ICONS_DIR, 'new-icon.png'),
    path.join(ICONS_DIR, 'icon.png')
);
fs.copyFileSync(
    path.join(ICONS_DIR, 'new-icon.png'),
    path.join(ICONS_DIR, 'app-icon.png')
);

console.log('Copied new-icon.png to icon.png and app-icon.png');
console.log('Done!');
