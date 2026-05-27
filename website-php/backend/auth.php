<?php
require_once __DIR__ . '/config.php';
require_once __DIR__ . '/security.php';

sendSecurityHeaders();

if (session_status() === PHP_SESSION_NONE) {
    session_set_cookie_params([
        'lifetime' => SESSION_LIFETIME,
        'path' => '/',
        'domain' => '',
        'secure' => isset($_SERVER['HTTPS']),
        'httponly' => true,
        'samesite' => 'Lax',
    ]);
    session_start();
}

function isLoggedIn(): bool
{
    if (!isset($_SESSION['admin_logged_in']) || $_SESSION['admin_logged_in'] !== true) {
        return false;
    }
    if (isset($_SESSION['login_time'])) {
        $elapsed = time() - (int)$_SESSION['login_time'];
        if ($elapsed > SESSION_LIFETIME) {
            logout();
            return false;
        }
    }
    return true;
}

function requireLogin(): void
{
    if (!isLoggedIn()) {
        header('Location: login.php');
        exit;
    }
}

function getAdminPasswordHash(): string
{
    require_once __DIR__ . '/db.php';
    $db = getDB();
    $row = $db->prepare("SELECT value FROM settings WHERE key = ?");
    $row->execute(['admin_password_hash']);
    $result = $row->fetch();
    return $result ? $result['value'] : ADMIN_DEFAULT_PASSWORD_HASH;
}

function updateAdminPassword(string $newPassword): bool
{
    require_once __DIR__ . '/db.php';
    $db = getDB();
    $hash = password_hash($newPassword, PASSWORD_BCRYPT);
    $db->prepare("INSERT OR REPLACE INTO settings (key, value) VALUES ('admin_password_hash', ?)")->execute([$hash]);
    return true;
}

function login(string $username, string $password): bool
{
    if (isLoginBlocked()) {
        return false;
    }

    if ($username === ADMIN_USERNAME && password_verify($password, getAdminPasswordHash())) {
        clearLoginAttempts();
        recordLoginAttempt(true);
        session_regenerate_id(true);
        $_SESSION['admin_logged_in'] = true;
        $_SESSION['admin_username'] = $username;
        $_SESSION['login_time'] = time();
        return true;
    }

    recordLoginAttempt(false);
    return false;
}

function getLoginBlockedMessage(): string
{
    return '登录尝试次数过多，请 15 分钟后再试';
}

function logout(): void
{
    $_SESSION = [];
    if (ini_get('session.use_cookies')) {
        $params = session_get_cookie_params();
        setcookie(session_name(), '', time() - 42000, $params['path'], $params['domain'], $params['secure'], $params['httponly']);
    }
    session_destroy();
}

function csrfToken(): string
{
    if (empty($_SESSION['csrf_token'])) {
        $_SESSION['csrf_token'] = bin2hex(random_bytes(32));
    }
    return $_SESSION['csrf_token'];
}

function csrfField(): string
{
    return '<input type="hidden" name="csrf_token" value="' . csrfToken() . '">';
}

function verifyCsrf(string $token): bool
{
    return isset($_SESSION['csrf_token']) && hash_equals($_SESSION['csrf_token'], $token);
}

function requireCsrf(): void
{
    $token = $_POST['csrf_token'] ?? '';
    if (!verifyCsrf($token)) {
        http_response_code(403);
        exit('CSRF 验证失败，请刷新页面后重试');
    }
}