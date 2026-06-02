<?php
header('Content-Type: application/json; charset=utf-8');

// ========== 发版时修改这里（各平台独立控制） ==========
$releases = [
    'windows' => [
        'latest_version' => '1.2.6',
        'release_notes'  => "更新内容：\n- 增加ai翻译之前自动备份\n- \n- \n- ",
        'release_date'   => '2026-05-31',
    ],
    'linux' => [
        'latest_version' => '1.2.6',
        'release_notes'  => "更新内容：\n- 增加ai翻译之前自动备份\n- \n- \n- ",
        'release_date'   => '2026-05-31',
    ],
    'macos' => [
        'latest_version' => '1.2.6',
        'release_notes'  => "更新内容：\n- 增加ai翻译之前自动备份\n- \n- \n- ",
        'release_date'   => '2026-05-31',
    ],
];
// ===================================

$current_version = $_GET['version'] ?? '0.0.0';
$os   = $_GET['os']   ?? 'windows';
$arch = $_GET['arch'] ?? 'x86_64';

$platform = isset($releases[$os]) ? $os : 'windows';
$release  = $releases[$platform];

$latest_version = $release['latest_version'];
$release_notes  = $release['release_notes'];
$release_date   = $release['release_date'];
$has_update = version_compare($latest_version, $current_version, '>');

$base_url = 'https://wp.svlmod.cn/d/SVL/SVL';

if ($platform === 'linux') {
    $download_url = $base_url . '/linux/SVL_' . $latest_version . '_amd64.AppImage';
} elseif ($platform === 'macos') {
    if ($arch === 'aarch64') {
        $download_url = $base_url . '/macos/SVL_' . $latest_version . '_aarch64.dmg';
    } else {
        $download_url = $base_url . '/macos/SVL_' . $latest_version . '_x64.dmg';
    }
} else {
    $download_url = $base_url . '/windows/SVL_' . $latest_version . '_x64-setup.exe';
}

echo json_encode([
    'has_update'      => $has_update,
    'current_version' => $current_version,
    'latest_version'  => $latest_version,
    'download_url'    => $download_url,
    'release_notes'   => $release_notes,
    'release_date'    => $release_date,
    'file_size'       => null,
    'sha256'          => null,
    'force_update'    => true,
], JSON_UNESCAPED_UNICODE);
