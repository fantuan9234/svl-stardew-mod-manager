<?php
$pageTitle = '公告';

$allAnnouncements = [
    [
        'title' => 'v2.0 版本正式发布',
        'date' => '2026-05-15',
        'category' => '更新',
        'content' => '全新界面设计，支持 SMAPI 4.0，MOD 自动更新功能上线！现在你可以更轻松地管理你的 MOD 收藏。'
    ],
    [
        'title' => '修复了部分 MOD 加载问题',
        'date' => '2026-05-10',
        'category' => '修复',
        'content' => '解决了 Content Patcher 1.30+ 版本的兼容性问题，提升了 MOD 加载稳定性。'
    ],
    [
        'title' => '欢迎加入 Discord 社区',
        'date' => '2026-05-01',
        'category' => '社区',
        'content' => '与其他农场主交流心得，获取最新更新资讯，参与功能讨论。'
    ],
    [
        'title' => 'v1.5 版本更新',
        'date' => '2026-04-20',
        'category' => '更新',
        'content' => '新增 MOD 批量导入功能，优化了搜索体验，修复了若干已知问题。'
    ],
    [
        'title' => 'macOS 版本现已可用',
        'date' => '2026-04-10',
        'category' => '更新',
        'content' => '经过测试，macOS 版本现已正式发布，欢迎 Mac 用户体验。'
    ],
    [
        'title' => '关于 SMAPI 3.18 兼容性说明',
        'date' => '2026-03-28',
        'category' => '说明',
        'content' => 'SMAPI 3.18 版本更改了部分 API，管理器已适配，请更新到最新版本。'
    ]
];

$categoryColors = [
    '更新' => 'tag',
    '修复' => 'tag-green',
    '社区' => 'tag-blue',
    '说明' => 'tag-red'
];

include 'header.php';
?>

<main class="flex-1 py-12">
    <div class="max-w-4xl mx-auto px-4 sm:px-6">
        <div class="mb-10">
            <h1 class="section-title">公告</h1>
            <p class="section-subtitle">软件更新、功能发布和重要通知</p>
        </div>

        <div class="space-y-4">
            <?php foreach ($allAnnouncements as $item): ?>
            <div class="card">
                <div class="flex flex-col sm:flex-row sm:items-center gap-3 mb-3">
                    <span class="tag <?php echo $categoryColors[$item['category']] ?? 'tag'; ?> w-fit">
                        <?php echo $item['category']; ?>
                    </span>
                    <span class="text-xs" style="color: var(--text-secondary);"><?php echo $item['date']; ?></span>
                </div>
                <h3 class="font-semibold text-base mb-2" style="color: var(--text);"><?php echo $item['title']; ?></h3>
                <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo $item['content']; ?></p>
            </div>
            <?php endforeach; ?>
        </div>
    </div>
</main>

<?php include 'footer.php'; ?>
