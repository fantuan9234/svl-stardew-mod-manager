<?php
require_once __DIR__ . '/../backend/auth.php';
require_once __DIR__ . '/../backend/db.php';
requireLogin();
initDatabase();

$db = getDB();
$message = '';
$error = '';

$uploadDir = __DIR__ . '/../uploads/announcements';
if (!is_dir($uploadDir)) {
    @mkdir($uploadDir, 0777, true);
}
if (!is_dir($uploadDir)) {
    $error = '无法创建上传目录: ' . $uploadDir . '，请手动创建并设置权限';
}

$action = $_GET['action'] ?? 'list';
$id = isset($_GET['id']) ? (int)$_GET['id'] : 0;

if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    requireCsrf();
    try {
        $postAction = $_POST['post_action'] ?? '';

        if ($postAction === 'create') {
            $title = trim($_POST['title'] ?? '');
            if ($title === '') {
                $error = '标题不能为空';
            } else {
                $category = trim($_POST['category'] ?? '更新');
                $content = trim($_POST['content'] ?? '');
                $image_url = trim($_POST['image_url'] ?? '');
                $is_pinned = isset($_POST['is_pinned']) ? 1 : 0;
                $stmt = $db->prepare("INSERT INTO announcements (title, category, content, image_url, is_pinned) VALUES (?, ?, ?, ?, ?)");
                $stmt->execute([$title, $category, $content, $image_url, $is_pinned]);
                $message = '公告已发布';
                $action = 'list';
            }
        } elseif ($postAction === 'update') {
            $updateId = (int)($_POST['id'] ?? 0);
            $title = trim($_POST['title'] ?? '');
            if ($title === '' || $updateId <= 0) {
                $error = '标题不能为空';
            } else {
                $category = trim($_POST['category'] ?? '更新');
                $content = trim($_POST['content'] ?? '');
                $image_url = trim($_POST['image_url'] ?? '');
                $is_pinned = isset($_POST['is_pinned']) ? 1 : 0;
                $stmt = $db->prepare("UPDATE announcements SET title=?, category=?, content=?, image_url=?, is_pinned=?, updated_at=datetime('now','localtime') WHERE id=?");
                $stmt->execute([$title, $category, $content, $image_url, $is_pinned, $updateId]);
                $message = '公告已更新';
                $action = 'list';
            }
        } elseif ($postAction === 'delete') {
            $deleteId = (int)($_POST['id'] ?? 0);
            if ($deleteId > 0) {
                $stmt = $db->prepare("DELETE FROM announcements WHERE id=?");
                $stmt->execute([$deleteId]);
                $message = '公告已删除';
            }
            $action = 'list';
        } elseif ($postAction === 'toggle_pin') {
            $pinId = (int)($_POST['id'] ?? 0);
            if ($pinId > 0) {
                $row = $db->prepare("SELECT is_pinned FROM announcements WHERE id=?");
                $row->execute([$pinId]);
                $row = $row->fetch();
                if ($row) {
                    $newPin = $row['is_pinned'] ? 0 : 1;
                    $db->prepare("UPDATE announcements SET is_pinned=?, updated_at=datetime('now','localtime') WHERE id=?")->execute([$newPin, $pinId]);
                    $message = $newPin ? '公告已置顶' : '已取消置顶';
                }
            }
            $action = 'list';
        }
    } catch (Exception $e) {
        $error = '操作失败：' . $e->getMessage();
    }
}

$categories = ['更新', '修复', '社区', '说明', '活动', '其他'];

$editItem = null;
if ($action === 'edit' && $id > 0) {
    $stmt = $db->prepare("SELECT * FROM announcements WHERE id=?");
    $stmt->execute([$id]);
    $editItem = $stmt->fetch();
    if (!$editItem) {
        $action = 'list';
        $error = '公告不存在';
    }
}

if ($action === 'new') {
    $editItem = null;
}

