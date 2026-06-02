<?php
require_once __DIR__ . '/../backend/security.php';
require_once __DIR__ . '/../backend/db.php';

sendSecurityHeaders();
initDatabase();

if ($_SERVER['REQUEST_METHOD'] === 'GET') {
    $platform = trim($_GET['platform'] ?? 'windows');
    if (!in_array($platform, ['windows', 'macos', 'linux'])) {
        $platform = 'windows';
    }

    $db = getDB();
    $stmt = $db->prepare("SELECT * FROM versions WHERE platform = ? AND is_latest = 1 LIMIT 1");
    $stmt->execute([$platform]);
    $version = $stmt->fetch();

    if (!$version) {
        $stmt = $db->prepare("SELECT * FROM versions WHERE platform = ? ORDER BY created_at DESC LIMIT 1");
        $stmt->execute([$platform]);
        $version = $stmt->fetch();
    }

    if (!$version || empty($version['download_url'])) {
        http_response_code(404);
        echo '未找到可用版本';
        exit;
    }

    $ip = $_SERVER['REMOTE_ADDR'] ?? '0.0.0.0';
    $ipHash = hash('sha256', $ip);
    $versionStr = $version['version'] ?? '';
    try {
        $stmt = $db->prepare("INSERT INTO downloads (ip_hash, version, platform) VALUES (?, ?, ?)");
        $stmt->execute([$ipHash, $versionStr, $platform]);
        // Increment version download count
        if ($versionStr) {
            $db->prepare("UPDATE versions SET download_count = download_count + 1 WHERE version = ? AND platform = ?")
               ->execute([$versionStr, $platform]);
        }
    } catch (Exception $e) {
    }

    header('Location: ' . $version['download_url'], true, 302);
    exit;
}

header('Content-Type: application/json; charset=utf-8');

if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    http_response_code(405);
    echo json_encode(['error' => 'Method not allowed'], JSON_UNESCAPED_UNICODE);
    exit;
}

$platform = trim($_POST['platform'] ?? 'unknown');
if ($platform !== 'unknown' && !in_array($platform, ['windows', 'macos', 'linux', 'android', 'ios'])) {
    $platform = 'unknown';
}

$ip = $_SERVER['REMOTE_ADDR'] ?? '0.0.0.0';
$ipHash = hash('sha256', $ip);

try {
    $db = getDB();
    $stmt = $db->prepare("INSERT INTO downloads (ip_hash, version, platform) VALUES (?, ?, ?)");
    $stmt->execute([$ipHash, '', $platform]);

    $total = $db->query("SELECT COUNT(DISTINCT ip_hash) FROM downloads")->fetchColumn();
    echo json_encode(['success' => true, 'total' => (int)$total], JSON_UNESCAPED_UNICODE);
} catch (Exception $e) {
    http_response_code(500);
    echo json_encode(['error' => 'Server error'], JSON_UNESCAPED_UNICODE);
}
