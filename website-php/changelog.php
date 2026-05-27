<?php
$pageTitle = '更新日志';

require_once __DIR__ . '/backend/security.php';
sendSecurityHeaders();

require_once __DIR__ . '/backend/db.php';
initDatabase();

require_once __DIR__ . '/backend/language.php';

$db = getDB();
$versions = $db->query("SELECT * FROM changelog ORDER BY release_date DESC, id DESC")->fetchAll();

$typeLabels = [
    'new' => ['新增', 'tag-green'],
    'fix' => ['修复', 'tag-red'],
    'improve' => ['优化', 'tag-blue'],
    'update' => ['更新', 'tag'],
    'release' => ['发布', 'tag-green'],
    'other' => ['其他', 'tag-blue'],
];

$releaseTypeDot = [
    'release' => 'var(--brand)',
    'update' => '#3b82f6',
    'fix' => '#ef4444',
    'other' => 'var(--text-tertiary)',
];

$releaseTypeIcon = [
    'release' => '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z"/></svg>',
    'update' => '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/></svg>',
    'fix' => '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/></svg>',
    'other' => '<svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>',
];

include 'header.php';
?>

<main class="flex-1 py-24">
    <div class="max-w-3xl mx-auto px-6">
        <div class="mb-14">
            <span class="section-label">Changelog</span>
            <h1 class="section-title"><?php echo t('changelog_title'); ?></h1>
            <p class="section-subtitle mt-4"><?php echo t('changelog_subtitle'); ?></p>
        </div>

        <div class="relative">
            <div class="absolute left-[23px] top-0 bottom-0 w-px timeline-line"></div>

            <div class="space-y-10">
                <?php foreach ($versions as $i => $release): ?>
                <?php $changes = json_decode($release['changes'], true) ?: []; ?>
                <div class="relative flex gap-6 timeline-item" style="opacity:0;transform:translateY(24px);transition:opacity 0.6s cubic-bezier(0.4,0,0.2,1),transform 0.6s cubic-bezier(0.4,0,0.2,1);">
                    <div class="relative z-10 flex-shrink-0">
                        <div class="w-12 h-12 rounded-2xl flex items-center justify-center timeline-dot" style="background: var(--surface); border: 2px solid <?php echo $releaseTypeDot[$release['release_type']] ?? 'var(--text-tertiary)'; ?>; color: <?php echo $releaseTypeDot[$release['release_type']] ?? 'var(--text-tertiary)'; ?>;">
                            <?php echo $releaseTypeIcon[$release['release_type']] ?? $releaseTypeIcon['other']; ?>
                        </div>
                    </div>
                    <div class="card flex-1 py-6 px-7 timeline-card" style="border-left: 3px solid <?php echo $releaseTypeDot[$release['release_type']] ?? 'var(--text-tertiary)'; ?>;">
                        <div class="flex flex-wrap items-center gap-3 mb-3">
                            <span class="text-lg font-bold" style="color: var(--text);"><?php echo h($release['version']); ?></span>
                            <?php $typeInfo = $typeLabels[$release['release_type']] ?? ['其他', 'tag-blue']; ?>
                            <span class="tag <?php echo $typeInfo[1]; ?>"><?php echo $typeInfo[0]; ?></span>
                            <span class="text-xs ml-auto" style="color: var(--text-tertiary);"><?php echo h($release['release_date']); ?></span>
                        </div>
                        <h3 class="font-semibold text-base mb-4" style="color: var(--text);"><?php echo h($release['title']); ?></h3>
                        <?php if (!empty($changes)): ?>
                        <ul class="space-y-3">
                            <?php foreach ($changes as $change): ?>
                            <li class="flex items-start gap-3 text-sm leading-relaxed" style="color: var(--text-secondary);">
                                <?php $changeInfo = $typeLabels[$change['type']] ?? ['其他', 'tag-blue']; ?>
                                <span class="tag <?php echo $changeInfo[1]; ?> mt-0.5 flex-shrink-0" style="font-size: 10px; padding: 1px 6px;"><?php echo $changeInfo[0]; ?></span>
                                <span><?php echo h($change['text']); ?></span>
                            </li>
                            <?php endforeach; ?>
                        </ul>
                        <?php endif; ?>
                    </div>
                </div>
                <?php endforeach; ?>
                <?php if (empty($versions)): ?>
                <div class="text-center py-16" style="color: var(--text-tertiary);">
                    <p>暂无更新日志</p>
                </div>
                <?php endif; ?>
            </div>
        </div>
    </div>
</main>

<style>
.timeline-line {
    background: linear-gradient(to bottom, var(--brand), var(--border) 30%, var(--border) 70%, transparent);
    opacity: 0.4;
}
.timeline-dot {
    transition: transform 0.3s, box-shadow 0.3s;
}
.timeline-item:hover .timeline-dot {
    transform: scale(1.1);
    box-shadow: 0 0 16px rgba(212,168,67,0.2);
}
.timeline-card {
    transition: transform 0.3s, box-shadow 0.3s;
}
.timeline-item:hover .timeline-card {
    transform: translateX(4px);
    box-shadow: 0 4px 24px rgba(0,0,0,0.08);
}
body.light-theme .timeline-item:hover .timeline-card {
    box-shadow: 0 4px 24px rgba(0,0,0,0.04);
}
</style>

<script>
(function(){
    var items = document.querySelectorAll('.timeline-item');
    if (!items.length) return;
    var observer = new IntersectionObserver(function(entries) {
        entries.forEach(function(entry) {
            if (entry.isIntersecting) {
                var idx = Array.prototype.indexOf.call(items, entry.target);
                entry.target.style.transitionDelay = (idx * 0.08) + 's';
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
                observer.unobserve(entry.target);
            }
        });
    }, { threshold: 0.1, rootMargin: '0px 0px -30px 0px' });
    items.forEach(function(item) { observer.observe(item); });
})();
</script>

<?php include 'footer.php'; ?>
