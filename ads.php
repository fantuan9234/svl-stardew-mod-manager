<?php
$pageTitle = '推广';

$ads = [
    [
        'title' => 'SMAPI 官方文档',
        'desc' => '星露谷物语 MOD 加载器的官方文档，开发者必备参考。包含完整的 API 参考和教程。',
        'link' => 'https://smapi.io/docs',
        'tag' => '官方',
        'tagClass' => 'tag-green'
    ],
    [
        'title' => 'Nexus Mods',
        'desc' => '最大的星露谷物语 MOD 社区，拥有数万款玩家创作的 MOD。发现、下载、分享你的创意。',
        'link' => 'https://www.nexusmods.com/stardewvalley',
        'tag' => '社区',
        'tagClass' => 'tag-blue'
    ],
    [
        'title' => 'Stardew Valley Wiki',
        'desc' => '最全面的星露谷物语百科，包含游戏机制、物品、NPC 等详细信息。新玩家的最佳入门指南。',
        'link' => 'https://stardewvalleywiki.com',
        'tag' => '百科',
        'tagClass' => 'tag'
    ],
    [
        'title' => 'Stardew Valley Discord',
        'desc' => '官方 Discord 社区，与其他农场主交流心得，获取最新游戏资讯。',
        'link' => 'https://discord.gg/stardewvalley',
        'tag' => '社区',
        'tagClass' => 'tag-blue'
    ]
];

include 'header.php';
?>

<main class="flex-1 py-12">
    <div class="max-w-4xl mx-auto px-4 sm:px-6">
        <div class="mb-10">
            <h1 class="section-title">推广</h1>
            <p class="section-subtitle">星露谷生态相关的优质资源和社区</p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
            <?php foreach ($ads as $ad): ?>
            <a href="<?php echo $ad['link']; ?>" target="_blank" rel="noopener noreferrer" class="card block" style="text-decoration: none; color: inherit;">
                <div class="flex items-center gap-3 mb-3">
                    <span class="tag <?php echo $ad['tagClass']; ?>"><?php echo $ad['tag']; ?></span>
                </div>
                <h3 class="font-semibold text-base mb-2" style="color: var(--text);"><?php echo $ad['title']; ?></h3>
                <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo $ad['desc']; ?></p>
                <div class="mt-4 flex items-center gap-1 text-sm font-medium" style="color: var(--brand);">
                    <span>访问网站</span>
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/>
                    </svg>
                </div>
            </a>
            <?php endforeach; ?>
        </div>

        <div class="mt-10 card text-center" style="background-color: rgba(212, 168, 67, 0.05); border-style: dashed;">
            <svg class="w-10 h-10 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" style="color: var(--brand);">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M11 5.882V19.24a1.76 1.76 0 01-3.417.592l-2.147-6.15M18 13a3 3 0 100-6M5.436 13.683A4.001 4.001 0 017 6h1.832c4.1 0 7.625-1.234 9.168-3v14c-1.543-1.766-5.067-3-9.168-3H7a3.988 3.988 0 01-1.564-.317z"/>
            </svg>
            <h3 class="font-semibold text-base mb-2" style="color: var(--text);">想要展示你的项目？</h3>
            <p class="text-sm mb-4" style="color: var(--text-secondary);">如果你有与星露谷物语相关的优质项目，欢迎联系我们。</p>
            <a href="contact.php" class="btn-primary text-sm">联系我们</a>
        </div>
    </div>
</main>

<?php include 'footer.php'; ?>
