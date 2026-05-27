<?php
$pageTitle = '联系我们';

$socials = [
    [
        'name' => 'GitHub',
        'value' => 'github.com/stardew-manager',
        'url' => 'https://github.com',
        'icon' => '<path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>'
    ],
    [
        'name' => '邮箱',
        'value' => 'support@stardewmanager.com',
        'url' => 'mailto:support@stardewmanager.com',
        'icon' => '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/>'
    ],
    [
        'name' => 'Bilibili',
        'value' => '@星露谷管理器',
        'url' => 'https://bilibili.com',
        'icon' => '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"/>'
    ],
    [
        'name' => '抖音',
        'value' => '@星露谷管理器',
        'url' => 'https://douyin.com',
        'icon' => '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3"/>'
    ]
];

include 'header.php';
?>

<main class="flex-1 py-12">
    <div class="max-w-4xl mx-auto px-4 sm:px-6">
        <div class="mb-10">
            <h1 class="section-title">联系我们</h1>
            <p class="section-subtitle">有问题或建议？欢迎随时联系</p>
        </div>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-5 mb-10">
            <?php foreach ($socials as $social): ?>
            <a href="<?php echo $social['url']; ?>" target="_blank" rel="noopener noreferrer" class="card flex items-center gap-4" style="text-decoration: none; color: inherit;">
                <div class="w-12 h-12 rounded-xl flex items-center justify-center flex-shrink-0" style="background-color: rgba(212, 168, 67, 0.1); color: var(--brand);">
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <?php echo $social['icon']; ?>
                    </svg>
                </div>
                <div>
                    <h3 class="font-semibold text-sm" style="color: var(--text);"><?php echo $social['name']; ?></h3>
                    <p class="text-sm" style="color: var(--text-secondary);"><?php echo $social['value']; ?></p>
                </div>
            </a>
            <?php endforeach; ?>
        </div>

        <div class="card">
            <h2 class="font-semibold text-lg mb-6" style="color: var(--text);">发送反馈</h2>
            <form action="#" method="post" class="space-y-5">
                <div class="grid grid-cols-1 sm:grid-cols-2 gap-5">
                    <div>
                        <label class="block text-sm font-medium mb-2" style="color: var(--text);">姓名</label>
                        <input type="text" name="name" placeholder="你的名字" class="w-full px-4 py-3 rounded-xl text-sm" style="border: 1px solid var(--border); background-color: var(--bg); color: var(--text); outline: none;" required>
                    </div>
                    <div>
                        <label class="block text-sm font-medium mb-2" style="color: var(--text);">邮箱</label>
                        <input type="email" name="email" placeholder="your@email.com" class="w-full px-4 py-3 rounded-xl text-sm" style="border: 1px solid var(--border); background-color: var(--bg); color: var(--text); outline: none;" required>
                    </div>
                </div>
                <div>
                    <label class="block text-sm font-medium mb-2" style="color: var(--text);">主题</label>
                    <select name="subject" class="w-full px-4 py-3 rounded-xl text-sm" style="border: 1px solid var(--border); background-color: var(--bg); color: var(--text); outline: none;">
                        <option value="feedback">功能建议</option>
                        <option value="bug">Bug 反馈</option>
                        <option value="coop">商务合作</option>
                        <option value="other">其他</option>
                    </select>
                </div>
                <div>
                    <label class="block text-sm font-medium mb-2" style="color: var(--text);">内容</label>
                    <textarea name="message" rows="5" placeholder="请详细描述你的问题或建议..." class="w-full px-4 py-3 rounded-xl text-sm resize-none" style="border: 1px solid var(--border); background-color: var(--bg); color: var(--text); outline: none;" required></textarea>
                </div>
                <button type="submit" class="btn-primary w-full sm:w-auto justify-center">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8"/>
                    </svg>
                    发送反馈
                </button>
            </form>
        </div>
    </div>
</main>

<?php include 'footer.php'; ?>
