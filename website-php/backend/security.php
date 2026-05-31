<?php
require_once __DIR__ . '/config.php';

function sendSecurityHeaders(): void
{
    header('X-Content-Type-Options: nosniff');
    header('X-Frame-Options: DENY');
    header('Referrer-Policy: strict-origin-when-cross-origin');
    header('Permissions-Policy: camera=(), microphone=(), geolocation=(), interest-cohort=()');

    if (isset($_SERVER['HTTPS']) && $_SERVER['HTTPS'] === 'on') {
        header('Strict-Transport-Security: max-age=31536000; includeSubDomains; preload');
    }

    header("Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.tailwindcss.com; style-src 'self' 'unsafe-inline' https://fonts.googleapis.com https://cdn.tailwindcss.com; font-src 'self' https://fonts.gstatic.com; img-src 'self' data: https:; connect-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self';");
}

function getClientIp(): string
{
    if (!empty($_SERVER['HTTP_X_FORWARDED_FOR'])) {
        $ips = explode(',', $_SERVER['HTTP_X_FORWARDED_FOR']);
        return trim($ips[0]);
    }
    return $_SERVER['REMOTE_ADDR'] ?? '0.0.0.0';
}

function checkRateLimit(string $action, int $maxRequests, int $windowSeconds): bool
{
    require_once __DIR__ . '/db.php';
    $db = getDB();

    $key = $action . ':' . getClientIp();
    $now = time();
    $cutoff = $now - $windowSeconds;

    $db->prepare("DELETE FROM rate_limits WHERE action_key = ? AND timestamp < ?")->execute([$key, $cutoff]);

    $count = $db->prepare("SELECT COUNT(*) FROM rate_limits WHERE action_key = ? AND timestamp >= ?");
    $count->execute([$key, $cutoff]);

    if ((int)$count->fetchColumn() >= $maxRequests) {
        return false;
    }

    $db->prepare("INSERT INTO rate_limits (action_key, timestamp) VALUES (?, ?)")->execute([$key, $now]);
    return true;
}

function recordLoginAttempt(bool $success): void
{
    require_once __DIR__ . '/db.php';
    $db = getDB();
    $ip = getClientIp();
    $now = time();

    $db->prepare("INSERT INTO login_attempts (ip, success, timestamp) VALUES (?, ?, ?)")->execute([$ip, $success ? 1 : 0, $now]);
}

function isLoginBlocked(): bool
{
    require_once __DIR__ . '/db.php';
    $db = getDB();
    $ip = getClientIp();
    $cutoff = time() - 900;

    $db->prepare("DELETE FROM login_attempts WHERE timestamp < ?")->execute([$cutoff]);

    $count = $db->prepare("SELECT COUNT(*) FROM login_attempts WHERE ip = ? AND success = 0 AND timestamp >= ?");
    $count->execute([$ip, $cutoff]);

    return (int)$count->fetchColumn() >= 5;
}

function clearLoginAttempts(): void
{
    require_once __DIR__ . '/db.php';
    $db = getDB();
    $ip = getClientIp();
    $db->prepare("DELETE FROM login_attempts WHERE ip = ?")->execute([$ip]);
}

function backupDatabase(): array
{
    $backupDir = DATA_DIR . '/backups';
    if (!is_dir($backupDir)) {
        mkdir($backupDir, 0755, true);
    }

    $htaccess = $backupDir . '/.htaccess';
    if (!file_exists($htaccess)) {
        file_put_contents($htaccess, "Deny from all\n");
    }

    $timestamp = date('Ymd_His');
    $backupFile = $backupDir . '/site_' . $timestamp . '.db';

    if (!copy(DB_PATH, $backupFile)) {
        return ['success' => false, 'error' => '备份失败：无法复制数据库文件'];
    }

    $existing = glob($backupDir . '/site_*.db');
    if (count($existing) > 10) {
        usort($existing, function ($a, $b) {
            return filemtime($a) - filemtime($b);
        });
        foreach (array_slice($existing, 0, count($existing) - 10) as $old) {
            @unlink($old);
        }
    }

    $size = filesize($backupFile);
    return [
        'success' => true,
        'file' => basename($backupFile),
        'size' => $size,
    ];
}

function getBackups(): array
{
    $backupDir = DATA_DIR . '/backups';
    if (!is_dir($backupDir)) {
        return [];
    }
    $files = glob($backupDir . '/site_*.db');
    $backups = [];
    foreach ($files as $file) {
        $backups[] = [
            'name' => basename($file),
            'size' => filesize($file),
            'time' => filemtime($file),
        ];
    }
    usort($backups, function ($a, $b) {
        return $b['time'] - $a['time'];
    });
    return $backups;
}