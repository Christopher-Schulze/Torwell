from playwright.sync_api import Page, expect, sync_playwright
import time

def verify_design_showcase(page: Page):
    # 1. Arrange: Go to the design showcase page
    # Using local dev server port
    page.goto("http://localhost:5173/design-showcase")

    # 2. Act: Wait for animations to settle
    # The showcase has staggered animations (up to 800ms delay + spring physics)
    # We wait a bit longer to ensure everything is visible
    time.sleep(2)

    # 3. Assert: Check for key elements
    # Check for the main title
    expect(page.get_by_text("Torwell.84")).to_be_visible()

    # Check for the "Tor Connection" card
    expect(page.get_by_text("Tor Connection")).to_be_visible()

    # Check for the "Identity Control" card
    expect(page.get_by_text("Identity Control")).to_be_visible()

    # Check for the "Disconnect" button
    expect(page.get_by_role("button", name="Disconnect")).to_be_visible()

    # 4. Screenshot
    page.screenshot(path="verification/design_showcase.png")

if __name__ == "__main__":
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()
        try:
            verify_design_showcase(page)
        finally:
            browser.close()
