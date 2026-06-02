<?php
require_once __DIR__ . '/../backend/auth.php';
require_once __DIR__ . '/../backend/db.php';
requireLogin();
initDatabase();

$db = getDB();
$message = '';
$error = '';

if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    requireCsrf();
    try {
        $postAction = $_POST['post_action'] ?? '';

        if ($postAction === 'mark_read') {
            $id = (int)($_POST['id'] ?? 0);
            if ($id > 0) {
                $db->prepare("UPDATE contacts SET is_read = 1 WHERE id = ?")->execute([$id]);
                $message = '已标记为已读';
            }
        } elseif ($postAction === 'mark_all_read') {
            $db->exec("UPDATE contacts SET is_read = 1 WHERE is_read = 0");
            $message = '已全部标记为已读';
        } elseif ($postAction === 'reply') {
            $id = (int)($_POST['id'] ?? 0);
            $reply = trim($_POST['admin_reply'] ?? '');
            if ($id > 0) {
                $now = now_cn();
                $db->prepare("UPDATE contacts SET admin_reply = ?, replied_at = ?, is_read = 1 WHERE id = ?")->execute([$reply, $now, $id]);
                $message = '回复成功';
            }
        } elseif ($postAction === 'delete') {
            $id = (int)($_POST['id'] ?? 0);
            if ($id > 0) {
                $db->prepare("DELETE FROM contacts WHERE id = ?")->execute([$id]);
                $message = '已删除';
            }
        }
    } catch (Exception $e) {
        $error = '操作失败：' . $e->getMessage();
    }
}

$items = $db->query("SELECT * FROM contacts ORDER BY created_at DESC")->fetchAll();
$unreadCount = $db->query("SELECT COUNT(*) FROM contacts WHERE is_read = 0")->fetchColumn();
?>
<div class="main-content">
    <?php echo csrfField(); ?>
    <div class="flex items-center justify-between mb-8">
        <h1 class="text-2xl font-bold">联系消息</h1>
        <?php if ($unreadCount > 0): ?>
        <button type="button" onclick="contactMarkAllRead()" class="btn btn-ghost">
            全部标记已读 (<?php echo $unreadCount; ?>)
        </button>
        <?php endif; ?>
    </div>

    <?php if ($message): ?><div class="msg-success"><?php echo h($message); ?></div><?php endif; ?>
    <?php if ($error): ?><div class="msg-error"><?php echo h($error); ?></div><?php endif; ?>

    <div class="table-wrapper">
        <table>
            <thead>
                <tr>
                    <th>姓名</th>
                    <th>微信</th>
                    <th>主题</th>
                    <th>内容</th>
                    <th>状态</th>
                    <th>时间</th>
                    <th>操作</th>
                </tr>
            </thead>
            <tbody>
                <?php if (empty($items)): ?>
                <tr><td colspan="7" style="text-align:center; color: var(--text-secondary); padding: 40px;">暂无消息</td></tr>
                <?php else: ?>
                <?php foreach ($items as $c): ?>
                <tr class="<?php echo $c['is_read'] ? '' : 'row-unread'; ?>">
                    <td><?php echo h($c['name']); ?></td>
                    <td>
                        <div class="contact-wechat">
                            <span style="color: var(--text-secondary);"><?php echo h($c['email']); ?></span>
                            <?php if ($c['email']): ?>
                            <button type="button" class="btn-copy-wechat" onclick="copyWechat('<?php echo h(addslashes($c['email'])); ?>', this)" title="复制微信号">
                                <svg width="14" height="14" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"/></svg>
                            </button>
                            <?php endif; ?>
                        </div>
                    </td>
                    <td><?php echo h($c['subject']); ?></td>
                    <td class="message-cell" onclick="this.querySelector('.message-full').classList.toggle('show'); var hint=this.querySelector('.message-expand-hint'); if(hint) hint.textContent = hint.textContent === '展开' ? '收起' : '展开';">
                        <div class="message-short"><?php echo h(mb_strlen($c['message']) > 60 ? mb_substr($c['message'], 0, 60) . '...' : $c['message']); ?></div>
                        <?php if (mb_strlen($c['message']) > 60): ?>
                        <div class="message-full"><?php echo nl2br(h($c['message'])); ?></div>
                        <div class="message-expand-hint">展开</div>
                        <?php endif; ?>
                        <?php if ($c['admin_reply']): ?>
                        <div class="existing-reply">
                            <div class="reply-label">我的回复</div>
                            <div class="reply-content"><?php echo nl2br(h($c['admin_reply'])); ?></div>
                            <div class="reply-time"><?php echo h(format_cn($c['replied_at'])); ?></div>
                        </div>
                        <?php endif; ?>
                    </td>
                    <td>
                        <?php if ($c['admin_reply']): ?>
                        <span class="badge badge-replied">已回复</span>
                        <?php elseif ($c['is_read']): ?>
                        <span class="badge badge-read">已读</span>
                        <?php else: ?>
                        <span class="badge badge-unread">未读</span>
                        <?php endif; ?>
                    </td>
                    <td style="color: var(--text-secondary); font-size: 13px;"><?php echo h(format_cn($c['created_at'])); ?></td>
                    <td>
                        <div class="flex gap-2">
                            <?php if (!$c['is_read']): ?>
                            <button type="button" onclick="contactMarkRead(<?php echo $c['id']; ?>)" class="btn btn-primary btn-sm">已读</button>
                            <?php endif; ?>
                            <button type="button" onclick="openContactReplyModal(<?php echo $c['id']; ?>, '<?php echo h(addslashes($c['admin_reply'])); ?>', '<?php echo h(addslashes($c['name'])); ?>')" class="btn btn-ghost btn-sm"><?php echo $c['admin_reply'] ? '编辑回复' : '回复'; ?></button>
                            <button type="button" onclick="contactDelete(<?php echo $c['id']; ?>)" class="btn btn-danger btn-sm">删除</button>
                        </div>
                    </td>
                </tr>
                <?php endforeach; ?>
                <?php endif; ?>
            </tbody>
        </table>
    </div>
