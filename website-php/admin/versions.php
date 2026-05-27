<?php
require_once __DIR__ . '/../backend/auth.php';
require_once __DIR__ . '/../backend/db.php';
requireLogin();
initDatabase();

$db = getDB();
$message = '';
$error = '';

$action = $_GET['action'] ?? 'list';
$id = isset($_GET['id']) ? (int)$_GET['id'] : 0;

if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    requireCsrf();
    try {
        $postAction = $_POST['post_action'] ?? '';

        if ($postAction === 'create') {
            $version = trim($_POST['version'] ?? '');
            if ($version === '') {
                $error = '版本号不能为空';
            } else {
                $changelog = trim($_POST['changelog'] ?? '');
                $download_url = trim($_POST['download_url'] ?? '');
                $platform = trim($_POST['platform'] ?? 'windows');
                $is_latest = isset($_POST['is_latest']) ? 1 : 0;
                if ($is_latest) {
                    $db->prepare("UPDATE versions SET is_latest=0 WHERE platform=?")->execute([$platform]);
                }
                $stmt = $db->prepare("INSERT INTO versions (version, changelog, download_url, platform, is_latest) VALUES (?, ?, ?, ?, ?)");
                $stmt->execute([$version, $changelog, $download_url, $platform, $is_latest]);
                $message = '版本已创建';
                $action = 'list';
            }
        } elseif ($postAction === 'update') {
            $updateId = (int)($_POST['id'] ?? 0);
            $version = trim($_POST['version'] ?? '');
            if ($version === '' || $updateId <= 0) {
                $error = '版本号不能为空';
            } else {
                $changelog = trim($_POST['changelog'] ?? '');
                $download_url = trim($_POST['download_url'] ?? '');
                $platform = trim($_POST['platform'] ?? 'windows');
                $is_latest = isset($_POST['is_latest']) ? 1 : 0;
                if ($is_latest) {
                    $db->prepare("UPDATE versions SET is_latest=0 WHERE platform=?")->execute([$platform]);
                }
                $stmt = $db->prepare("UPDATE versions SET version=?, changelog=?, download_url=?, platform=?, is_latest=? WHERE id=?");
                $stmt->execute([$version, $changelog, $download_url, $platform, $is_latest, $updateId]);
                $message = '版本已更新';
                $action = 'list';
            }
        } elseif ($postAction === 'delete') {
            $deleteId = (int)($_POST['id'] ?? 0);
            if ($deleteId > 0) {
                $stmt = $db->prepare("DELETE FROM versions WHERE id=?");
                $stmt->execute([$deleteId]);
                $message = '版本已删除';
            }
            $action = 'list';
        } elseif ($postAction === 'toggle_latest') {
            $latestId = (int)($_POST['id'] ?? 0);
            if ($latestId > 0) {
                $row = $db->prepare("SELECT is_latest, platform FROM versions WHERE id=?");
                $row->execute([$latestId]);
                $row = $row->fetch();
                if ($row) {
                    $newLatest = $row['is_latest'] ? 0 : 1;
                    if ($newLatest) {
                        $db->prepare("UPDATE versions SET is_latest=0 WHERE platform=?")->execute([$row['platform']]);
                    }
                    $db->prepare("UPDATE versions SET is_latest=? WHERE id=?")->execute([$newLatest, $latestId]);
                    $message = $newLatest ? '已设为最新版本' : '已取消最新版本';
                }
            }
            $action = 'list';
        }
    } catch (Exception $e) {
        $error = '操作失败：' . $e->getMessage();
    }
}

$platforms = ['windows', 'macos', 'linux'];

$editItem = null;
if ($action === 'edit' && $id > 0) {
    $stmt = $db->prepare("SELECT * FROM versions WHERE id=?");
    $stmt->execute([$id]);
    $editItem = $stmt->fetch();
    if (!$editItem) {
        $action = 'list';
        $error = '版本不存在';
    }
}

if ($action === 'new') {
    $editItem = null;
}

