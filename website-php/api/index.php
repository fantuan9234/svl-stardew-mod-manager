<?php
header('Content-Type: application/json; charset=utf-8');
header('Access-Control-Allow-Origin: *');
header('Access-Control-Allow-Methods: GET, POST, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type');

if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    http_response_code(204);
    exit;
}

require_once __DIR__ . '/../backend/db.php';
require_once __DIR__ . '/../backend/security.php';
require_once __DIR__ . '/../backend/config.php';

initDatabase();
$db = getDB();

$action = $_GET['action'] ?? '';
$method = $_SERVER['REQUEST_METHOD'];

try {
    switch ($action) {
        case 'announcements':
            $limit = min((int)($_GET['limit'] ?? 10), 50);
            $offset = max((int)($_GET['offset'] ?? 0), 0);
            $stmt = $db->prepare("SELECT id, title, category, content, is_pinned, created_at, updated_at FROM announcements ORDER BY is_pinned DESC, created_at DESC LIMIT ? OFFSET ?");
            $stmt->execute([$limit, $offset]);
            $items = $stmt->fetchAll();
            $total = $db->query("SELECT COUNT(*) FROM announcements")->fetchColumn();
            echo json_encode(['success' => true, 'data' => $items, 'total' => (int)$total]);
            break;

        case 'latest_version':
            $stmt = $db->prepare("SELECT version, changelog, download_url, platform, created_at FROM versions WHERE is_latest = 1 ORDER BY created_at DESC LIMIT 1");
            $stmt->execute();
            $version = $stmt->fetch();
            if (!$version) {
                $version = ['version' => '1.0.0', 'changelog' => '', 'download_url' => '', 'platform' => 'windows', 'created_at' => date('Y-m-d H:i:s')];
            }
            echo json_encode(['success' => true, 'data' => $version]);
            break;

        case 'check_update':
            if ($method !== 'GET') {
                http_response_code(405);
                echo json_encode(['success' => false, 'error' => 'Method not allowed']);
                break;
            }
            $currentVersion = trim($_GET['current'] ?? '');
            if ($currentVersion === '') {
                echo json_encode(['success' => false, 'error' => 'Missing current version']);
                break;
            }
            $stmt = $db->prepare("SELECT version, changelog, download_url, platform, created_at FROM versions WHERE is_latest = 1 ORDER BY created_at DESC LIMIT 1");
            $stmt->execute();
            $latest = $stmt->fetch();
            if (!$latest) {
                echo json_encode(['success' => true, 'data' => ['has_update' => false]]);
                break;
            }
            $hasUpdate = version_compare($latest['version'], $currentVersion, '>');
            echo json_encode(['success' => true, 'data' => ['has_update' => $hasUpdate, 'latest' => $latest]]);
            break;

        case 'download':
            if ($method !== 'POST') {
                http_response_code(405);
                echo json_encode(['success' => false, 'error' => 'Method not allowed']);
                break;
            }
            if (!checkRateLimit('download', 10, 60)) {
                http_response_code(429);
                echo json_encode(['success' => false, 'error' => 'Too many requests']);
                break;
            }
            $input = json_decode(file_get_contents('php://input'), true);
            $platform = in_array($input['platform'] ?? '', ['windows', 'macos', 'linux']) ? $input['platform'] : 'unknown';
            $ipHash = hash('sha256', getClientIp() . 'svl_salt_2026');

            $stmt = $db->prepare("INSERT INTO downloads (ip_hash, platform) VALUES (?, ?)");
            $stmt->execute([$ipHash, $platform]);

            $stmt = $db->prepare("SELECT version, download_url FROM versions WHERE is_latest = 1 AND platform = ? ORDER BY created_at DESC LIMIT 1");
            $stmt->execute([$platform]);
            $versionInfo = $stmt->fetch();

            echo json_encode([
                'success' => true,
                'data' => [
                    'download_url' => $versionInfo ? $versionInfo['download_url'] : '#',
                    'version' => $versionInfo ? $versionInfo['version'] : '1.0.0'
                ]
            ]);
            break;

        case 'track':
            if ($method !== 'POST') {
                http_response_code(405);
                echo json_encode(['success' => false, 'error' => 'Method not allowed']);
                break;
            }
            if (!checkRateLimit('track', 30, 60)) {
                http_response_code(429);
                echo json_encode(['success' => false, 'error' => 'Too many requests']);
                break;
            }
            $input = json_decode(file_get_contents('php://input'), true);
            $page = trim($input['page'] ?? '');
            $ipHash = hash('sha256', getClientIp() . 'svl_salt_2026');
            $ua = substr($_SERVER['HTTP_USER_AGENT'] ?? '', 0, 500);
            $referer = substr($_SERVER['HTTP_REFERER'] ?? '', 0, 500);

            $stmt = $db->prepare("INSERT INTO visitors (ip_hash, page, user_agent, referer) VALUES (?, ?, ?, ?)");
            $stmt->execute([$ipHash, $page, $ua, $referer]);
            echo json_encode(['success' => true]);
            break;

        case 'stats':
            if ($method !== 'GET') {
                http_response_code(405);
                echo json_encode(['success' => false, 'error' => 'Method not allowed']);
                break;
            }
            $days = min((int)($_GET['days'] ?? 7), 30);
            $stats = [];

            for ($i = $days - 1; $i >= 0; $i--) {
                $date = date('Y-m-d', strtotime("-{$i} days"));
                $stmt = $db->prepare("SELECT COUNT(*) as pv, COUNT(DISTINCT ip_hash) as uv FROM visitors WHERE date(created_at) = ?");
                $stmt->execute([$date]);
                $row = $stmt->fetch();
                $stats[] = ['date' => $date, 'pv' => (int)$row['pv'], 'uv' => (int)$row['uv']];
            }

            $totalDownloads = $db->query("SELECT COUNT(*) FROM downloads")->fetchColumn();

            echo json_encode([
                'success' => true,
                'data' => [
                    'daily' => $stats,
                    'total_downloads' => (int)$totalDownloads
                ]
            ]);
            break;

        default:
            http_response_code(400);
            echo json_encode(['success' => false, 'error' => 'Unknown action']);
            break;
    }
} catch (Exception $e) {
    http_response_code(500);
    echo json_encode(['success' => false, 'error' => 'Internal server error']);
}
