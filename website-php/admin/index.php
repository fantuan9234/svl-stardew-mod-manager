<?php
require_once __DIR__ . '/../backend/auth.php';
require_once __DIR__ . '/../backend/db.php';
requireLogin();
initDatabase();

if (!defined('ADMIN_LAYOUT') && basename($_SERVER['SCRIPT_NAME']) === 'index.php') {
    header('Location: ' . SITE_URL . '/admin/layout.php');
    exit;
}

$db = getDB();
$message = '';

$announcementCount = $db->query("SELECT COUNT(*) FROM announcements")->fetchColumn();
$contactCount = $db->query("SELECT COUNT(*) FROM contacts")->fetchColumn();
$unreadCount = $db->query("SELECT COUNT(*) FROM contacts WHERE is_read = 0")->fetchColumn();
$downloadCount = $db->query("SELECT COUNT(*) FROM downloads")->fetchColumn();
$versionCount = $db->query("SELECT COUNT(*) FROM versions")->fetchColumn();

$today = date('Y-m-d');
$todayPV = $db->prepare("SELECT COUNT(*) FROM visitors WHERE date(created_at) = ?");
$todayPV->execute([$today]);
$todayPV = (int)$todayPV->fetchColumn();
$todayUV = $db->prepare("SELECT COUNT(DISTINCT ip_hash) FROM visitors WHERE date(created_at) = ?");
$todayUV->execute([$today]);
$todayUV = (int)$todayUV->fetchColumn();

$recentContacts = $db->query("SELECT * FROM contacts ORDER BY created_at DESC LIMIT 5")->fetchAll();
$latestVersion = $db->query("SELECT version, platform, created_at FROM versions WHERE is_latest = 1 ORDER BY created_at DESC LIMIT 1")->fetch();
?>
<div class="main-content">
        <h1 class="text-2xl font-bold mb-8">仪表盘</h1>

        <?php if ($message): ?><div class="msg-success"><?php echo h($message); ?></div><?php endif; ?>

        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-10">
            <div class="stat-card">
                <div class="stat-value" style="color: var(--brand);"><?php echo $todayPV; ?></div>
                <div class="stat-label">今日访问</div>
            </div>
            <div class="stat-card">
                <div class="stat-value"><?php echo $todayUV; ?></div>
                <div class="stat-label">今日访客</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" style="color: #22c55e;"><?php echo $downloadCount; ?></div>
                <div class="stat-label">总下载量</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" style="color: #3b82f6;"><?php echo $versionCount; ?></div>
                <div class="stat-label">版本数</div>
            </div>
        </div>

        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-10">
            <div class="stat-card">
                <div class="stat-value"><?php echo $announcementCount; ?></div>
                <div class="stat-label">公告总数</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" style="color: #ef4444;"><?php echo $unreadCount; ?></div>
                <div class="stat-label">未读消息</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" style="color: #3b82f6;"><?php echo $contactCount; ?></div>
                <div class="stat-label">消息总数</div>
            </div>
        </div>

        <?php if ($latestVersion): ?>
        <div class="stat-card mb-10" style="border-left: 3px solid var(--brand);">
            <div class="flex items-center justify-between">
                <div>
                    <div class="text-sm" style="color: var(--text-secondary);">最新版本</div>
                    <div class="text-xl font-bold mt-1" style="color: var(--brand);">v<?php echo h($latestVersion['version']); ?> <span class="text-xs font-normal" style="color: var(--text-secondary);"><?php echo h($latestVersion['platform']); ?></span></div>
                </div>
                <div class="text-right">
                    <div class="text-xs" style="color: var(--text-tertiary);">发布时间</div>
                    <div class="text-sm mt-1" style="color: var(--text-secondary);"><?php echo h($latestVersion['created_at']); ?></div>
                </div>
            </div>
        </div>
        <?php endif; ?>

        <div class="mb-4">
            <h2 class="text-lg font-semibold mb-1">最近联系消息</h2>
            <p class="text-sm" style="color: var(--text-secondary);">最新 5 条消息</p>
        </div>
        <div class="table-wrapper">
            <table>
                <thead>
                    <tr>
                        <th>姓名</th>
                        <th>微信</th>
                        <th>主题</th>
                        <th>状态</th>
                        <th>时间</th>
                    </tr>
                </thead>
                <tbody>
                    <?php if (empty($recentContacts)): ?>
                    <tr><td colspan="5" style="text-align:center; color: var(--text-secondary); padding: 40px;">暂无消息</td></tr>
                    <?php else: ?>
                    <?php foreach ($recentContacts as $c): ?>
                    <tr>
                        <td><?php echo h($c['name']); ?></td>
                        <td style="color: var(--text-secondary);"><?php echo h($c['email']); ?></td>
                        <td><?php echo h($c['subject']); ?></td>
                        <td><span class="badge <?php echo $c['is_read'] ? 'badge-read' : 'badge-unread'; ?>"><?php echo $c['is_read'] ? '已读' : '未读'; ?></span></td>
                        <td style="color: var(--text-secondary); font-size: 13px;"><?php echo h(format_cn($c['created_at'])); ?></td>
                    </tr>
                    <?php endforeach; ?>
                    <?php endif; ?>
                </tbody>
            </table>
        </div>
</div>