$items = $db->query("SELECT * FROM versions ORDER BY is_latest DESC, created_at DESC")->fetchAll();
?>
<div class="main-content">
    <?php echo csrfField(); ?>
    <div class="flex items-center justify-between mb-8">
        <h1 class="text-2xl font-bold">版本管理</h1>
        <button type="button" onclick="versionNav('new')" class="btn btn-primary">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/></svg>
            新建版本
        </button>
    </div>

    <?php if ($message): ?><div class="msg-success"><?php echo h($message); ?></div><?php endif; ?>
    <?php if ($error): ?><div class="msg-error"><?php echo h($error); ?></div><?php endif; ?>

    <?php if ($action === 'new' || $action === 'edit'): ?>
    <div class="form-card">
        <h2 class="text-lg font-semibold mb-6"><?php echo $action === 'edit' ? '编辑版本' : '新建版本'; ?></h2>
        <form id="versionForm" onsubmit="return versionSubmit(event)">
            <?php echo csrfField(); ?>
            <input type="hidden" name="post_action" value="<?php echo $action === 'edit' ? 'update' : 'create'; ?>">
            <?php if ($action === 'edit'): ?>
            <input type="hidden" name="id" value="<?php echo $editItem['id']; ?>">
            <?php endif; ?>
            <div class="grid grid-cols-2 gap-4 mb-4">
                <div>
                    <label class="label">版本号</label>
                    <input type="text" name="version" class="input-field" required value="<?php echo h($editItem['version'] ?? ''); ?>" placeholder="例如：v2.1.0">
                </div>
                <div>
                    <label class="label">平台</label>
                    <select name="platform" class="input-field">
                        <?php foreach ($platforms as $p): ?>
                        <option value="<?php echo h($p); ?>" <?php echo ($editItem['platform'] ?? 'windows') === $p ? 'selected' : ''; ?>><?php echo h($p); ?></option>
                        <?php endforeach; ?>
                    </select>
                </div>
            </div>
            <div class="mb-4">
                <label class="label">下载链接</label>
                <input type="text" name="download_url" class="input-field" value="<?php echo h($editItem['download_url'] ?? ''); ?>" placeholder="https://...">
            </div>
            <div class="mb-4">
                <label class="label">更新日志</label>
                <textarea name="changelog" class="input-field" rows="4"><?php echo h($editItem['changelog'] ?? ''); ?></textarea>
            </div>
            <div class="mb-6">
                <label class="checkbox-label">
                    <input type="checkbox" name="is_latest" value="1" <?php echo ($editItem['is_latest'] ?? 0) ? 'checked' : ''; ?> style="accent-color: var(--brand);">
                    设为该平台最新版本
                </label>
            </div>
            <div class="flex gap-3">
                <button type="submit" class="btn btn-primary"><?php echo $action === 'edit' ? '保存修改' : '创建版本'; ?></button>
                <button type="button" onclick="versionNav('list')" class="btn btn-ghost">取消</button>
            </div>
        </form>
    </div>
    <?php endif; ?>

    <div class="table-wrapper">
        <table>
            <thead>
                <tr>
                    <th>版本号</th>
                    <th>平台</th>
                    <th>最新</th>
                    <th>下载链接</th>
                    <th>日期</th>
                    <th>操作</th>
                </tr>
            </thead>
            <tbody>
                <?php if (empty($items)): ?>
                <tr><td colspan="6" style="text-align:center; color: var(--text-secondary); padding: 40px;">暂无版本</td></tr>
                <?php else: ?>
                <?php foreach ($items as $item): ?>
                <tr>
                    <td class="font-medium"><?php echo h($item['version']); ?></td>
                    <td><span class="badge badge-platform-<?php echo h($item['platform']); ?>"><?php echo h($item['platform']); ?></span></td>
                    <td>
                        <button type="button" onclick="versionToggleLatest(<?php echo $item['id']; ?>)" class="btn btn-sm <?php echo $item['is_latest'] ? 'btn-primary' : 'btn-ghost'; ?>" title="<?php echo $item['is_latest'] ? '取消最新' : '设为最新'; ?>">
                            <svg class="w-3.5 h-3.5" fill="<?php echo $item['is_latest'] ? 'currentColor' : 'none'; ?>" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11.049 2.927c.3-.921 1.603-.921 1.902 0l1.519 4.674a1 1 0 00.95.69h4.915c.969 0 1.371 1.24.588 1.81l-3.976 2.888a1 1 0 00-.363 1.118l1.518 4.674c.3.922-.755 1.688-1.538 1.118l-3.976-2.888a1 1 0 00-1.176 0l-3.976 2.888c-.783.57-1.838-.197-1.538-1.118l1.518-4.674a1 1 0 00-.363-1.118l-3.976-2.888c-.784-.57-.38-1.81.588-1.81h4.914a1 1 0 00.951-.69l1.519-4.674z"/></svg>
                        </button>
                    </td>
                    <td style="max-width:200px; overflow:hidden; text-overflow:ellipsis; white-space:nowrap; color: var(--text-secondary); font-size: 13px;"><?php echo h($item['download_url']); ?></td>
                    <td style="color: var(--text-secondary); font-size: 13px;"><?php echo h($item['created_at']); ?></td>
                    <td>
                        <div class="flex gap-2">
                            <button type="button" onclick="versionNav('edit', <?php echo $item['id']; ?>)" class="btn btn-ghost btn-sm">编辑</button>
                            <button type="button" onclick="versionDelete(<?php echo $item['id']; ?>)" class="btn btn-danger btn-sm">删除</button>
                        </div>
                    </td>
                </tr>
                <?php endforeach; ?>
                <?php endif; ?>
            </tbody>
        </table>
    </div>
</div>

<script>
function versionNav(action, id) {
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    var params = 'action=' + encodeURIComponent(action);
    if (id) params += '&id=' + encodeURIComponent(id);

    var xhr = new XMLHttpRequest();
    xhr.open('GET', 'layout.php?page=versions&' + params, true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4 && xhr.status === 200) {
            document.getElementById('mainContent').innerHTML = xhr.responseText;
            rebindScripts();
        }
    };
    xhr.send();
}

function versionSubmit(e) {
    e.preventDefault();
    var form = e.target;
    var formData = new FormData(form);

    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=versions', true);
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

function versionToggleLatest(id) {
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) { alert('页面已过期，请刷新后重试'); return; }
    var formData = new FormData();
    formData.append('post_action', 'toggle_latest');
    formData.append('id', id);
    formData.append('csrf_token', csrfToken.value);

    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=versions', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4 && xhr.status === 200) {
            document.getElementById('mainContent').innerHTML = xhr.responseText;
            rebindScripts();
        }
    };
    xhr.send(formData);
}

function versionDelete(id) {
    if (!confirm('确定要删除此版本吗？')) return;
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) { alert('页面已过期，请刷新后重试'); return; }
    var formData = new FormData();
    formData.append('post_action', 'delete');
    formData.append('id', id);
    formData.append('csrf_token', csrfToken.value);

    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=versions', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                document.getElementById('mainContent').innerHTML = xhr.responseText;
                rebindScripts();
            } else if (xhr.status === 403) {
                alert('CSRF 验证失败，请刷新页面后重试');
            } else {
                alert('删除失败，请刷新页面后重试');
            }
        }
    };
    xhr.send(formData);
}

function rebindScripts() {
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
}
</script>
