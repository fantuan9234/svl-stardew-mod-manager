<?php
$pageTitle = '公告';

require_once __DIR__ . '/backend/security.php';
sendSecurityHeaders();

require_once __DIR__ . '/backend/db.php';
initDatabase();

require_once __DIR__ . '/backend/language.php';

function renderAnnouncementContent(string $content): string
{
    $content = h($content);
    $content = preg_replace(
        '#\[img\](.+?)\[/img\]#i',
        '<img src="$1" alt="公告图片" class="announce-img" loading="lazy">',
        $content
    );
    $content = preg_replace(
        '#\[url=(.+?)\](.+?)\[/url\]#i',
        '<a href="$1" target="_blank" rel="noopener noreferrer" class="announce-link">$2</a>',
        $content
    );
    $content = preg_replace(
        '#\[url\](.+?)\[/url\]#i',
        '<a href="$1" target="_blank" rel="noopener noreferrer" class="announce-link">$1</a>',
        $content
    );
    $content = preg_replace(
        '#(?<!href=["\'])(?<!src=["\'])https?://[^\s<\)]+#i',
        '<a href="$0" target="_blank" rel="noopener noreferrer" class="announce-link">$0</a>',
        $content
    );
    $content = nl2br($content);
    return $content;
}

$db = getDB();
$items = $db->query("SELECT * FROM announcements ORDER BY is_pinned DESC, created_at DESC")->fetchAll();

$cats = [
    '更新' => 'tag',
    '修复' => 'tag-green',
    '社区' => 'tag-blue',
    '说明' => 'tag-red',
    '活动' => 'tag',
    '其他' => 'tag-blue'
];

$catIcons = [
    '更新' => '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/></svg>',
    '修复' => '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/></svg>',
    '社区' => '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z"/></svg>',
    '说明' => '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>',
    '活动' => '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"/></svg>',
    '其他' => '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h.01M12 12h.01M19 12h.01M6 12a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0z"/></svg>',
];

include 'header.php';
?>

<main class="flex-1 py-24">
    <div class="max-w-3xl mx-auto px-6">
        <div class="mb-14">
            <span class="section-label">News</span>
            <h1 class="section-title"><?php echo t('announcements_title'); ?></h1>
            <p class="section-subtitle mt-4"><?php echo t('announcements_subtitle'); ?></p>
        </div>

        <div class="space-y-5">
            <?php foreach ($items as $i => $item): ?>
            <div class="announce-card card group" style="opacity:0;transform:translateY(20px);transition:opacity 0.5s cubic-bezier(0.4,0,0.2,1),transform 0.5s cubic-bezier(0.4,0,0.2,1);">
                <div class="flex items-start gap-5">
                    <div class="w-11 h-11 rounded-xl flex items-center justify-center flex-shrink-0 announce-icon" style="background: var(--brand-dim); color: var(--brand);">
                        <?php echo $catIcons[$item['category']] ?? $catIcons['其他']; ?>
                    </div>
                    <div class="flex-1 min-w-0">
                        <div class="flex flex-wrap items-center gap-2.5 mb-2">
                            <?php $catClass = $cats[$item['category']] ?? 'tag'; ?>
                            <span class="tag <?php echo $catClass; ?>"><?php echo t('announcements_cat_' . $item['category']); ?></span>
                            <?php if ($item['is_pinned']): ?>
                            <span class="tag" style="background: rgba(212,168,67,0.12); color: var(--brand);">
                                <svg class="w-3 h-3 inline -mt-0.5 mr-0.5" fill="currentColor" viewBox="0 0 24 24"><path d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"/></svg>
                                <?php echo t('announcements_pinned'); ?>
                            </span>
                            <?php endif; ?>
                            <span class="text-xs ml-auto" style="color: var(--text-tertiary);"><?php echo h($item['created_at']); ?></span>
                        </div>
                        <h3 class="font-semibold text-base mb-2" style="color: var(--text);"><?php echo h($item['title']); ?></h3>
                        <?php if (!empty($item['image_url'])): ?>
                        <img src="<?php echo h($item['image_url']); ?>" alt="<?php echo h($item['title']); ?>" class="announce-img" loading="lazy">
                        <?php endif; ?>
                        <div class="text-sm leading-relaxed announce-content" style="color: var(--text-secondary);"><?php echo renderAnnouncementContent($item['content']); ?></div>
                    </div>
                </div>
            </div>
            <?php endforeach; ?>
            <?php if (empty($items)): ?>
            <div class="text-center py-16" style="color: var(--text-tertiary);">
                <p><?php echo t('announcements_empty'); ?></p>
            </div>
            <?php endif; ?>
        </div>
    </div>
</main>

<style>
.announce-card {
    transition: transform 0.3s, box-shadow 0.3s, border-color 0.3s;
}
.announce-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 32px rgba(0,0,0,0.06);
    border-color: var(--brand-dim);
}
body.light-theme .announce-card:hover {
    box-shadow: 0 8px 32px rgba(0,0,0,0.03);
}
.announce-icon {
    transition: transform 0.3s, background 0.3s;
}
.announce-card:hover .announce-icon {
    transform: scale(1.08);
    background: rgba(212,168,67,0.2);
}
.announce-link {
    color: var(--brand);
    text-decoration: none;
    border-bottom: 1px solid rgba(212,168,67,0.3);
    transition: border-color 0.2s, opacity 0.2s;
}
.announce-link:hover {
    border-bottom-color: var(--brand);
    opacity: 0.85;
}
.announce-img {
    max-width: 100%;
    max-height: 280px;
    width: auto;
    object-fit: contain;
    border-radius: 12px;
    margin: 12px 0;
    border: 1px solid var(--border);
}
.announce-content {
    word-break: break-word;
    overflow-wrap: break-word;
}
</style>

<script>
(function(){
    var cards = document.querySelectorAll('.announce-card');
    if (!cards.length) return;
    var observer = new IntersectionObserver(function(entries) {
        entries.forEach(function(entry) {
            if (entry.isIntersecting) {
                var idx = Array.prototype.indexOf.call(cards, entry.target);
                entry.target.style.transitionDelay = (idx * 0.06) + 's';
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
                observer.unobserve(entry.target);
            }
        });
    }, { threshold: 0.1, rootMargin: '0px 0px -20px 0px' });
    cards.forEach(function(card) { observer.observe(card); });
})();
</script>

<?php include 'footer.php'; ?>
