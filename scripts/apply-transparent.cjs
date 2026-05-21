const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const ICONS_DIR = path.join(__dirname, '..', 'src-tauri', 'icons');

async function removeBackground(inputPath, outputPath) {
    const image = sharp(inputPath);
    const { width, height } = await image.metadata();
    
    const raw = await image.raw().ensureAlpha().toBuffer({ resolveWithObject: true });
    const { data, info } = raw;
    const channels = info.channels;
    
    const samplePoints = [
        [2, 2], [width - 3, 2], [2, height - 3], [width - 3, height - 3],
        [Math.floor(width / 2), 2], [Math.floor(width / 2), height - 3],
        [2, Math.floor(height / 2)], [width - 3, Math.floor(height / 2)],
    ];
    
    let totalR = 0, totalG = 0, totalB = 0;
    for (const [x, y] of samplePoints) {
        const idx = (y * width + x) * channels;
        totalR += data[idx];
        totalG += data[idx + 1];
        totalB += data[idx + 2];
    }
    const bgR = totalR / samplePoints.length;
    const bgG = totalG / samplePoints.length;
    const bgB = totalB / samplePoints.length;
    const bgBrightness = (bgR + bgG + bgB) / 3;
    const isDarkBg = bgBrightness < 80;
    
    for (let y = 0; y < height; y++) {
        for (let x = 0; x < width; x++) {
            const idx = (y * width + x) * channels;
            const r = data[idx];
            const g = data[idx + 1];
            const b = data[idx + 2];
            const brightness = (r + g + b) / 3;
            
            const dist = Math.sqrt(
                Math.pow(r - bgR, 2) +
                Math.pow(g - bgG, 2) +
                Math.pow(b - bgB, 2)
            );
            
            let isBackground = false;
            
            if (isDarkBg) {
                if (dist < 35) {
                    isBackground = true;
                } else if (brightness < 60 && dist < 60) {
                    isBackground = true;
                }
                
                if (brightness > 100) {
                    isBackground = false;
                }
                
                const isGreen = g > r + 20 && g > b + 20;
                const isRed = r > g + 20 && r > b + 20;
                const isYellow = r > 150 && g > 100 && b < 80;
                
                if (isGreen || isRed || isYellow) {
                    isBackground = false;
                }
            } else {
                if (dist < 40) {
                    isBackground = true;
                }
            }
            
            if (isBackground) {
                data[idx + 3] = 0;
            } else {
                data[idx + 3] = 255;
            }
        }
    }
    
    await sharp(data, {
        raw: { width, height, channels }
    }).png().toFile(outputPath);
}

async function processIcon(name) {
    const inputPath = path.join(ICONS_DIR, name);
    if (!fs.existsSync(inputPath)) {
        console.log(`Skip (not found): ${name}`);
        return;
    }
    
    const ext = path.extname(name);
    const baseName = path.basename(name, ext);
    const tempPath = path.join(ICONS_DIR, `${baseName}-temp.png`);
    
    console.log(`Processing: ${name}`);
    await removeBackground(inputPath, tempPath);
    
    // Replace original
    fs.unlinkSync(inputPath);
    fs.renameSync(tempPath, inputPath);
    console.log(`Replaced: ${name}`);
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
    
    for (const icon of iconsToProcess) {
        await processIcon(icon);
    }
    
    // Clean up -transparent files
    const files = fs.readdirSync(ICONS_DIR);
    for (const file of files) {
        if (file.includes('-transparent')) {
            fs.unlinkSync(path.join(ICONS_DIR, file));
            console.log(`Cleaned: ${file}`);
        }
    }
    
    console.log('\nAll icons updated with transparent background!');
}

main().catch(console.error);
