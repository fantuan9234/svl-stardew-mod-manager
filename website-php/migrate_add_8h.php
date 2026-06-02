<?php
// 自包含时区迁移脚本：不依赖 config.php/db.php
// 直接通过 PDO 打开数据库，避免服务器上旧 config.php 缓存问题

$basePath = __DIR__;
$dataDir = $basePath . '/data';
$dbPath = $dataDir . '/site.db';

header('Content-Type: text/html; charset=utf-8');
?><!DOCTYPE html>
<html lang="zh-CN">
<head>
<meta charset="UTF-8">
<title>时区迁移：所有时间戳 +8 小时</title>
<style>
body { font-family: -apple-system, "PingFang SC", "Microsoft YaHei", sans-serif; max-width: 720px; margin: 40px auto; padding: 0 20px; line-height: 1.6; color: #1f2937; }
h2 { color: #111827; border-bottom: 2px solid #e5e7eb; padding-bottom: 8px; }
.ok { color: #16a34a; font-weight: 600; }
.warn { color: #d97706; font-weight: 600; }
.err { color: #dc2626; font-weight: 600; }
.btn { display: inline-block; background: #2563eb; color: #fff; padding: 10px 24px; border-radius: 6px; text-decoration: none; font-weight: 600; margin: 8px 4px 8px 0; border: none; cursor: pointer; font-size: 14px; }
.btn-danger { background: #dc2626; }
.btn-secondary { background: #6b7280; }
code { background: #f3f4f6; padding: 2px 6px; border-radius: 3px; font-size: 13px; }
.box { background: #f9fafb; border: 1px solid #e5e7eb; border-radius: 8px; padding: 16px; margin: 16px 0; }
ul { margin: 4px 0; }
</style>
</head>
<body>
<h2>时区迁移：将所有时间戳 +8 小时</h2>

<?php
if (!file_exists($dbPath)) {
    echo '<div class="box"><p class="err">✗ 找不到数据库文件：<code>' . htmlspecialchars($dbPath) . '</code></p>';
    echo '<p>请确认此脚本放在网站根目录（与 <code>backend/</code> 同级）。</p></div>';
    exit;
}

try {
    $pdo = new PDO('sqlite:' . $dbPath, null, null, [
        PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
        PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
    ]);
} catch (Exception $e) {
    echo '<div class="box"><p class="err">✗ 打开数据库失败：' . htmlspecialchars($e->getMessage()) . '</p></div>';
    exit;
}

$flag = $pdo->prepare("SELECT value FROM settings WHERE key = ?");
$flag->execute(['tz_migrated_to_cn_v1']);
$alreadyMigrated = (bool)$flag->fetch();

$action = $_GET['action'] ?? '';

if ($alreadyMigrated && $action !== 'force') {
    echo '<div class="box"><p class="ok">✓ 迁移已经执行过了</p>';
    echo '<p>数据库已设置 <code>tz_migrated_to_cn_v1</code> 标志，跳过执行。</p>';
    echo '<p>如果你确认需要重新执行（比如手动恢复了旧备份），可以点击下面的按钮强制重跑：</p>';
    echo '<p><a class="btn btn-danger" href="?action=force" onclick="return confirm(\'确定要强制重新执行迁移吗？\\n\\n如果你的数据已经修正过了，这会让时间再 +8 小时而错位！\')">强制重新执行</a>';
    echo '<a class="btn btn-secondary" href="?action=reset_flag">重置迁移标志</a></p>';
    echo '<p class="warn">提示：验证时间显示正确后，请删除此文件 <code>migrate_add_8h.php</code>。</p></div>';
    exit;
}

if ($action === 'reset_flag') {
    $pdo->prepare("DELETE FROM settings WHERE key = ?")->execute(['tz_migrated_to_cn_v1']);
    echo '<div class="box"><p class="ok">✓ 标志已重置</p>';
    echo '<p><a class="btn" href="migrate_add_8h.php">返回</a></p></div>';
    exit;
}

if ($action !== 'force' && $action !== 'do') {
    $sample = $pdo->query("SELECT created_at FROM contacts ORDER BY id DESC LIMIT 1")->fetch();
    $needMigrate = false;
    $sampleInfo = '';
    if ($sample && !empty($sample['created_at'])) {
        $stored = strtotime($sample['created_at']);
        $now = time();
        $diff = abs($now - $stored);
        if ($diff > 12 * 3600) {
            $needMigrate = true;
            $sampleInfo = '最新一条联系消息时间: ' . $sample['created_at']
                . '，与当前时间相差 ' . round($diff / 3600, 1) . ' 小时（>12h，判断为时区错误）';
        } else {
            $sampleInfo = '最新一条联系消息时间: ' . $sample['created_at']
                . '，与当前时间相差 ' . round($diff / 3600, 1) . ' 小时（≤12h，数据看起来已是正确时区）';
        }
    } else {
        $sampleInfo = '联系消息表为空，跳过预检';
    }

    echo '<div class="box">';
    echo '<p><strong>预检结果：</strong>' . htmlspecialchars($sampleInfo) . '</p>';
    if ($needMigrate) {
        echo '<p class="warn">建议执行迁移，将所有时间戳 +8 小时（UTC → Asia/Shanghai）。</p>';
        echo '<p><a class="btn" href="?action=do" onclick="return confirm(\'确定要执行迁移吗？\\n\\n脚本会自动先备份数据库。\')">执行迁移</a></p>';
    } else {
        echo '<p class="ok">数据看起来已经是正确时区，无需迁移。</p>';
        echo '<p>如果仍要执行，可以强制：<a class="btn btn-danger" href="?action=force" onclick="return confirm(\'确定要强制执行吗？\')">强制执行迁移</a></p>';
    }
    echo '</div>';
    exit;
}

echo '<div class="box">';
echo '<p><strong>正在执行迁移...</strong></p>';

$backupDir = $dataDir . '/backups';
if (!is_dir($backupDir)) {
    @mkdir($backupDir, 0755, true);
}
$htaccess = $backupDir . '/.htaccess';
if (!file_exists($htaccess)) {
    @file_put_contents($htaccess, "Deny from all\n");
}
$backupFile = $backupDir . '/site_before_tz_migration_' . date('Ymd_His') . '.db';
if (@copy($dbPath, $backupFile)) {
    echo '<p class="ok">✓ 备份已创建: <code>' . htmlspecialchars($backupFile) . '</code></p>';
} else {
    echo '<p class="err">✗ 备份失败，请检查 <code>' . htmlspecialchars($dataDir) . '</code> 目录权限</p>';
    exit;
}

$migrations = [
    'contacts'     => ['created_at', 'replied_at'],
    'announcements'=> ['created_at', 'updated_at'],
    'downloads'    => ['created_at'],
    'visitors'     => ['created_at'],
    'versions'     => ['created_at'],
    'feedback'     => ['created_at', 'updated_at'],
    'changelog'    => ['created_at', 'updated_at'],
];

$totalUpdated = 0;
foreach ($migrations as $table => $columns) {
    $quotedTable = $pdo->quote($table);
    $tableExists = $pdo->query("SELECT name FROM sqlite_master WHERE type='table' AND name=$quotedTable")->fetch();
    if (!$tableExists) {
        echo '<p>- ' . htmlspecialchars($table) . '：表不存在，跳过</p>';
        continue;
    }
    $colInfo = $pdo->query("PRAGMA table_info($table)")->fetchAll();
    $existingCols = array_column($colInfo, 'name');
    echo '<p><strong>' . htmlspecialchars($table) . '：</strong></p><ul>';
    foreach ($columns as $column) {
        if (!in_array($column, $existingCols, true)) {
            echo '<li>- ' . htmlspecialchars($column) . '：列不存在，跳过</li>';
            continue;
        }
        $count = (int)$pdo->query("SELECT COUNT(*) FROM $table WHERE $column IS NOT NULL AND $column != ''")->fetchColumn();
        $pdo->exec("UPDATE $table SET $column = datetime($column, '+8 hours') WHERE $column IS NOT NULL AND $column != ''");
        echo '<li>- ' . htmlspecialchars($column) . '：更新 ' . $count . ' 行</li>';
        $totalUpdated += $count;
    }
    echo '</ul>';
}

$pdo->prepare("INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)")->execute(['tz_migrated_to_cn_v1', '1']);

echo '<h3 class="ok">✓ 迁移完成！共更新 ' . $totalUpdated . ' 行。</h3>';
echo '<p><strong>接下来请：</strong></p>';
echo '<ol>';
echo '<li>打开后台「联系消息」页面，确认历史消息时间已修正为北京时间</li>';
echo '<li>如果一切正常，<span class="warn">务必删除此文件 <code>migrate_add_8h.php</code></span>（防止他人意外触发）</li>';
echo '<li>如果出现问题，可以使用备份 <code>' . htmlspecialchars(basename($backupFile)) . '</code> 恢复</li>';
echo '</ol>';
echo '</div>';
?>
</body>
</html>
