<?php
// 捕获所有 PHP 错误和异常，确保始终返回 JSON
set_error_handler(function($errno, $errstr, $errfile, $errline) {
    header('Content-Type: application/json; charset=utf-8');
    http_response_code(500);
    echo json_encode(['success' => false, 'error' => '服务器内部错误: ' . $errstr . ' (行 ' . $errline . ')']);
    exit;
});

set_exception_handler(function($e) {
    header('Content-Type: application/json; charset=utf-8');
    http_response_code(500);
    echo json_encode(['success' => false, 'error' => '服务器异常: ' . $e->getMessage()]);
    exit;
});

require_once __DIR__ . '/../backend/config.php';
require_once __DIR__ . '/../backend/security.php';
require_once __DIR__ . '/../backend/db.php';

// 手动启动 session，不依赖 auth.php 的完整初始化
if (session_status() === PHP_SESSION_NONE) {
    $isSecure = (!empty($_SERVER['HTTPS']) && $_SERVER['HTTPS'] !== 'off')
        || (!empty($_SERVER['HTTP_X_FORWARDED_PROTO']) && $_SERVER['HTTP_X_FORWARDED_PROTO'] === 'https');

    session_set_cookie_params([
        'lifetime' => SESSION_LIFETIME,
        'path' => '/',
        'domain' => '',
        'secure' => $isSecure,
        'httponly' => true,
        'samesite' => 'Lax',
    ]);
    session_start();
}

header('Content-Type: application/json; charset=utf-8');

// 检查登录状态
if (empty($_SESSION['admin_logged_in']) || $_SESSION['admin_logged_in'] !== true) {
    http_response_code(401);
    echo json_encode(['success' => false, 'error' => '未登录或会话已过期，请重新登录']);
    exit;
}

if ($_SERVER['REQUEST_METHOD'] !== 'POST') {
    echo json_encode(['success' => false, 'error' => '非法请求']);
    exit;
}

// 验证 CSRF
$csrfToken = $_POST['csrf_token'] ?? '';
if (empty($_SESSION['csrf_token']) || !hash_equals($_SESSION['csrf_token'], $csrfToken)) {
    http_response_code(403);
    echo json_encode(['success' => false, 'error' => '页面已过期，请刷新后重试']);
    exit;
}

// 检查上传文件
if (empty($_FILES['image']) || $_FILES['image']['error'] !== UPLOAD_ERR_OK) {
    $errMsg = '请选择图片文件';
    if (!empty($_FILES['image'])) {
        switch ($_FILES['image']['error']) {
            case UPLOAD_ERR_INI_SIZE:
            case UPLOAD_ERR_FORM_SIZE:
                $errMsg = '文件太大，超过了服务器限制（' . ini_get('upload_max_filesize') . '）';
                break;
            case UPLOAD_ERR_PARTIAL:
                $errMsg = '文件上传不完整，请重试';
                break;
            case UPLOAD_ERR_NO_TMP_DIR:
                $errMsg = '服务器临时目录缺失，请联系管理员';
                break;
            case UPLOAD_ERR_CANT_WRITE:
                $errMsg = '服务器写入失败，请联系管理员';
                break;
        }
    }
    echo json_encode(['success' => false, 'error' => $errMsg]);
    exit;
}

$file = $_FILES['image'];
$allowedTypes = ['image/jpeg', 'image/png', 'image/gif', 'image/webp'];
$allowedExts = ['jpg', 'jpeg', 'png', 'gif', 'webp'];

$ext = strtolower(pathinfo($file['name'], PATHINFO_EXTENSION));
if (!in_array($ext, $allowedExts)) {
    echo json_encode(['success' => false, 'error' => '仅支持 JPG、PNG、GIF、WebP 格式']);
    exit;
}

if (class_exists('finfo')) {
    $finfo = new finfo(FILEINFO_MIME_TYPE);
    $realType = $finfo->file($file['tmp_name']);
    if ($realType && !in_array($realType, $allowedTypes)) {
        echo json_encode(['success' => false, 'error' => '仅支持 JPG、PNG、GIF、WebP 格式（检测到: ' . $realType . '）']);
        exit;
    }
} elseif (function_exists('mime_content_type')) {
    $realType = mime_content_type($file['tmp_name']);
    if ($realType && !in_array($realType, $allowedTypes)) {
        echo json_encode(['success' => false, 'error' => '仅支持 JPG、PNG、GIF、WebP 格式（检测到: ' . $realType . '）']);
        exit;
    }
}

if ($file['size'] > 5 * 1024 * 1024) {
    echo json_encode(['success' => false, 'error' => '图片大小不能超过 5MB']);
    exit;
}

$uploadDir = __DIR__ . '/../uploads/announcements';
if (!is_dir($uploadDir)) {
    @mkdir($uploadDir, 0777, true);
}
if (!is_dir($uploadDir)) {
    echo json_encode(['success' => false, 'error' => '无法创建上传目录: ' . $uploadDir . '，请手动创建并设置权限为 777']);
    exit;
}

if (!is_writable($uploadDir)) {
    @chmod($uploadDir, 0777);
}
if (!is_writable($uploadDir)) {
    echo json_encode(['success' => false, 'error' => '上传目录不可写: ' . $uploadDir . '，请在服务器执行: chmod 777 ' . $uploadDir]);
    exit;
}

$extMap = ['image/jpeg' => 'jpg', 'image/png' => 'png', 'image/gif' => 'gif', 'image/webp' => 'webp'];
$fileExt = isset($realType) && isset($extMap[$realType]) ? $extMap[$realType] : $ext;
$filename = date('Ymd_His') . '_' . bin2hex(random_bytes(4)) . '.' . $fileExt;
$filepath = $uploadDir . '/' . $filename;

if (!move_uploaded_file($file['tmp_name'], $filepath)) {
    echo json_encode(['success' => false, 'error' => '图片保存失败，请检查目录权限']);
    exit;
}

$imageUrl = SITE_URL . '/uploads/announcements/' . $filename;
echo json_encode(['success' => true, 'url' => $imageUrl]);
