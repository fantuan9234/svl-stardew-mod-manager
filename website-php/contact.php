<?php
$pageTitle = '联系我们';

require_once __DIR__ . '/backend/security.php';
sendSecurityHeaders();
require_once __DIR__ . '/backend/language.php';

$socials = [
    ['name' => 'Bilibili', 'handle' => '@饭团923', 'url' => 'https://space.bilibili.com/3546621436496190?spm_id_from=333.40164.0.0', 'color' => '#fb7299', 'bg' => 'rgba(251,114,153,0.1)', 'icon' => '<svg class="w-6 h-6" viewBox="0 0 24 24" fill="currentColor"><path d="M17.813 4.653h.854c1.51.054 2.769.578 3.773 1.574 1.004.995 1.524 2.249 1.56 3.76v7.36c-.036 1.51-.556 2.769-1.56 3.773s-2.262 1.524-3.773 1.56H5.333c-1.51-.036-2.769-.556-3.773-1.56S.036 18.858 0 17.347v-7.36c.036-1.511.556-2.765 1.56-3.76 1.004-.996 2.262-1.52 3.773-1.574h.774l-1.174-1.12a1.234 1.234 0 0 1-.373-.906c0-.356.124-.658.373-.907l.027-.027c.267-.249.573-.373.92-.373.347 0 .653.124.92.373L9.653 4.44c.071.071.134.142.187.213h4.267a.836.836 0 0 1 .16-.213l2.853-2.747c.267-.249.573-.373.92-.373.347 0 .662.151.929.4.267.249.391.551.391.907 0 .355-.124.657-.373.906zM5.333 7.24c-.746.018-1.373.276-1.88.773-.506.498-.769 1.13-.786 1.894v7.52c.017.764.28 1.395.786 1.893.507.498 1.134.756 1.88.773h13.334c.746-.017 1.373-.275 1.88-.773.506-.498.769-1.129.786-1.893v-7.52c-.017-.765-.28-1.396-.786-1.894-.507-.497-1.134-.755-1.88-.773zM8 11.107c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c0-.373.129-.689.386-.947.258-.257.574-.386.947-.386zm8 0c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c.017-.391.15-.711.4-.96.249-.249.56-.373.933-.373z"/></svg>'],
    ['name' => '抖音', 'handle' => '@饭团', 'url' => 'https://www.douyin.com/user/self?from_tab_name=main', 'color' => '#fff', 'bg' => '#1a1a1a', 'icon' => '<svg class="w-6 h-6" viewBox="0 0 24 24" fill="currentColor"><path d="M12.525.02c1.31-.02 2.61-.01 3.91-.02.08 1.53.63 3.09 1.75 4.17 1.12 1.11 2.7 1.62 4.24 1.79v4.03c-1.44-.05-2.89-.35-4.2-.97-.57-.26-1.1-.59-1.62-.93-.01 2.92.01 5.84-.02 8.75-.08 1.4-.54 2.79-1.35 3.94-1.31 1.92-3.58 3.17-5.91 3.21-1.43.08-2.86-.31-4.08-1.03-2.02-1.19-3.44-3.37-3.65-5.71-.02-.5-.03-1-.01-1.49.18-1.9 1.12-3.72 2.58-4.96 1.66-1.44 3.98-2.13 6.15-1.72.02 1.48-.04 2.96-.04 4.44-.99-.32-2.15-.23-3.02.37-.63.41-1.11 1.04-1.36 1.75-.21.51-.15 1.07-.14 1.61.24 1.64 1.82 3.02 3.5 2.87 1.12-.01 2.19-.66 2.77-1.61.19-.33.4-.67.41-1.06.1-1.79.06-3.57.07-5.36.01-4.03-.01-8.05.02-12.07z"/></svg>']
];

include 'header.php';
?>

