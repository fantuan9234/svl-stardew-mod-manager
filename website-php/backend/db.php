<?php
require_once __DIR__ . '/config.php';

function getDB(): PDO
{
    static $pdo = null;
    if ($pdo === null) {
        $pdo = new PDO('sqlite:' . DB_PATH, null, null, [
            PDO::ATTR_ERRMODE => PDO::ERRMODE_EXCEPTION,
            PDO::ATTR_DEFAULT_FETCH_MODE => PDO::FETCH_ASSOC,
            PDO::ATTR_EMULATE_PREPARES => false,
        ]);
        $pdo->exec('PRAGMA journal_mode=WAL');
        $pdo->exec('PRAGMA foreign_keys=ON');
    }
    return $pdo;
}

function initDatabase(): void
{
    $db = getDB();

    $db->exec("CREATE TABLE IF NOT EXISTS announcements (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        title TEXT NOT NULL,
        category TEXT NOT NULL DEFAULT '更新',
        content TEXT NOT NULL DEFAULT '',
        is_pinned INTEGER NOT NULL DEFAULT 0,
        created_at DATETIME NOT NULL DEFAULT (datetime('now','localtime')),
        updated_at DATETIME NOT NULL DEFAULT (datetime('now','localtime'))
    )");

    $db->exec("CREATE TABLE IF NOT EXISTS contacts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL DEFAULT '',
        email TEXT NOT NULL DEFAULT '',
        subject TEXT NOT NULL DEFAULT '',
        message TEXT NOT NULL,
        is_read INTEGER NOT NULL DEFAULT 0,
        created_at DATETIME NOT NULL DEFAULT (datetime('now','localtime'))
    )");

    $db->exec("CREATE TABLE IF NOT EXISTS downloads (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        ip_hash TEXT NOT NULL,
        version TEXT NOT NULL DEFAULT '',
        platform TEXT NOT NULL DEFAULT 'unknown',
        created_at DATETIME NOT NULL DEFAULT (datetime('now','localtime'))
    )");

    $db->exec("CREATE TABLE IF NOT EXISTS settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    )");

    $db->exec("CREATE TABLE IF NOT EXISTS login_attempts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        ip TEXT NOT NULL,
        success INTEGER NOT NULL DEFAULT 0,
        timestamp INTEGER NOT NULL
    )");

    $db->exec("CREATE TABLE IF NOT EXISTS rate_limits (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        action_key TEXT NOT NULL,
        timestamp INTEGER NOT NULL
    )");

    $db->exec("CREATE TABLE IF NOT EXISTS versions (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        version TEXT NOT NULL,
        changelog TEXT NOT NULL DEFAULT '',
        download_url TEXT NOT NULL DEFAULT '',
        platform TEXT NOT NULL DEFAULT 'windows',
        is_latest INTEGER NOT NULL DEFAULT 0,
        download_count INTEGER NOT NULL DEFAULT 0,
        created_at DATETIME NOT NULL DEFAULT (datetime('now','localtime'))
    )");

    $db->exec("CREATE TABLE IF NOT EXISTS visitors (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        ip_hash TEXT NOT NULL,
        page TEXT NOT NULL DEFAULT '',
        user_agent TEXT NOT NULL DEFAULT '',
        referer TEXT NOT NULL DEFAULT '',
        created_at DATETIME NOT NULL DEFAULT (datetime('now','localtime'))
    )");

    $db->exec("CREATE TABLE IF NOT EXISTS feedback (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        type TEXT NOT NULL DEFAULT 'suggestion',
        content TEXT NOT NULL,
        contact TEXT NOT NULL DEFAULT '',
        app_version TEXT NOT NULL DEFAULT '',
        os_info TEXT NOT NULL DEFAULT '',
        status TEXT NOT NULL DEFAULT 'pending',
        admin_reply TEXT NOT NULL DEFAULT '',
        created_at DATETIME NOT NULL DEFAULT (datetime('now','localtime')),
        updated_at DATETIME NOT NULL DEFAULT (datetime('now','localtime'))
    )");

    $db->exec("CREATE TABLE IF NOT EXISTS changelog (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        version TEXT NOT NULL,
        release_type TEXT NOT NULL DEFAULT 'update',
        title TEXT NOT NULL,
        changes TEXT NOT NULL DEFAULT '',
        release_date TEXT NOT NULL,
        created_at DATETIME NOT NULL DEFAULT (datetime('now','localtime')),
        updated_at DATETIME NOT NULL DEFAULT (datetime('now','localtime'))
    )");

    $db->exec("CREATE INDEX IF NOT EXISTS idx_login_attempts_ip_time ON login_attempts(ip, timestamp)");
    $db->exec("CREATE INDEX IF NOT EXISTS idx_rate_limits_key_time ON rate_limits(action_key, timestamp)");
    $db->exec("CREATE INDEX IF NOT EXISTS idx_visitors_date ON visitors(created_at)");
    $db->exec("CREATE INDEX IF NOT EXISTS idx_visitors_page ON visitors(page)");
    $db->exec("CREATE INDEX IF NOT EXISTS idx_feedback_status ON feedback(status)");

    // Schema migration: add missing columns for existing tables
    $cols = [];
    $result = $db->query("PRAGMA table_info(announcements)");
    while ($row = $result->fetch(PDO::FETCH_ASSOC)) {
        $cols[] = $row['name'];
    }
    if (!in_array('is_pinned', $cols)) {
        $db->exec("ALTER TABLE announcements ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0");
    }
    if (!in_array('updated_at', $cols)) {
        $db->exec("ALTER TABLE announcements ADD COLUMN updated_at DATETIME NOT NULL DEFAULT (datetime('now','localtime'))");
    }
    if (!in_array('image_url', $cols)) {
        $db->exec("ALTER TABLE announcements ADD COLUMN image_url TEXT NOT NULL DEFAULT ''");
    }

    // Migration for contacts: add is_read column if missing
    $contactCols = [];
    $result = $db->query("PRAGMA table_info(contacts)");
    while ($row = $result->fetch(PDO::FETCH_ASSOC)) {
        $contactCols[] = $row['name'];
    }
    if (!in_array('is_read', $contactCols)) {
        $db->exec("ALTER TABLE contacts ADD COLUMN is_read INTEGER NOT NULL DEFAULT 0");
    }
    if (!in_array('name', $contactCols)) {
        $db->exec("ALTER TABLE contacts ADD COLUMN name TEXT NOT NULL DEFAULT ''");
    }
    if (!in_array('email', $contactCols)) {
        $db->exec("ALTER TABLE contacts ADD COLUMN email TEXT NOT NULL DEFAULT ''");
    }
    if (!in_array('subject', $contactCols)) {
        $db->exec("ALTER TABLE contacts ADD COLUMN subject TEXT NOT NULL DEFAULT ''");
    }
    if (!in_array('admin_reply', $contactCols)) {
        $db->exec("ALTER TABLE contacts ADD COLUMN admin_reply TEXT NOT NULL DEFAULT ''");
    }
    if (!in_array('replied_at', $contactCols)) {
        $db->exec("ALTER TABLE contacts ADD COLUMN replied_at DATETIME DEFAULT NULL");
    }
    if (!in_array('device_id', $contactCols)) {
        $db->exec("ALTER TABLE contacts ADD COLUMN device_id TEXT NOT NULL DEFAULT ''");
    }

    // Migration for versions: add alt download fields and download_count
    $versionCols = [];
    $result = $db->query("PRAGMA table_info(versions)");
    while ($row = $result->fetch(PDO::FETCH_ASSOC)) {
        $versionCols[] = $row['name'];
    }
    if (!in_array('download_url_alt', $versionCols)) {
        $db->exec("ALTER TABLE versions ADD COLUMN download_url_alt TEXT NOT NULL DEFAULT ''");
    }
    if (!in_array('download_label_alt', $versionCols)) {
        $db->exec("ALTER TABLE versions ADD COLUMN download_label_alt TEXT NOT NULL DEFAULT ''");
    }
    if (!in_array('download_count', $versionCols)) {
        $db->exec("ALTER TABLE versions ADD COLUMN download_count INTEGER NOT NULL DEFAULT 0");
    }

    // Migration for downloads: add version field
    $downloadCols = [];
    $result = $db->query("PRAGMA table_info(downloads)");
    while ($row = $result->fetch(PDO::FETCH_ASSOC)) {
        $downloadCols[] = $row['name'];
    }
    if (!in_array('version', $downloadCols)) {
        $db->exec("ALTER TABLE downloads ADD COLUMN version TEXT NOT NULL DEFAULT ''");
    }

    // Only insert seed data once using settings flag
    $seeded = $db->prepare("SELECT value FROM settings WHERE key = ?");
    $seeded->execute(['seed_data_v2']);
    if (!$seeded->fetch()) {
        $count = $db->query("SELECT COUNT(*) FROM announcements")->fetchColumn();
        if ($count == 0) {
            $now = date('Y-m-d H:i:s');
            $stmt = $db->prepare("INSERT INTO announcements (title, category, content, is_pinned, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)");
            $items = [
                ['v2.0 版本正式发布', '更新', '全新界面设计，支持 SMAPI 4.0，MOD 自动更新功能上线', 0, '2026-05-15 12:00:00', '2026-05-15 12:00:00'],
                ['修复了部分 MOD 加载问题', '修复', '解决了 Content Patcher 1.30+ 版本的兼容性问题', 0, '2026-05-10 10:00:00', '2026-05-10 10:00:00'],
                ['欢迎加入 Discord 社区', '社区', '与其他农场主交流心得，获取最新更新资讯', 0, '2026-05-01 14:00:00', '2026-05-01 14:00:00'],
            ];
            foreach ($items as $item) {
                $stmt->execute($item);
            }
        }
        $db->prepare("INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)")->execute(['seed_data_v2', '1']);
    }

    // Seed changelog data — only insert once, never re-insert after user deletes
    $changelogSeeded = $db->prepare("SELECT value FROM settings WHERE key = ?");
    $changelogSeeded->execute(['seed_changelog_v1']);
    if (!$changelogSeeded->fetch()) {
        $count = $db->query("SELECT COUNT(*) FROM changelog")->fetchColumn();
        if ($count == 0) {
            $stmt = $db->prepare("INSERT INTO changelog (version, release_type, title, changes, release_date, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)");
            $items = [
                [
                    'v1.0.2',
                    'update',
                    'N 网一键下载 & 稳定性提升',
                    json_encode([
                        ['type' => 'new', 'text' => '新增 Nexus Mods 内置浏览器，登录后可搜索并一键下载安装 MOD'],
                        ['type' => 'new', 'text' => '新增 MOD 依赖关系可视化图表'],
                        ['type' => 'fix', 'text' => '修复部分 MOD 解压后文件夹嵌套导致无法识别的问题'],
                        ['type' => 'fix', 'text' => '修复 SMAPI 路径检测在 GOG 版本上的兼容性问题'],
                        ['type' => 'improve', 'text' => '优化 MOD 列表加载速度，大幅减少卡顿'],
                        ['type' => 'improve', 'text' => '升级冲突检测引擎，支持更多 MOD 类型的兼容性检查'],
                    ], JSON_UNESCAPED_UNICODE),
                    '2025-05-18',
                    '2025-05-18 12:00:00',
                    '2025-05-18 12:00:00'
                ],
                [
                    'v1.0.1',
                    'fix',
                    'Bug 修复 & 体验优化',
                    json_encode([
                        ['type' => 'fix', 'text' => '修复配置方案切换时部分 MOD 启用状态未正确保存的问题'],
                        ['type' => 'fix', 'text' => '修复备份恢复后游戏启动报错的兼容性问题'],
                        ['type' => 'fix', 'text' => '修复深色模式下部分文字对比度不足的问题'],
                        ['type' => 'improve', 'text' => '优化拖拽安装的文件解析逻辑，支持更多压缩格式'],
                        ['type' => 'improve', 'text' => '改进版本更新检查的稳定性'],
                    ], JSON_UNESCAPED_UNICODE),
                    '2025-04-22',
                    '2025-04-22 10:00:00',
                    '2025-04-22 10:00:00'
                ],
                [
                    'v1.0.0',
                    'release',
                    'SVL 正式发布 🎉',
                    json_encode([
                        ['type' => 'new', 'text' => '一键安装 MOD — 拖拽 .zip 文件即可自动安装'],
                        ['type' => 'new', 'text' => '自动冲突检测 — 智能分析 MOD 依赖和兼容性'],
                        ['type' => 'new', 'text' => '智能备份恢复 — 修改前自动备份，一键回滚'],
                        ['type' => 'new', 'text' => 'MOD 版本管理 — 自动检测更新，支持版本锁定'],
                        ['type' => 'new', 'text' => '配置方案切换 — 保存多套 MOD 配置，一键切换'],
                        ['type' => 'new', 'text' => '极速启动游戏 — 内置 SMAPI 集成，多平台支持'],
                    ], JSON_UNESCAPED_UNICODE),
                    '2025-03-15',
                    '2025-03-15 12:00:00',
                    '2025-03-15 12:00:00'
                ],
            ];
            foreach ($items as $item) {
                $stmt->execute($item);
            }
        }
        $db->prepare("INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)")->execute(['seed_changelog_v1', '1']);
    }
}