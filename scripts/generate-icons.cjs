const sharp = require('sharp');
const fs = require('fs');
const path = require('path');

const ICONS_DIR = path.join(__dirname, '..', 'src-tauri', 'icons');
const SOURCE_ICON = path.join(ICONS_DIR, 'icon.png');

async function generateAllIcons() {
    const source = sharp(SOURCE_ICON);
    const { width, height } = await source.metadata();
    
    console.log(`Source icon: ${width}x${height}`);
    
    // Generate Windows ICO with multiple sizes
    const icoSizes = [16, 32, 48, 64, 128, 256];
    for (const size of icoSizes) {
        const outputPath = path.join(ICONS_DIR, `${size}x${size}.png`);
        await sharp(SOURCE_ICON)
            .resize(size, size, { fit: 'contain', background: { r: 0, g: 0, b: 0, alpha: 0 } })
            .png()
            .toFile(outputPath);
        console.log(`Generated: ${size}x${size}.png`);
    }
    
    // Generate app icon
    await sharp(SOURCE_ICON)
        .png()
        .toFile(path.join(ICONS_DIR, 'app-icon.png'));
    console.log('Generated: app-icon.png');
    
    // Generate iOS icons
    const iosSizes = [
        ['AppIcon-20x20@2x.png', 40],
        ['AppIcon-20x20@3x.png', 60],
        ['AppIcon-29x29@2x.png', 58],
        ['AppIcon-29x29@3x.png', 87],
        ['AppIcon-40x40@2x.png', 80],
        ['AppIcon-40x40@3x.png', 120],
        ['AppIcon-60x60@2x.png', 120],
        ['AppIcon-60x60@3x.png', 180],
        ['AppIcon-76x76@2x.png', 152],
        ['AppIcon-83.5x83.5@2x.png', 167],
        ['AppIcon-1024x1024@1x.png', 1024],
    ];
    
    const iosDir = path.join(ICONS_DIR, 'ios');
    if (!fs.existsSync(iosDir)) fs.mkdirSync(iosDir, { recursive: true });
    
    for (const [name, size] of iosSizes) {
        await sharp(SOURCE_ICON)
            .resize(size, size, { fit: 'contain' })
            .png()
            .toFile(path.join(iosDir, name));
        console.log(`Generated: ios/${name}`);
    }
    
    // Generate Android mipmap icons
    const androidSizes = [
        ['mipmap-mdpi', 48],
        ['mipmap-hdpi', 72],
        ['mipmap-xhdpi', 96],
        ['mipmap-xxhdpi', 144],
        ['mipmap-xxxhdpi', 192],
    ];
    
    const androidDir = path.join(ICONS_DIR, 'android');
    for (const [folder, size] of androidSizes) {
        const dirPath = path.join(androidDir, folder);
        if (!fs.existsSync(dirPath)) fs.mkdirSync(dirPath, { recursive: true });
        
        await sharp(SOURCE_ICON)
            .resize(size, size, { fit: 'contain' })
            .png()
            .toFile(path.join(dirPath, 'ic_launcher.png'));
        console.log(`Generated: android/${folder}/ic_launcher.png`);
    }
    
    // Generate Windows Store icons
    const storeSizes = [
        ['Square44x44Logo.png', 44],
        ['Square71x71Logo.png', 71],
        ['Square89x89Logo.png', 89],
        ['Square107x107Logo.png', 107],
        ['Square142x142Logo.png', 142],
        ['Square150x150Logo.png', 150],
        ['Square284x284Logo.png', 284],
        ['Square310x310Logo.png', 310],
        ['StoreLogo.png', 50],
    ];
    
    for (const [name, size] of storeSizes) {
        await sharp(SOURCE_ICON)
            .resize(size, size, { fit: 'contain' })
            .png()
            .toFile(path.join(ICONS_DIR, name));
        console.log(`Generated: ${name}`);
    }
    
    // Copy icon.png to a temp file for tauri icon command
    fs.copyFileSync(SOURCE_ICON, path.join(ICONS_DIR, 'source-for-build.png'));
    console.log('\nGenerated source-for-build.png for tauri icon command');
    console.log('All icons generated successfully!');
}

generateAllIcons().catch(console.error);
