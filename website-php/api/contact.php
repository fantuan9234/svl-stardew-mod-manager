<?php
require_once __DIR__ . '/../backend/security.php';
require_once __DIR__ . '/../backend/db.php';

sendSecurityHeaders();
initDatabase();

if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    http_response_code(405);
    echo json_encode(['error' => 'Method not allowed'], JSON_UNESCAPED_UNICODE);
    exit;
}

if (!checkRateLimit('contact', 3, 3600)) {
    http_response_code(429);
    echo json_encode(['error' => '发送过于频繁，请稍后再试'], JSON_UNESCAPED_UNICODE);
    exit;
}

$name = trim($_POST['name'] ?? '');
$email = trim($_POST['email'] ?? '');
$subject = trim($_POST['subject'] ?? '');
$message = trim($_POST['message'] ?? '');
$deviceId = trim($_POST['device_id'] ?? '');

if ($message === '') {
    http_response_code(400);
    echo json_encode(['error' => '消息不能为空'], JSON_UNESCAPED_UNICODE);
    exit;
}

if (mb_strlen($message) > 5000) {
    http_response_code(400);
    echo json_encode(['error' => '消息内容过长，请控制在5000字以内'], JSON_UNESCAPED_UNICODE);
    exit;
}

if ($name !== '' && mb_strlen($name) > 100) {
    http_response_code(400);
    echo json_encode(['error' => '姓名过长'], JSON_UNESCAPED_UNICODE);
    exit;
}

if ($subject !== '' && mb_strlen($subject) > 200) {
    http_response_code(400);
    echo json_encode(['error' => '主题过长'], JSON_UNESCAPED_UNICODE);
    exit;
}

if ($deviceId === '' || strlen($deviceId) !== 32) {
    http_response_code(400);
    echo json_encode(['error' => '设备标识无效，请刷新页面后重试'], JSON_UNESCAPED_UNICODE);
    exit;
}

try {
    $db = getDB();
    $stmt = $db->prepare("INSERT INTO contacts (name, email, subject, message, device_id) VALUES (?, ?, ?, ?, ?)");
    $stmt->execute([$name, $email, $subject, $message, $deviceId]);
    echo json_encode(['success' => true, 'message' => '消息已发送'], JSON_UNESCAPED_UNICODE);
} catch (Exception $e) {
    http_response_code(500);
    echo json_encode(['error' => '服务器错误，请稍后重试'], JSON_UNESCAPED_UNICODE);
}
