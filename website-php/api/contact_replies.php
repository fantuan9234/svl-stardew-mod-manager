<?php
header('Content-Type: application/json; charset=utf-8');
header('Access-Control-Allow-Origin: *');
header('Access-Control-Allow-Methods: GET, OPTIONS');
header('Access-Control-Allow-Headers: Content-Type');

if ($_SERVER['REQUEST_METHOD'] === 'OPTIONS') {
    http_response_code(204);
    exit;
}

require_once __DIR__ . '/../backend/db.php';
require_once __DIR__ . '/../backend/security.php';

initDatabase();

$deviceId = trim($_GET['device_id'] ?? '');

if ($deviceId === '' || strlen($deviceId) !== 32) {
    http_response_code(400);
    echo json_encode(['success' => false, 'error' => 'Invalid device id'], JSON_UNESCAPED_UNICODE);
    exit;
}

try {
    $db = getDB();
    $stmt = $db->prepare("SELECT id, name, email, subject, message, admin_reply, created_at, replied_at FROM contacts WHERE device_id = ? ORDER BY created_at DESC LIMIT 20");
    $stmt->execute([$deviceId]);
    $items = $stmt->fetchAll();
    foreach ($items as &$item) {
        $item['created_at'] = format_cn($item['created_at']);
        $item['replied_at'] = format_cn($item['replied_at']);
    }
    unset($item);
    echo json_encode(['success' => true, 'data' => $items], JSON_UNESCAPED_UNICODE);
} catch (Exception $e) {
    http_response_code(500);
    echo json_encode(['success' => false, 'error' => 'Server error'], JSON_UNESCAPED_UNICODE);
}
