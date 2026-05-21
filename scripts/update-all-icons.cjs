const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const ICONS_DIR = path.join(__dirname, '..', 'src-tauri', 'icons');

async function createIcoFromPng(pngPath, icoPath) {
    // ICO file needs multiple sizes
    const sizes = [16, 32, 48, 64, 128, 256];
    const buffers = [];
    
    for (const size of sizes) {
        const buf = await sharp(pngPath)
            .resize(size, size, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
            .png()
            .toBuffer();
        buffers.push(buf);
    }
    
    // For now, just use the largest PNG as base
    // Full ICO creation requires a specialized library
    // We'll copy the PNG to a temporary location and let Tauri handle ICO generation during build
    fs.copyFileSync(pngPath, icoPath.replace('.ico', '.png'));
    console.log(`Note: ICO requires specialized tool. PNG copy created.`);
}

async function updateAndroidIcons() {
    const androidDir = path.join(ICONS_DIR, 'android');
    if (!fs.existsSync(androidDir)) return;
    
    const mipmapDirs = fs.readdirSync(androidDir).filter(d => d.startsWith('mipmap-'));
    
    for (const dir of mipmapDirs) {
        const launcherPath = path.join(androidDir, dir, 'ic_launcher.png');
        const foregroundPath = path.join(androidDir, dir, 'ic_launcher_foreground.png');
        
        if (fs.existsSync(launcherPath)) {
            // Process launcher icon
            const image = sharp(launcherPath);
            const { width, height } = await image.metadata();
            const raw = await image.raw().ensureAlpha().toBuffer({ resolveWithObject: true });
            const { data, info } = raw;
            const channels = info.channels;
            
            // Simple background removal for Android icons
            for (let y = 0; y < height; y++) {
                for (let x = 0; x < width; x++) {
                    const idx = (y * width + x) * channels;
                    const r = data[idx];
                    const g = data[idx + 1];
                    const b = data[idx + 2];
                    const brightness = (r + g + b) / 3;
                    
                    if (brightness < 50) {
                        data[idx + 3] = 0;
                    }
                }
            }
            
            await sharp(data, {
                raw: { width, height, channels }
            }).png().toFile(launcherPath);
            console.log(`Updated: ${dir}/ic_launcher.png`);
        }
        
        if (fs.existsSync(foregroundPath)) {
            // Foreground usually already transparent, but process anyway
            const image = sharp(foregroundPath);
            const { width, height } = await image.metadata();
            const raw = await image.raw().ensureAlpha().toBuffer({ resolveWithObject: true });
            const { data, info } = raw;
            const channels = info.channels;
            
            for (let y = 0; y < height; y++) {
                for (let x = 0; x < width; x++) {
                    const idx = (y * width + x) * channels;
                    const r = data[idx];
                    const g = data[idx + 1];
                    const b = data[idx + 2];
                    const brightness = (r + g + b) / 3;
                    
                    if (brightness < 30) {
                        data[idx + 3] = 0;
                    }
                }
            }
            
            await sharp(data, {
                raw: { width, height, channels }
            }).png().toFile(foregroundPath);
            console.log(`Updated: ${dir}/ic_launcher_foreground.png`);
        }
    }
}

async function updateIosIcons() {
    const iosDir = path.join(ICONS_DIR, 'ios');
    if (!fs.existsSync(iosDir)) return;
    
    const files = fs.readdirSync(iosDir).filter(f => f.endsWith('.png'));
    
    for (const file of files) {
        const filePath = path.join(iosDir, file);
        const image = sharp(filePath);
        const { width, height } = await image.metadata();
        const raw = await image.raw().ensureAlpha().toBuffer({ resolveWithObject: true });
        const { data, info } = raw;
        const channels = info.channels;
        
        for (let y = 0; y < height; y++) {
            for (let x = 0; x < width; x++) {
                const idx = (y * width + x) * channels;
                const r = data[idx];
                const g = data[idx + 1];
                const b = data[idx + 2];
                const brightness = (r + g + b) / 3;
                
                if (brightness < 50) {
                    data[idx + 3] = 0;
                }
            }
        }
        
        await sharp(data, {
            raw: { width, height, channels }
        }).png().toFile(filePath);
        console.log(`Updated: ios/${file}`);
    }
}

async function main() {
    console.log('Updating Android icons...');
    await updateAndroidIcons();
    
    console.log('\nUpdating iOS icons...');
    await updateIosIcons();
    
    console.log('\nAll platform icons updated!');
    console.log('\nNote: For .ico and .icns files, you may need to regenerate them using:');
    console.log('  npm run tauri icon <path-to-source-png>');
}

main().catch(console.error);