<main class="flex-1 py-24">
    <div class="max-w-6xl mx-auto px-6">
        <div class="text-center mb-16">
            <span class="section-label">Follow Us</span>
            <h1 class="section-title mb-4"><?php echo t('contact_follow_title'); ?></h1>
            <p class="section-subtitle mx-auto"><?php echo t('contact_follow_subtitle'); ?></p>
        </div>
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4 max-w-3xl mx-auto">
            <?php foreach ($socials as $s): ?>
            <a href="<?php echo $s['url']; ?>" target="_blank" rel="noopener" class="social-card text-center group">
                <div class="w-12 h-12 rounded-xl mx-auto mb-4 flex items-center justify-center transition-transform duration-300 group-hover:scale-110" style="background: <?php echo $s['bg']; ?>; color: <?php echo $s['color']; ?>;">
                    <?php echo $s['icon']; ?>
                </div>
                <h3 class="font-semibold text-sm mb-1" style="color: var(--text);"><?php echo $s['name']; ?></h3>
                <p class="text-xs" style="color: var(--text-tertiary);"><?php echo $s['handle']; ?></p>
            </a>
            <?php endforeach; ?>
        </div>
    </div>

    <div class="divider max-w-6xl mx-auto mt-20 mb-20"></div>

    <div class="max-w-2xl mx-auto px-6">
        <div class="text-center mb-12">
            <span class="section-label">Contact</span>
            <h2 class="text-2xl md:text-3xl font-bold mb-4" style="color: var(--text);"><?php echo t('contact_form_title'); ?></h2>
            <p class="section-subtitle mx-auto"><?php echo t('contact_form_subtitle'); ?></p>
        </div>

        <form id="contactForm" class="space-y-5" novalidate>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-5">
                <div class="contact-form-group">
                    <label class="block text-sm font-medium mb-2" style="color: var(--text-secondary);"><?php echo t('contact_name'); ?></label>
                    <input type="text" name="name" class="contact-input" placeholder="<?php echo t('contact_name_placeholder'); ?>">
                </div>
                <div class="contact-form-group">
                    <label class="block text-sm font-medium mb-2" style="color: var(--text-secondary);"><?php echo t('contact_wechat'); ?></label>
                    <input type="text" name="email" class="contact-input" placeholder="<?php echo t('contact_wechat_placeholder'); ?>">
                </div>
            </div>
            <div class="contact-form-group">
                <label class="block text-sm font-medium mb-2" style="color: var(--text-secondary);"><?php echo t('contact_subject'); ?></label>
                <input type="text" name="subject" class="contact-input" placeholder="<?php echo t('contact_subject_placeholder'); ?>">
            </div>
            <div class="contact-form-group">
                <label class="block text-sm font-medium mb-2" style="color: var(--text-secondary);"><?php echo t('contact_content'); ?> <span style="color: #ef4444;">*</span></label>
                <textarea name="message" class="contact-input" rows="5" placeholder="<?php echo t('contact_content_placeholder'); ?>" required></textarea>
            </div>
            <div id="formFeedback" class="hidden text-sm rounded-lg p-4 mb-2"></div>
            <button type="submit" class="btn-primary w-full justify-center" id="submitBtn">
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/></svg>
                <?php echo t('contact_submit'); ?>
            </button>
        </form>

        <div id="replySection" class="hidden mt-10">
            <div class="text-center mb-6">
                <span class="section-label">Reply</span>
                <h3 class="text-xl font-bold mb-2" style="color: var(--text);">我的消息与回复</h3>
                <p class="section-subtitle mx-auto">以下是你通过此设备发送的消息及管理员的回复</p>
            </div>
            <div id="replyList" class="space-y-4"></div>
        </div>
    </div>
</main>

<style>
.contact-input {
    width: 100%;
    padding: 14px 18px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    color: var(--text);
    font-size: 14px;
    outline: none;
    transition: border-color 0.3s, box-shadow 0.3s, background 0.3s;
    font-family: inherit;
}
.contact-input:focus {
    border-color: var(--brand);
    box-shadow: 0 0 0 3px rgba(212,168,67,0.1);
    background: var(--bg);
}
.contact-input::placeholder { color: var(--text-tertiary); }
.contact-form-group {
    position: relative;
}
.contact-form-group label {
    transition: color 0.2s;
}
.contact-form-group:focus-within label {
    color: var(--brand);
}
.feedback-success {
    background: rgba(34,197,94,0.1);
    border: 1px solid rgba(34,197,94,0.2);
    color: #22c55e;
}
.feedback-error {
    background: rgba(239,68,68,0.1);
    border: 1px solid rgba(239,68,68,0.2);
    color: #ef4444;
}
body.light-theme .feedback-success {
    background: rgba(34,197,94,0.15);
    border-color: rgba(34,197,94,0.3);
}
body.light-theme .feedback-error {
    background: rgba(239,68,68,0.1);
    border-color: rgba(239,68,68,0.3);
}
.social-card {
    padding: 24px 16px;
    border-radius: 16px;
    background: var(--surface);
    border: 1px solid var(--border);
    text-decoration: none;
    transition: transform 0.3s, box-shadow 0.3s, border-color 0.3s;
}
.social-card:hover {
    transform: translateY(-4px);
    box-shadow: 0 8px 32px rgba(0,0,0,0.08);
    border-color: var(--brand-dim);
}
body.light-theme .social-card:hover {
    box-shadow: 0 8px 32px rgba(0,0,0,0.04);
}
.reply-card {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 20px;
}
.reply-card .reply-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
}
.reply-card .reply-status {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 4px 10px;
    border-radius: 6px;
    font-size: 12px;
    font-weight: 500;
}
.reply-status.replied {
    background: rgba(0,184,148,0.1);
    border: 1px solid rgba(0,184,148,0.2);
    color: #00b894;
}
.reply-status.pending {
    background: rgba(255,165,0,0.1);
    border: 1px solid rgba(255,165,0,0.2);
    color: #ff8c00;
}
.reply-card .reply-message {
    font-size: 14px;
    line-height: 1.6;
    color: var(--text);
    margin-bottom: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--border);
}
.reply-card .reply-admin {
    background: rgba(0,184,148,0.05);
    border: 1px solid rgba(0,184,148,0.12);
    border-radius: 10px;
    padding: 14px;
}
.reply-card .reply-admin-label {
    font-size: 11px;
    font-weight: 600;
    color: #00b894;
    margin-bottom: 6px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
}
.reply-card .reply-admin-content {
    font-size: 14px;
    line-height: 1.6;
    color: var(--text);
}
.reply-card .reply-time {
    font-size: 12px;
    color: var(--text-tertiary);
    margin-top: 8px;
}
</style>

