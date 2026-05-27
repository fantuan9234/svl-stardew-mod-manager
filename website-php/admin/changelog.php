<?php
require_once __DIR__ . '/../backend/auth.php';
require_once __DIR__ . '/../backend/db.php';
requireLogin();
initDatabase();

$db = getDB();
$currentPage = basename($_SERVER['PHP_SELF']);
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
            $title = trim($_POST['title'] ?? '');
            if ($version === '' || $title === '') {
                $error = '版本号和标题不能为空';
            } else {
                $release_type = trim($_POST['release_type'] ?? 'update');
                $release_date = trim($_POST['release_date'] ?? date('Y-m-d'));
                $changes = [];
                $changeTexts = $_POST['change_text'] ?? [];
                $changeTypes = $_POST['change_type'] ?? [];
                for ($i = 0; $i < count($changeTexts); $i++) {
                    $text = trim($changeTexts[$i] ?? '');
                    if ($text !== '') {
                        $changes[] = ['type' => $changeTypes[$i] ?? 'new', 'text' => $text];
                    }
                }
                $stmt = $db->prepare("INSERT INTO changelog (version, release_type, title, changes, release_date) VALUES (?, ?, ?, ?, ?)");
                $stmt->execute([$version, $release_type, $title, json_encode($changes, JSON_UNESCAPED_UNICODE), $release_date]);
                $message = '版本记录已添加';
                $action = 'list';
            }
        } elseif ($postAction === 'update') {
            $updateId = (int)($_POST['id'] ?? 0);
            $version = trim($_POST['version'] ?? '');
            $title = trim($_POST['title'] ?? '');
            if ($version === '' || $title === '' || $updateId <= 0) {
                $error = '版本号和标题不能为空';
            } else {
                $release_type = trim($_POST['release_type'] ?? 'update');
                $release_date = trim($_POST['release_date'] ?? date('Y-m-d'));
                $changes = [];
                $changeTexts = $_POST['change_text'] ?? [];
                $changeTypes = $_POST['change_type'] ?? [];
                for ($i = 0; $i < count($changeTexts); $i++) {
                    $text = trim($changeTexts[$i] ?? '');
                    if ($text !== '') {
                        $changes[] = ['type' => $changeTypes[$i] ?? 'new', 'text' => $text];
                    }
                }
                $stmt = $db->prepare("UPDATE changelog SET version=?, release_type=?, title=?, changes=?, release_date=?, updated_at=datetime('now','localtime') WHERE id=?");
                $stmt->execute([$version, $release_type, $title, json_encode($changes, JSON_UNESCAPED_UNICODE), $release_date, $updateId]);
                $message = '版本记录已更新';
                $action = 'list';
            }
        } elseif ($postAction === 'delete') {
            $deleteId = (int)($_POST['id'] ?? 0);
            if ($deleteId > 0) {
                $stmt = $db->prepare("DELETE FROM changelog WHERE id=?");
                $stmt->execute([$deleteId]);
                $message = '版本记录已删除';
            }
            $action = 'list';
        }
    } catch (Exception $e) {
        $error = '操作失败：' . $e->getMessage();
    }
}

$releaseTypes = ['update' => '更新', 'fix' => '修复', 'release' => '发布', 'other' => '其他'];
$changeTypes = ['new' => '新增', 'fix' => '修复', 'improve' => '优化'];

$editItem = null;
$editChanges = [];
if ($action === 'edit' && $id > 0) {
    $stmt = $db->prepare("SELECT * FROM changelog WHERE id=?");
    $stmt->execute([$id]);
    $editItem = $stmt->fetch();
    if (!$editItem) {
        $action = 'list';
        $error = '记录不存在';
    } else {
        $editChanges = json_decode($editItem['changes'], true) ?: [];
    }
}

if ($action === 'new') {
    $editItem = null;
    $editChanges = [['type' => 'new', 'text' => '']];
}

