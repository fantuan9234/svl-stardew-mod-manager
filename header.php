<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title><?php echo isset($pageTitle) ? $pageTitle . ' - ' : ''; ?>星露谷管理器</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <link rel="icon" type="image/png" href="assets/icon.png">
    <style>
        :root {
            --bg: #fafafa;
            --card: #ffffff;
            --text: #1a1a1a;
            --text-secondary: #666666;
            --border: #e5e5e5;
            --brand: #d4a843;
            --brand-hover: #b8922e;
        }

        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            background-color: var(--bg);
            color: var(--text);
            line-height: 1.6;
        }

        .nav-link {
            color: var(--text-secondary);
            text-decoration: none;
            font-size: 14px;
            font-weight: 500;
            padding: 8px 16px;
            border-radius: 8px;
            transition: all 0.2s ease;
        }

        .nav-link:hover {
            color: var(--text);
            background-color: #f0f0f0;
        }

        .nav-link.active {
            color: var(--brand);
            background-color: rgba(212, 168, 67, 0.08);
        }

        .btn-primary {
            display: inline-flex;
            align-items: center;
            gap: 8px;
            padding: 10px 24px;
            background-color: var(--brand);
            color: white;
            font-size: 14px;
            font-weight: 600;
            border-radius: 10px;
            text-decoration: none;
            transition: all 0.2s ease;
            border: none;
            cursor: pointer;
        }

        .btn-primary:hover {
            background-color: var(--brand-hover);
            transform: translateY(-1px);
            box-shadow: 0 4px 12px rgba(212, 168, 67, 0.3);
        }

        .btn-secondary {
            display: inline-flex;
            align-items: center;
            gap: 8px;
            padding: 10px 24px;
            background-color: white;
            color: var(--text);
            font-size: 14px;
            font-weight: 600;
            border-radius: 10px;
            text-decoration: none;
            transition: all 0.2s ease;
            border: 1px solid var(--border);
            cursor: pointer;
        }

        .btn-secondary:hover {
            background-color: #f5f5f5;
            border-color: #d0d0d0;
        }

        .card {
            background-color: var(--card);
            border: 1px solid var(--border);
            border-radius: 16px;
            padding: 24px;
            transition: all 0.2s ease;
        }

        .card:hover {
            box-shadow: 0 4px 20px rgba(0, 0, 0, 0.06);
            transform: translateY(-2px);
        }

        .section-title {
            font-size: 28px;
            font-weight: 700;
            color: var(--text);
            margin-bottom: 8px;
        }

        .section-subtitle {
            font-size: 15px;
            color: var(--text-secondary);
        }

        .tag {
            display: inline-block;
            padding: 4px 12px;
            background-color: rgba(212, 168, 67, 0.1);
            color: var(--brand);
            font-size: 12px;
            font-weight: 600;
            border-radius: 20px;
        }

        .tag-green {
            background-color: rgba(91, 140, 90, 0.1);
            color: #5b8c5a;
        }

        .tag-blue {
            background-color: rgba(59, 130, 246, 0.1);
            color: #3b82f6;
        }

        .tag-red {
            background-color: rgba(239, 68, 68, 0.1);
            color: #ef4444;
        }

        @media (max-width: 768px) {
            .section-title {
                font-size: 22px;
            }
        }
    </style>
</head>
<body class="min-h-screen flex flex-col">
    <header class="sticky top-0 z-50 bg-white/80 backdrop-blur-md border-b" style="border-color: var(--border);">
        <div class="max-w-6xl mx-auto px-4 sm:px-6">
            <div class="flex items-center justify-between h-16">
                <a href="index.php" class="flex items-center gap-3 text-decoration-none">
                    <img src="assets/icon.png" alt="SVL" class="w-9 h-9 rounded-xl">
                    <span class="font-semibold text-lg" style="color: var(--text);">星露谷管理器</span>
                </a>

                <nav class="hidden md:flex items-center gap-1">
                    <a href="index.php" class="nav-link <?php echo basename($_SERVER['PHP_SELF']) == 'index.php' ? 'active' : '';">首页</a>
                    <a href="announcements.php" class="nav-link <?php echo basename($_SERVER['PHP_SELF']) == 'announcements.php' ? 'active' : '';">公告</a>
                    <a href="ads.php" class="nav-link <?php echo basename($_SERVER['PHP_SELF']) == 'ads.php' ? 'active' : '';">推广</a>
                    <a href="contact.php" class="nav-link <?php echo basename($_SERVER['PHP_SELF']) == 'contact.php' ? 'active' : '';">联系</a>
                </nav>

                <div class="flex items-center gap-3">
                    <a href="#download" class="btn-primary hidden sm:inline-flex">下载软件</a>
                    <button id="mobileMenuBtn" class="md:hidden p-2 rounded-lg hover:bg-gray-100">
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"/>
                        </svg>
                    </button>
                </div>
            </div>
        </div>

        <div id="mobileMenu" class="hidden md:hidden border-t" style="border-color: var(--border);">
            <div class="max-w-6xl mx-auto px-4 py-3 space-y-1">
                <a href="index.php" class="block px-4 py-2 rounded-lg text-sm font-medium <?php echo basename($_SERVER['PHP_SELF']) == 'index.php' ? 'text-amber-600 bg-amber-50' : 'text-gray-600 hover:bg-gray-50'; ?>">首页</a>
                <a href="announcements.php" class="block px-4 py-2 rounded-lg text-sm font-medium <?php echo basename($_SERVER['PHP_SELF']) == 'announcements.php' ? 'text-amber-600 bg-amber-50' : 'text-gray-600 hover:bg-gray-50'; ?>">公告</a>
                <a href="ads.php" class="block px-4 py-2 rounded-lg text-sm font-medium <?php echo basename($_SERVER['PHP_SELF']) == 'ads.php' ? 'text-amber-600 bg-amber-50' : 'text-gray-600 hover:bg-gray-50'; ?>">推广</a>
                <a href="contact.php" class="block px-4 py-2 rounded-lg text-sm font-medium <?php echo basename($_SERVER['PHP_SELF']) == 'contact.php' ? 'text-amber-600 bg-amber-50' : 'text-gray-600 hover:bg-gray-50'; ?>">联系</a>
            </div>
        </div>
    </header>

    <script>
        document.getElementById('mobileMenuBtn').addEventListener('click', function() {
            document.getElementById('mobileMenu').classList.toggle('hidden');
        });
    </script>