$items = $db->query("SELECT * FROM announcements ORDER BY is_pinned DESC, created_at DESC")->fetchAll();
?>
<div class="main-content">
    <?php echo csrfField(); ?>
    <div class="flex items-center justify-between mb-8">
        <h1 class="text-2xl font-bold">公告管理</h1>
        <button type="button" onclick="announceNav('new')" class="btn btn-primary">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/></svg>
            新建公告
        </button>
    </div>

    <?php if ($message): ?><div class="msg-success"><?php echo h($message); ?></div><?php endif; ?>
    <?php if ($error): ?><div class="msg-error"><?php echo h($error); ?></div><?php endif; ?>

    <?php if ($action === 'new' || $action === 'edit'): ?>
    <div class="form-card">
        <h2 class="text-lg font-semibold mb-6"><?php echo $action === 'edit' ? '编辑公告' : '新建公告'; ?></h2>
        <form id="announceForm" onsubmit="return announceSubmit(event)">
            <?php echo csrfField(); ?>
            <input type="hidden" name="post_action" value="<?php echo $action === 'edit' ? 'update' : 'create'; ?>">
            <?php if ($action === 'edit'): ?>
            <input type="hidden" name="id" value="<?php echo $editItem['id']; ?>">
            <?php endif; ?>
            <div class="grid grid-cols-2 gap-4 mb-4">
                <div>
                    <label class="label">标题</label>
                    <input type="text" name="title" class="input-field" required value="<?php echo h($editItem['title'] ?? ''); ?>">
                </div>
                <div>
                    <label class="label">分类</label>
                    <select name="category" class="input-field">
                        <?php foreach ($categories as $cat): ?>
                        <option value="<?php echo h($cat); ?>" <?php echo ($editItem['category'] ?? '更新') === $cat ? 'selected' : ''; ?>><?php echo h($cat); ?></option>
                        <?php endforeach; ?>
                    </select>
                </div>
            </div>
            <div class="mb-4">
                <label class="label">内容</label>
                <textarea name="content" class="input-field" rows="4" placeholder="支持输入链接（自动识别）或使用 [url=链接]文字[/url] 和 [img]图片链接[/img]"><?php echo h($editItem['content'] ?? ''); ?></textarea>
            </div>
            <div class="mb-4">
                <label class="label">公告图片（可选）</label>
                <div class="flex gap-3 items-start">
                    <div class="flex-1">
                        <input type="text" name="image_url" id="imageUrlField" class="input-field" placeholder="图片 URL（上传后自动填入）" value="<?php echo h($editItem['image_url'] ?? ''); ?>">
                    </div>
                    <label class="btn btn-primary btn-sm cursor-pointer whitespace-nowrap" style="padding: 8px 16px; font-size: 13px;">
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"/></svg>
                        上传图片
                        <input type="file" name="image" id="imageUpload" accept="image/jpeg,image/png,image/gif,image/webp" class="hidden" onchange="uploadAnnouncementImage(this)">
                    </label>
                </div>
                <div id="imagePreview" class="mt-3" style="<?php echo empty($editItem['image_url'] ?? '') ? 'display:none;' : ''; ?>">
                    <img id="imagePreviewImg" src="<?php echo h($editItem['image_url'] ?? ''); ?>" alt="预览" style="max-width: 160px; max-height: 100px; object-fit: contain; border-radius: 8px; border: 1px solid var(--border);">
                    <button type="button" onclick="removeAnnouncementImage()" class="btn btn-ghost btn-sm mt-2" style="font-size: 12px; padding: 4px 10px;">移除图片</button>
                </div>
                <div id="imageUploadProgress" class="mt-2" style="display:none;">
                    <div style="background: var(--border); border-radius: 4px; height: 4px; overflow: hidden;">
                        <div id="imageUploadBar" style="background: var(--brand); height: 100%; width: 0%; transition: width 0.3s;"></div>
                    </div>
                    <span class="text-xs" style="color: var(--text-secondary);">上传中...</span>
                </div>
            </div>
            <div class="mb-6">
                <label class="checkbox-label">
                    <input type="checkbox" name="is_pinned" value="1" <?php echo ($editItem['is_pinned'] ?? 0) ? 'checked' : ''; ?> style="accent-color: var(--brand);">
                    置顶此公告
                </label>
            </div>
            <div class="flex gap-3">
                <button type="submit" class="btn btn-primary"><?php echo $action === 'edit' ? '保存修改' : '发布公告'; ?></button>
                <button type="button" onclick="announceNav('list')" class="btn btn-ghost">取消</button>
            </div>
        </form>
    </div>
    <?php endif; ?>

    <div class="table-wrapper">
        <table>
            <thead>
                <tr>
                    <th>标题</th>
                    <th>分类</th>
                    <th>置顶</th>
                    <th>日期</th>
                    <th>操作</th>
                </tr>
            </thead>
            <tbody>
                <?php if (empty($items)): ?>
                <tr><td colspan="5" style="text-align:center; color: var(--text-secondary); padding: 40px;">暂无公告</td></tr>
                <?php else: ?>
                <?php foreach ($items as $item): ?>
                <tr>
                    <td><?php echo h($item['title']); ?></td>
                    <td><span class="badge badge-cat"><?php echo h($item['category']); ?></span></td>
                    <td>
                        <button type="button" onclick="announceTogglePin(<?php echo $item['id']; ?>)" class="btn btn-sm <?php echo $item['is_pinned'] ? 'btn-primary' : 'btn-ghost'; ?>" title="<?php echo $item['is_pinned'] ? '取消置顶' : '置顶'; ?>">
                            <svg class="w-3.5 h-3.5" fill="<?php echo $item['is_pinned'] ? 'currentColor' : 'none'; ?>" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z"/></svg>
                        </button>
                    </td>
                    <td style="color: var(--text-secondary); font-size: 13px;"><?php echo h($item['created_at']); ?></td>
                    <td>
                        <div class="flex gap-2">
                            <button type="button" onclick="announceNav('edit', <?php echo $item['id']; ?>)" class="btn btn-ghost btn-sm">编辑</button>
                            <button type="button" onclick="announceDelete(<?php echo $item['id']; ?>)" class="btn btn-danger btn-sm">删除</button>
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
function uploadAnnouncementImage(input) {
    if (!input.files || !input.files[0]) return;
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) { alert('页面已过期，请刷新后重试'); return; }

    var formData = new FormData();
    formData.append('csrf_token', csrfToken.value);
    formData.append('image', input.files[0]);

    var progress = document.getElementById('imageUploadProgress');
    var bar = document.getElementById('imageUploadBar');
    progress.style.display = 'block';
    bar.style.width = '0%';

    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'upload-image.php', true);
    xhr.upload.onprogress = function(e) {
        if (e.lengthComputable) {
            bar.style.width = Math.round((e.loaded / e.total) * 100) + '%';
        }
    };
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            progress.style.display = 'none';
            if (xhr.status === 200) {
                try {
                    var resp = JSON.parse(xhr.responseText);
                    if (resp.success) {
                        document.getElementById('imageUrlField').value = resp.url;
                        document.getElementById('imagePreviewImg').src = resp.url;
                        document.getElementById('imagePreview').style.display = 'block';
                    } else {
                        alert(resp.error || '上传失败');
                    }
                } catch(e) {
                    var preview = xhr.responseText.substring(0, 500);
                    console.error('Upload response (not JSON):', xhr.responseText.substring(0, 2000));
                    alert('上传失败：服务器返回了非预期内容。\n\n原始响应前500字符：\n' + preview + '\n\n请按 F12 打开控制台查看完整错误信息。');
                }
            } else if (xhr.status === 403) {
                alert('页面已过期，请刷新后重试');
            } else {
                alert('上传失败（HTTP ' + xhr.status + '），请刷新页面后重试');
            }
            input.value = '';
        }
    };
    xhr.send(formData);
}

