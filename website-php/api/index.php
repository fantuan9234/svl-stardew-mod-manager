<?php
header('Content-Type: application/json; charset=utf-8');

// ========== 发版时修改这里 ==========
$latest_version = '1.2.5';
$release_notes  = "更新内容：\n- 跨平台适配：支持 Linux 和 macOS\n- 自动更新支持多平台\n- 翻译字典优化\n- 修复取消翻译无效等问题";
$release_date   = '2026-05-31';
// ===================================

$current_version = $_GET['version'] ?? '0.0.0';
$os   = $_GET['os']   ?? 'windows';
$arch = $_GET['arch'] ?? 'x86_64';
$has_update = version_compare($latest_version, $current_version, '>');

// 云盘下载链接，格式：https://wp.svlmod.cn/d/SVL/SVL/{系统}/SVL_{版本号}_{架构}.{后缀}
$base_url = 'https://wp.svlmod.cn/d/SVL/SVL';

if ($os === 'linux') {
    // https://wp.svlmod.cn/d/SVL/SVL/linux/SVL_1.2.5_amd64.AppImage
    $download_url = $base_url . '/linux/SVL_' . $latest_version . '_amd64.AppImage';
} elseif ($os === 'macos') {
    if ($arch === 'aarch64') {
        // https://wp.svlmod.cn/d/SVL/SVL/macos/SVL_1.2.5_aarch64.dmg
        $download_url = $base_url . '/macos/SVL_' . $latest_version . '_aarch64.dmg';
    } else {
        // https://wp.svlmod.cn/d/SVL/SVL/macos/SVL_1.2.5_x64.dmg
        $download_url = $base_url . '/macos/SVL_' . $latest_version . '_x64.dmg';
    }
} else {
    // https://wp.svlmod.cn/d/SVL/SVL/windows/SVL_1.2.5_x64-setup.exe
    $download_url = $base_url . '/windows/SVL_' . $latest_version . '_x64-setup.exe';
}

$download_url_alt = null;
$download_label_alt = null;

echo json_encode([
    'has_update'      => $has_update,
    'current_version' => $current_version,
    'latest_version'  => $latest_version,
    'download_url'      => $download_url,
    'download_url_alt'  => $download_url_alt ?? null,
    'download_label_alt' => $download_label_alt ?? null,
    'release_notes'     => $release_notes,
    'release_date'    => $release_date,
    'file_size'       => null,
    'sha256'          => null,
    'force_update'    => true,
], JSON_UNESCAPED_UNICODE);
