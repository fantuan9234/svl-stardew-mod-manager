const fs = require('fs');
const path = require('path');

// Simple PHP to static HTML converter
function renderPhpFile(phpFile, outputFile) {
    // Read all files
    let headerContent = fs.readFileSync(path.join(__dirname, 'header.php'), 'utf8');
    let mainContent = fs.readFileSync(phpFile, 'utf8');
    let footerContent = fs.readFileSync(path.join(__dirname, 'footer.php'), 'utf8');

    // Remove PHP includes from main content
    mainContent = mainContent.replace(/<\?php[\s\S]*?include\s+['"]header\.php['"];[\s\S]*?\?>/g, '');
    mainContent = mainContent.replace(/<\?php[\s\S]*?include\s+['"]footer\.php['"];[\s\S]*?\?>/g, '');

    // Combine all content
    let content = headerContent + '\n' + mainContent + '\n' + footerContent;

    // ===== Process PHP variables =====

    // Replace $pageTitle in title tag
    content = content.replace(/<\?php\s*echo\s+isset\(\$pageTitle\)\s*\?\s*\$pageTitle\s*\.\s*'\s*-\s*'\s*:\s*'';\s*\?>/g, '');
    content = content.replace(/<\?php\s*echo\s+\$pageTitle;\s*\?>/g, '首页');

    // Replace $currentPage conditionals for mobile menu highlighting
    content = content.replace(/<\?php\s*echo\s*\$currentPage=='index\.php'\s*\?\s*'text-white bg-white\/10'\s*:\s*'text-gray-400 hover:bg-white\/5';\s*\?>/g, 'text-white bg-white/10');
    content = content.replace(/<\?php\s*echo\s*\$currentPage=='announcements\.php'\s*\?\s*'text-white bg-white\/10'\s*:\s*'text-gray-400 hover:bg-white\/5';\s*\?>/g, 'text-gray-400 hover:bg-white/5');
    content = content.replace(/<\?php\s*echo\s*\$currentPage=='contact\.php'\s*\?\s*'text-white bg-white\/10'\s*:\s*'text-gray-400 hover:bg-white\/5';\s*\?>/g, 'text-gray-400 hover:bg-white/5');

    // Replace $siteConfig values
    content = content.replace(/<\?php\s*echo\s+htmlspecialchars\(\$siteConfig\['name'\]\);\s*\?>/g, 'SVL - 星露谷物语模组管理器');
    content = content.replace(/<\?php\s*echo\s+htmlspecialchars\(\$siteConfig\['version'\]\);\s*\?>/g, '1.0.2');
    content = content.replace(/<\?php\s*echo\s+htmlspecialchars\(\$siteConfig\['license'\]\);\s*\?>/g, 'MIT');
    content = content.replace(/<\?php\s*echo\s+htmlspecialchars\(\$siteConfig\['size'\]\);\s*\?>/g, '约 15MB');
    content = content.replace(/<\?php\s*echo\s+htmlspecialchars\(\$siteConfig\['github'\]\);\s*\?>/g, 'https://github.com/your-username/svl');
    content = content.replace(/<\?php\s*echo\s+htmlspecialchars\(\$siteConfig\['author'\]\);\s*\?>/g, 'SVL Team');
    content = content.replace(/<\?php\s*echo\s+\$siteConfig\['year'\];\s*\?>/g, new Date().getFullYear().toString());
    content = content.replace(/<\?php\s*echo\s+htmlspecialchars\(\$siteConfig\['tagline'\]\);\s*\?>/g, '让 MOD 管理变得简单、高效、愉悦');
    content = content.replace(/<\?php\s*echo\s+htmlspecialchars\(\$siteConfig\['description'\]\);\s*\?>/g, '专为星露谷物语打造的 MOD 管理工具');

    // Handle SEO meta block - remove the PHP array definition
    content = content.replace(/<\?php\s*\$seoMeta\s*=\s*\[[\s\S]*?\];\s*\$meta\s*=\s*\$seoMeta\[\$pageTitle\]\s*\?\?\s*\[[\s\S]*?\];\s*\?>/g, '');
    content = content.replace(/<\?php\s*echo\s+\$meta\[0\];\s*\?>/g, 'SVL — 一键安装、自动检测冲突、智能备份恢复的 MOD 管理工具');
    content = content.replace(/<\?php\s*echo\s+\$meta\[1\];\s*\?>/g, '星露谷物语,MOD管理器,SMAPI,MOD安装,MOD冲突检测,星露谷MOD,星露谷物语MOD');

    // Replace $currentPage in footer
    content = content.replace(/<\?php\s*echo\s+\$currentPage;\s*\?>/g, 'index.php');
    content = content.replace(/<\?php\s*echo\s+htmlspecialchars\(\$currentPage\);\s*\?>/g, 'index.php');

    // Remove any remaining PHP tags
    content = content.replace(/<\?php[\s\S]*?\?>/g, '');

    // Clean up multiple blank lines
    content = content.replace(/\n{3,}/g, '\n\n');

    fs.writeFileSync(outputFile, content, 'utf8');
    console.log('Rendered: ' + outputFile);
}

// Render index.php
renderPhpFile(
    path.join(__dirname, 'index.php'),
    path.join(__dirname, 'index-preview.html')
);

console.log('Preview file created successfully!');