function removeAnnouncementImage() {
    document.getElementById('imageUrlField').value = '';
    document.getElementById('imagePreviewImg').src = '';
    document.getElementById('imagePreview').style.display = 'none';
}

function announceNav(action, id) {
    var params = 'action=' + encodeURIComponent(action);
    if (id) params += '&id=' + encodeURIComponent(id);
    var xhr = new XMLHttpRequest();
    xhr.open('GET', 'layout.php?page=announcements&' + params, true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4 && xhr.status === 200) {
            document.getElementById('mainContent').innerHTML = xhr.responseText;
            rebindScripts();
        }
    };
    xhr.send();
}

function announceSubmit(e) {
    e.preventDefault();
    var form = e.target;
    var formData = new FormData(form);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=announcements', true);
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

function announceTogglePin(id) {
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) { alert('页面已过期，请刷新后重试'); return; }
    var formData = new FormData();
    formData.append('post_action', 'toggle_pin');
    formData.append('id', id);
    formData.append('csrf_token', csrfToken.value);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=announcements', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4 && xhr.status === 200) {
            document.getElementById('mainContent').innerHTML = xhr.responseText;
            rebindScripts();
        }
    };
    xhr.send(formData);
}

function announceDelete(id) {
    if (!confirm('确定要删除这条公告吗？')) return;
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) { alert('页面已过期，请刷新后重试'); return; }
    var formData = new FormData();
    formData.append('post_action', 'delete');
    formData.append('id', id);
    formData.append('csrf_token', csrfToken.value);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=announcements', true);
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
        if (oldScript.src) { newScript.src = oldScript.src; }
        else { newScript.textContent = oldScript.textContent; }
        oldScript.parentNode.replaceChild(newScript, oldScript);
    });
}
</script>
