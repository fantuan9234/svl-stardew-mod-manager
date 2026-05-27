<?php
require_once __DIR__ . '/../backend/auth.php';
require_once __DIR__ . '/../backend/db.php';
requireLogin();
initDatabase();

$db = getDB();
$currentPage = basename($_SERVER['PHP_SELF']);

$unreadCount = $db->query("SELECT COUNT(*) FROM contacts WHERE is_read = 0")->fetchColumn();

if (isset($_SERVER['HTTP_X_REQUESTED_WITH']) && $_SERVER['HTTP_X_REQUESTED_WITH'] === 'XMLHttpRequest') {
    $page = $_GET['page'] ?? 'index';
    $allowed = ['index', 'announcements', 'changelog', 'contacts', 'versions', 'stats', 'settings'];
    if (in_array($page, $allowed) && file_exists($page . '.php')) {
        include $page . '.php';
    }
    exit;
}
?>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>管理后台</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;500;600;700&family=Noto+Sans+SC:wght@300;400;500;700&display=swap" rel="stylesheet">
    <style>
        :root {
            --bg: #0c0c0e;
            --surface: #141416;
            --border: rgba(255,255,255,0.06);
            --text: #f0f0f0;
            --text-secondary: #999;
            --brand: #d4a843;
        }
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: 'Outfit', 'Noto Sans SC', sans-serif;
            background: var(--bg);
            color: var(--text);
            min-height: 100vh;
        }
        .sidebar {
            position: fixed;
            top: 0; left: 0; bottom: 0;
            width: 220px;
            background: var(--surface);
            border-right: 1px solid var(--border);
            padding: 24px 16px;
            z-index: 10;
        }
        .main-content {
            margin-left: 220px;
            padding: 32px;
            min-height: 100vh;
        }
        .nav-link {
            display: flex;
            align-items: center;
            gap: 10px;
            padding: 10px 14px;
            border-radius: 10px;
            color: var(--text-secondary);
            text-decoration: none;
            font-size: 14px;
            transition: all 0.2s;
            margin-bottom: 4px;
            cursor: pointer;
            border: none;
            background: none;
            width: 100%;
            font-family: inherit;
        }
        .nav-link:hover, .nav-link.active { background: rgba(255,255,255,0.05); color: var(--text); }
        .nav-link.active { color: var(--brand); }
        .nav-link .badge {
            margin-left: auto;
        }
        .stat-card {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 14px;
            padding: 24px;
        }
        .stat-value { font-size: 28px; font-weight: 700; margin-bottom: 4px; }
        .stat-label { font-size: 13px; color: var(--text-secondary); }
        .table-wrapper {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 14px;
            overflow: hidden;
        }
        table { width: 100%; border-collapse: collapse; }
        th {
            text-align: left;
            padding: 14px 20px;
            font-size: 12px;
            font-weight: 500;
            color: var(--text-secondary);
            text-transform: uppercase;
            letter-spacing: 0.5px;
            border-bottom: 1px solid var(--border);
            background: rgba(255,255,255,0.02);
        }
        td {
            padding: 14px 20px;
            font-size: 14px;
            border-bottom: 1px solid rgba(255,255,255,0.03);
        }
        tr:last-child td { border-bottom: none; }
        .badge {
            display: inline-block;
            padding: 3px 10px;
            border-radius: 20px;
            font-size: 11px;
            font-weight: 500;
        }
        .badge-unread { background: rgba(212,168,67,0.15); color: #d4a843; }
        .badge-read { background: rgba(255,255,255,0.05); color: #666; }

        .page-loading {
            position: fixed;
            top: 0; left: 220px; right: 0; bottom: 0;
            background: var(--bg);
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 5;
            opacity: 0;
            pointer-events: none;
            transition: opacity 0.2s ease;
        }
        .page-loading.show {
            opacity: 1;
            pointer-events: auto;
        }
        .page-loading-spinner {
            width: 32px; height: 32px;
            border: 2px solid var(--border);
            border-top-color: var(--brand);
            border-radius: 50%;
            animation: spin 0.8s linear infinite;
        }
        @keyframes spin { to { transform: rotate(360deg); } }

        .nav-link.loading { opacity: 0.6; pointer-events: none; }

        .form-card {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 24px;
            margin-bottom: 24px;
        }
        .label {
            display: block;
            font-size: 13px;
            font-weight: 500;
            color: var(--text-secondary);
            margin-bottom: 6px;
        }
        .input-field {
            width: 100%;
            padding: 10px 14px;
            border: 1px solid var(--border);
            border-radius: 8px;
            background: var(--bg);
            color: var(--text);
            font-size: 14px;
            outline: none;
            transition: border-color 0.2s;
            box-sizing: border-box;
        }
        .input-field:focus { border-color: var(--brand); }
        .checkbox-label {
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 14px;
            color: var(--text);
            cursor: pointer;
        }
        .msg-success {
            background: rgba(0,184,148,0.1);
            border: 1px solid rgba(0,184,148,0.3);
            color: #00b894;
            padding: 12px 16px;
            border-radius: 8px;
            margin-bottom: 16px;
            font-size: 14px;
        }
        .msg-error {
            background: rgba(255,71,87,0.1);
            border: 1px solid rgba(255,71,87,0.3);
            color: #ff4757;
            padding: 12px 16px;
            border-radius: 8px;
            margin-bottom: 16px;
            font-size: 14px;
        }
        .badge {
            display: inline-flex;
            align-items: center;
            padding: 4px 10px;
            border-radius: 6px;
            font-size: 12px;
            font-weight: 500;
        }
        .badge-platform-windows { background: rgba(0,120,212,0.15); color: #0078d4; }
        .badge-platform-macos { background: rgba(120,120,120,0.15); color: #888; }
        .badge-platform-linux { background: rgba(212,168,67,0.15); color: #d4a843; }
        .badge-release { background: rgba(0,184,148,0.15); color: #00b894; }
        .badge-update { background: rgba(0,120,212,0.15); color: #0078d4; }
        .badge-fix { background: rgba(255,71,87,0.15); color: #ff4757; }
        .badge-cat { background: rgba(212,168,67,0.15); color: var(--brand); }
        .badge-replied { background: rgba(0,184,148,0.15); color: #00b894; }
        .badge-read { background: rgba(255,255,255,0.05); color: #666; }
        .badge-unread { background: rgba(255,71,87,0.15); color: #ff4757; }
        .badge-bug { background: rgba(255,71,87,0.15); color: #ff4757; }
        .badge-suggestion { background: rgba(0,120,212,0.15); color: #0078d4; }
        .badge-praise { background: rgba(0,184,148,0.15); color: #00b894; }
        .badge-other { background: rgba(120,120,120,0.15); color: #888; }
        .badge-pending { background: rgba(255,165,0,0.15); color: #ff8c00; }
        .badge-processing { background: rgba(0,120,212,0.15); color: #0078d4; }
        .badge-resolved { background: rgba(0,184,148,0.15); color: #00b894; }
        .badge-closed { background: rgba(120,120,120,0.15); color: #888; }

        .row-unread, .row-pending { background: rgba(212,168,67,0.04); }
        .message-cell, .content-cell { cursor: pointer; max-width: 300px; }
        .message-short, .content-short { font-size: 13px; }
        .message-full, .content-full { display: none; margin-top: 6px; font-size: 13px; line-height: 1.6; }
        .message-full.show, .content-full.show { display: block; }
        .message-expand-hint, .content-expand-hint { font-size: 11px; color: var(--brand); margin-top: 4px; }
        .existing-reply { margin-top: 8px; padding: 8px 12px; background: rgba(0,184,148,0.08); border-radius: 6px; font-size: 12px; color: var(--text-secondary); }
        .status-select { padding: 4px 8px; border-radius: 6px; border: 1px solid var(--border); background: var(--bg); color: var(--text); font-size: 12px; }
        .backup-item { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; border: 1px solid var(--border); border-radius: 8px; margin-bottom: 6px; }

        .contact-wechat { display: flex; align-items: center; gap: 6px; }
        .btn-copy-wechat { background: none; border: none; color: var(--text-secondary); cursor: pointer; padding: 2px; border-radius: 4px; display: flex; align-items: center; transition: color 0.2s; }
        .btn-copy-wechat:hover { color: var(--brand); }
        .existing-reply { margin-top: 8px; padding: 10px 12px; background: rgba(0,184,148,0.06); border: 1px solid rgba(0,184,148,0.15); border-radius: 8px; }
        .reply-label { font-size: 11px; font-weight: 600; color: #00b894; margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.5px; }
        .reply-content { font-size: 13px; line-height: 1.6; color: var(--text); }
        .reply-time { font-size: 11px; color: var(--text-secondary); margin-top: 6px; }

        .reply-modal-overlay { display: none; position: fixed; inset: 0; background: rgba(0,0,0,0.5); z-index: 100; align-items: center; justify-content: center; }
        .reply-modal-overlay.show { display: flex; }
        .reply-modal { background: var(--surface); border-radius: 12px; padding: 24px; width: 480px; max-width: 90vw; }
        .reply-modal h3 { font-size: 16px; font-weight: 600; margin-bottom: 16px; }
        .reply-modal textarea { width: 100%; padding: 10px 14px; border: 1px solid var(--border); border-radius: 8px; background: var(--bg); color: var(--text); font-size: 14px; resize: vertical; box-sizing: border-box; }

        .chart-container { padding: 20px 0; }
        .chart-bars { display: flex; align-items: flex-end; gap: 12px; height: 200px; padding: 0 8px; }
        .chart-bar-group { flex: 1; display: flex; flex-direction: column; align-items: center; gap: 6px; }
        .chart-bar-value { font-size: 11px; color: var(--text-secondary); font-weight: 500; }
        .chart-bar-wrapper { display: flex; gap: 3px; align-items: flex-end; height: 160px; }
        .chart-bar { width: 16px; border-radius: 3px 3px 0 0; min-height: 2px; }
        .chart-bar-pv { background: var(--brand); }
        .chart-bar-uv { background: rgba(212,168,67,0.3); }
        .chart-bar-label { font-size: 11px; color: var(--text-tertiary); }
        .chart-legend { display: flex; gap: 16px; justify-content: center; margin-top: 12px; }
        .chart-legend-item { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--text-secondary); }
        .chart-legend-dot { width: 10px; height: 10px; border-radius: 2px; }
    </style>
</head>
<body>
    <div class="sidebar">
        <div class="mb-8 px-3">
            <h2 class="text-lg font-bold">管理后台</h2>
            <p class="text-xs mt-1" style="color: var(--text-secondary);"><?php echo h(SITE_NAME); ?></p>
        </div>
        <nav>
            <button class="nav-link active" data-page="index" onclick="loadPage('index', this)">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"/></svg>
                仪表盘
            </button>
            <button class="nav-link" data-page="announcements" onclick="loadPage('announcements', this)">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"/></svg>
                公告管理
            </button>
            <button class="nav-link" data-page="changelog" onclick="loadPage('changelog', this)">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-3 7h3m-3 4h3m-6-4h.01M9 16h.01"/></svg>
                更新日志
            </button>
            <button class="nav-link" data-page="contacts" onclick="loadPage('contacts', this)">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/></svg>
                联系消息
                <?php if ($unreadCount > 0): ?><span class="badge badge-unread"><?php echo $unreadCount; ?></span><?php endif; ?>
            </button>
            <button class="nav-link" data-page="versions" onclick="loadPage('versions', this)">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z"/></svg>
                版本管理
            </button>
            <button class="nav-link" data-page="stats" onclick="loadPage('stats', this)">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"/></svg>
                访客统计
            </button>
            <button class="nav-link" data-page="settings" onclick="loadPage('settings', this)">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/></svg>
                系统设置
            </button>
        </nav>
        <div class="absolute bottom-6 left-4 right-4">
            <a href="logout.php" class="nav-link">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"/></svg>
                退出登录
            </a>
            <a href="../index.php" target="_blank" class="nav-link mt-2">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14"/></svg>
                打开网站
            </a>
        </div>
    </div>

    <div class="page-loading" id="pageLoader">
        <div class="page-loading-spinner"></div>
    </div>

    <div class="main-content" id="mainContent">
        <?php include 'index.php'; ?>
    </div>

    <script>
    var currentPage = 'index';
    var isLoading = false;

    function loadPage(page, el) {
        if (isLoading || page === currentPage) return;
        isLoading = true;

        document.querySelectorAll('.nav-link').forEach(function(n) {
            n.classList.remove('active', 'loading');
        });
        if (el) el.classList.add('active', 'loading');

        document.getElementById('pageLoader').classList.add('show');

        var xhr = new XMLHttpRequest();
        xhr.open('GET', 'layout.php?page=' + encodeURIComponent(page), true);
        xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
        xhr.onreadystatechange = function() {
            if (xhr.readyState === 4) {
                isLoading = false;
                document.getElementById('pageLoader').classList.remove('show');
                document.querySelectorAll('.nav-link').forEach(function(n) {
                    n.classList.remove('loading');
                });

                if (xhr.status === 200) {
                    document.getElementById('mainContent').innerHTML = xhr.responseText;
                    currentPage = page;
                    window.scrollTo(0, 0);

                    var scripts = document.getElementById('mainContent').querySelectorAll('script');
                    scripts.forEach(function(oldScript) {
                        var newScript = document.createElement('script');
                        if (oldScript.src) {
                            newScript.src = oldScript.src;
                        } else {
                            newScript.textContent = oldScript.textContent;
                        }
                        oldScript.parentNode.replaceChild(newScript, oldScript);
                    });
                } else {
                    alert('加载失败，请刷新页面重试');
                }
            }
        };
        xhr.send();
    }

    document.addEventListener('DOMContentLoaded', function() {
        var hash = window.location.hash.replace('#', '');
        if (hash) {
            var btn = document.querySelector('.nav-link[data-page="' + hash + '"]');
            if (btn) loadPage(hash, btn);
        }
    });
    </script>
</body>
</html>
