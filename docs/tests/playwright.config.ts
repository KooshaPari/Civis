import { defineConfig, devices } from '@playwright/test'
import { DOCS_PORT } from '../.vitepress/constants.mjs'

/** Dedicated port so docs E2E does not collide with the main frontend on 5173. */
const docsBaseURL = process.env.BASE_URL || `http://localhost:${DOCS_PORT}`

export default defineConfig({
  testDir: './e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: process.env.CI ? 'list' : 'html',
  use: {
    baseURL: docsBaseURL,
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: `vitepress dev --port ${DOCS_PORT}`,
    cwd: '..',
    url: docsBaseURL,
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },
})
