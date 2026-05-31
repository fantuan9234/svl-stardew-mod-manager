<?php
$pageTitle = '常见问题';

require_once __DIR__ . '/backend/security.php';
sendSecurityHeaders();

require_once __DIR__ . '/backend/language.php';

$faqItems = [];
for ($i = 1; $i <= 8; $i++) {
    $q = t('faq_q' . $i);
    $a = t('faq_a' . $i);
    if ($q && $a && $q !== 'faq_q' . $i) {
        $faqItems[] = ['q' => $q, 'a' => $a];
    }
}

$faqLdJson = [
    '@context' => 'https://schema.org',
    '@type' => 'FAQPage',
    'mainEntity' => array_map(function($item) {
        return [
            '@type' => 'Question',
            'name' => $item['q'],
            'acceptedAnswer' => [
                '@type' => 'Answer',
                'text' => $item['a']
            ]
        ];
    }, $faqItems)
];

include 'header.php';
?>
<script type="application/ld+json"><?php echo json_encode($faqLdJson, JSON_UNESCAPED_UNICODE | JSON_PRETTY_PRINT); ?></script>

<main class="flex-1 py-24">
    <div class="max-w-3xl mx-auto px-6">
        <div class="mb-14">
            <span class="section-label"><?php echo t('faq_label'); ?></span>
            <h1 class="section-title"><?php echo t('faq_title'); ?></h1>
            <p class="section-subtitle mt-4"><?php echo t('faq_subtitle'); ?></p>
        </div>

        <div class="space-y-4">
            <?php foreach ($faqItems as $i => $item): ?>
            <div class="faq-item card" style="padding: 0; overflow: hidden;">
                <button class="faq-question" onclick="toggleFaq(this)" aria-expanded="false">
                    <span class="faq-q-icon">
                        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                    </span>
                    <span class="faq-q-text"><?php echo h($item['q']); ?></span>
                    <span class="faq-arrow">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/></svg>
                    </span>
                </button>
                <div class="faq-answer">
                    <div class="faq-answer-inner">
                        <?php echo h($item['a']); ?>
                    </div>
                </div>
            </div>
            <?php endforeach; ?>
        </div>

        <div class="mt-16 text-center">
            <p class="text-sm mb-4" style="color: var(--text-secondary);"><?php echo t('faq_subtitle'); ?></p>
            <a href="contact.php" class="btn-secondary">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"/></svg>
                <?php echo t('nav_contact'); ?>
            </a>
        </div>
    </div>
</main>

<style>
.faq-item {
    transition: border-color 0.3s, box-shadow 0.3s;
}
.faq-item:hover {
    border-color: var(--brand-dim);
}
.faq-item.open {
    border-color: var(--brand-dim);
    box-shadow: 0 4px 20px rgba(212,168,67,0.06);
}
.faq-question {
    display: flex;
    align-items: center;
    gap: 14px;
    width: 100%;
    padding: 20px 24px;
    background: none;
    border: none;
    cursor: pointer;
    text-align: left;
    color: var(--text);
    font-size: 15px;
    font-weight: 600;
    font-family: inherit;
    transition: color 0.2s;
}
.faq-question:hover {
    color: var(--brand);
}
.faq-q-icon {
    flex-shrink: 0;
    width: 36px;
    height: 36px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: var(--brand-dim);
    color: var(--brand);
}
.faq-q-text {
    flex: 1;
}
.faq-arrow {
    flex-shrink: 0;
    color: var(--text-tertiary);
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.faq-item.open .faq-arrow {
    transform: rotate(180deg);
    color: var(--brand);
}
.faq-answer {
    max-height: 0;
    overflow: hidden;
    transition: max-height 0.4s cubic-bezier(0.4, 0, 0.2, 1), opacity 0.3s;
    opacity: 0;
}
.faq-item.open .faq-answer {
    max-height: 500px;
    opacity: 1;
}
.faq-answer-inner {
    padding: 0 24px 20px 74px;
    color: var(--text-secondary);
    font-size: 14px;
    line-height: 1.8;
}
@media (max-width: 640px) {
    .faq-answer-inner {
        padding-left: 24px;
    }
}
</style>

<script>
function toggleFaq(btn) {
    var item = btn.closest('.faq-item');
    var wasOpen = item.classList.contains('open');
    document.querySelectorAll('.faq-item.open').forEach(function(el) {
        el.classList.remove('open');
        el.querySelector('.faq-question').setAttribute('aria-expanded', 'false');
    });
    if (!wasOpen) {
        item.classList.add('open');
        btn.setAttribute('aria-expanded', 'true');
    }
}
</script>

<?php include 'footer.php'; ?>