$items = $db->query("SELECT * FROM changelog ORDER BY release_date DESC, id DESC")->fetchAll();
?>
<div class="main-content">
    <?php echo csrfField(); ?>
    <div class="flex items-center justify-between mb-8">
        <h1 class="text-2xl font-bold">更新日志管理</h1>
        <button type="button" onclick="changelogNav('new')" class="btn btn-primary">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/></svg>
            添加版本
        </button>
    </div>

    <div id="changelogMsg">
    <?php if ($message): ?><div class="msg-success" style="padding:12px 16px; border-radius:8px; background:rgba(34,197,94,0.1); color:#22c55e; margin-bottom:16px; font-size:14px;"><?php echo h($message); ?></div><?php endif; ?>
    <?php if ($error): ?><div class="msg-error" style="padding:12px 16px; border-radius:8px; background:rgba(239,68,68,0.1); color:#ef4444; margin-bottom:16px; font-size:14px;"><?php echo h($error); ?></div><?php endif; ?>
    </div>

    <?php if ($action === 'new' || $action === 'edit'): ?>
    <div class="form-card" style="background: var(--surface); border: 1px solid var(--border); border-radius: 14px; padding: 24px;">
        <h2 class="text-lg font-semibold mb-6"><?php echo $action === 'edit' ? '编辑版本记录' : '添加新版本'; ?></h2>
        <form method="post" id="changelogForm" onsubmit="return changelogSubmit(this);">
            <?php echo csrfField(); ?>
            <input type="hidden" name="post_action" value="<?php echo $action === 'edit' ? 'update' : 'create'; ?>">
            <?php if ($action === 'edit'): ?>
            <input type="hidden" name="id" value="<?php echo $editItem['id']; ?>">
            <?php endif; ?>
            <div class="grid grid-cols-3 gap-4 mb-4">
                <div>
                    <label class="label" style="display:block; font-size:13px; font-weight:500; color:var(--text-secondary); margin-bottom:6px;">版本号</label>
                    <input type="text" name="version" class="input-field" style="width:100%; padding:8px 12px; border-radius:8px; border:1px solid var(--border); background:var(--bg); color:var(--text); font-size:14px;" required placeholder="v1.0.3" value="<?php echo h($editItem['version'] ?? ''); ?>">
                </div>
                <div>
                    <label class="label" style="display:block; font-size:13px; font-weight:500; color:var(--text-secondary); margin-bottom:6px;">类型</label>
                    <select name="release_type" class="input-field" style="width:100%; padding:8px 12px; border-radius:8px; border:1px solid var(--border); background:var(--bg); color:var(--text); font-size:14px;">
                        <?php foreach ($releaseTypes as $val => $label): ?>
                        <option value="<?php echo h($val); ?>" <?php echo ($editItem['release_type'] ?? 'update') === $val ? 'selected' : ''; ?>><?php echo h($label); ?></option>
                        <?php endforeach; ?>
                    </select>
                </div>
                <div>
                    <label class="label" style="display:block; font-size:13px; font-weight:500; color:var(--text-secondary); margin-bottom:6px;">发布日期</label>
                    <input type="date" name="release_date" class="input-field" style="width:100%; padding:8px 12px; border-radius:8px; border:1px solid var(--border); background:var(--bg); color:var(--text); font-size:14px;" value="<?php echo h($editItem['release_date'] ?? date('Y-m-d')); ?>">
                </div>
            </div>
            <div class="mb-4">
                <label class="label" style="display:block; font-size:13px; font-weight:500; color:var(--text-secondary); margin-bottom:6px;">标题</label>
                <input type="text" name="title" class="input-field" style="width:100%; padding:8px 12px; border-radius:8px; border:1px solid var(--border); background:var(--bg); color:var(--text); font-size:14px;" required placeholder="版本标题" value="<?php echo h($editItem['title'] ?? ''); ?>">
            </div>
            <div class="mb-6">
                <div class="flex items-center justify-between mb-3">
                    <label class="label" style="display:block; font-size:13px; font-weight:500; color:var(--text-secondary);">变更列表</label>
                    <button type="button" onclick="addChangeRow()" class="btn btn-ghost btn-sm" style="padding:4px 12px; border-radius:6px; border:1px solid var(--border); background:transparent; color:var(--brand); cursor:pointer; font-size:12px;">+ 添加条目</button>
                </div>
                <div id="changesContainer">
                    <?php if (!empty($editChanges)): ?>
                        <?php foreach ($editChanges as $i => $change): ?>
                        <div class="change-row flex gap-2 mb-2 items-center">
                            <select name="change_type[]" style="padding:6px 8px; border-radius:6px; border:1px solid var(--border); background:var(--bg); color:var(--text); font-size:13px; width:80px; flex-shrink:0;">
                                <?php foreach ($changeTypes as $val => $label): ?>
                                <option value="<?php echo h($val); ?>" <?php echo $change['type'] === $val ? 'selected' : ''; ?>><?php echo h($label); ?></option>
                                <?php endforeach; ?>
                            </select>
                            <input type="text" name="change_text[]" style="flex:1; padding:6px 12px; border-radius:6px; border:1px solid var(--border); background:var(--bg); color:var(--text); font-size:13px;" value="<?php echo h($change['text']); ?>" placeholder="变更描述...">
                            <button type="button" onclick="this.parentElement.remove()" style="padding:6px; border-radius:6px; border:1px solid var(--border); background:transparent; color:#ef4444; cursor:pointer; font-size:13px; flex-shrink:0;">✕</button>
                        </div>
                        <?php endforeach; ?>
                    <?php else: ?>
                    <div class="change-row flex gap-2 mb-2 items-center">
                        <select name="change_type[]" style="padding:6px 8px; border-radius:6px; border:1px solid var(--border); background:var(--bg); color:var(--text); font-size:13px; width:80px; flex-shrink:0;">
                            <?php foreach ($changeTypes as $val => $label): ?>
                            <option value="<?php echo h($val); ?>"><?php echo h($label); ?></option>
                            <?php endforeach; ?>
                        </select>
                        <input type="text" name="change_text[]" style="flex:1; padding:6px 12px; border-radius:6px; border:1px solid var(--border); background:var(--bg); color:var(--text); font-size:13px;" placeholder="变更描述...">
                        <button type="button" onclick="this.parentElement.remove()" style="padding:6px; border-radius:6px; border:1px solid var(--border); background:transparent; color:#ef4444; cursor:pointer; font-size:13px; flex-shrink:0;">✕</button>
                    </div>
                    <?php endif; ?>
                </div>
            </div>
            <div class="flex gap-3">
                <button type="submit" class="btn btn-primary" style="padding:8px 20px; border-radius:8px; background:var(--brand); color:#000; font-weight:600; border:none; cursor:pointer; font-size:14px;"><?php echo $action === 'edit' ? '保存修改' : '添加版本'; ?></button>
                <button type="button" onclick="changelogNav('list')" class="btn btn-ghost" style="padding:8px 20px; border-radius:8px; border:1px solid var(--border); background:transparent; color:var(--text); cursor:pointer; font-size:14px;">取消</button>
            </div>
        </form>
    </div>
    <?php endif; ?>

    <?php if ($action === 'list'): ?>
    <div class="table-wrapper">
        <table>
            <thead>
                <tr>
                    <th>版本号</th>
                    <th>类型</th>
                    <th>标题</th>
                    <th>变更数</th>
                    <th>日期</th>
                    <th>操作</th>
                </tr>
            </thead>
            <tbody>
                <?php if (empty($items)): ?>
                <tr><td colspan="6" style="text-align:center; color: var(--text-secondary); padding: 40px;">暂无更新日志</td></tr>
                <?php else: ?>
                <?php foreach ($items as $item): ?>
                <?php $changes = json_decode($item['changes'], true) ?: []; ?>
                <tr>
                    <td><strong><?php echo h($item['version']); ?></strong></td>
                    <td><span class="badge badge-cat" style="background:rgba(212,168,67,0.15); color:var(--brand); padding:3px 10px; border-radius:20px; font-size:11px;"><?php echo h($releaseTypes[$item['release_type']] ?? $item['release_type']); ?></span></td>
                    <td><?php echo h($item['title']); ?></td>
                    <td style="color: var(--text-secondary);"><?php echo count($changes); ?> 项</td>
                    <td style="color: var(--text-secondary); font-size: 13px;"><?php echo h($item['release_date']); ?></td>
                    <td>
                        <div class="flex gap-2">
                            <button type="button" onclick="changelogNav('edit', <?php echo $item['id']; ?>)" class="btn btn-ghost btn-sm" style="padding:4px 12px; border-radius:6px; border:1px solid var(--border); background:transparent; color:var(--text); cursor:pointer; font-size:12px;">编辑</button>
                            <button type="button" onclick="changelogDelete(this, <?php echo $item['id']; ?>)" class="btn btn-danger btn-sm" style="padding:4px 12px; border-radius:6px; border:1px solid rgba(239,68,68,0.3); background:transparent; color:#ef4444; cursor:pointer; font-size:12px;">删除</button>
                        </div>
                    </td>
                </tr>
                <?php endforeach; ?>
                <?php endif; ?>
            </tbody>
        </table>
    </div>
    <?php endif; ?>
