<?php
require_once __DIR__ . '/../backend/auth.php';
require_once __DIR__ . '/../backend/db.php';
requireLogin();
initDatabase();

$db = getDB();
$message = '';
$error = '';

$typeLabels = ['bug' => '缺陷', 'suggestion' => '建议', 'praise' => '好评', 'other' => '其他'];
$typeBadgeClass = ['bug' => 'badge-bug', 'suggestion' => 'badge-suggestion', 'praise' => 'badge-praise', 'other' => 'badge-other'];
$statusLabels = ['pending' => '待处理', 'processing' => '处理中', 'resolved' => '已解决', 'closed' => '已关闭'];
$statusBadgeClass = ['pending' => 'badge-pending', 'processing' => 'badge-processing', 'resolved' => 'badge-resolved', 'closed' => 'badge-closed'];

if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    requireCsrf();
    try {
        $postAction = $_POST['post_action'] ?? '';

        if ($postAction === 'update_status') {
            $id = (int)($_POST['id'] ?? 0);
            $status = $_POST['status'] ?? '';
            if ($id > 0 && isset($statusLabels[$status])) {
                $db->prepare("UPDATE feedback SET status = ?, updated_at = datetime('now','localtime') WHERE id = ?")->execute([$status, $id]);
                $message = '状态已更新';
            }
        } elseif ($postAction === 'reply') {
            $id = (int)($_POST['id'] ?? 0);
            $reply = trim($_POST['admin_reply'] ?? '');
            if ($id > 0) {
                $db->prepare("UPDATE feedback SET admin_reply = ?, status = 'resolved', updated_at = datetime('now','localtime') WHERE id = ?")->execute([$reply, $id]);
                $message = '回复成功';
            }
        } elseif ($postAction === 'delete') {
            $id = (int)($_POST['id'] ?? 0);
            if ($id > 0) {
                $db->prepare("DELETE FROM feedback WHERE id = ?")->execute([$id]);
                $message = '已删除';
            }
        }
    } catch (Exception $e) {
        $error = '操作失败：' . $e->getMessage();
    }
}

$items = $db->query("SELECT * FROM feedback ORDER BY created_at DESC")->fetchAll();
$pendingCount = $db->query("SELECT COUNT(*) FROM feedback WHERE status = 'pending'")->fetchColumn();
?>
<div class="main-content">
    <?php echo csrfField(); ?>
    <h1 class="text-2xl font-bold mb-8">反馈管理</h1>

    <?php if ($message): ?><div class="msg-success"><?php echo h($message); ?></div><?php endif; ?>
    <?php if ($error): ?><div class="msg-error"><?php echo h($error); ?></div><?php endif; ?>

    <div class="table-wrapper">
        <table>
            <thead>
                <tr>
                    <th>类型</th>
                    <th>内容</th>
                    <th>联系方式</th>
                    <th>应用版本</th>
                    <th>状态</th>
                    <th>时间</th>
                    <th>操作</th>
                </tr>
            </thead>
            <tbody>
                <?php if (empty($items)): ?>
                <tr><td colspan="7" style="text-align:center; color: var(--text-secondary); padding: 40px;">暂无反馈</td></tr>
                <?php else: ?>
                <?php foreach ($items as $f): ?>
                <tr class="<?php echo $f['status'] === 'pending' ? 'row-pending' : ''; ?>">
                    <td><span class="badge <?php echo $typeBadgeClass[$f['type']] ?? 'badge-other'; ?>"><?php echo h($typeLabels[$f['type']] ?? $f['type']); ?></span></td>
                    <td class="content-cell" onclick="this.querySelector('.content-full').classList.toggle('show'); var hint=this.querySelector('.content-expand-hint'); if(hint) hint.textContent = hint.textContent === '展开' ? '收起' : '展开';">
                        <div class="content-short"><?php echo h(mb_strlen($f['content']) > 60 ? mb_substr($f['content'], 0, 60) . '...' : $f['content']); ?></div>
                        <?php if (mb_strlen($f['content']) > 60): ?>
                        <div class="content-full"><?php echo nl2br(h($f['content'])); ?></div>
                        <div class="content-expand-hint">展开</div>
                        <?php endif; ?>
                        <?php if ($f['admin_reply']): ?>
                        <div class="existing-reply">回复：<?php echo nl2br(h($f['admin_reply'])); ?></div>
                        <?php endif; ?>
                    </td>
                    <td style="color: var(--text-secondary);"><?php echo h($f['contact']) ?: '-'; ?></td>
                    <td style="color: var(--text-secondary);"><?php echo h($f['app_version']) ?: '-'; ?></td>
                    <td><span class="badge <?php echo $statusBadgeClass[$f['status']] ?? 'badge-pending'; ?>"><?php echo h($statusLabels[$f['status']] ?? $f['status']); ?></span></td>
                    <td style="color: var(--text-secondary); font-size: 13px;"><?php echo h($f['created_at']); ?></td>
                    <td>
                        <div class="flex gap-2 items-center">
                            <select class="status-select" onchange="feedbackUpdateStatus(<?php echo $f['id']; ?>, this.value)">
                                <?php foreach ($statusLabels as $key => $label): ?>
                                <option value="<?php echo $key; ?>" <?php echo $f['status'] === $key ? 'selected' : ''; ?>><?php echo $label; ?></option>
                                <?php endforeach; ?>
                            </select>
                            <button type="button" class="btn btn-ghost btn-sm" onclick="openReplyModal(<?php echo $f['id']; ?>, '<?php echo h(addslashes($f['admin_reply'])); ?>')">回复</button>
                            <button type="button" class="btn btn-danger btn-sm" onclick="feedbackDelete(<?php echo $f['id']; ?>)">删除</button>
                        </div>
                    </td>
                </tr>
                <?php endforeach; ?>
                <?php endif; ?>
            </tbody>
        </table>
    </div>