</div>

<div id="contactReplyModal" class="reply-modal-overlay" onclick="if(event.target===this)closeContactReplyModal()">
    <div class="reply-modal">
        <h3>回复 <span id="replyContactName"></span></h3>
        <form id="contactReplyForm" onsubmit="return contactReply(event)">
            <?php echo csrfField(); ?>
            <input type="hidden" name="post_action" value="reply">
            <input type="hidden" name="id" id="contactReplyId">
            <textarea name="admin_reply" id="contactReplyText" class="input-field" rows="5" placeholder="输入回复内容，方便记录沟通情况..."></textarea>
            <div style="display:flex; gap:8px; margin-top:12px; justify-content:flex-end;">
                <button type="button" class="btn btn-ghost" onclick="closeContactReplyModal()">取消</button>
                <button type="submit" class="btn btn-primary">提交回复</button>
            </div>
        </form>
    </div>
</div>

<script>
function contactMarkRead(id) {
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) { alert('页面已过期，请刷新后重试'); return; }
    var formData = new FormData();
    formData.append('post_action', 'mark_read');
    formData.append('id', id);
    formData.append('csrf_token', csrfToken.value);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=contacts', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4 && xhr.status === 200) {
            document.getElementById('mainContent').innerHTML = xhr.responseText;
            rebindScripts();
        }
    };
    xhr.send(formData);
}

function contactMarkAllRead() {
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) { alert('页面已过期，请刷新后重试'); return; }
    var formData = new FormData();
    formData.append('post_action', 'mark_all_read');
    formData.append('csrf_token', csrfToken.value);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=contacts', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4 && xhr.status === 200) {
            document.getElementById('mainContent').innerHTML = xhr.responseText;
            rebindScripts();
        }
    };
    xhr.send(formData);
}

function contactReply(e) {
    e.preventDefault();
    var form = e.target;
    var formData = new FormData(form);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=contacts', true);
    xhr.setRequestHeader('X-Requested-With', 'XMLHttpRequest');
    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                closeContactReplyModal();
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

function contactDelete(id) {
    if (!confirm('确定要删除此消息吗？')) return;
    var csrfToken = document.querySelector('input[name="csrf_token"]');
    if (!csrfToken) { alert('页面已过期，请刷新后重试'); return; }
    var formData = new FormData();
    formData.append('post_action', 'delete');
    formData.append('id', id);
    formData.append('csrf_token', csrfToken.value);
    var xhr = new XMLHttpRequest();
    xhr.open('POST', 'layout.php?page=contacts', true);
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

function openContactReplyModal(id, existingReply, name) {
    document.getElementById('contactReplyId').value = id;
    document.getElementById('contactReplyText').value = existingReply || '';
    document.getElementById('replyContactName').textContent = name || '';
    document.getElementById('contactReplyModal').classList.add('show');
}

function closeContactReplyModal() {
    document.getElementById('contactReplyModal').classList.remove('show');
}

function copyWechat(text, btn) {
    event.stopPropagation();
    if (navigator.clipboard) {
        navigator.clipboard.writeText(text).then(function() {
            showCopySuccess(btn);
        });
    } else {
        var ta = document.createElement('textarea');
        ta.value = text;
        ta.style.position = 'fixed';
        ta.style.left = '-9999px';
        document.body.appendChild(ta);
        ta.select();
        document.execCommand('copy');
        document.body.removeChild(ta);
        showCopySuccess(btn);
    }
}

function showCopySuccess(btn) {
    var orig = btn.innerHTML;
    btn.innerHTML = '<svg width="14" height="14" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"/></svg>';
    btn.style.color = '#00b894';
    setTimeout(function() {
        btn.innerHTML = orig;
        btn.style.color = '';
    }, 1500);
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
