<?php
require_once __DIR__ . '/../backend/auth.php';
require_once __DIR__ . '/../backend/db.php';
requireLogin();
initDatabase();

$db = getDB();

$today = date('Y-m-d');
$todayStats = $db->prepare("SELECT COUNT(*) as pv, COUNT(DISTINCT ip_hash) as uv FROM visitors WHERE date(created_at) = ?");
$todayStats->execute([$today]);
$todayRow = $todayStats->fetch(PDO::FETCH_ASSOC);
$todayPV = $todayRow ? (int)$todayRow['pv'] : 0;
$todayUV = $todayRow ? (int)$todayRow['uv'] : 0;

$downloadCount = $db->query("SELECT COUNT(*) FROM downloads")->fetchColumn();



$dailyRows = $db->query("SELECT date(created_at) as d, COUNT(*) as pv, COUNT(DISTINCT ip_hash) as uv FROM visitors GROUP BY d ORDER BY d DESC LIMIT 7")->fetchAll(PDO::FETCH_ASSOC);
$dailyRows = array_reverse($dailyRows);

$topPages = $db->query("SELECT page, COUNT(*) as count FROM visitors WHERE page != '' GROUP BY page ORDER BY count DESC LIMIT 10")->fetchAll(PDO::FETCH_ASSOC);

$currentPage = basename($_SERVER['PHP_SELF']);
?>
<div class="main-content">
        <h1 class="text-2xl font-bold mb-8">访客统计</h1>

        <div class="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-10">
            <div class="stat-card">
                <div class="stat-value" style="color: var(--brand);"><?php echo $todayPV; ?></div>
                <div class="stat-label">今日访问量</div>
            </div>
            <div class="stat-card">
                <div class="stat-value"><?php echo $todayUV; ?></div>
                <div class="stat-label">今日访客数</div>
            </div>
            <div class="stat-card">
                <div class="stat-value"><?php echo $downloadCount; ?></div>
                <div class="stat-label">总下载量</div>
            </div>
            <div class="stat-card">
                <div class="stat-value" style="color: #3b82f6;"><?php echo $db->query("SELECT COUNT(*) FROM contacts")->fetchColumn(); ?></div>
                <div class="stat-label">联系消息</div>
            </div>
        </div>

        <div class="mb-10">
            <h2 class="text-lg font-semibold mb-1">7天访问趋势</h2>
            <p class="text-sm mb-4" style="color: var(--text-secondary);">近7天网站访问情况</p>
            <div class="chart-container">
                <?php
                $maxPV = 1;
                foreach ($dailyRows as $row) {
                    if ((int)$row['pv'] > $maxPV) $maxPV = (int)$row['pv'];
                }
                ?>
                <div class="chart-bars">
                    <?php foreach ($dailyRows as $row): ?>
                    <?php
                    $pvHeight = $maxPV > 0 ? round(((int)$row['pv'] / $maxPV) * 160) : 4;
                    $uvHeight = $maxPV > 0 ? round(((int)$row['uv'] / $maxPV) * 160) : 4;
                    if ($pvHeight < 4) $pvHeight = 4;
                    if ($uvHeight < 4) $uvHeight = 4;
                    $label = substr($row['d'], 5);
                    ?>
                    <div class="chart-bar-group">
                        <div class="chart-bar-value"><?php echo (int)$row['pv']; ?></div>
                        <div class="chart-bar-wrapper">
                            <div class="chart-bar chart-bar-uv" style="height: <?php echo $uvHeight; ?>px;"></div>
                            <div class="chart-bar chart-bar-pv" style="height: <?php echo $pvHeight; ?>px;"></div>
                        </div>
                        <div class="chart-bar-label"><?php echo h($label); ?></div>
                    </div>
                    <?php endforeach; ?>
                    <?php if (empty($dailyRows)): ?>
                    <div style="text-align:center; color: var(--text-secondary); width:100%; padding: 40px;">暂无数据</div>
                    <?php endif; ?>
                </div>
                <?php if (!empty($dailyRows)): ?>
                <div class="chart-legend">
                    <div class="chart-legend-item">
                        <div class="chart-legend-dot" style="background: var(--brand);"></div>
                        访问量
                    </div>
                    <div class="chart-legend-item">
                        <div class="chart-legend-dot" style="background: rgba(212,168,67,0.3);"></div>
                        访客数
                    </div>
                </div>
                <?php endif; ?>
            </div>
        </div>

        <div class="mb-4">
            <h2 class="text-lg font-semibold mb-1">热门页面</h2>
            <p class="text-sm" style="color: var(--text-secondary);">访问量前10的页面</p>
        </div>
        <div class="table-wrapper">
            <table>
                <thead>
                    <tr>
                        <th>排名</th>
                        <th>页面</th>
                        <th>访问次数</th>
                    </tr>
                </thead>
                <tbody>
                    <?php if (empty($topPages)): ?>
                    <tr><td colspan="3" style="text-align:center; color: var(--text-secondary); padding: 40px;">暂无数据</td></tr>
                    <?php else: ?>
                    <?php $rank = 1; foreach ($topPages as $p): ?>
                    <tr>
                        <td style="color: var(--text-secondary);"><?php echo $rank++; ?></td>
                        <td><?php echo h($p['page']); ?></td>
                        <td style="color: var(--brand);"><?php echo (int)$p['count']; ?></td>
                    </tr>
                    <?php endforeach; ?>
                    <?php endif; ?>
                </tbody>
            </table>
        </div>
    </div>
