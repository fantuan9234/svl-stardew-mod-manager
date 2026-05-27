<?php
require_once __DIR__ . '/backend/security.php';
require_once __DIR__ . '/backend/db.php';
require_once __DIR__ . '/backend/language.php';

sendSecurityHeaders();
initDatabase();

$pageTitle = '首页';

$db = getDB();
$latestWindows = $db->prepare("SELECT * FROM versions WHERE platform = 'windows' AND is_latest = 1 LIMIT 1");
$latestWindows->execute();
$latestWin = $latestWindows->fetch();
if (!$latestWin) {
    $fallback = $db->prepare("SELECT * FROM versions WHERE platform = 'windows' ORDER BY created_at DESC LIMIT 1");
    $fallback->execute();
    $latestWin = $fallback->fetch();
}

$primaryVersion = $latestWin;
$displayVersion = $primaryVersion ? $primaryVersion['version'] : 'v1.1.0';
$displayDate = $primaryVersion ? date('Y-m-d', strtotime($primaryVersion['created_at'])) : '2026-05-17';
$displayUrl = $primaryVersion ? $primaryVersion['download_url'] : '#';

include 'header.php';
?>

<main class="flex-1">
    <!-- Hero -->
    <section class="relative min-h-[90vh] flex items-center overflow-hidden">
        <canvas id="heroCanvas"></canvas>
        <div class="hero-glow hero-glow-1"></div>
        <div class="hero-glow hero-glow-2"></div>
        <div class="hero-glow hero-glow-3"></div>

        <div class="relative z-10 max-w-6xl mx-auto px-6 w-full py-20">
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
                <!-- Left: Content -->
                <div class="text-center lg:text-left">
                    <div class="animate-fade-up">
                        <!-- 优化：添加 will-change 和硬件加速 -->
                        <img src="assets/icon.png" alt="<?php echo t('site_name'); ?>" class="w-16 h-16 rounded-2xl mx-auto lg:mx-0 mb-6" style="will-change: transform; animation: float 6s ease-in-out infinite;">
                    </div>
                    <h1 class="animate-fade-up delay-1 section-title mb-5 hero-title-animate" style="text-shadow: 0 2px 16px rgba(0,0,0,0.08);">
                        <?php echo t('hero_title_pre'); ?><span class="gradient-text"><?php echo t('hero_title_highlight'); ?></span>
                    </h1>
                    <p class="animate-fade-up delay-2 text-base mb-3" style="color: var(--text-secondary); text-shadow: 0 1px 8px rgba(0,0,0,0.05);">
                        <?php echo t('site_desc'); ?>
                    </p>
                    <p class="animate-fade-up delay-2 text-sm mb-10" style="color: var(--text-tertiary);">
                        <?php echo t('site_subdesc'); ?>
                    </p>
                    <div class="animate-fade-up delay-3 flex flex-col sm:flex-row items-center justify-center lg:justify-start gap-3">
                        <a href="#download" class="btn-primary">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
                            <?php echo t('hero_btn_download'); ?>
                        </a>
                        <a href="https://space.bilibili.com/3546621436496190?spm_id_from=333.40164.0.0" target="_blank" class="btn-ghost" title="B站主页">
                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor"><path d="M17.813 4.653h.854c1.51.054 2.769.578 3.773 1.574 1.004.995 1.524 2.249 1.56 3.76v7.36c-.036 1.51-.556 2.769-1.56 3.773s-2.262 1.524-3.773 1.56H5.333c-1.51-.036-2.769-.556-3.773-1.56S.036 18.858 0 17.347v-7.36c.036-1.511.556-2.765 1.56-3.76 1.004-.996 2.262-1.52 3.773-1.574h.774l-1.174-1.12a1.234 1.234 0 0 1-.373-.906c0-.356.124-.658.373-.907l.027-.027c.267-.249.573-.373.92-.373.347 0 .653.124.92.373L9.653 4.44c.071.071.134.142.187.213h4.267a.836.836 0 0 1 .16-.213l2.853-2.747c.267-.249.573-.373.92-.373.347 0 .662.151.929.4.267.249.391.551.391.907 0 .355-.124.657-.373.906zM5.333 7.24c-.746.018-1.373.276-1.88.773-.506.498-.769 1.13-.786 1.894v7.52c.017.764.28 1.395.786 1.893.507.498 1.134.756 1.88.773h13.334c.746-.017 1.373-.275 1.88-.773.506-.498.769-1.129.786-1.893v-7.52c-.017-.765-.28-1.396-.786-1.894-.507-.497-1.134-.755-1.88-.773zM8 11.107c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c0-.373.129-.689.386-.947.258-.257.574-.386.947-.386zm8 0c.373 0 .684.124.933.373.25.249.383.569.4.96v1.173c-.017.391-.15.711-.4.96-.249.25-.56.374-.933.374s-.684-.125-.933-.374c-.25-.249-.383-.569-.4-.96V12.44c.017-.391.15-.711.4-.96.249-.249.56-.373.933-.373z"/></svg>
                        </a>
                        <a href="https://www.douyin.com/user/self?from_tab_name=main" target="_blank" class="btn-ghost" title="抖音主页">
                            <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor"><path d="M12.525.02c1.31-.02 2.61-.01 3.91-.02.08 1.53.63 3.09 1.75 4.17 1.12 1.11 2.7 1.62 4.24 1.79v4.03c-1.44-.05-2.89-.35-4.2-.97-.57-.26-1.1-.59-1.62-.93-.01 2.92.01 5.84-.02 8.75-.08 1.4-.54 2.79-1.35 3.94-1.31 1.92-3.58 3.17-5.91 3.21-1.43.08-2.86-.31-4.08-1.03-2.02-1.19-3.44-3.37-3.65-5.71-.02-.5-.03-1-.01-1.49.18-1.9 1.12-3.72 2.58-4.96 1.66-1.44 3.98-2.13 6.15-1.72.02 1.48-.04 2.96-.04 4.44-.99-.32-2.15-.23-3.02.37-.63.41-1.11 1.04-1.36 1.75-.21.51-.15 1.07-.14 1.61.24 1.64 1.82 3.02 3.5 2.87 1.12-.01 2.19-.66 2.77-1.61.19-.33.4-.67.41-1.06.1-1.79.06-3.57.07-5.36.01-4.03-.01-8.05.02-12.07z"/></svg>
                        </a>
                    </div>
                </div>
                <!-- Right: Screenshot -->
                <div class="animate-fade-up delay-2 hidden lg:block">
                    <div class="relative group" style="will-change: transform;">
                        <!-- 优化：减少 blur 滤镜强度 -->
                        <div class="absolute -inset-1 rounded-2xl opacity-20 group-hover:opacity-35 transition-opacity duration-300" style="background: linear-gradient(135deg, rgba(212,168,67,0.25) 0%, rgba(0,212,255,0.1) 100%); filter: blur(15px);"></div>
                        <img src="assets/screenshot.png" alt="<?php echo t('site_name'); ?>" loading="lazy" class="relative rounded-xl border transition-transform duration-300 group-hover:scale-[1.02]" style="border-color: rgba(255,255,255,0.08); box-shadow: 0 20px 60px rgba(0,0,0,0.5);">
                    </div>
                </div>
            </div>
        </div>

        <div class="absolute bottom-8 left-1/2 -translate-x-1/2 animate-fade-in delay-5">
            <div class="w-6 h-10 rounded-full border-2 flex justify-center pt-2" style="border-color: var(--text-tertiary);">
                <div class="w-1 h-2 rounded-full animate-bounce" style="background: var(--text-secondary);"></div>
            </div>
        </div>
    </section>

    <!-- Sponsor -->
    <section class="py-20">
        <div class="max-w-6xl mx-auto px-6">
            <div class="text-center mb-8">
                <span class="text-xs font-semibold uppercase tracking-widest" style="color: var(--text-tertiary);"><?php echo t('sponsor_label'); ?></span>
            </div>
            <a href="https://yy.0play.cn/auth/register?ref=REF1330FA2E" target="_blank" rel="noopener" class="card block group relative overflow-hidden sponsor-card" style="text-decoration: none; color: inherit; background: linear-gradient(135deg, #0a1a2e 0%, #112240 50%, #0d1b2a 100%); border-color: rgba(0,212,255,0.08);">
                <div class="absolute inset-0 opacity-30" style="background-image: linear-gradient(rgba(255,255,255,0.02) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.02) 1px, transparent 1px); background-size: 32px 32px;"></div>
                <div class="absolute top-0 right-0 w-[300px] h-[300px] opacity-20" style="background: radial-gradient(circle, rgba(0,212,255,0.15) 0%, transparent 70%);"></div>
                <div class="absolute bottom-0 left-0 w-[250px] h-[250px] opacity-15 sponsor-glow" style="background: radial-gradient(circle, rgba(212,168,67,0.12) 0%, transparent 70%); transition: opacity 0.5s;"></div>

                <div class="relative z-10 flex flex-col md:flex-row items-start md:items-center gap-6">
                    <div class="flex-1">
                        <div class="flex items-center gap-2 mb-3">
                            <span class="tag" style="background: rgba(0,184,148,0.15); color: #00d4ff; font-size: 11px;"><?php echo t('sponsor_tag'); ?></span>
                        </div>
                        <div class="text-xl md:text-2xl font-bold mb-2" style="color: #fff;"><?php echo t('sponsor_title'); ?></div>
                        <p class="text-sm font-medium mb-4" style="color: #00d4ff;"><?php echo t('sponsor_sub'); ?></p>
                        <div class="flex flex-wrap gap-2">
                            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium" style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); color: rgba(255,255,255,0.75);">
                                <svg class="w-3 h-3" fill="none" stroke="#00ff88" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"/></svg>
                                <?php echo t('sponsor_feat_1'); ?>
                            </span>
                            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium" style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); color: rgba(255,255,255,0.75);">
                                <svg class="w-3 h-3" fill="none" stroke="#00ff88" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"/></svg>
                                <?php echo t('sponsor_feat_2'); ?>
                            </span>
                            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium" style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); color: rgba(255,255,255,0.75);">
                                <svg class="w-3 h-3" fill="none" stroke="#00ff88" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"/></svg>
                                <?php echo t('sponsor_feat_3'); ?>
                            </span>
                            <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium" style="background: rgba(255,255,255,0.06); border: 1px solid rgba(255,255,255,0.08); color: rgba(255,255,255,0.75);">
                                <svg class="w-3 h-3" fill="none" stroke="#00ff88" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7"/></svg>
                                <?php echo t('sponsor_feat_4'); ?>
                            </span>
                        </div>
                    </div>
                    <div class="flex items-center gap-2 text-sm font-medium transition-all duration-300 group-hover:translate-x-1" style="color: #00d4ff;">
                        <span><?php echo t('sponsor_link'); ?></span>
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7"/></svg>
                    </div>
                </div>
            </a>
        </div>
    </section>

    <!-- 新增：功能详细介绍 -->
    <section class="py-24" id="features">
        <div class="max-w-6xl mx-auto px-6">
            <div class="text-center mb-16">
                <span class="section-label">Features</span>
                <h2 class="section-title mb-4"><?php echo t('features_title'); ?></h2>
                <p class="section-subtitle mx-auto"><?php echo t('features_subtitle'); ?></p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <!-- 功能 1 -->
                <div class="card feature-card reveal-card" style="will-change: transform;">
                    <div class="card-glow" style="top: -50px; left: -50px;"></div>
                    <div class="w-12 h-12 rounded-xl flex items-center justify-center mb-5 card-icon" style="background: rgba(212,168,67,0.1);">
                        <svg class="w-6 h-6" fill="none" stroke="#d4a843" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"/></svg>
                    </div>
                    <h3 class="font-semibold text-lg mb-2" style="color: var(--text);"><?php echo t('feat_1_title'); ?></h3>
                    <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo t('feat_1_desc'); ?></p>
                </div>

                <!-- 功能 2 -->
                <div class="card feature-card reveal-card" style="will-change: transform;">
                    <div class="card-glow" style="top: -50px; right: -50px;"></div>
                    <div class="w-12 h-12 rounded-xl flex items-center justify-center mb-5 card-icon" style="background: rgba(239,68,68,0.1);">
                        <svg class="w-6 h-6" fill="none" stroke="#ef4444" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"/></svg>
                    </div>
                    <h3 class="font-semibold text-lg mb-2" style="color: var(--text);"><?php echo t('feat_2_title'); ?></h3>
                    <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo t('feat_2_desc'); ?></p>
                </div>

                <!-- 功能 3 -->
                <div class="card feature-card reveal-card" style="will-change: transform;">
                    <div class="card-glow" style="bottom: -50px; left: -50px;"></div>
                    <div class="w-12 h-12 rounded-xl flex items-center justify-center mb-5 card-icon" style="background: rgba(34,197,94,0.1);">
                        <svg class="w-6 h-6" fill="none" stroke="#22c55e" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                    </div>
                    <h3 class="font-semibold text-lg mb-2" style="color: var(--text);"><?php echo t('feat_3_title'); ?></h3>
                    <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo t('feat_3_desc'); ?></p>
                </div>

                <!-- 功能 4 -->
                <div class="card feature-card reveal-card" style="will-change: transform;">
                    <div class="card-glow" style="top: -50px; left: -50px;"></div>
                    <div class="w-12 h-12 rounded-xl flex items-center justify-center mb-5 card-icon" style="background: rgba(59,130,246,0.1);">
                        <svg class="w-6 h-6" fill="none" stroke="#3b82f6" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4"/></svg>
                    </div>
                    <h3 class="font-semibold text-lg mb-2" style="color: var(--text);"><?php echo t('feat_4_title'); ?></h3>
                    <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo t('feat_4_desc'); ?></p>
                </div>

                <!-- 功能 5 -->
                <div class="card feature-card reveal-card" style="will-change: transform;">
                    <div class="card-glow" style="top: -50px; right: -50px;"></div>
                    <div class="w-12 h-12 rounded-xl flex items-center justify-center mb-5 card-icon" style="background: rgba(168,85,247,0.1);">
                        <svg class="w-6 h-6" fill="none" stroke="#a855f7" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"/><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/></svg>
                    </div>
                    <h3 class="font-semibold text-lg mb-2" style="color: var(--text);"><?php echo t('feat_5_title'); ?></h3>
                    <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo t('feat_5_desc'); ?></p>
                </div>

                <!-- 功能 6 -->
                <div class="card feature-card reveal-card" style="will-change: transform;">
                    <div class="card-glow" style="bottom: -50px; right: -50px;"></div>
                    <div class="w-12 h-12 rounded-xl flex items-center justify-center mb-5 card-icon" style="background: rgba(20,184,166,0.1);">
                        <svg class="w-6 h-6" fill="none" stroke="#14b8a6" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M13 10V3L4 14h7v7l9-11h-7z"/></svg>
                    </div>
                    <h3 class="font-semibold text-lg mb-2" style="color: var(--text);"><?php echo t('feat_6_title'); ?></h3>
                    <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo t('feat_6_desc'); ?></p>
                </div>

                <!-- 功能 7 -->
                <div class="card feature-card reveal-card" style="will-change: transform;">
                    <div class="card-glow" style="top: -50px; left: -50px;"></div>
                    <div class="w-12 h-12 rounded-xl flex items-center justify-center mb-5 card-icon" style="background: rgba(250,176,5,0.1);">
                        <svg class="w-6 h-6" fill="none" stroke="#fab005" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z"/></svg>
                    </div>
                    <h3 class="font-semibold text-lg mb-2" style="color: var(--text);"><?php echo t('feat_7_title'); ?></h3>
                    <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo t('feat_7_desc'); ?></p>
                </div>
            </div>
        </div>
    </section>

    <!-- 新增：使用流程 -->
    <section class="py-24" style="background: var(--surface);">
        <div class="max-w-6xl mx-auto px-6">
            <div class="text-center mb-16">
                <span class="section-label">How It Works</span>
                <h2 class="section-title mb-4"><?php echo t('how_title'); ?></h2>
                <p class="section-subtitle mx-auto"><?php echo t('how_subtitle'); ?></p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-8 relative">
                <div class="hidden md:block step-connector"></div>
                <div class="hidden md:block step-connector" style="left: calc(66.66% + 16px);"></div>
                <div class="text-center">
                    <div class="w-16 h-16 rounded-2xl flex items-center justify-center mx-auto mb-6 step-number" style="background: var(--brand-dim); color: var(--brand); font-size: 28px; font-weight: 700; transition: transform 0.3s, box-shadow 0.3s;">1</div>
                    <h3 class="font-semibold text-lg mb-3" style="color: var(--text);"><?php echo t('how_1_title'); ?></h3>
                    <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo t('how_1_desc'); ?></p>
                </div>
                <div class="text-center">
                    <div class="w-16 h-16 rounded-2xl flex items-center justify-center mx-auto mb-6 step-number" style="background: var(--brand-dim); color: var(--brand); font-size: 28px; font-weight: 700; transition: transform 0.3s, box-shadow 0.3s;">2</div>
                    <h3 class="font-semibold text-lg mb-3" style="color: var(--text);"><?php echo t('how_2_title'); ?></h3>
                    <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo t('how_2_desc'); ?></p>
                </div>
                <div class="text-center">
                    <div class="w-16 h-16 rounded-2xl flex items-center justify-center mx-auto mb-6 step-number" style="background: var(--brand-dim); color: var(--brand); font-size: 28px; font-weight: 700; transition: transform 0.3s, box-shadow 0.3s;">3</div>
                    <h3 class="font-semibold text-lg mb-3" style="color: var(--text);"><?php echo t('how_3_title'); ?></h3>
                    <p class="text-sm leading-relaxed" style="color: var(--text-secondary);"><?php echo t('how_3_desc'); ?></p>
                </div>
            </div>
        </div>
    </section>

    <div class="divider max-w-6xl mx-auto"></div>

    <!-- Download CTA -->
    <section id="download" class="py-28">
        <div class="max-w-6xl mx-auto px-6">
            <div class="relative card text-center py-20 overflow-hidden download-card">
                <div class="absolute top-0 left-1/2 -translate-x-1/2 w-[500px] h-[300px] opacity-30" style="background: radial-gradient(ellipse, rgba(212,168,67,0.15) 0%, transparent 70%);"></div>
                <div class="absolute top-6 left-6 w-20 h-20 rounded-full opacity-10 download-orb" style="background: var(--brand); filter: blur(30px); animation: float 6s ease-in-out infinite;"></div>
                <div class="absolute bottom-8 right-8 w-16 h-16 rounded-full opacity-10 download-orb" style="background: #3b82f6; filter: blur(25px); animation: float 8s ease-in-out 2s infinite;"></div>
                <div class="absolute top-1/2 right-[10%] w-1 h-1 rounded-full opacity-30" style="background: var(--brand); box-shadow: 0 0 6px var(--brand);"></div>
                <div class="absolute top-[30%] left-[8%] w-1 h-1 rounded-full opacity-20" style="background: #3b82f6; box-shadow: 0 0 6px #3b82f6;"></div>
                <div class="absolute bottom-[25%] left-[15%] w-1 h-1 rounded-full opacity-25" style="background: var(--brand); box-shadow: 0 0 6px var(--brand);"></div>
                <div class="relative z-10">
                    <h2 class="text-3xl md:text-4xl font-bold mb-3" style="color: var(--text); text-shadow: 0 2px 12px rgba(0,0,0,0.15);"><?php echo t('download_title'); ?></h2>
                    <p class="text-sm mb-8" style="color: var(--text-secondary); text-shadow: 0 1px 8px rgba(0,0,0,0.1);"><?php echo t('download_subtitle'); ?></p>

                    <div class="flex flex-col sm:flex-row items-center justify-center gap-3 mb-4">
                        <?php if ($latestWin && $latestWin['download_url']): ?>
                        <a href="<?php echo h($latestWin['download_url']); ?>" class="btn-primary btn-primary-pulse" id="downloadBtn" data-platform="windows" onclick="recordDownload('windows')">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
                            <span><?php echo t('download_btn_windows'); ?></span>
                        </a>
                        <?php else: ?>
                        <a href="#" class="btn-primary opacity-50 pointer-events-none">
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/></svg>
                            <?php echo t('download_btn_none'); ?>
                        </a>
                        <?php endif; ?>
                    </div>

                    <?php if ($primaryVersion && $primaryVersion['changelog']): ?>
                    <div class="max-w-md mx-auto mt-4 mb-2">
                        <details class="text-left" style="color: var(--text-tertiary);">
                            <summary class="cursor-pointer text-xs hover:text-[var(--text-secondary)] transition-colors"><?php echo t('download_changelog'); ?></summary>
                            <p class="text-xs mt-2 leading-relaxed" style="color: var(--text-secondary);"><?php echo nl2br(h($primaryVersion['changelog'])); ?></p>
                        </details>
                    </div>
                    <?php endif; ?>

                    <p class="text-xs mt-6" style="color: var(--text-secondary); opacity: 0.7;"><?php echo h($displayVersion); ?> &middot; <?php echo t('download_updated'); ?> <?php echo h($displayDate); ?></p>
                </div>
            </div>
        </div>
    </section>
</main>

<?php include 'footer.php'; ?>
