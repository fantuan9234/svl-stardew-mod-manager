<?php
require_once __DIR__ . '/../backend/security.php';
require_once __DIR__ . '/../backend/db.php';

sendSecurityHeaders();
initDatabase();

header('Content-Type: application/json; charset=utf-8');

$platform = trim($_GET['platform'] ?? 'windows');
if (!in_array($platform, ['windows', 'macos', 'linux'])) {
    $platform = 'windows';
}

try {
    $db = getDB();
    $stmt = $db->prepare("SELECT version, changelog, download_url, platform, is_latest, created_at FROM versions WHERE platform = ? AND is_latest = 1 LIMIT 1");
    $stmt->execute([$platform]);
    $latest = $stmt->fetch();

    if (!$latest) {
        $stmt = $db->prepare("SELECT version, changelog, download_url, platform, is_latest, created_at FROM versions WHERE platform = ? ORDER BY created_at DESC LIMIT 1");
        $stmt->execute([$platform]);
        $latest = $stmt->fetch();
    }

    if (!$latest) {
        echo json_encode(['error' => '暂无可用版本'], JSON_UNESCAPED_UNICODE);
        exit;
    }

    $allPlatforms = $db->query("SELECT DISTINCT platform FROM versions ORDER BY platform")->fetchAll(PDO::FETCH_COLUMN);

    echo json_encode([
        'version' => $latest['version'],
        'changelog' => $latest['changelog'],
        'download_url' => $latest['download_url'],
        'platform' => $latest['platform'],
        'is_latest' => (bool)$latest['is_latest'],
        'created_at' => $latest['created_at'],
        'available_platforms' => $allPlatforms,
    ], JSON_UNESCAPED_UNICODE);
} catch (Exception $e) {
    http_response_code(500);
    echo json_encode(['error' => '服务器错误'], JSON_UNESCAPED_UNICODE);
}
