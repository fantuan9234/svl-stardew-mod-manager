<?php
require_once __DIR__ . '/../backend/auth.php';
require_once __DIR__ . '/../backend/db.php';
require_once __DIR__ . '/../backend/security.php';
requireLogin();
initDatabase();

$db = getDB();
$message = '';
$error = '';

if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    requireCsrf();
    $postAction = $_POST['post_action'] ?? '';

    if ($postAction === 'change_password') {
        $currentPassword = $_POST['current_password'] ?? '';
        $newPassword = $_POST['new_password'] ?? '';
        $confirmPassword = $_POST['confirm_password'] ?? '';

        if ($currentPassword === '' || $newPassword === '' || $confirmPassword === '') {
            $error = '所有密码字段都必须填写';
        } elseif (!password_verify($currentPassword, getAdminPasswordHash())) {
            $error = '当前密码不正确';
        } elseif (strlen($newPassword) < 8) {
            $error = '新密码至少需要 8 个字符';
        } elseif ($newPassword !== $confirmPassword) {
            $error = '两次输入的新密码不一致';
        } else {
            updateAdminPassword($newPassword);
            $message = '密码已成功修改，下次登录时生效';
        }
    } elseif ($postAction === 'backup') {
        $result = backupDatabase();
        if ($result['success']) {
            $message = '备份成功：' . $result['file'] . '（' . round($result['size'] / 1024, 1) . ' KB）';
        } else {
            $error = $result['error'];
        }
    }
}

$backups = getBackups();
?>
<div class="main-content">
    <?php echo csrfField(); ?>
    <h1 class="text-2xl font-bold mb-8">系统设置</h1>

    <?php if ($message): ?><div class="msg-success"><?php echo h($message); ?></div><?php endif; ?>
    <?php if ($error): ?><div class="msg-error"><?php echo h($error); ?></div><?php endif; ?>

    <div class="form-card">
        <h2 class="text-lg font-semibold mb-6">修改管理员密码</h2>
        <form id="passwordForm" onsubmit="return settingsSubmit(event)">
            <?php echo csrfField(); ?>
            <input type="hidden" name="post_action" value="change_password">
            <div class="mb-4">
                <label class="label">当前密码</label>
                <input type="password" name="current_password" class="input-field" required autocomplete="current-password">
            </div>
            <div class="mb-4">
                <label class="label">新密码（至少 8 个字符）</label>
                <input type="password" name="new_password" class="input-field" required minlength="8" autocomplete="new-password">
            </div>
            <div class="mb-6">
                <label class="label">确认新密码</label>
                <input type="password" name="confirm_password" class="input-field" required minlength="8" autocomplete="new-password">
            </div>
            <button type="submit" class="btn btn-primary">修改密码</button>
        </form>
    </div>

    <div class="form-card">
        <h2 class="text-lg font-semibold mb-6">数据库备份</h2>
        <p class="text-sm mb-4" style="color: var(--text-secondary);">创建当前数据库的备份文件，自动保留最近 10 个备份。</p>
        <button type="button" onclick="settingsBackup()" class="btn btn-primary">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 10v6m0 0l-3-3m3 3l3-3m2 8H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/></svg>
            创建备份
        </button>

        <?php if (!empty($backups)): ?>
        <div class="mt-6">
            <h3 class="text-sm font-medium mb-3" style="color: var(--text-secondary);">历史备份（<?php echo count($backups); ?> 个）</h3>
            <?php foreach ($backups as $b): ?>
            <div class="backup-item">
                <div>
                    <span class="text-sm"><?php echo h($b['name']); ?></span>
                    <span class="text-xs ml-3" style="color: var(--text-tertiary);"><?php echo round($b['size'] / 1024, 1); ?> KB</span>
                </div>
                <span class="text-xs" style="color: var(--text-tertiary);"><?php echo date('Y-m-d H:i:s', $b['time']); ?></span>
            </div>
            <?php endforeach; ?>
        </div>
        <?php endif; ?>
    </div>
</div>

<script>
function settingsSubmit(e) {
    e.preventDefault();
    var form = e.target;
    var formData = new FormData(form);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=settings', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                document.getElementById('mainContent').innerHTML = xhr.responseText;
                rebindScripts();
            } else if (xhr.status === 403) {
                alert('CSRF 验证失败，请刷新页面后重试');
            } else {
                alert('操作失败，请刷新页面后重试');
            }
        }
    };
    xhr.send(formData);
    return false;
}

function settingsBackup() {
    if (!confirm('确定要创建新的数据库备份吗？')) return;
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) { alert('页面已过期，请刷新后重试'); return; }
    var formData = new FormData();
    formData.append('post_action', 'backup');
    formData.append('csrf_token', csrfToken.value);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=settings', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4 && xhr.status === 200) {
            document.getElementById('mainContent').innerHTML = xhr.responseText;
            rebindScripts();
        }
    };
    xhr.send(formData);
}

function rebindScripts() {
    var scripts = document.getElementById('mainContent').querySelectorAll('script');
    scripts.forEach(function(oldScript) {
        var newScript = document.createElement('script');
        if (oldScript.src) { newScript.src = oldScript.src; }
        else { newScript.textContent = oldScript.textContent; }
        oldScript.parentNode.replaceChild(newScript, oldScript);
    });
}
</script>