</div>

<script>
function addChangeRow() {
    var container = document.getElementById('changesContainer');
    var row = document.createElement('div');
    row.className = 'change-row flex gap-2 mb-2 items-center';
    row.innerHTML = '<select name="change_type[]" style="padding:6px 8px; border-radius:6px; border:1px solid var(--border); background:var(--bg); color:var(--text); font-size:13px; width:80px; flex-shrink:0;"><option value="new">新增</option><option value="fix">修复</option><option value="improve">优化</option></select><input type="text" name="change_text[]" style="flex:1; padding:6px 12px; border-radius:6px; border:1px solid var(--border); background:var(--bg); color:var(--text); font-size:13px;" placeholder="变更描述..."><button type="button" onclick="this.parentElement.remove()" style="padding:6px; border-radius:6px; border:1px solid var(--border); background:transparent; color:#ef4444; cursor:pointer; font-size:13px; flex-shrink:0;">✕</button>';
    container.appendChild(row);
    row.querySelector('input').focus();
}

function changelogNav(action, id) {
    var url = 'layout.php?page=changelog';
    if (action && action !== 'list') {
        url += '&action=' + encodeURIComponent(action);
    }
    if (id) {
        url += '&id=' + parseInt(id);
    }
    var xhr = new XMLHttpRequest();
    xhr.open('GET', url, true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4 && xhr.status === 200) {
            document.getElementById('mainContent').innerHTML = xhr.responseText;
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
    };
    xhr.send();
}

function changelogSubmit(form) {
    var formData = new FormData(form);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=changelog', true);
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

function changelogDelete(btn, id) {
    if (!confirm('确定要删除这条版本记录吗？')) return;
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) {
        alert('页面已过期，请刷新后重试');
        return;
    }
    var formData = new FormData();
    formData.append('post_action', 'delete');
    formData.append('id', id);
    formData.append('csrf_token', csrfToken.value);

    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=changelog', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                document.getElementById('mainContent').innerHTML = xhr.responseText;
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
            } else if (xhr.status === 403) {
                alert('CSRF 验证失败，请刷新页面后重试');
            } else {
                alert('删除失败，请刷新页面后重试');
            }
        }
    };
    xhr.send(formData);
}
</script>
