from playwright.sync_api import sync_playwright

with sync_playwright() as p:
    browser = p.chromium.launch(headless=True)
    page = browser.new_page(viewport={'width': 1440, 'height': 900})
    page.goto('http://localhost:8080')
    page.wait_for_load_state('networkidle')
    page.wait_for_timeout(2000)

    # Full page screenshot
    page.screenshot(path='screenshot_full.png', full_page=True)

    # Hero section screenshot
    page.screenshot(path='screenshot_hero.png')

    # Scroll to features
    page.evaluate('window.scrollTo(0, 800)')
    page.wait_for_timeout(500)
    page.screenshot(path='screenshot_features.png')

    # Scroll to showcase
    page.evaluate('window.scrollTo(0, 1800)')
    page.wait_for_timeout(500)
    page.screenshot(path='screenshot_showcase.png')

    # Scroll to download
    page.evaluate('window.scrollTo(0, document.body.scrollHeight)')
    page.wait_for_timeout(500)
    page.screenshot(path='screenshot_download.png')

    browser.close()
    print('Screenshots saved!')
