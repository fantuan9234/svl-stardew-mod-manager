<?php
$pageTitle = '404';

require_once __DIR__ . '/backend/security.php';
sendSecurityHeaders();
require_once __DIR__ . '/backend/language.php';

include 'header.php';
?>

<main class="flex-1 flex items-center justify-center py-24">
    <div class="text-center px-6">
        <div class="mb-8">
            <span class="text-8xl font-bold gradient-text" style="line-height: 1;">404</span>
        </div>
        <h1 class="text-2xl font-semibold mb-4" style="color: var(--text);">页面走丢了</h1>
        <p class="text-base mb-8 max-w-md mx-auto" style="color: var(--text-secondary);">你访问的页面不存在，可能已被移动或删除。不如回首页看看？</p>
        <a href="index.php" class="btn-primary inline-flex items-center gap-2">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/></svg>
            返回首页
        </a>
    </div>
</main>

<?php include 'footer.php'; ?>
