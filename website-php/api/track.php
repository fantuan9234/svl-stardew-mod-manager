<?php
require_once __DIR__ . '/../backend/security.php';
require_once __DIR__ . '/../backend/db.php';

sendSecurityHeaders();
initDatabase();

header('Content-Type: application/json; charset=utf-8');

if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    http_response_code(405);
    echo json_encode(['error' => 'Method not allowed'], JSON_UNESCAPED_UNICODE);
    exit;
}

$input = json_decode(file_get_contents('php://input'), true);
$ip = $_SERVER['REMOTE_ADDR'] ?? '0.0.0.0';
$ipHash = hash('sha256', $ip);
$userAgent = $_SERVER['HTTP_USER_AGENT'] ?? '';
$referer = $_SERVER['HTTP_REFERER'] ?? '';

try {
    $db = getDB();
    $inserted = 0;

    // Handle batch upload (cached data)
    if (!empty($input['batch']) && is_array($input['batch'])) {
        $stmt = $db->prepare("INSERT INTO visitors (ip_hash, page, user_agent, referer) VALUES (?, ?, ?, ?)");
        foreach ($input['batch'] as $item) {
            $page = trim($item['page'] ?? 'index.php');
            $stmt->execute([$ipHash, $page, $userAgent, $referer]);
            $inserted++;
        }
        echo json_encode(['success' => true, 'inserted' => $inserted], JSON_UNESCAPED_UNICODE);
        exit;
    }

    // Handle single request
    $page = trim($input['page'] ?? 'index.php');
    $stmt = $db->prepare("INSERT INTO visitors (ip_hash, page, user_agent, referer) VALUES (?, ?, ?, ?)");
    $stmt->execute([$ipHash, $page, $userAgent, $referer]);
    echo json_encode(['success' => true], JSON_UNESCAPED_UNICODE);
} catch (Exception $e) {
    http_response_code(500);
    echo json_encode(['error' => 'Server error'], JSON_UNESCAPED_UNICODE);
}
