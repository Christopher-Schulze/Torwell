from playwright.sync_api import sync_playwright
import time

def run():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()

        try:
            # Navigate to the app
            page.goto("http://localhost:1420")

            # Wait for load
            page.wait_for_selector(".tw-surface", timeout=5000)

            # Open Settings Modal (ActionCard emits 'openSettings')
            # We need to find the settings button. In ActionCard.svelte it's a button with a Settings icon.
            # It has aria-label="Open settings"
            page.click('button[aria-label="Open settings"]')

            # Wait for modal
            page.wait_for_selector('h2:text("Settings")', timeout=5000)

            # Check for the new Connectivity section
            if page.is_visible("text=Connectivity"):
                print("Connectivity section found")
            else:
                print("Connectivity section NOT found")

            # Check for the toggle
            if page.is_visible("text=System-wide Routing (VPN Mode)"):
                 print("System Routing toggle found")
            else:
                 print("System Routing toggle NOT found")

            # Take screenshot
            page.screenshot(path="verification/settings_modal.png")

        except Exception as e:
            print(f"Error: {e}")
            page.screenshot(path="verification/error.png")
        finally:
            browser.close()

if __name__ == "__main__":
    run()
