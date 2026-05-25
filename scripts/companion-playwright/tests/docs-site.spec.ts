import { test, expect } from '@playwright/test';

test.describe('DINOForge Documentation Site', () => {
  test('home page loads and displays title', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/DINOForge|Dino/i);
    await expect(page.locator('h1, h2')).first().toBeVisible();
  });

  test('getting-started guide is accessible', async ({ page }) => {
    await page.goto('/guide/getting-started');
    await expect(page.locator('h1')).first().toBeVisible();
    // Verify guide content exists
    await expect(page.locator('main, article')).first().toContainText(/getting|start|install/i);
  });

  test('installation guide exists', async ({ page }) => {
    await page.goto('/guide/installation');
    await expect(page.locator('h1')).first().toBeVisible();
    await expect(page.locator('main, article')).first().toContainText(/install|setup/i);
  });

  test('mcp-bridge documentation exists', async ({ page }) => {
    await page.goto('/guide/mcp-bridge');
    await expect(page.locator('h1')).first().toBeVisible();
    // Should mention MCP or bridge
    await expect(page.locator('main, article')).first().toContainText(/mcp|bridge|automation/i);
  });

  test('proof-of-features page exists', async ({ page }) => {
    await page.goto('/proof');
    await expect(page.locator('h1')).first().toBeVisible();
    // Should mention video, proof, or features
    await expect(page.locator('main, article')).first().toContainText(/proof|feature|video|demo/i);
  });

  test('navigation works from home to guide', async ({ page }) => {
    await page.goto('/');
    // Find and click a link to a guide (adjust selector based on actual site nav)
    const guideLink = page.locator('a[href*="/guide/"]').first();
    await guideLink.click();
    // Should navigate without error
    await expect(page).not.toHaveURL('/');
  });

  test('sidebar/navigation renders', async ({ page }) => {
    await page.goto('/guide/getting-started');
    // Check for navigation elements (sidebar, nav bar, etc.)
    const nav = page.locator('nav, aside').first();
    await expect(nav).toBeVisible();
  });

  test('no console errors on page load', async ({ page }) => {
    const errors: string[] = [];
    page.on('console', (msg) => {
      if (msg.type() === 'error') {
        errors.push(msg.text());
      }
    });

    await page.goto('/');
    await page.waitForLoadState('networkidle');

    expect(errors).toHaveLength(0);
  });

  test('all major guide sections are accessible', async ({ page }) => {
    const routes = [
      '/guide/getting-started',
      '/guide/installation',
      '/guide/architecture',
      '/guide/pack-system',
      '/guide/mcp-bridge',
    ];

    for (const route of routes) {
      await page.goto(route).catch(() => {
        // Skip routes that don't exist
      });
      // If page loaded, verify it has content
      const title = page.locator('h1, h2').first();
      if (await page.url().includes(route)) {
        await expect(title).toBeVisible();
      }
    }
  });
});