<script>
function getDeviceId() {
    var key = 'svl_device_id';
    var id = localStorage.getItem(key);
    if (!id) {
        id = '';
        var chars = 'abcdef0123456789';
        for (var i = 0; i < 32; i++) {
            id += chars.charAt(Math.floor(Math.random() * chars.length));
        }
        localStorage.setItem(key, id);
    }
    return id;
}

async function loadReplies() {
    var deviceId = getDeviceId();
    try {
        var resp = await fetch('api/contact_replies.php?device_id=' + encodeURIComponent(deviceId));
        var data = await resp.json();
        if (data.success && data.data.length > 0) {
            var section = document.getElementById('replySection');
            var list = document.getElementById('replyList');
            list.innerHTML = '';
            data.data.forEach(function(item) {
                var hasReply = item.admin_reply && item.admin_reply.trim() !== '';
                var html = '<div class="reply-card">';
                html += '<div class="reply-header">';
                html += '<span class="text-sm font-medium" style="color: var(--text);">' + (item.subject || '无主题') + '</span>';
                if (hasReply) {
                    html += '<span class="reply-status replied">已回复</span>';
                } else {
                    html += '<span class="reply-status pending">等待回复</span>';
                }
                html += '</div>';
                html += '<div class="reply-message">' + escapeHtml(item.message) + '</div>';
                if (hasReply) {
                    html += '<div class="reply-admin">';
                    html += '<div class="reply-admin-label">管理员回复</div>';
                    html += '<div class="reply-admin-content">' + escapeHtml(item.admin_reply).replace(/\n/g, '<br>') + '</div>';
                    html += '<div class="reply-time">回复于 ' + item.replied_at + '</div>';
                    html += '</div>';
                }
                html += '<div class="reply-time" style="margin-top: 8px;">发送于 ' + item.created_at + '</div>';
                html += '</div>';
                list.innerHTML += html;
            });
            section.classList.remove('hidden');
        }
    } catch (e) {
        console.error('加载回复失败', e);
    }
}

function escapeHtml(text) {
    var div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

document.getElementById('contactForm').addEventListener('submit', async function(e) {
    e.preventDefault();
    var btn = document.getElementById('submitBtn');
    var fb = document.getElementById('formFeedback');
    var originalText = btn.innerHTML;

    fb.classList.add('hidden');

    var message = this.querySelector('[name="message"]').value.trim();
    if (!message) {
        fb.className = 'text-sm rounded-lg p-4 mb-2 feedback-error';
        fb.textContent = '<?php echo t('contact_error_empty'); ?>';
        fb.classList.remove('hidden');
        return;
    }

    btn.innerHTML = '<svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24"><circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle><path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"></path></svg> <?php echo t('contact_sending'); ?>';
    btn.style.pointerEvents = 'none';
    btn.style.opacity = '0.7';

    try {
        var formData = new FormData(this);
        formData.append('device_id', getDeviceId());
        var resp = await fetch('api/contact.php', { method: 'POST', body: formData });
        var data = await resp.json();

        if (data.success) {
            fb.className = 'text-sm rounded-lg p-4 mb-2 feedback-success';
            fb.textContent = '<?php echo t('contact_success'); ?>';
            this.reset();
            setTimeout(loadReplies, 500);
        } else {
            fb.className = 'text-sm rounded-lg p-4 mb-2 feedback-error';
            fb.textContent = data.error || '<?php echo t('contact_error_send'); ?>';
        }
    } catch (err) {
        fb.className = 'text-sm rounded-lg p-4 mb-2 feedback-error';
        fb.textContent = '<?php echo t('contact_error_network'); ?>';
    }

    fb.classList.remove('hidden');
    btn.innerHTML = originalText;
    btn.style.pointerEvents = '';
    btn.style.opacity = '';
});

loadReplies();
</script>

<?php include 'footer.php'; ?>
