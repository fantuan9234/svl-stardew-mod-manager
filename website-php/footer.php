    <footer role="contentinfo" style="border-top: 1px solid var(--border);">
        <div class="max-w-6xl mx-auto px-6 py-16">
            <div class="grid grid-cols-1 md:grid-cols-12 gap-10">
                <div class="md:col-span-5">
                    <div class="flex items-center gap-3 mb-5">
                        <img src="assets/icon.png" alt="<?php echo t('site_name'); ?>" class="w-7 h-7 rounded-lg">
                        <span class="font-semibold text-[15px]" style="color: var(--text);"><?php echo t('site_name'); ?></span>
                    </div>
                    <p class="text-sm leading-relaxed max-w-sm" style="color: var(--text-secondary);">
                        <?php echo t('footer_desc'); ?>
                    </p>
                </div>
                <div class="md:col-span-3 md:col-start-7">
                    <h4 class="font-semibold text-xs uppercase tracking-widest mb-5" style="color: var(--text-tertiary);"><?php echo t('footer_nav'); ?></h4>
                    <div class="space-y-3">
                        <a href="index.php" class="block text-sm transition-colors duration-300 hover:text-white" style="color: var(--text-secondary); text-decoration: none;" title="<?php echo t('footer_home_title'); ?>"><?php echo t('footer_home_link'); ?></a>
                        <a href="announcements.php" class="block text-sm transition-colors duration-300 hover:text-white" style="color: var(--text-secondary); text-decoration: none;" title="<?php echo t('footer_announcements_title'); ?>"><?php echo t('footer_announcements_link'); ?></a>
                        <a href="changelog.php" class="block text-sm transition-colors duration-300 hover:text-white" style="color: var(--text-secondary); text-decoration: none;" title="<?php echo t('nav_changelog'); ?>"><?php echo t('nav_changelog'); ?></a>
                        <a href="contact.php" class="block text-sm transition-colors duration-300 hover:text-white" style="color: var(--text-secondary); text-decoration: none;" title="<?php echo t('footer_contact_title'); ?>"><?php echo t('footer_contact_link'); ?></a>
                    </div>
                </div>
                <div class="md:col-span-3">
                    <h4 class="font-semibold text-xs uppercase tracking-widest mb-5" style="color: var(--text-tertiary);"><?php echo t('footer_follow'); ?></h4>
                    <div class="space-y-3">
                        <a href="https://space.bilibili.com/3546621436496190?spm_id_from=333.40164.0.0" target="_blank" rel="noopener" class="block text-sm transition-colors duration-300 hover:text-white" style="color: var(--text-secondary); text-decoration: none;">Bilibili</a>
                        <a href="https://www.douyin.com/user/self?from_tab_name=main" target="_blank" rel="noopener" class="block text-sm transition-colors duration-300 hover:text-white" style="color: var(--text-secondary); text-decoration: none;">抖音</a>
                    </div>
                </div>
            </div>
            <div class="divider mt-12 mb-6"></div>
            <div class="flex flex-col items-center gap-3 text-center">
                <div class="flex flex-col sm:flex-row items-center gap-2 sm:gap-6">
                    <p class="text-xs" style="color: var(--text-tertiary);">
                        &copy; <?php echo date('Y'); ?> <?php echo t('footer_copyright'); ?>
                    </p>
                    <p class="text-xs" style="color: var(--text-tertiary);">
                        <?php echo t('footer_disclaimer'); ?>
                    </p>
                </div>
                <p class="text-xs">
                    <a href="https://beian.miit.gov.cn/" target="_blank" rel="noopener noreferrer" style="color: var(--text-secondary); text-decoration: none; transition: color 0.3s ease;">陕ICP备2026008479号-2</a>
                </p>
            </div>
        </div>
    </footer>
    <script>
    (function(){
        var bp = document.createElement('script');
        var curProtocol = window.location.protocol.split(':')[0];
        if (curProtocol === 'https') {
            bp.src = 'https://zz.bdstatic.com/linksubmit/push.js';
        } else {
            bp.src = 'http://push.zhanzhang.baidu.com/push.js';
        }
        var s = document.getElementsByTagName('script')[0];
        s.parentNode.insertBefore(bp, s);
    })();

    // Visitor tracking
    (function(){
        var page = window.location.pathname.split('/').pop() || 'index.php';
        try {
            var xhr = new XMLHttpRequest();
            xhr.open('POST', 'api/index.php?action=track', true);
            xhr.setRequestHeader('Content-Type', 'application/json');
            xhr.send(JSON.stringify({page: page}));
        } catch(e) {}
    })();

    // Download tracking
    function recordDownload(platform) {
        try {
            var data = JSON.stringify({platform: platform});
            if (navigator.sendBeacon) {
                navigator.sendBeacon('api/index.php?action=download', new Blob([data], {type: 'application/json'}));
            } else {
                var xhr = new XMLHttpRequest();
                xhr.open('POST', 'api/index.php?action=download', false);
                xhr.setRequestHeader('Content-Type', 'application/json');
                xhr.send(data);
            }
        } catch(e) {}
    }

    // Hero particle canvas
    (function(){
        var canvas = document.getElementById('heroCanvas');
        if (!canvas) return;
        var ctx = canvas.getContext('2d');
        var particles = [];
        var PARTICLE_COUNT = 60;

        function resize() {
            var section = canvas.parentElement;
            canvas.width = section.offsetWidth;
            canvas.height = section.offsetHeight;
        }
        resize();
        window.addEventListener('resize', resize);

        for (var i = 0; i < PARTICLE_COUNT; i++) {
            particles.push({
                x: Math.random() * canvas.width,
                y: Math.random() * canvas.height,
                r: Math.random() * 1.5 + 0.3,
                dx: (Math.random() - 0.5) * 0.3,
                dy: (Math.random() - 0.5) * 0.3,
                alpha: Math.random() * 0.5 + 0.1
            });
        }

        function draw() {
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            for (var i = 0; i < particles.length; i++) {
                var p = particles[i];
                p.x += p.dx;
                p.y += p.dy;
                if (p.x < 0) p.x = canvas.width;
                if (p.x > canvas.width) p.x = 0;
                if (p.y < 0) p.y = canvas.height;
                if (p.y > canvas.height) p.y = 0;

                ctx.beginPath();
                ctx.arc(p.x, p.y, p.r, 0, Math.PI * 2);
                ctx.fillStyle = 'rgba(212,168,67,' + p.alpha + ')';
                ctx.fill();

                for (var j = i + 1; j < particles.length; j++) {
                    var p2 = particles[j];
                    var dist = Math.sqrt((p.x - p2.x) * (p.x - p2.x) + (p.y - p2.y) * (p.y - p2.y));
                    if (dist < 120) {
                        ctx.beginPath();
                        ctx.moveTo(p.x, p.y);
                        ctx.lineTo(p2.x, p2.y);
                        ctx.strokeStyle = 'rgba(212,168,67,' + (0.06 * (1 - dist / 120)) + ')';
                        ctx.lineWidth = 0.5;
                        ctx.stroke();
                    }
                }
            }
            requestAnimationFrame(draw);
        }
        draw();
    })();

    // Reveal cards on scroll
    (function(){
        var cards = document.querySelectorAll('.reveal-card');
        if (!cards.length) return;
        var observer = new IntersectionObserver(function(entries) {
            entries.forEach(function(entry) {
                if (entry.isIntersecting) {
                    var idx = Array.prototype.indexOf.call(cards, entry.target);
                    entry.target.style.transitionDelay = (idx * 0.1) + 's';
                    entry.target.classList.add('revealed');
                    observer.unobserve(entry.target);
                }
            });
        }, { threshold: 0.1, rootMargin: '0px 0px -40px 0px' });
        cards.forEach(function(card) { observer.observe(card); });
    })();

    // Scroll progress bar
    (function(){
        var bar = document.createElement('div');
        bar.id = 'scrollProgress';
        bar.style.cssText = 'position:fixed;top:0;left:0;height:2px;background:var(--brand);z-index:9998;width:0;transition:none;pointer-events:none;';
        document.body.appendChild(bar);
        window.addEventListener('scroll', function(){
            var scrollTop = document.documentElement.scrollTop || document.body.scrollTop;
            var scrollHeight = document.documentElement.scrollHeight - document.documentElement.clientHeight;
            var progress = scrollHeight > 0 ? (scrollTop / scrollHeight) * 100 : 0;
            bar.style.width = progress + '%';
        }, {passive: true});
    })();

    // Back to top button
    (function(){
        var btn = document.createElement('button');
        btn.id = 'backToTop';
        btn.setAttribute('aria-label', '返回顶部');
        btn.innerHTML = '<svg width="18" height="18" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 15l7-7 7 7"/></svg>';
        btn.style.cssText = 'position:fixed;bottom:32px;right:32px;width:44px;height:44px;border-radius:12px;border:1px solid var(--border);background:var(--surface);color:var(--text-secondary);cursor:pointer;display:flex;align-items:center;justify-content:center;opacity:0;transform:translateY(16px);transition:opacity 0.3s,transform 0.3s,background 0.2s;pointer-events:none;z-index:100;box-shadow:0 4px 16px rgba(0,0,0,0.15);';
        document.body.appendChild(btn);
        btn.addEventListener('mouseenter', function(){ btn.style.background = 'var(--brand)'; btn.style.color = '#fff'; btn.style.borderColor = 'var(--brand)'; });
        btn.addEventListener('mouseleave', function(){ btn.style.background = 'var(--surface)'; btn.style.color = 'var(--text-secondary)'; btn.style.borderColor = 'var(--border)'; });
        btn.addEventListener('click', function(){ window.scrollTo({top:0,behavior:'smooth'}); });
        window.addEventListener('scroll', function(){
            var show = (document.documentElement.scrollTop || document.body.scrollTop) > 400;
            btn.style.opacity = show ? '1' : '0';
            btn.style.transform = show ? 'translateY(0)' : 'translateY(16px)';
            btn.style.pointerEvents = show ? 'auto' : 'none';
        }, {passive: true});
    })();

    // Header scroll effect
    (function(){
        var header = document.getElementById('siteHeader');
        var inner = document.getElementById('headerInner');
        if (!header || !inner) return;
        window.addEventListener('scroll', function(){
            var scrolled = (document.documentElement.scrollTop || document.body.scrollTop) > 20;
            header.style.borderBottomColor = scrolled ? 'var(--border)' : 'transparent';
            header.style.boxShadow = scrolled ? '0 1px 24px rgba(0,0,0,0.08)' : 'none';
            inner.style.height = scrolled ? '56px' : '68px';
        }, {passive: true});
    })();

    // Hero title stagger animation
    (function(){
        var heroTitle = document.querySelector('.hero-title-animate');
        if (!heroTitle) return;
        var text = heroTitle.textContent;
        heroTitle.innerHTML = '';
        var chars = text.split('');
        chars.forEach(function(ch, i){
            var span = document.createElement('span');
            span.textContent = ch === ' ' ? '\u00A0' : ch;
            span.style.cssText = 'display:inline-block;opacity:0;transform:translateY(20px);animation:heroCharIn 0.5s cubic-bezier(0.4,0,0.2,1) forwards;animation-delay:' + (0.6 + i * 0.04) + 's;';
            heroTitle.appendChild(span);
        });
        if (!document.getElementById('heroCharStyle')) {
            var style = document.createElement('style');
            style.id = 'heroCharStyle';
            style.textContent = '@keyframes heroCharIn{to{opacity:1;transform:translateY(0)}}';
            document.head.appendChild(style);
        }
    })();
    </script>
</body>
</html>
