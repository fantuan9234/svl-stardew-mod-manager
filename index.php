<?php
$pageTitle = '首页';

$announcements = [
    [
        'title' => 'v2.0 版本正式发布',
        'date' => '2026-05-15',
        'summary' => '全新界面设计，支持 SMAPI 4.0，MOD 自动更新功能上线。'
    ],
    [
        'title' => '修复了部分 MOD 加载问题',
        'date' => '2026-05-10',
        'summary' => '解决了 Content Patcher 1.30+ 版本的兼容性问题。'
    ]
];

$features = [
    [
        'title' => '一键安装 MOD',
        'desc' => '拖拽或点击即可安装，自动处理依赖关系，无需手动配置。'
    ],
    [
        'title' => '自动检测冲突',
        'desc' => '智能分析 MOD 兼容性，提前预警潜在冲突，保护游戏存档。'
    ],
    [
        'title' => '备份与恢复',
        'desc' => '一键备份所有配置，换设备或重装后快速恢复完美农场。'
    ],
    [
        'title' => '自动更新',
        'desc' => '后台检查 MOD 更新，发现新版本即时提醒，一键批量升级。'
    ]
];

$socialLinks = [
    [
        'name' => '抖音',
        'handle' => '@星露谷管理器',
        'url' => 'https://douyin.com',
        'color' => '#1a1a1a',
        'bgColor' => '#f5f5f5'
    ],
    [
        'name' => 'Bilibili',
        'handle' => '@星露谷管理器',
        'url' => 'https://bilibili.com',
        'color' => '#fb7299',
        'bgColor' => '#fff0f3'
    ],
    [
        'name' => '快手',
        'handle' => '@星露谷管理器',
        'url' => 'https://kuaishou.com',
        'color' => '#ff5000',
        'bgColor' => '#fff5f0'
    ],
    [
        'name' => '小红书',
        'handle' => '@星露谷管理器',
        'url' => 'https://xiaohongshu.com',
        'color' => '#ff2442',
        'bgColor' => '#fff0f2'
    ]
];

include 'header.php';
?>

<main class="flex-1">
    <!-- Hero -->
    <section class="py-16 md:py-24">
        <div class="max-w-6xl mx-auto px-4 sm:px-6 text-center">
            <img src="assets/icon.png" alt="星露谷管理器" class="w-24 h-24 md:w-32 md:h-32 mx-auto mb-8 rounded-3xl shadow-lg">
            <h1 class="text-4xl md:text-5xl font-bold mb-4" style="color: var(--text);">星露谷管理器</h1>
            <p class="text-lg md:text-xl mb-4" style="color: var(--text-secondary); max-width: 600px; margin: 0 auto 16px;">
                专为星露谷物语打造的 MOD 管理工具
            </p>
            <p class="text-base mb-10" style="color: var(--text-secondary); max-width: 500px; margin: 0 auto 40px;">
                一键安装、自动检测冲突、智能备份恢复，让你的农场生活更加轻松
            </p>
            <div class="flex flex-col sm:flex-row items-center justify-center gap-4">
                <a href="#download" class="btn-primary text-base px-8 py-3">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
                    </svg>
                    免费下载
                </a>
                <a href="https://github.com" target="_blank" class="btn-secondary text-base px-8 py-3">
                    <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                    </svg>
                    GitHub
                </a>
            </div>
            <div class="mt-6 flex items-center justify-center gap-6 text-sm" style="color: var(--text-secondary);">
                <span class="flex items-center gap-2">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                    </svg>
                    完全免费
                </span>
                <span class="flex items-center gap-2">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                    </svg>
                    开源
                </span>
                <span class="flex items-center gap-2">
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/>
                    </svg>
                    50K+ 下载
                </span>
            </div>
        </div>
    </section>

    <!-- Features -->
    <section class="py-16" style="background-color: white;">
        <div class="max-w-6xl mx-auto px-4 sm:px-6">
            <div class="text-center mb-12">
                <h2 class="section-title">核心功能</h2>
                <p class="section-subtitle">简洁高效，让 MOD 管理零门槛</p>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
                <?php foreach ($features as $feature): ?>
                <div class="card">
                    <h3 class="font-semibold text-base mb-2" style="color: var(--text);"><?php echo $feature['title']; ?></h3>
                    <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo $feature['desc']; ?></p>
                </div>
                <?php endforeach; ?>
            </div>
        </div>
    </section>

    <!-- Social Media -->
    <section class="py-16">
        <div class="max-w-6xl mx-auto px-4 sm:px-6">
            <div class="text-center mb-12">
                <h2 class="section-title">关注我们</h2>
                <p class="section-subtitle">在短视频平台获取更多教程和资讯</p>
            </div>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                <?php foreach ($socialLinks as $social): ?>
                <a href="<?php echo $social['url']; ?>" target="_blank" rel="noopener noreferrer" class="card text-center" style="text-decoration: none; color: inherit;">
                    <div class="w-12 h-12 rounded-xl mx-auto mb-3 flex items-center justify-center text-lg font-bold" style="background-color: <?php echo $social['bgColor']; ?>; color: <?php echo $social['color']; ?>">
                        <?php echo mb_substr($social['name'], 0, 1); ?>
                    </div>
                    <h3 class="font-semibold text-sm mb-1" style="color: var(--text);"><?php echo $social['name']; ?></h3>
                    <p class="text-xs" style="color: var(--text-secondary);"><?php echo $social['handle']; ?></p>
                </a>
                <?php endforeach; ?>
            </div>
        </div>
    </section>

    <!-- Latest Announcements -->
    <section class="py-16" style="background-color: white;">
        <div class="max-w-6xl mx-auto px-4 sm:px-6">
            <div class="flex items-center justify-between mb-8">
                <div>
                    <h2 class="section-title">最新公告</h2>
                    <p class="section-subtitle">及时了解软件更新动态</p>
                </div>
                <a href="announcements.php" class="text-sm font-medium flex items-center gap-1" style="color: var(--brand); text-decoration: none;">
                    查看全部
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                    </svg>
                </a>
            </div>
            <div class="space-y-4">
                <?php foreach ($announcements as $item): ?>
                <div class="card flex items-start gap-4">
                    <div class="w-2 h-2 rounded-full mt-2 flex-shrink-0" style="background-color: var(--brand);"></div>
                    <div class="flex-1">
                        <div class="flex items-center gap-3 mb-1">
                            <h3 class="font-semibold text-sm" style="color: var(--text);"><?php echo $item['title']; ?></h3>
                            <span class="text-xs" style="color: var(--text-secondary);"><?php echo $item['date']; ?></span>
                        </div>
                        <p class="text-sm" style="color: var(--text-secondary);"><?php echo $item['summary']; ?></p>
                    </div>
                </div>
                <?php endforeach; ?>
            </div>
        </div>
    </section>

    <!-- Download CTA -->
    <section id="download" class="py-20">
        <div class="max-w-6xl mx-auto px-4 sm:px-6 text-center">
            <div class="card max-w-2xl mx-auto" style="background-color: var(--text); color: white;">
                <h2 class="text-2xl font-bold mb-3">开始你的 MOD 之旅</h2>
                <p class="text-sm mb-6 opacity-80">免费下载，支持 Windows / macOS / Linux</p>
                <a href="#" class="btn-primary inline-flex" style="background-color: var(--brand);">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
                    </svg>
                    立即下载
                </a>
                <p class="text-xs mt-4 opacity-50">v2.0.0 | 更新于 2026-05-15</p>
            </div>
        </div>
    </section>
</main>

<?php include 'footer.php'; ?>
