<?php
require_once __DIR__ . '/backend/db.php';
initDatabase();

header('Content-Type: application/xml; charset=utf-8');

$baseUrl = 'https://svlmod.cn';
$now = date('c');

$pages = [
    ['loc' => $baseUrl . '/', 'lastmod' => $now, 'changefreq' => 'daily', 'priority' => '1.0'],
    ['loc' => $baseUrl . '/announcements.php', 'lastmod' => $now, 'changefreq' => 'daily', 'priority' => '0.8'],
    ['loc' => $baseUrl . '/contact.php', 'lastmod' => $now, 'changefreq' => 'monthly', 'priority' => '0.5'],
];

$langs = ['zh' => 'zh', 'zh-TW' => 'zh-Hant', 'en' => 'en'];

try {
    $db = getDB();
    $announcements = $db->query("SELECT id, created_at, updated_at FROM announcements ORDER BY created_at DESC")->fetchAll();
    foreach ($announcements as $a) {
        $pages[] = [
            'loc' => $baseUrl . '/announcements.php#' . $a['id'],
            'lastmod' => date('c', strtotime($a['updated_at'] ?? $a['created_at'])),
            'changefreq' => 'monthly',
            'priority' => '0.6',
        ];
    }

    $versions = $db->query("SELECT version, created_at FROM versions ORDER BY created_at DESC")->fetchAll();
    foreach ($versions as $v) {
        $pages[] = [
            'loc' => $baseUrl . '/?v=' . urlencode($v['version']),
            'lastmod' => date('c', strtotime($v['created_at'])),
            'changefreq' => 'monthly',
            'priority' => '0.5',
        ];
    }
} catch (Exception $e) {
}

echo '<?xml version="1.0" encoding="UTF-8"?>' . "\n";
?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"
        xmlns:xhtml="http://www.w3.org/1999/xhtml"
        xmlns:image="http://www.google.com/schemas/sitemap-image/1.1">
<?php foreach ($pages as $p): ?>
    <url>
        <loc><?php echo h($p['loc']); ?></loc>
        <lastmod><?php echo h($p['lastmod'] ?? $now); ?></lastmod>
        <changefreq><?php echo h($p['changefreq']); ?></changefreq>
        <priority><?php echo h($p['priority']); ?></priority>
<?php foreach ($langs as $lg => $hl): ?>
        <xhtml:link rel="alternate" hreflang="<?php echo h($hl); ?>" href="<?php echo h($p['loc']); ?><?php echo strpos($p['loc'], '?') === false ? '?' : '&amp;'; ?>lang=<?php echo h($lg); ?>"/>
<?php endforeach; ?>
        <xhtml:link rel="alternate" hreflang="x-default" href="<?php echo h($p['loc']); ?>"/>
    </url>
<?php endforeach; ?>
</urlset>