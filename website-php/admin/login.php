<?php
require_once __DIR__ . '/../backend/auth.php';

$error = '';

if (isLoggedIn()) {
    header('Location: index.php');
    exit;
}

if ($_SERVER['REQUEST_METHOD'] === 'POST') {
    requireCsrf();

    if (isLoginBlocked()) {
        $error = getLoginBlockedMessage();
    } else {
        $username = trim($_POST['username'] ?? '');
        $password = $_POST['password'] ?? '';
        if (login($username, $password)) {
            header('Location: index.php');
            exit;
        }

        if (isLoginBlocked()) {
            $error = getLoginBlockedMessage();
        } else {
            $error = '用户名或密码错误';
        }
    }
}
?>
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>登录 - 管理后台</title>
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
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .login-card {
            background: var(--surface);
            border: 1px solid var(--border);
            border-radius: 16px;
            padding: 40px;
            width: 100%;
            max-width: 400px;
        }
        .input-field {
            width: 100%;
            padding: 12px 16px;
            background: rgba(255,255,255,0.04);
            border: 1px solid var(--border);
            border-radius: 10px;
            color: var(--text);
            font-size: 14px;
            outline: none;
            transition: border-color 0.2s;
        }
        .input-field:focus { border-color: var(--brand); }
        .btn-submit {
            width: 100%;
            padding: 12px;
            background: var(--brand);
            color: #0c0c0e;
            border: none;
            border-radius: 10px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            transition: opacity 0.2s;
        }
        .btn-submit:hover { opacity: 0.9; }
        .btn-submit:disabled { opacity: 0.5; cursor: not-allowed; }
        .error-msg {
            background: rgba(239,68,68,0.1);
            border: 1px solid rgba(239,68,68,0.2);
            color: #ef4444;
            padding: 10px 14px;
            border-radius: 8px;
            font-size: 13px;
            margin-bottom: 16px;
        }
    </style>
</head>
<body>
    <div class="login-card">
        <div class="text-center mb-8">
            <h1 class="text-xl font-bold mb-2">管理后台</h1>
            <p class="text-sm" style="color: var(--text-secondary);">请登录以继续</p>
        </div>
        <?php if ($error): ?>
        <div class="error-msg"><?php echo h($error); ?></div>
        <?php endif; ?>
        <form method="post">
            <?php echo csrfField(); ?>
            <div class="mb-4">
                <label class="block text-sm font-medium mb-2" style="color: var(--text-secondary);">用户名</label>
                <input type="text" name="username" class="input-field" required autocomplete="username" <?php echo isLoginBlocked() ? 'disabled' : ''; ?>>
            </div>
            <div class="mb-6">
                <label class="block text-sm font-medium mb-2" style="color: var(--text-secondary);">密码</label>
                <input type="password" name="password" class="input-field" required autocomplete="current-password" <?php echo isLoginBlocked() ? 'disabled' : ''; ?>>
            </div>
            <button type="submit" class="btn-submit" <?php echo isLoginBlocked() ? 'disabled' : ''; ?>>登 录</button>
        </form>
        <div class="text-center mt-6">
            <a href="../index.php" class="text-sm transition-colors hover:text-white" style="color: var(--text-secondary); text-decoration: none;">返回网站</a>
        </div>
    </div>
</body>
</html>