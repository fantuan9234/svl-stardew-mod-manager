<?php
require_once __DIR__ . '/backend/db.php';
initDatabase();

header('Content-Type: application/rss+xml; charset=utf-8');

$baseUrl = 'https://svlmod.cn';
$now = date('r');

$items = '';
try {
    $db = getDB();
    $announcements = $db->query("SELECT * FROM announcements ORDER BY is_pinned DESC, created_at DESC LIMIT 20")->fetchAll();
    foreach ($announcements as $a) {
        $link = $baseUrl . '/announcements.php#' . $a['id'];
        $pubDate = date('r', strtotime($a['created_at']));
        $items .= <<<ITEM
        <item>
            <title><![CDATA[{$a['title']}]]></title>
            <link>{$link}</link>
            <description><![CDATA[{$a['content']}]]></description>
            <pubDate>{$pubDate}</pubDate>
            <guid isPermaLink="false">announcement-{$a['id']}</guid>
            <category>{$a['category']}</category>
        </item>

ITEM;
    }
} catch (Exception $e) {
}

echo '<?xml version="1.0" encoding="UTF-8"?>' . "\n";
?>
<rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom">
    <channel>
        <title>SVL ModManager - 公告</title>
        <link><?php echo $baseUrl; ?></link>
        <description>SVL ModManager 更新公告 — 版本更新、修复说明、社区活动</description>
        <language>zh-CN</language>
        <lastBuildDate><?php echo $now; ?></lastBuildDate>
        <atom:link href="<?php echo $baseUrl; ?>/rss.php" rel="self" type="application/rss+xml"/>
        <generator>SVL ModManager RSS Generator</generator>
        <image>
            <url><?php echo $baseUrl; ?>/assets/icon.png</url>
            <title>SVL ModManager</title>
            <link><?php echo $baseUrl; ?></link>
        </image>
<?php echo $items; ?>
    </channel>
</rss>