</div>

<div id="replyModal" class="reply-modal-overlay" onclick="if(event.target===this)closeReplyModal()">
    <div class="reply-modal">
        <h3>回复反馈</h3>
        <form id="replyForm" onsubmit="return feedbackReply(event)">
            <?php echo csrfField(); ?>
            <input type="hidden" name="post_action" value="reply">
            <input type="hidden" name="id" id="replyId">
            <textarea name="admin_reply" id="replyText" class="input-field" rows="4" placeholder="输入回复内容..."></textarea>
            <div class="modal-actions" style="display:flex; gap:8px; margin-top:12px; justify-content:flex-end;">
                <button type="button" class="btn btn-ghost" onclick="closeReplyModal()">取消</button>
                <button type="submit" class="btn btn-primary">提交回复</button>
            </div>
        </form>
    </div>
</div>

<script>
function feedbackUpdateStatus(id, status) {
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) { alert('页面已过期，请刷新后重试'); return; }
    var formData = new FormData();
    formData.append('post_action', 'update_status');
    formData.append('id', id);
    formData.append('status', status);
    formData.append('csrf_token', csrfToken.value);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=feedback', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4 && xhr.status === 200) {
            document.getElementById('mainContent').innerHTML = xhr.responseText;
            rebindScripts();
        }
    };
    xhr.send(formData);
}

function feedbackReply(e) {
    e.preventDefault();
    var form = e.target;
    var formData = new FormData(form);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=feedback', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                closeReplyModal();
                document.getElementById('mainContent').innerHTML = xhr.responseText;
                rebindScripts();
            } else {
                alert('回复失败，请刷新页面后重试');
            }
        }
    };
    xhr.send(formData);
    return false;
}

function feedbackDelete(id) {
    if (!confirm('确定删除此反馈？')) return;
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) { alert('页面已过期，请刷新后重试'); return; }
    var formData = new FormData();
    formData.append('post_action', 'delete');
    formData.append('id', id);
    formData.append('csrf_token', csrfToken.value);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=feedback', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                document.getElementById('mainContent').innerHTML = xhr.responseText;
                rebindScripts();
            } else {
                alert('删除失败，请刷新页面后重试');
            }
        }
    };
    xhr.send(formData);
}

function openReplyModal(id, existingReply) {
    document.getElementById('replyId').value = id;
    document.getElementById('replyText').value = existingReply || '';
    document.getElementById('replyModal').classList.add('show');
}

function closeReplyModal() {
    document.getElementById('replyModal').classList.remove('show');
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
