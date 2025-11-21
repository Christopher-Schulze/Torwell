from playwright.sync_api import sync_playwright
import time

def verify_ui():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        page = browser.new_page()

        # Mock Tauri API
        page.add_init_script("""
            window.__TAURI__ = {
              tauri: {
                invoke: async (cmd, args) => {
                  console.log('Mock invoke:', cmd, args);
                  if (cmd === 'request_token') return 'mock-token';
                  if (cmd === 'load_metrics') {
                    return Array.from({length: 30}, (_, i) => ({
                      time: Date.now() - (30 - i) * 1000,
                      memoryMB: 500 + Math.sin(i) * 50,
                      circuitCount: 5,
                      latencyMs: 100 + Math.random() * 50,
                      oldestAge: 0,
                      avgCreateMs: 200,
                      failedAttempts: 0,
                      cpuPercent: 10 + Math.random() * 5,
                      networkBytes: 1000 + Math.random() * 500,
                      networkTotal: 1000000 + i * 1000,
                      complete: true
                    }));
                  }
                  if (cmd === 'get_status_summary') {
                      return { total_traffic_bytes: 5000000 };
                  }
                  return null;
                }
              },
              event: {
                listen: (event, handler) => {
                    console.log('Mock listen:', event);
                    return Promise.resolve(() => {});
                }
              }
            };
        """)

        # Navigate to Dashboard
        print("Navigating to Dashboard...")
        try:
            page.goto("http://localhost:5173/dashboard", timeout=10000)
            page.wait_for_load_state("networkidle")
            # Wait for content
            page.wait_for_selector("text=SYSTEM RESOURCES", timeout=5000)
            page.screenshot(path="verification/dashboard.png")
            print("Captured verification/dashboard.png")
        except Exception as e:
            print(f"Failed to capture dashboard: {e}")
            page.screenshot(path="verification/dashboard_error.png")

        # Navigate to Network Monitor
        print("Navigating to Network Monitor...")
        try:
            page.goto("http://localhost:5173/network", timeout=10000)
            page.wait_for_load_state("networkidle")
            # Wait for content - NetworkMonitor uses GlassCard now
            page.wait_for_selector("text=CPU Load", timeout=5000)
            page.screenshot(path="verification/network.png")
            print("Captured verification/network.png")
        except Exception as e:
            print(f"Failed to capture network: {e}")
            page.screenshot(path="verification/network_error.png")

        browser.close()

if __name__ == "__main__":
    verify_ui()
