const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const ICONS_DIR = path.join(__dirname, '..', 'src-tauri', 'icons');

async function addDarkBackground(inputPath, outputPath) {
    const image = sharp(inputPath);
    const { width, height } = await image.metadata();
    const raw = await image.raw().ensureAlpha().toBuffer({ resolveWithObject: true });
    const { data, info } = raw;
    const channels = info.channels;
    
    const bgColor = { r: 15, g: 20, b: 35 };
    
    for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
            const idx = (y * width + x) * channels;
            
            if (data[idx + 3] === 0) {
                data[idx] = bgColor.r;
                data[idx + 1] = bgColor.g;
                data[idx + 2] = bgColor.b;
                data[idx + 3] = 255;
            }
        }
    }
    
    await sharp(data, {
        raw: { width, height, channels }
    }).png().toFile(outputPath);
}

async function main() {
    const iconsToProcess = [
        'icon.png',
        'app-icon.png',
        '128x128.png',
        '128x128@2x.png',
        '64x64.png',
        '32x32.png',
        'Square150x150Logo.png',
        'Square284x284Logo.png',
        'Square310x310Logo.png',
        'Square44x44Logo.png',
        'Square71x71Logo.png',
        'Square89x89Logo.png',
        'Square107x107Logo.png',
        'Square142x142Logo.png',
        'StoreLogo.png',
    ];
    
    for (const name of iconsToProcess) {
        const inputPath = path.join(ICONS_DIR, name);
        if (!fs.existsSync(inputPath)) continue;
        
        console.log(`Processing: ${name}`);
        
        const tempPath = path.join(ICONS_DIR, 'temp-icon.png');
        await addDarkBackground(inputPath, tempPath);
        
        fs.unlinkSync(inputPath);
        fs.renameSync(tempPath, inputPath);
        console.log(`Restored: ${name}`);
    }
    
    console.log('\nAll icons restored to dark background version!');
}

main().catch(console.error);
