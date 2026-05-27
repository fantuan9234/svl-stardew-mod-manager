<!DOCTYPE html>
<html lang="<?php echo $currentLang === 'zh-TW' ? 'zh-TW' : ($currentLang === 'en' ? 'en' : 'zh-CN'); ?>">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title><?php echo (isset($pageTitle) && $pageTitle !== '首页') ? $pageTitle . ' - ' : ''; ?><?php echo t('site_name'); ?></title>
<?php
$seoMeta = [
    '首页' => [t('seo_home_desc'), '星露谷物语MOD,星露谷物语MOD管理器,星露谷物语MOD安装,星露谷MOD管理,Stardew Valley MOD,星露谷物语,MOD管理器,SMAPI,MOD安装,MOD冲突检测'],
    '公告' => [t('seo_announcements_desc'), '星露谷物语MOD管理器,更新日志,公告,版本更新,星露谷物语'],
    '更新日志' => [t('seo_changelog_desc'), 'SVL,更新日志,版本历史,Changelog,星露谷物语MOD管理器'],
    '联系我们' => [t('seo_contact_desc'), '星露谷物语MOD管理器,联系我们,反馈,合作'],
];
$meta = $seoMeta[$pageTitle] ?? [t('seo_home_desc'), 'Stardew Valley,MOD Manager,SMAPI,MOD install'];
$pageSlugMap = ['首页' => '', '公告' => 'announcements.php', '更新日志' => 'changelog.php', '联系我们' => 'contact.php'];
$pageSlug = $pageSlugMap[$pageTitle] ?? '';
$canonicalUrl = 'https://svlmod.cn/' . $pageSlug;
?>
    <meta name="description" content="<?php echo $meta[0]; ?>">
    <meta name="keywords" content="<?php echo $meta[1]; ?>">
    <meta name="robots" content="index, follow, max-image-preview:large, max-snippet:-1, max-video-preview:-1">
    <meta name="baidu-site-verification" content="code-请替换为你的百度站长验证码">
    <meta name="author" content="SVL Team">
<?php
$currentPageFile = basename($_SERVER['SCRIPT_NAME']);
$currentPage = $currentPageFile ?: 'index.php';
?>
    <meta property="og:site_name" content="<?php echo t('site_name'); ?>">
    <meta property="og:title" content="<?php echo $pageTitle; ?> - <?php echo t('site_name'); ?>">
    <meta property="og:description" content="<?php echo $meta[0]; ?>">
    <meta property="og:url" content="<?php echo h($canonicalUrl); ?>">
    <meta property="og:image" content="https://svlmod.cn/assets/og-image.png">
    <meta property="og:image:width" content="1200">
    <meta property="og:image:height" content="630">
    <meta property="og:image:alt" content="<?php echo t('site_name'); ?>">
    <meta property="og:type" content="website">
    <meta property="og:locale" content="<?php echo $currentLang === 'zh-TW' ? 'zh_TW' : ($currentLang === 'en' ? 'en_US' : 'zh_CN'); ?>">
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="<?php echo $pageTitle; ?> - <?php echo t('site_name'); ?>">
    <meta name="twitter:description" content="<?php echo $meta[0]; ?>">
    <meta name="twitter:image" content="https://svlmod.cn/assets/og-image.png">
    <link rel="canonical" href="<?php echo h($canonicalUrl); ?>">
    <link rel="alternate" hreflang="zh" href="<?php echo h($canonicalUrl); ?>?lang=zh">
    <link rel="alternate" hreflang="zh-Hant" href="<?php echo h($canonicalUrl); ?>?lang=zh-TW">
    <link rel="alternate" hreflang="en" href="<?php echo h($canonicalUrl); ?>?lang=en">
    <link rel="alternate" hreflang="x-default" href="<?php echo h($canonicalUrl); ?>">
    <script type="application/ld+json">
    {
        "@context": "https://schema.org",
        "@type": "SoftwareApplication",
        "name": "<?php echo t('site_name'); ?>",
        "applicationCategory": "UtilitiesApplication",
        "operatingSystem": "Windows",
        "description": "<?php echo t('seo_home_desc'); ?>",
        "url": "https://svlmod.cn",
        "image": "https://svlmod.cn/assets/icon.png",
        "offers": {
            "@type": "Offer",
            "price": "0",
            "priceCurrency": "CNY"
        },
        "author": {
            "@type": "Organization",
            "name": "SVL Team"
        },
        "inLanguage": ["zh-CN", "zh-TW", "en"]
    }
    </script>
<?php
$breadcrumbMap = [
    '首页' => [['name' => t('nav_home'), 'url' => 'https://svlmod.cn/']],
    '公告' => [['name' => t('nav_home'), 'url' => 'https://svlmod.cn/'], ['name' => t('nav_announcements'), 'url' => 'https://svlmod.cn/announcements.php']],
    '更新日志' => [['name' => t('nav_home'), 'url' => 'https://svlmod.cn/'], ['name' => t('nav_changelog'), 'url' => 'https://svlmod.cn/changelog.php']],
    '联系我们' => [['name' => t('nav_home'), 'url' => 'https://svlmod.cn/'], ['name' => t('nav_contact'), 'url' => 'https://svlmod.cn/contact.php']],
];
$breadcrumbItems = $breadcrumbMap[$pageTitle] ?? $breadcrumbMap['首页'];
$breadcrumbLdJson = [
    '@context' => 'https://schema.org',
    '@type' => 'BreadcrumbList',
    'itemListElement' => array_map(function($i, $idx) {
        return [
            '@type' => 'ListItem',
            'position' => $idx + 1,
            'name' => $i['name'],
            'item' => $i['url']
        ];
    }, $breadcrumbItems, array_keys($breadcrumbItems))
];

