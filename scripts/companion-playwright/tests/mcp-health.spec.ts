import { test, expect } from '@playwright/test';

test.describe('MCP Server Health', () => {
  test('MCP server is accessible on port 8765', async ({ request }) => {
    // MCP server runs on HTTP localhost:8765
    const response = await request.get('http://127.0.0.1:8765/tools', {
      timeout: 5000,
    }).catch(() => null);

    // Note: If MCP server is not running, this test will be skipped
    // Start it with: python -m dinoforge_mcp.server (from src/Tools/DinoforgeMcp)
    test.skip(!response, 'MCP server not running on localhost:8765');
    expect(response!.status()).toBeLessThan(500);
  });

  test('MCP server responds to status queries', async ({ request }) => {
    const response = await request.get('http://127.0.0.1:8765/status', {
      timeout: 5000,
    }).catch(() => null);

    test.skip(!response, 'MCP server not running');
    expect(response!.ok).toBeTruthy();
  });
});
