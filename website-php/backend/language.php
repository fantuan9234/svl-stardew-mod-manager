<?php
$supportedLangs = ['zh', 'zh-TW', 'en'];
$supportedLangNames = ['zh' => '简体中文', 'zh-TW' => '繁體中文', 'en' => 'English'];

$currentLang = 'zh';

if (isset($_GET['lang']) && in_array($_GET['lang'], $supportedLangs)) {
    $currentLang = $_GET['lang'];
    setcookie('lang', $currentLang, time() + 86400 * 365, '/', '', isset($_SERVER['HTTPS']), true);
} elseif (isset($_COOKIE['lang']) && in_array($_COOKIE['lang'], $supportedLangs)) {
    $currentLang = $_COOKIE['lang'];
} elseif (isset($_SERVER['HTTP_ACCEPT_LANGUAGE'])) {
    $acceptLang = strtolower(substr($_SERVER['HTTP_ACCEPT_LANGUAGE'], 0, 5));
    if (strpos($acceptLang, 'zh-tw') !== false || strpos($acceptLang, 'zh-hk') !== false) {
        $currentLang = 'zh-TW';
    } elseif (strpos($acceptLang, 'zh') !== false) {
        $currentLang = 'zh';
    } elseif (strpos($acceptLang, 'en') !== false) {
        $currentLang = 'en';
    }
}

$langFile = __DIR__ . '/../i18n/' . $currentLang . '.php';
if (!file_exists($langFile)) {
    $langFile = __DIR__ . '/../i18n/zh.php';
}

$lang = require $langFile;

function t(string $key, string $default = ''): string
{
    global $lang;
    return $lang[$key] ?? ($default ?: $key);
}

function langUrl(string $targetLang): string
{
    $uri = $_SERVER['REQUEST_URI'];
    $path = parse_url($uri, PHP_URL_PATH);
    $query = parse_url($uri, PHP_URL_QUERY);
    parse_str($query ?? '', $params);
    $params['lang'] = $targetLang;
    return $path . '?' . http_build_query($params);
}