$orgLdJson = [
    '@context' => 'https://schema.org',
    '@type' => 'Organization',
    'name' => t('site_name'),
    'url' => 'https://svlmod.cn',
    'logo' => 'https://svlmod.cn/assets/icon.png',
    'sameAs' => [
        'https://space.bilibili.com/3546621436496190',
        'https://www.douyin.com/user/self?from_tab_name=main',
    ],
];

$webSiteLdJson = [
    '@context' => 'https://schema.org',
    '@type' => 'WebSite',
    'name' => t('site_name'),
    'url' => 'https://svlmod.cn',
    'potentialAction' => [
        '@type' => 'SearchAction',
        'target' => 'https://svlmod.cn/?s={search_term_string}',
        'query-input' => 'required name=search_term_string',
    ],
];

$webPageLdJson = [
    '@context' => 'https://schema.org',
    '@type' => 'WebPage',
    'name' => ($pageTitle ?? t('nav_home')) . ' - ' . t('site_name'),
    'description' => $meta[0],
    'url' => $canonicalUrl,
    'inLanguage' => $currentLang === 'zh-TW' ? 'zh-Hant' : ($currentLang === 'en' ? 'en' : 'zh-CN'),
    'isPartOf' => ['@id' => 'https://svlmod.cn/#website'],
    'breadcrumb' => ['@id' => '#breadcrumb'],
];
?>
    <script type="application/ld+json"><?php echo json_encode($breadcrumbLdJson, JSON_UNESCAPED_UNICODE | JSON_PRETTY_PRINT); ?></script>
    <script type="application/ld+json"><?php echo json_encode($orgLdJson, JSON_UNESCAPED_UNICODE | JSON_PRETTY_PRINT); ?></script>
    <script type="application/ld+json"><?php echo json_encode($webSiteLdJson, JSON_UNESCAPED_UNICODE | JSON_PRETTY_PRINT); ?></script>
    <script type="application/ld+json"><?php echo json_encode($webPageLdJson, JSON_UNESCAPED_UNICODE | JSON_PRETTY_PRINT); ?></script>
    <link rel="dns-prefetch" href="//cdn.tailwindcss.com">
    <link rel="dns-prefetch" href="//fonts.googleapis.com">
    <link rel="dns-prefetch" href="//fonts.gstatic.com">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700&family=Noto+Sans+SC:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="icon" type="image/png" href="assets/icon.png">
    <style>
        :root {
            --bg: #0c0c0e;
            --surface: #141416;
            --surface-hover: #1c1c1f;
            --text: #f5f5f7;
            --text-secondary: #86868b;
            --text-tertiary: #525256;
            --border: rgba(255,255,255,0.06);
            --border-hover: rgba(255,255,255,0.1);
            --brand: #d4a843;
            --brand-dim: rgba(212,168,67,0.12);
            --shadow-sm: 0 2px 8px rgba(0,0,0,0.2);
            --shadow-md: 0 8px 30px rgba(0,0,0,0.3);
            --shadow-lg: 0 20px 60px rgba(0,0,0,0.4);
            --header-bg: rgba(12,12,14,0.85);
            --input-bg: rgba(255,255,255,0.03);
            --tag-green-bg: rgba(91,140,90,0.15);
            --tag-green-text: #6abf69;
            --tag-blue-bg: rgba(100,130,180,0.15);
            --tag-blue-text: #6a9fd8;
            --tag-red-bg: rgba(180,80,60,0.15);
            --tag-red-text: #d87868;
            --success-bg: rgba(91,140,90,0.15);
            --success-border: rgba(91,140,90,0.3);
            --success-text: #6abf69;
            --error-bg: rgba(180,80,60,0.1);
            --error-border: rgba(180,80,60,0.25);
            --error-text: #d87868;
            --gradient-text-start: #f5f5f7;
            --gradient-text-end: #d4a843;
            --sponsor-bg: linear-gradient(135deg, #0a1a2e 0%, #112240 50%, #0d1b2a 100%);
            --sponsor-grid: rgba(255,255,255,0.02);
            --sponsor-glow: rgba(0,212,255,0.15);
            --sponsor-title: #fff;
            --sponsor-subtitle: #00d4ff;
            --sponsor-desc: rgba(255,255,255,0.5);
            --sponsor-badge-bg: rgba(255,255,255,0.06);
            --sponsor-badge-border: rgba(255,255,255,0.08);
            --sponsor-badge-text: rgba(255,255,255,0.75);
            --sponsor-link: #00d4ff;
            --sponsor-tag-bg: rgba(0,255,136,0.1);
            --sponsor-tag-text: #00ff88;
            --download-bg: linear-gradient(135deg, rgba(212,168,67,0.08) 0%, rgba(20,20,22,0.8) 50%, rgba(12,12,14,0.9) 100%);
            --screenshot-border: rgba(255,255,255,0.08);
            --screenshot-shadow: 0 25px 80px rgba(0,0,0,0.4);
            --screenshot-glow: linear-gradient(135deg, rgba(212,168,67,0.15) 0%, rgba(180,170,150,0.08) 100%);
        }

        body.light-theme {
            --bg: #f0ede8;
            --surface: #faf8f5;
            --surface-hover: #f2efea;
            --text: #2d2a26;
            --text-secondary: #5a5450;
            --text-tertiary: #7a7470;
            --border: rgba(60,50,40,0.08);
            --border-hover: rgba(60,50,40,0.15);
            --brand: #c4953a;
            --brand-dim: rgba(196,149,58,0.12);
            --shadow-sm: 0 2px 8px rgba(60,50,40,0.04);
            --shadow-md: 0 8px 30px rgba(60,50,40,0.06);
            --shadow-lg: 0 20px 60px rgba(60,50,40,0.08);
            --header-bg: rgba(240,237,232,0.9);
            --input-bg: rgba(60,50,40,0.03);
            --tag-green-bg: rgba(91,140,90,0.1);
            --tag-green-text: #5b8c5a;
            --tag-blue-bg: rgba(100,130,180,0.1);
            --tag-blue-text: #5a7fb8;
            --tag-red-bg: rgba(180,80,60,0.1);
            --tag-red-text: #b05040;
            --success-bg: rgba(91,140,90,0.12);
            --success-border: rgba(91,140,90,0.25);
            --success-text: #4a7a49;
            --error-bg: rgba(180,80,60,0.08);
            --error-border: rgba(180,80,60,0.2);
            --error-text: #b05040;
            --gradient-text-start: #2d2a26;
            --gradient-text-end: #c4953a;
            --sponsor-bg: linear-gradient(135deg, #f5f0e8 0%, #ebe5db 50%, #f0ebe0 100%);
            --sponsor-grid: rgba(60,50,40,0.04);
            --sponsor-glow: rgba(196,149,58,0.12);
            --sponsor-title: #2d2a26;
            --sponsor-subtitle: #8a6d3b;
            --sponsor-desc: rgba(60,50,40,0.55);
            --sponsor-badge-bg: rgba(196,149,58,0.08);
            --sponsor-badge-border: rgba(196,149,58,0.12);
            --sponsor-badge-text: rgba(60,50,40,0.65);
            --sponsor-link: #8a6d3b;
            --sponsor-tag-bg: rgba(91,140,90,0.1);
            --sponsor-tag-text: #5b8c5a;
            --download-bg: linear-gradient(135deg, rgba(196,149,58,0.06) 0%, rgba(240,237,232,0.8) 50%, rgba(235,232,227,0.9) 100%);
            --screenshot-border: rgba(60,50,40,0.08);
            --screenshot-shadow: 0 25px 80px rgba(60,50,40,0.08);
            --screenshot-glow: linear-gradient(135deg, rgba(196,149,58,0.15) 0%, rgba(180,170,150,0.08) 100%);
        }

        body {
            font-family: 'Outfit', 'Noto Sans SC', -apple-system, sans-serif;
            background-color: var(--bg);
            color: var(--text);
            line-height: 1.6;
            -webkit-font-smoothing: antialiased;
            transition: background-color 0.3s ease;
        }

        body.theme-transitioning {
            transition: none;
        }
        body.theme-transitioning * {
            transition: none !important;
        }

        body::after {
            content: '';
            position: fixed;
            inset: 0;
            background: radial-gradient(ellipse 80% 50% at 20% 40%, rgba(212,168,67,0.04) 0%, transparent 60%);
            pointer-events: none;
            z-index: -1;
            opacity: 0;
        }
        body.light-theme::after {
            opacity: 1;
        }

        * { margin: 0; padding: 0; box-sizing: border-box; }
        html { scroll-behavior: smooth; }

        .card {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 20px;
            padding: 32px;
            transition: transform 0.3s ease, box-shadow 0.3s ease;
            position: relative;
            overflow: hidden;
        }
        .card:hover {
            transform: translateY(-4px);
            box-shadow: var(--shadow-lg);
            border-color: var(--border-hover);
        }
        .card::before {
            content: '';
            position: absolute;
            top: 0; left: 0; right: 0;
            height: 1px;
            background: linear-gradient(90deg, transparent, var(--brand-dim), transparent);
        }

        .btn-primary {
            display: inline-flex; align-items: center; gap: 10px;
            padding: 12px 28px;
            background: var(--brand);
            color: var(--bg);
            font-size: 14px; font-weight: 600;
            border-radius: 100px;
            text-decoration: none;
            border: none; cursor: pointer;
            position: relative; overflow: hidden;
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }
        .btn-primary:hover {
            transform: translateY(-2px);
            box-shadow: 0 8px 30px var(--brand-dim);
        }

        .btn-ghost {
            display: inline-flex; align-items: center; gap: 10px;
            padding: 12px 28px;
            background: transparent;
            color: var(--text);
            font-size: 14px; font-weight: 500;
            border-radius: 100px;
            text-decoration: none;
            border: 1px solid var(--border);
            cursor: pointer;
            transition: transform 0.3s ease, background 0.3s ease;
        }
        .btn-ghost:hover {
            background: var(--surface-hover);
            transform: translateY(-2px);
        }

        .social-card {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 20px;
            padding: 32px;
            transition: transform 0.3s ease, box-shadow 0.3s ease;
        }
        .social-card:hover {
            transform: translateY(-4px);
            box-shadow: var(--shadow-md);
        }

        .divider {
            height: 1px;
            background: linear-gradient(90deg, transparent, var(--border), transparent);
        }

        .gradient-text {
            background: linear-gradient(135deg, var(--gradient-text-start) 0%, var(--gradient-text-end) 100%);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
        }

        .contact-input {
            width: 100%;
            padding: 14px 18px;
            background: var(--input-bg);
            border: 1px solid var(--border);
            border-radius: 14px;
            color: var(--text);
            font-size: 15px;
            font-family: inherit;
            outline: none;
            transition: border-color 0.3s ease, box-shadow 0.3s ease;
        }
        .contact-input:focus {
            border-color: var(--brand);
            box-shadow: 0 0 0 3px var(--brand-dim);
        }
        .contact-input::placeholder { color: var(--text-tertiary); }

        .section-label {
            display: inline-block;
            font-size: 12px;
            font-weight: 600;
            letter-spacing: 0.15em;
            text-transform: uppercase;
            color: var(--brand);
            margin-bottom: 16px;
        }

        .tag-green { background: var(--tag-green-bg); color: var(--tag-green-text); }
        .tag-blue { background: var(--tag-blue-bg); color: var(--tag-blue-text); }
        .tag-red { background: var(--tag-red-bg); color: var(--tag-red-text); }

        .feedback-success { background: var(--success-bg); border: 1px solid var(--success-border); color: var(--success-text); }
        .feedback-error { background: var(--error-bg); border: 1px solid var(--error-border); color: var(--error-text); }

        .sponsor-card {
            background: var(--sponsor-bg);
            border: 1px solid var(--border);
            border-radius: 20px;
            padding: 32px;
            position: relative;
            overflow: hidden;
            transition: transform 0.3s ease;
        }
        .sponsor-card:hover { transform: translateY(-2px); }
        .sponsor-grid {
            position: absolute;
            inset: 0;
            background-image: linear-gradient(var(--sponsor-grid) 1px, transparent 1px), linear-gradient(90deg, var(--sponsor-grid) 1px, transparent 1px);
            background-size: 32px 32px;
            pointer-events: none;
        }
        .sponsor-glow {
            position: absolute;
            top: 0; right: 0;
            width: 300px; height: 300px;
            background: radial-gradient(circle, var(--sponsor-glow) 0%, transparent 70%);
            pointer-events: none;
        }
        .sponsor-title { color: var(--sponsor-title); }
        .sponsor-subtitle { color: var(--sponsor-subtitle); }
        .sponsor-desc { color: var(--sponsor-desc); }
        .sponsor-badge {
            background: var(--sponsor-badge-bg);
            border: 1px solid var(--sponsor-badge-border);
            color: var(--sponsor-badge-text);
        }
        .sponsor-link { color: var(--sponsor-link); }
        .sponsor-tag {
            background: var(--sponsor-tag-bg);
            color: var(--sponsor-tag-text);
        }

        .download-card {
            background: var(--download-bg);
            border: 1px solid var(--border);
            border-radius: 20px;
        }

        .screenshot-glow { background: var(--screenshot-glow); }
        .screenshot-img {
            border: 1px solid var(--screenshot-border);
            box-shadow: var(--screenshot-shadow);
        }

        .collapse-nav {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 14px;
            overflow: hidden;
            transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
            min-width: 200px;
            position: relative;
        }
        .collapse-nav-wrapper {
            position: absolute;
            top: 100%; left: 0; right: 0;
            margin-top: 8px;
            z-index: 100;
            opacity: 0;
            transform: translateY(-8px);
            pointer-events: none;
            transition: opacity 0.3s ease, transform 0.3s ease;
        }
        .collapse-nav-wrapper.open {
            opacity: 1;
            transform: translateY(0);
            pointer-events: auto;
        }
        .collapse-nav-header {
            display: flex; align-items: center; justify-content: space-between;
            padding: 10px 16px;
            cursor: pointer;
            transition: background 0.3s ease;
            gap: 12px;
        }
        .collapse-nav-header:hover { background: var(--surface-hover); }
        .collapse-nav-body {
            max-height: 0;
            overflow: hidden;
            transition: max-height 0.4s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.3s ease;
            opacity: 0;
        }
        .collapse-nav.open .collapse-nav-body {
            max-height: 400px;
            opacity: 1;
        }
        .collapse-nav.open .collapse-arrow { transform: rotate(180deg); }
        .collapse-arrow {
            transition: transform 0.4s cubic-bezier(0.4, 0, 0.2, 1);
            color: var(--text-secondary);
            flex-shrink: 0;
        }
        .collapse-item {
            display: flex; align-items: center; gap: 10px;
            padding: 10px 16px;
            font-size: 14px;
            color: var(--text-secondary);
            text-decoration: none;
            transition: all 0.2s ease;
            border-top: 1px solid var(--border);
            cursor: pointer;
        }
        .collapse-item:hover {
            background: var(--surface-hover);
            color: var(--text);
        }
        .collapse-item.active {
            background: var(--brand-dim);
            color: var(--brand);
        }

        .nav-pill {
            background: var(--input-bg);
            border: 1px solid var(--border);
            border-radius: 100px;
            padding: 4px;
            gap: 0;
        }
        .nav-glow {
            position: absolute;
            top: 4px; bottom: 4px;
            border-radius: 100px;
            background: var(--brand);
            opacity: 0;
            transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
            pointer-events: none;
            z-index: 0;
            box-shadow: 0 2px 12px var(--brand-dim);
        }
        .nav-link {
            position: relative;
            z-index: 1;
            display: inline-flex; align-items: center;
            padding: 8px 20px;
            border-radius: 100px;
            font-size: 14px; font-weight: 500;
            color: var(--text-secondary);
            text-decoration: none;
            transition: color 0.3s ease;
            cursor: pointer;
        }
        .nav-link:hover {
            color: var(--text);
        }
        .nav-link.active {
            color: var(--bg);
        }

        .lang-switcher {
            position: relative;
            display: inline-flex;
        }
        .lang-btn {
            display: flex; align-items: center; gap: 4px;
            padding: 6px 10px;
            border-radius: 10px;
            font-size: 13px; font-weight: 500;
            color: var(--text-secondary);
            background: transparent;
            border: 1px solid var(--border);
            cursor: pointer;
            transition: all 0.2s ease;
        }
        .lang-btn:hover {
            background: var(--surface-hover);
            color: var(--text);
            border-color: var(--border-hover);
        }
        .lang-dropdown {
            position: absolute;
            top: calc(100% + 6px);
            right: 0;
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 4px;
            min-width: 130px;
            box-shadow: var(--shadow-md);
            opacity: 0;
            transform: translateY(-6px);
            pointer-events: none;
            transition: opacity 0.2s ease, transform 0.2s ease;
            z-index: 200;
        }
        .lang-dropdown.open {
            opacity: 1;
            transform: translateY(0);
            pointer-events: auto;
        }
        .lang-option {
            display: flex; align-items: center; gap: 8px;
            padding: 8px 12px;
            border-radius: 8px;
            font-size: 13px;
            color: var(--text-secondary);
            text-decoration: none;
            transition: all 0.15s ease;
        }
        .lang-option:hover {
            background: var(--surface-hover);
            color: var(--text);
        }
        .lang-option.active {
            color: var(--brand);
            background: var(--brand-dim);
        }

        ::-webkit-scrollbar { width: 6px; }
        ::-webkit-scrollbar-track { background: var(--bg); }
        ::-webkit-scrollbar-thumb { background: var(--text-tertiary); border-radius: 3px; }
        ::-webkit-scrollbar-thumb:hover { background: var(--text-secondary); }

        @keyframes fadeUp {
            from { opacity: 0; transform: translateY(30px); }
            to { opacity: 1; transform: translateY(0); }
        }
        @keyframes fadeIn {
            from { opacity: 0; }
            to { opacity: 1; }
        }
        @keyframes float {
            0%, 100% { transform: translateY(0); }
            50% { transform: translateY(-10px); }
        }
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.5; }
        }
        @keyframes spin {
            to { transform: rotate(360deg); }
        }
        .animate-fade-up { animation: fadeUp 0.8s ease forwards; opacity: 0; }
        .animate-fade-in { animation: fadeIn 1s ease forwards; opacity: 0; }
        .delay-1 { animation-delay: 0.1s; }
        .delay-2 { animation-delay: 0.2s; }
        .delay-3 { animation-delay: 0.3s; }
        .delay-4 { animation-delay: 0.4s; }
        .delay-5 { animation-delay: 0.5s; }

        #heroCanvas {
            position: absolute;
            inset: 0;
            width: 100%;
            height: 100%;
            pointer-events: none;
            z-index: 0;
        }

        .hero-glow {
            position: absolute;
            border-radius: 50%;
            pointer-events: none;
            filter: blur(80px);
            opacity: 0;
            animation: glowPulse 8s ease-in-out infinite;
        }
        .hero-glow-1 {
            width: 400px; height: 400px;
            top: 10%; left: 15%;
            background: rgba(212,168,67,0.08);
            animation-delay: 0s;
        }
        .hero-glow-2 {
            width: 300px; height: 300px;
            bottom: 15%; right: 10%;
            background: rgba(0,212,255,0.05);
            animation-delay: 3s;
        }
        .hero-glow-3 {
            width: 250px; height: 250px;
            top: 50%; left: 50%;
            transform: translate(-50%, -50%);
            background: rgba(212,168,67,0.06);
            animation-delay: 5s;
        }
        @keyframes glowPulse {
            0%, 100% { opacity: 0.3; transform: scale(1); }
            50% { opacity: 0.7; transform: scale(1.15); }
        }
        .hero-glow-3 {
            animation-name: glowPulseCenter;
        }
        @keyframes glowPulseCenter {
            0%, 100% { opacity: 0.3; transform: translate(-50%, -50%) scale(1); }
            50% { opacity: 0.6; transform: translate(-50%, -50%) scale(1.2); }
        }

        .feature-card {
            position: relative;
            overflow: hidden;
        }
        .feature-card::after {
            content: '';
            position: absolute;
            inset: 0;
            border-radius: 20px;
            padding: 1px;
            background: linear-gradient(135deg, transparent 40%, var(--brand-dim) 50%, transparent 60%);
            -webkit-mask: linear-gradient(#fff 0 0) content-box, linear-gradient(#fff 0 0);
            -webkit-mask-composite: xor;
            mask-composite: exclude;
            opacity: 0;
            transition: opacity 0.5s ease;
            pointer-events: none;
        }
        .feature-card:hover::after {
            opacity: 1;
        }
        .feature-card .card-glow {
            position: absolute;
            width: 200px; height: 200px;
            border-radius: 50%;
            background: var(--brand);
            filter: blur(80px);
            opacity: 0;
            transition: opacity 0.5s ease;
            pointer-events: none;
            z-index: 0;
        }
        .feature-card:hover .card-glow {
            opacity: 0.06;
        }
        .feature-card .card-icon {
            transition: transform 0.4s cubic-bezier(0.4, 0, 0.2, 1), box-shadow 0.4s ease;
        }
        .feature-card:hover .card-icon {
            transform: scale(1.1) translateY(-2px);
            box-shadow: 0 8px 24px rgba(0,0,0,0.2);
        }

        .reveal-card {
            opacity: 0;
            transform: translateY(40px);
            transition: opacity 0.7s cubic-bezier(0.4, 0, 0.2, 1), transform 0.7s cubic-bezier(0.4, 0, 0.2, 1);
        }
        .reveal-card.revealed {
            opacity: 1;
            transform: translateY(0);
        }

        .btn-primary-pulse {
            position: relative;
        }
        .btn-primary-pulse::before {
            content: '';
            position: absolute;
            inset: -4px;
            border-radius: 100px;
            background: var(--brand);
            opacity: 0;
            animation: btnPulse 3s ease-in-out infinite;
            z-index: -1;
        }
        @keyframes btnPulse {
            0%, 100% { opacity: 0; transform: scale(1); }
            50% { opacity: 0.15; transform: scale(1.05); }
        }

        .sponsor-card {
            transition: transform 0.4s, box-shadow 0.4s, border-color 0.4s;
        }
        .sponsor-card:hover {
            transform: translateY(-4px);
            box-shadow: 0 12px 48px rgba(0,212,255,0.08);
            border-color: rgba(0,212,255,0.2);
        }
        .sponsor-card:hover .sponsor-glow {
            opacity: 0.4 !important;
        }
        .sponsor-card::after {
            content: '';
            position: absolute;
            top: -50%;
            left: -50%;
            width: 200%;
            height: 200%;
            background: conic-gradient(from 0deg, transparent, rgba(0,212,255,0.08), transparent, rgba(212,168,67,0.06), transparent);
            animation: sponsorBorderSpin 8s linear infinite;
            opacity: 0;
            transition: opacity 0.5s;
            pointer-events: none;
        }
        .sponsor-card:hover::after {
            opacity: 1;
        }
        @keyframes sponsorBorderSpin {
            to { transform: rotate(360deg); }
        }

        .noise-overlay {
            position: fixed;
            inset: 0;
            pointer-events: none;
            z-index: 9999;
            opacity: 0.015;
            background-image: url("data:image/svg+xml,%3Csvg viewBox='0 0 256 256' xmlns='http://www.w3.org/2000/svg'%3E%3Cfilter id='n'%3E%3CfeTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='4' stitchTiles='stitch'/%3E%3C/filter%3E%3Crect width='100%25' height='100%25' filter='url(%23n)'/%3E%3C/svg%3E");
            background-repeat: repeat;
            background-size: 256px 256px;
        }
        body.light-theme .noise-overlay { opacity: 0.008; }

        @keyframes pageIn {
            from { opacity: 0; transform: translateY(8px); }
            to { opacity: 1; transform: translateY(0); }
        }
        main {
            animation: pageIn 0.4s cubic-bezier(0.4,0,0.2,1) forwards;
        }

        ::-webkit-scrollbar { width: 6px; }
        ::-webkit-scrollbar-track { background: transparent; }
        ::-webkit-scrollbar-thumb { background: rgba(212,168,67,0.2); border-radius: 3px; }
        ::-webkit-scrollbar-thumb:hover { background: rgba(212,168,67,0.35); }
        body.light-theme ::-webkit-scrollbar-thumb { background: rgba(212,168,67,0.15); }
        body.light-theme ::-webkit-scrollbar-thumb:hover { background: rgba(212,168,67,0.3); }

        .step-connector {
            position: absolute;
            top: 40px;
            left: calc(33.33% + 16px);
            width: calc(33.33% - 32px);
            height: 2px;
            background: linear-gradient(90deg, var(--brand-dim), var(--border), var(--brand-dim));
            opacity: 0.5;
        }
        .step-connector::after {
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            width: 0;
            height: 100%;
            background: var(--brand);
            animation: stepLine 2s ease-in-out 1s forwards;
        }
        @keyframes stepLine {
            to { width: 100%; }
        }

        .step-number:hover {
            transform: scale(1.12) translateY(-2px);
            box-shadow: 0 8px 24px rgba(212,168,67,0.15);
        }

        @media (max-width: 768px) {
            .section-title { font-size: 32px; }
            .hero-glow-1 { width: 250px; height: 250px; }
            .hero-glow-2 { width: 180px; height: 180px; }
        }
    </style>
</head>
<body class="min-h-screen flex flex-col">
    <div class="noise-overlay"></div>
    <header role="banner" id="siteHeader" style="position: fixed; top: 0; left: 0; right: 0; z-index: 50; background: var(--header-bg); border-bottom: 1px solid transparent; backdrop-filter: blur(12px); -webkit-backdrop-filter: blur(12px); transition: border-color 0.3s, box-shadow 0.3s;">
        <div class="max-w-6xl mx-auto px-6">
            <div class="flex items-center justify-between h-[68px]" id="headerInner" style="transition: height 0.3s;">
                <a href="index.php" class="flex items-center gap-3" style="text-decoration: none;" aria-label="<?php echo t('site_name'); ?> - <?php echo t('nav_home'); ?>">
                    <img src="assets/icon.png" alt="<?php echo t('site_name'); ?>" class="w-8 h-8 rounded-lg">
                    <span class="font-semibold text-[15px]" style="color: var(--text);"><?php echo t('site_name'); ?></span>
                </a>
                <nav class="hidden md:flex items-center gap-1 relative nav-pill" aria-label="<?php echo t('footer_nav'); ?>">
                    <div class="nav-glow" id="navGlow"></div>
                    <a href="index.php" class="nav-link <?php echo $currentPage=='index.php'?'active':''; ?>" data-index="0" <?php echo $currentPage=='index.php'?'aria-current="page"':''; ?>><?php echo t('nav_home'); ?></a>
                    <a href="announcements.php" class="nav-link <?php echo $currentPage=='announcements.php'?'active':''; ?>" data-index="1" <?php echo $currentPage=='announcements.php'?'aria-current="page"':''; ?>><?php echo t('nav_announcements'); ?></a>
                    <a href="changelog.php" class="nav-link <?php echo $currentPage=='changelog.php'?'active':''; ?>" data-index="2" <?php echo $currentPage=='changelog.php'?'aria-current="page"':''; ?>><?php echo t('nav_changelog'); ?></a>
                    <a href="contact.php" class="nav-link <?php echo $currentPage=='contact.php'?'active':''; ?>" data-index="3" <?php echo $currentPage=='contact.php'?'aria-current="page"':''; ?>><?php echo t('nav_contact'); ?></a>
                </nav>

                <div class="flex items-center gap-2">
                    <a href="https://space.bilibili.com/3546621436496190" target="_blank" rel="noopener noreferrer" class="hidden sm:flex w-10 h-10 rounded-xl items-center justify-center transition-colors hover:bg-white/5" style="color: var(--text-secondary);" title="B站主页" aria-label="Bilibili - SVL 官方账号">
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor"><path d="M17.813 4.653h.854c1.51.054 2.769.578 3.773 1.574 1.004.995 1.524 2.249 1.56 3.76v7.36c-.036 1.51-.556 2.769-1.56 3.773s-2.262 1.524-3.773 1.56H5.333c-1.51-.036-2.769-.556-3.773-1.56S.036 18.858 0 17.347v-7.36c.036-1.511.556-2.765 1.56-3.76 1.004-.996 2.262-1.52 3.773-1.574h.774l-1.174-1.12a1.234 1.234 0 0 1-.373-.906c0-.356.124-.658.373-.907l.027-.027c.267-.249.573-.373.92-.373.347 0 .653.124.92.373L9.653 4.44c.071.071.134.142.187.213h4.267a.836.836 0 0 1 .16-.213l2.853-2.747c.267-.249.573-.373.92-.373.347 0 .662.151.929.4.267.249.391.551.391.907 0 .355-.124.657-.373.906zM5.333 7.24c-.746.018-1.373.276-1.88.773-.506.498-.769 1.13-.786 1.894v7.52c.017.764.28 1.395.786 1.893.507.498 1.134.756 1.88.773h13.334c.746-.017 1.373-.275 1.88-.773.506-.498.769-1.129.786-1.893v-7.52c-.017-.765-.28-1.396-.786-1.894-.507-.497-1.134-.755-1.88-.773zM8 11.107c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c0-.373.129-.689.386-.947.258-.257.574-.386.947-.386zm8 0c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c.017-.391.15-.711.4-.96.249-.249.56-.373.933-.373z"/></svg>
                    </a>
                    <a href="https://www.douyin.com/user/self?from_tab_name=main" target="_blank" rel="noopener noreferrer" class="hidden sm:flex w-10 h-10 rounded-xl items-center justify-center transition-colors hover:bg-white/5" style="color: var(--text-secondary);" title="抖音主页" aria-label="抖音 - 星露谷物语MOD管理器官方账号">
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor"><path d="M12.525.02c1.31-.02 2.61-.01 3.91-.02.08 1.53.63 3.09 1.75 4.17 1.12 1.11 2.7 1.62 4.24 1.79v4.03c-1.44-.05-2.89-.35-4.2-.97-.57-.26-1.1-.59-1.62-.93-.01 2.92.01 5.84-.02 8.75-.08 1.4-.54 2.79-1.35 3.94-1.31 1.92-3.58 3.17-5.91 3.21-1.43.08-2.86-.31-4.08-1.03-2.02-1.19-3.44-3.37-3.65-5.71-.02-.5-.03-1-.01-1.49.18-1.9 1.12-3.72 2.58-4.96 1.66-1.44 3.98-2.13 6.15-1.72.02 1.48-.04 2.96-.04 4.44-.99-.32-2.15-.23-3.02.37-.63.41-1.11 1.04-1.36 1.75-.21.51-.15 1.07-.14 1.61.24 1.64 1.82 3.02 3.5 2.87 1.12-.01 2.19-.66 2.77-1.61.19-.33.4-.67.41-1.06.1-1.79.06-3.57.07-5.36.01-4.03-.01-8.05.02-12.07z"/></svg>
                    </a>
                    <div class="lang-switcher">
                        <button class="lang-btn" id="langToggleBtn" onclick="document.getElementById('langDropdown').classList.toggle('open')">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"/></svg>
                            <span><?php echo $supportedLangNames[$currentLang]; ?></span>
                        </button>
                        <div class="lang-dropdown" id="langDropdown">
                            <?php foreach ($supportedLangs as $lg): ?>
                            <a href="<?php echo langUrl($lg); ?>" class="lang-option <?php echo $currentLang === $lg ? 'active' : ''; ?>">
                                <?php echo $supportedLangNames[$lg]; ?>
                            </a>
                            <?php endforeach; ?>
                        </div>
                    </div>
                    <button id="themeToggleBtn" class="w-10 h-10 rounded-xl flex items-center justify-center transition-colors hover:bg-white/5" style="color: var(--text-secondary);" title="<?php echo t('theme_toggle'); ?>">
                        <svg id="themeIconMoon" class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"/></svg>
                        <svg id="themeIconSun" class="w-5 h-5 hidden" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"/></svg>
                    </button>
                    <a href="index.php#download" class="btn-primary text-[13px] py-2.5 px-5"><?php echo t('nav_download'); ?></a>
                    <button id="mobileMenuBtn" class="md:hidden w-10 h-10 rounded-xl flex items-center justify-center" style="color: var(--text-secondary);" aria-label="菜单" aria-expanded="false">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/></svg>
                    </button>
                </div>
            </div>
            <div id="mobileMenu" class="hidden md:hidden pb-4" style="border-top: 1px solid var(--border);">
                <div class="flex flex-col gap-1 pt-3">
                    <a href="index.php" class="block px-4 py-3 rounded-xl text-sm font-medium <?php echo $currentPage=='index.php'?'text-white bg-white/10':'text-gray-400 hover:bg-white/5'; ?>"><?php echo t('nav_home'); ?></a>
                    <a href="announcements.php" class="block px-4 py-3 rounded-xl text-sm font-medium <?php echo $currentPage=='announcements.php'?'text-white bg-white/10':'text-gray-400 hover:bg-white/5'; ?>"><?php echo t('nav_announcements'); ?></a>
                    <a href="changelog.php" class="block px-4 py-3 rounded-xl text-sm font-medium <?php echo $currentPage=='changelog.php'?'text-white bg-white/10':'text-gray-400 hover:bg-white/5'; ?>"><?php echo t('nav_changelog'); ?></a>
                    <a href="contact.php" class="block px-4 py-3 rounded-xl text-sm font-medium <?php echo $currentPage=='contact.php'?'text-white bg-white/10':'text-gray-400 hover:bg-white/5'; ?>"><?php echo t('nav_contact'); ?></a>
                </div>
            </div>
        </div>
    </header>

    <div class="h-[68px]"></div>

    <script>
        (function() {
            var navPill = document.querySelector('.nav-pill');
            var glow = document.getElementById('navGlow');
            if (!navPill || !glow) return;
            var links = navPill.querySelectorAll('.nav-link');
            var activeLink = navPill.querySelector('.nav-link.active');

            function moveGlowTo(el) {
                if (!el) { glow.style.opacity = '0'; return; }
                glow.style.width = el.offsetWidth + 'px';
                glow.style.left = el.offsetLeft + 'px';
                glow.style.opacity = '1';
            }

            if (activeLink) moveGlowTo(activeLink);

            links.forEach(function(link) {
                link.addEventListener('mouseenter', function() {
                    moveGlowTo(link);
                });
                link.addEventListener('mouseleave', function() {
                    moveGlowTo(activeLink);
                });
            });

            window.addEventListener('resize', function() {
                moveGlowTo(activeLink);
            });
        })();

        document.getElementById('mobileMenuBtn').addEventListener('click', function() {
            document.getElementById('mobileMenu').classList.toggle('hidden');
        });
        document.addEventListener('click', function(e) {
            var btn = document.getElementById('langToggleBtn');
            var dd = document.getElementById('langDropdown');
            if (btn && dd && !btn.contains(e.target) && !dd.contains(e.target)) {
                dd.classList.remove('open');
            }
        });

        (function() {
            var btn = document.getElementById('themeToggleBtn');
            var iconMoon = document.getElementById('themeIconMoon');
            var iconSun = document.getElementById('themeIconSun');
            var saved = localStorage.getItem('theme');

            function applyTheme(isLight) {
                document.body.classList.add('theme-transitioning');
                document.body.offsetHeight;
                document.body.classList.toggle('light-theme', isLight);
                if (iconMoon) iconMoon.classList.toggle('hidden', isLight);
                if (iconSun) iconSun.classList.toggle('hidden', !isLight);
                requestAnimationFrame(function() {
                    document.body.classList.remove('theme-transitioning');
                });
            }

            if (saved === 'light') {
                document.body.classList.add('light-theme');
                if (iconMoon) iconMoon.classList.add('hidden');
                if (iconSun) iconSun.classList.remove('hidden');
            } else if (saved === 'dark') {
                document.body.classList.remove('light-theme');
                if (iconMoon) iconMoon.classList.remove('hidden');
                if (iconSun) iconSun.classList.add('hidden');
            } else {
                document.body.classList.add('light-theme');
                if (iconMoon) iconMoon.classList.add('hidden');
                if (iconSun) iconSun.classList.remove('hidden');
            }

            if (btn) {
                btn.addEventListener('click', function() {
                    var isLight = !document.body.classList.contains('light-theme');
                    applyTheme(isLight);
                    localStorage.setItem('theme', isLight ? 'light' : 'dark');
                });
            }
        })();
    </script>
