import { chromium } from 'playwright';
import fs from 'fs';
import path from 'path';
import crypto from 'crypto';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const MODEL_URL = 'https://sketchfab.com/3d-models/star-wars-low-poly-stormtrooper-7d55b6ca7935440aa59961197ea742ff';
const OUTPUT_DIR = '/c/Users/koosh/Dino/packs/warfare-starwars/assets/raw/sw_stormtrooper_sketchfab_001';
const OUTPUT_FILE = path.join(OUTPUT_DIR, 'source_download.glb');

async function downloadStormtrooper() {
  const browser = await chromium.launch({ headless: false });
  const context = await browser.newContext();
  const page = await context.newPage();

  try {
    console.log('Navigating to Sketchfab model page...');
    await page.goto(MODEL_URL, { waitUntil: 'networkidle' });

    console.log('Waiting for model viewer to load...');
    await page.waitForTimeout(3000);

    console.log('Looking for download button...');

    // Click the download button - try multiple selectors
    let downloadButtonFound = false;

    // Try to find the download button by text or aria-label
    const downloadButtons = await page.locator('[aria-label="Download"]').count();
    if (downloadButtons > 0) {
      console.log('Found download button by aria-label');
      await page.locator('[aria-label="Download"]').first().click();
      downloadButtonFound = true;
    }

    if (!downloadButtonFound) {
      // Try finding by button text
      const buttons = await page.locator('button').all();
      for (const button of buttons) {
        const text = await button.textContent();
        if (text && text.toLowerCase().includes('download')) {
          console.log('Found download button by text:', text);
          await button.click();
          downloadButtonFound = true;
          break;
        }
      }
    }

    if (!downloadButtonFound) {
      console.log('Could not find download button. Showing page state...');
      await page.screenshot({ path: '/tmp/sketchfab-page.png' });
      throw new Error('Download button not found');
    }

    console.log('Waiting for download dialog...');
    await page.waitForTimeout(1000);

    // Look for GLB format option
    console.log('Looking for GLB format option...');

    // Try to click GLB format if it appears as a button/option
    const glbOptions = await page.locator('text=GLB').all();
    if (glbOptions.length > 0) {
      console.log('Found GLB option, clicking...');
      await glbOptions[0].click();
      await page.waitForTimeout(500);
    }

    // Look for and click the final download button in the dialog
    console.log('Looking for final download confirmation...');

    const downloadConfirmButtons = await page.locator('button:has-text("Download")').all();
    if (downloadConfirmButtons.length > 0) {
      const lastButton = downloadConfirmButtons[downloadConfirmButtons.length - 1];
      console.log('Clicking download confirmation button');

      // Handle the download
      const downloadPromise = context.waitForEvent('download');
      await lastButton.click();

      const download = await downloadPromise;
      console.log(`Download started: ${download.suggestedFilename}`);

      // Save to target location
      await download.saveAs(OUTPUT_FILE);
      console.log(`File saved to: ${OUTPUT_FILE}`);
    } else {
      console.log('No download confirmation button found');
    }

  } catch (error) {
    console.error('Error during download:', error.message);
    throw error;
  } finally {
    await browser.close();
  }
}

function calculateSHA256(filePath) {
  const fileBuffer = fs.readFileSync(filePath);
  const hashSum = crypto.createHash('sha256');
  hashSum.update(fileBuffer);
  return hashSum.digest('hex');
}

async function main() {
  try {
    await downloadStormtrooper();

    // Verify file exists and get stats
    if (!fs.existsSync(OUTPUT_FILE)) {
      throw new Error(`Downloaded file not found at ${OUTPUT_FILE}`);
    }

    const stats = fs.statSync(OUTPUT_FILE);
    const fileSizeKB = stats.size / 1024;

    console.log(`\nFile verification:`);
    console.log(`  Size: ${fileSizeKB.toFixed(2)} KB`);

    if (stats.size < 500 * 1024) {
      console.warn(`Warning: File size (${fileSizeKB.toFixed(2)} KB) is less than 500 KB`);
    }

    // Calculate SHA256
    const sha256Hash = calculateSHA256(OUTPUT_FILE);
    console.log(`  SHA256: ${sha256Hash}`);

    // Generate manifest
    const now = new Date().toISOString();
    const manifest = {
      asset_id: 'sw_stormtrooper_sketchfab_001',
      source_url: 'https://sketchfab.com/3d-models/star-wars-low-poly-stormtrooper-7d55b6ca7935440aa59961197ea742ff',
      author_name: 'Oscar RP',
      polycount_estimate: 3222,
      sha256: sha256Hash,
      acquired_at_utc: now,
      technical_status: 'downloaded',
      ip_status: 'fan_star_wars_private_only',
      notes: 'Low-poly ready, ideal for gameplay',
      file_size_bytes: stats.size,
      file_size_kb: parseFloat(fileSizeKB.toFixed(2))
    };

    console.log('\n=== MANIFEST ===');
    console.log(JSON.stringify(manifest, null, 2));

    // Optionally save manifest
    const manifestPath = path.join(OUTPUT_DIR, 'manifest.json');
    fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
    console.log(`\nManifest saved to: ${manifestPath}`);

  } catch (error) {
    console.error('Fatal error:', error.message);
    process.exit(1);
  }
}

main();
