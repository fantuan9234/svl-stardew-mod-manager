(function() {
    console.log('[SVL] Auto-download script v45 loaded');

    var TARGET_MOD_ID = "SVL_TARGET_MOD_ID";
    var TARGET_FILE_ID = "SVL_TARGET_FILE_ID";
    var STORAGE_KEY = 'svl_dl_' + TARGET_MOD_ID + '_' + TARGET_FILE_ID;

    function svlLog(msg) { console.log('[SVL] ' + msg); }

    function svlIsVisible(el) {
        if (!el) return false;
        try {
            var style = window.getComputedStyle(el);
            if (style.display === 'none' || style.visibility === 'hidden') return false;
            if (parseFloat(style.opacity) <= 0) return false;
            var rect = el.getBoundingClientRect();
            if (rect.width <= 0 || rect.height <= 0) return false;
            if (rect.right <= 0 || rect.bottom <= 0) return false;
            if (rect.left >= window.innerWidth || rect.top >= window.innerHeight) return false;
            return true;
        } catch(e) { return false; }
    }

    function svlUpdateStatus(text, color) {
    }

    function svlClickElement(el, reason) {
        var text = (el.textContent || '').trim().substring(0, 40);
        var id = el.id || '';
        var cls = (el.className && typeof el.className === 'string') ? el.className.substring(0, 30) : '';
        var href = (el.getAttribute('href') || '').substring(0, 60);
        svlLog('CLICK: ' + reason + ' [' + el.tagName + '#' + id + '.' + cls + '] "' + text + '" href=' + href);
        el.scrollIntoView({ behavior: 'instant', block: 'center' });
        setTimeout(function() {
            try { el.click(); } catch(e) {}
            try { el.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true, view: window })); } catch(e2) {}
            try { el.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true, view: window })); } catch(e3) {}
            try { el.dispatchEvent(new MouseEvent('mouseup', { bubbles: true, cancelable: true, view: window })); } catch(e4) {}
        }, 500);
        return true;
    }

    function detectPhaseFromURL() {
        var path = window.location.pathname;
        var query = window.location.search;
        if (path.indexOf('/download') >= 0 || path.indexOf('DownloadPopUp') >= 0) return 2;
        if (query.indexOf('file_id=') >= 0) {
            var dl = document.querySelector('MOD-FILE-DOWNLOAD');
            if (dl && dl.shadowRoot && svlIsVisible(dl)) return 2;
            return 1;
        }
        if (path.indexOf('/mods/' + TARGET_MOD_ID) >= 0) return 1;
        return 1;
    }

    function svlDetectCaptcha() {
        if (document.querySelector('.cf-turnstile, [class*="turnstile"], iframe[src*="turnstile"]')) return true;
        if (document.querySelector('.h-captcha, iframe[src*="hcaptcha"], iframe[src*="h-captcha"]')) return true;
        if (document.querySelector('.g-recaptcha, iframe[src*="recaptcha"], iframe[src*="google.com/recaptcha"]')) return true;
        if (document.querySelector('#challenge-stage, [id*="challenge"], [class*="challenge"]')) return true;
        var iframes = document.querySelectorAll('iframe');
        for (var i = 0; i < iframes.length; i++) {
            var src = (iframes[i].getAttribute('src') || '').toLowerCase();
            if (src.indexOf('captcha') >= 0 || src.indexOf('challenge') >= 0 || src.indexOf('verify') >= 0) return true;
        }
        return false;
    }

    function svlLoadState() {
        try {
            var saved = sessionStorage.getItem(STORAGE_KEY);
            if (saved) {
                var parsed = JSON.parse(saved);
                var detected = detectPhaseFromURL();
                var effectivePhase = Math.max(parsed.phase, detected);
                svlLog('LoadState: saved=' + parsed.phase + ' url=' + detected + ' eff=' + effectivePhase);
                return {
                    phase: effectivePhase,
                    manualClicked: parsed.manualClicked || effectivePhase >= 2,
                    startTime: parsed.startTime || Date.now()
                };
            }
        } catch(e) {}
        return null;
    }

    function svlSaveState() {
        try { sessionStorage.setItem(STORAGE_KEY, JSON.stringify(svlState)); } catch(e) {}
    }

    var savedState = svlLoadState();
    var initPhase = savedState ? savedState.phase : detectPhaseFromURL();

    var svlState = {
        phase: initPhase,
        manualClicked: savedState ? savedState.manualClicked : (initPhase >= 2),
        downloadStarted: false,
        lastPath: window.location.pathname,
        phaseStartTime: Date.now(),
        startTime: savedState ? savedState.startTime : Date.now(),
        p2Retries: 0,
        didDump: false,
        captchaPaused: false,
        phaseRetryCount: 0
    };

    function advancePhase(newPhase) {
        svlState.phase = newPhase;
        svlState.phaseStartTime = Date.now();
        svlState.p2Retries = 0;
        svlState.didDump = false;
        svlState.phaseRetryCount = 0;
        if (newPhase >= 2) svlState.manualClicked = true;
        svlSaveState();
        var names = ['', '找Manual', '找SlowDownload'];
        svlUpdateStatus('<b>SVL</b> v45<br>P' + newPhase + ': ' + (names[newPhase] || ''));
        svlLog('>>> Phase -> ' + newPhase);
    }

    function svlInjectUI() {
    }

    function svlShowDebug(title, lines) {
    }

    function svlIsNavOrMenu(el) {
        var p = el;
        for (var d = 0; d < 8 && p && p !== document.body; d++) {
            var tag = p.tagName ? p.tagName.toLowerCase() : '';
            var prole = (p.getAttribute('role') || '').toLowerCase();
            // 只过滤明确的导航元素
            if (tag === 'nav' || tag === 'header') return true;
            if (prole === 'navigation') return true;
            p = p.parentElement;
        }
        return false;
    }

    function svlCollectShadowRoots(root, results) {
        var els = root.querySelectorAll('*');
        for (var i = 0; i < els.length; i++) {
            var el = els[i];
            if (el.shadowRoot) {
                results.push({host: el, shadowRoot: el.shadowRoot});
                svlCollectShadowRoots(el.shadowRoot, results);
            }
        }
    }

    function svlElemVis(el) {
        try {
            var style = window.getComputedStyle(el);
            if (style.display === 'none') return 'disp:none';
            if (style.visibility === 'hidden') return 'vis:hidden';
            if (parseFloat(style.opacity) <= 0) return 'opacity:0';
            var rect = el.getBoundingClientRect();
            if (rect.width <= 0 || rect.height <= 0) return 'size:0';
            if (rect.right <= 0 || rect.bottom <= 0) return 'offscreen-L/T';
            if (rect.left >= window.innerWidth || rect.top >= window.innerHeight) return 'offscreen-R/B';
            return 'VIS';
        } catch(e) { return 'err'; }
    }

    function svlFormatEl(el, idx, maxLen) {
        var dt = (el.textContent || '').trim().toLowerCase();
        var cls = (el.className && typeof el.className === 'string') ? el.className.substring(0, 40) : '';
        var dh = (el.getAttribute('href') || '').substring(0, 60);
        var vis = svlElemVis(el);
        var line = idx + '. [' + vis + '] [' + el.tagName;
        if (cls) line += '.' + cls;
        line += '] "' + dt.substring(0, maxLen || 80) + '"';
        if (dh) line += ' | href=' + dh;
        return line;
    }

    function svlDumpElements(phaseName) {
        var lines = [];
        lines.push('URL: ' + window.location.href.substring(0, 120));
        var keywords = ['slow', 'download', 'free', 'standard', 'mbit', 'mb/s', 'kbps', 'mbps'];
        var els = document.querySelectorAll('*');
        var dumped = 0;
        for (var d = 0; d < els.length && dumped < 200; d++) {
            var de = els[d];
            var dt = (de.textContent || '').trim().toLowerCase();
            if (dt.length < 2 || dt.length > 300) continue;
            var matched = false;
            for (var k = 0; k < keywords.length; k++) {
                if (dt.indexOf(keywords[k]) >= 0) { matched = true; break; }
            }
            if (!matched) continue;
            lines.push(svlFormatEl(de, dumped + 1, 80));
            dumped++;
        }
        if (dumped === 0) lines.push('(主DOM: 无任何元素含slow/download/mbit等关键词!)');

        var shadowRoots = [];
        svlCollectShadowRoots(document, shadowRoots);
        if (shadowRoots.length > 0) {
            lines.push('');
            lines.push('--- Shadow DOM (' + shadowRoots.length + '个shadow root) ---');
            for (var sr = 0; sr < shadowRoots.length; sr++) {
                var host = shadowRoots[sr].host;
                var hostTag = host.tagName;
                var hostCls = (host.className && typeof host.className === 'string') ? host.className.substring(0, 40) : '';
                var hostId = host.id || '';
                var hostVis = svlElemVis(host);
                lines.push('  Shadow#' + sr + ' host=[' + hostVis + '] [' + hostTag + (hostId ? '#' + hostId : '') + (hostCls ? '.' + hostCls : '') + ']');

                var srEls = shadowRoots[sr].shadowRoot.querySelectorAll('*');
                var srDumped = 0;
                for (var srd = 0; srd < srEls.length && srDumped < 50; srd++) {
                    var sre = srEls[srd];
                    var srt = (sre.textContent || '').trim().toLowerCase();
                    if (srt.length < 2 || srt.length > 300) continue;
                    var matched = false;
                    for (var k2 = 0; k2 < keywords.length; k2++) {
                        if (srt.indexOf(keywords[k2]) >= 0) { matched = true; break; }
                    }
                    if (!matched) continue;
                    lines.push('    ' + svlFormatEl(sre, srDumped + 1, 60));
                    srDumped++;
                }
                if (srDumped === 0) {
                    lines.push('    (无含关键词元素, 全部' + srEls.length + '个shadow子元素:)');
                    var shown = 0;
                    for (var srd2 = 0; srd2 < srEls.length && shown < 30; srd2++) {
                        var sre2 = srEls[srd2];
                        var srt2 = (sre2.textContent || '').trim();
                        if (srt2.length < 1) continue;
                        var stag = sre2.tagName;
                        var scls2 = (sre2.className && typeof sre2.className === 'string') ? sre2.className.substring(0, 30) : '';
                        lines.push('    [' + stag + (scls2 ? '.' + scls2 : '') + '] "' + srt2.substring(0, 50) + '"');
                        shown++;
                    }
                }
            }
        } else {
            lines.push('');
            lines.push('(无Shadow DOM)');
        }

        svlShowDebug('=== ' + phaseName + ' v45 dump ===', lines);
        return lines;
    }

    // ===== Phase 1: 找 Manual 按钮 =====
    function p1_findManual() {
        var phaseTime = Date.now() - svlState.phaseStartTime;

        if (phaseTime > 15000 && !svlState.didDump) {
            svlState.didDump = true;
            window.scrollTo(0, document.body.scrollHeight / 3);
            setTimeout(function() { svlDumpElements('P1'); }, 1000);
        }

        var els = document.querySelectorAll('a, button, [role="button"], .btn, [class*="btn"]');
        for (var i = 0; i < els.length; i++) {
            var el = els[i];
            if (!svlIsVisible(el)) continue;
            if (svlIsNavOrMenu(el)) continue;
            var text = (el.textContent || '').trim();
            var clean = text.replace(/[^a-zA-Z0-9]/g, ' ').replace(/\s+/g, ' ').trim().toLowerCase();
            if (clean.indexOf('vortex') >= 0) continue;
            if (clean.indexOf('manual') >= 0) {
                var href = el.getAttribute('href');
                if (href && href.indexOf('ModRequirementsPopUp') >= 0) {
                    svlLog('P1: navigating to popup URL: ' + href);
                    svlUpdateStatus('<b>SVL</b> v45<br>导航到弹窗...', 'rgba(200,150,0,0.9)');
                    svlState.manualClicked = true;
                    svlSaveState();
                    window.location.href = href;
                    return true;
                }
                svlClickElement(el, 'Manual-click');
                svlState.manualClicked = true;
                svlSaveState();
                advancePhase(2);
                return true;
            }
        }
        return false;
    }

    // ===== Phase 2: Shadow DOM穿透搜索 Slow Download =====
    function svlSearchAllDOM(keywords, excludeWords) {
        function checkEl(el) {
            if (!svlIsVisible(el)) return false;
            var text = (el.textContent || '').trim().toLowerCase();
            if (text.length < 2 || text.length > 200) return false;
            var href = (el.getAttribute('href') || '').toLowerCase();
            for (var ew = 0; ew < excludeWords.length; ew++) {
                if (text.indexOf(excludeWords[ew]) >= 0 || href.indexOf(excludeWords[ew]) >= 0) return false;
            }
            for (var kw = 0; kw < keywords.length; kw++) {
                if (text.indexOf(keywords[kw]) >= 0) return true;
            }
            var aria = (el.getAttribute('aria-label') || '').toLowerCase();
            for (var ak = 0; ak < keywords.length; ak++) {
                if (aria.indexOf(keywords[ak]) >= 0) return true;
            }
            return false;
        }

        var mainEls = document.querySelectorAll('*');
        for (var i = 0; i < mainEls.length && i < 3000; i++) {
            if (svlIsNavOrMenu(mainEls[i])) continue;
            if (checkEl(mainEls[i])) return mainEls[i];
        }

        var shadowRoots = [];
        svlCollectShadowRoots(document, shadowRoots);
        for (var s = 0; s < shadowRoots.length; s++) {
            var srEls = shadowRoots[s].shadowRoot.querySelectorAll('*');
            for (var j = 0; j < srEls.length; j++) {
                if (checkEl(srEls[j])) return srEls[j];
            }
        }

        return null;
    }

    function p2_findSlowDownload() {
        var phaseTime = Date.now() - svlState.phaseStartTime;

        if (phaseTime > 5000 && !svlState.didDump) {
            svlState.didDump = true;
            svlDumpElements('P2');
        }

        svlState.p2Retries++;

        // 策略1: 直接找 MOD-FILE-DOWNLOAD shadow DOM 里的 slow download
        var dl = document.querySelector('MOD-FILE-DOWNLOAD');
        if (dl && dl.shadowRoot && svlIsVisible(dl)) {
            var srBtns = dl.shadowRoot.querySelectorAll('button, a, [role="button"]');
            for (var b = 0; b < srBtns.length; b++) {
                var btn = srBtns[b];
                if (!svlIsVisible(btn)) continue;
                var bt = (btn.textContent || '').trim().toLowerCase();
                if (bt.indexOf('slow') >= 0 && bt.indexOf('download') >= 0) {
                    svlClickElement(btn, 'SlowDL-Shadow');
                    svlUpdateStatus('<b>SVL</b><br>慢速下载!', 'rgba(0,184,148,0.9)');
                    svlState.downloadStarted = true;
                    svlSaveState();
                    return true;
                }
            }
        }

        // 策略2: 全DOM+Shadow搜索 slow download
        var found = svlSearchAllDOM(
            ['slow download', 'free download', 'standard download', 'basic download'],
            ['fast', 'premium', 'supporter', 'member', 'history']
        );
        if (found) {
            svlClickElement(found, 'SlowDL-AllDOM');
            svlUpdateStatus('<b>SVL</b><br>慢速下载!', 'rgba(0,184,148,0.9)');
            svlState.downloadStarted = true;
            svlSaveState();
            return true;
        }

        // 策略3: desperate - 搜所有含slow的元素
        if (svlState.p2Retries >= 8) {
            svlLog('P2 desperate mode');
            var shadowRoots = [];
            svlCollectShadowRoots(document, shadowRoots);
            var allContexts = [{root: document, label: 'main'}];
            for (var s = 0; s < shadowRoots.length; s++) {
                allContexts.push({root: shadowRoots[s].shadowRoot, label: 'shadow#' + s});
            }
            for (var c = 0; c < allContexts.length; c++) {
                var ctxEls = allContexts[c].root.querySelectorAll('*');
                for (var j = 0; j < ctxEls.length && j < 2000; j++) {
                    var el = ctxEls[j];
                    if (!svlIsVisible(el)) continue;
                    var t = (el.textContent || '').trim().toLowerCase();
                    if (t.indexOf('slow') >= 0 && t.indexOf('download') >= 0) {
                        svlClickElement(el, 'Desperate-' + allContexts[c].label);
                        svlState.downloadStarted = true;
                        svlSaveState();
                        svlUpdateStatus('<b>SVL</b><br>慢速下载!', 'rgba(0,184,148,0.9)');
                        return true;
                    }
                }
            }
        }

        return false;
    }

    function svlMainLoop() {
        if (svlState.downloadStarted) return;

        var hasCaptcha = svlDetectCaptcha();
        if (hasCaptcha && !svlState.captchaPaused) {
            svlState.captchaPaused = true;
            svlState.phaseStartTime = Date.now();
            svlUpdateStatus('<b>SVL</b> v45<br><span style="color:#ff0">等待验证...</span>', 'rgba(200,150,0,0.9)');
            svlLog('CAPTCHA detected, pausing...');
        }
        if (!hasCaptcha && svlState.captchaPaused) {
            svlState.captchaPaused = false;
            svlState.phaseStartTime = Date.now();
            svlLog('CAPTCHA resolved, resuming...');
            var names = ['', '找Manual', '找SlowDownload'];
            svlUpdateStatus('<b>SVL</b> v45<br>P' + svlState.phase + ': ' + (names[svlState.phase] || ''));
        }

        var currentPath = window.location.pathname;
        if (currentPath !== svlState.lastPath) {
            svlLog('URL: ' + svlState.lastPath.substring(0, 50) + ' -> ' + currentPath.substring(0, 50));
            svlState.lastPath = currentPath;
        }

        var detected = detectPhaseFromURL();
        if (detected > svlState.phase) {
            svlLog('Phase auto-detect: ' + svlState.phase + ' -> ' + detected);
            advancePhase(detected);
        }

        if (!svlState.captchaPaused) {
            var phaseTime = Date.now() - svlState.phaseStartTime;
            var phaseTimeout = (svlState.phase === 1) ? 60000 : 60000;
            if (phaseTime > phaseTimeout) {
                svlState.phaseRetryCount++;
                svlLog('Phase ' + svlState.phase + ' timeout, retry #' + svlState.phaseRetryCount);
                advancePhase(svlState.phase);
            }
        }

        var names = ['', 'Manual', 'SlowDL'];
        svlLog('LOOP P' + svlState.phase + '(' + names[svlState.phase] + ') path=' + currentPath.substring(0, 30) +
               (svlState.captchaPaused ? ' [验证中]' : ''));

        switch (svlState.phase) {
            case 1: p1_findManual(); break;
            case 2: p2_findSlowDownload(); break;
        }

        setTimeout(svlMainLoop, 2000);
    }

    function svlInit() {
        setTimeout(svlInjectUI, 500);
        svlState.lastPath = window.location.pathname;
        svlLog('INIT mod=' + TARGET_MOD_ID + ' file=' + TARGET_FILE_ID + ' phase=' + svlState.phase + ' url=' + window.location.pathname.substring(0, 50));
        setTimeout(svlMainLoop, 8000);
    }

    if (document.readyState === 'complete' || document.readyState === 'interactive') {
        svlInit();
    } else {
        document.addEventListener('DOMContentLoaded', svlInit);
    }
})();