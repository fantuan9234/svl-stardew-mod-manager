<?php
date_default_timezone_set('Asia/Shanghai');

define('BASE_PATH', dirname(__DIR__));
define('DATA_DIR', BASE_PATH . '/data');
define('DB_PATH', DATA_DIR . '/site.db');

define('SITE_NAME', 'SVL - 星露谷物语模组管理器');
define('SITE_URL', 'https://svlmod.cn');

define('ADMIN_USERNAME', 'admin');
define('ADMIN_DEFAULT_PASSWORD_HASH', password_hash('svl2024admin', PASSWORD_BCRYPT));

define('SESSION_LIFETIME', 3600);

if (!is_dir(DATA_DIR)) {
    mkdir(DATA_DIR, 0755, true);
}

function h(string $str): string
{
    return htmlspecialchars($str, ENT_QUOTES, 'UTF-8');
}

function now_cn(): string
{
    $dt = new DateTime('now', new DateTimeZone('Asia/Shanghai'));
    return $dt->format('Y-m-d H:i:s');
}

function format_cn(?string $value): string
{
    if ($value === null || $value === '') {
        return '';
    }
    try {
        $dt = new DateTime($value);
        $dt->setTimezone(new DateTimeZone('Asia/Shanghai'));
        return $dt->format('Y-m-d H:i:s');
    } catch (Exception $e) {
        return $value;
    }
}