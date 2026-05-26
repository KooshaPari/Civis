const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

async function downloadCloneTrooper() {
  const browser = await chromium.launch({ headless: false });
  const context = await browser.newContext();
  const page = await context.newPage();

  // Setup download handler
  const downloadPath = 'C:\\Users\\koosh\\Dino\\packs\\warfare-starwars\\assets\\raw\\sw_clone_trooper_sketchfab_001';

  // Create directory if it doesn't exist
  if (!fs.existsSync(downloadPath)) {
    fs.mkdirSync(downloadPath, { recursive: true });
  }

  let downloadedFile = null;

  // Listen for downloads
  page.on('download', async (download) => {
    const fileName = 'source_download.glb';
    const filePath = path.join(downloadPath, fileName);
    await download.saveAs(filePath);
    downloadedFile = filePath;
    console.log(`Downloaded to: ${filePath}`);
  });

  try {
    // Navigate to Sketchfab model page
    console.log('Opening Sketchfab model page...');
    await page.goto('https://sketchfab.com/3d-models/star-wars-clone-trooper-9b0d260e63174ac8873cb56919cc76f9', {
      waitUntil: 'networkidle',
      timeout: 60000
    });

    console.log('Waiting for page to load...');
    await page.waitForTimeout(3000); // Wait for model viewer to load

    // Scroll down to find download button
    console.log('Looking for download button...');
    await page.evaluate(() => {
      window.scrollBy(0, 500);
    });

    await page.waitForTimeout(2000);

    // Try to find and click the download button
    const downloadButton = await page.locator('button:has-text("Download")').first();

    if (await downloadButton.isVisible()) {
      console.log('Found download button, clicking...');
      await downloadButton.click();
      await page.waitForTimeout(2000);
    } else {
      console.log('Download button not found via text, trying alternative selectors...');
      // Try alternative selectors
      const buttons = await page.locator('button').all();
      for (const button of buttons) {
        const text = await button.textContent();
        if (text && text.toLowerCase().includes('download')) {
          console.log(`Found button with text: ${text}`);
          await button.click();
          await page.waitForTimeout(2000);
          break;
        }
      }
    }

    // Look for GLB format option in dropdown
    console.log('Looking for GLB format option...');
    const glbOption = await page.locator('text=GLB').first();

    if (await glbOption.isVisible()) {
      console.log('Found GLB option, clicking...');
      await glbOption.click();
      await page.waitForTimeout(2000);
    } else {
      console.log('GLB option not immediately visible, checking dropdown...');
    }

    // Look for final download/confirm button
    console.log('Looking for download confirmation button...');
    await page.waitForTimeout(1000);

    // Try to find any clickable download action
    const downloadActions = await page.locator('a[download], button:has-text("Download")').all();
    if (downloadActions.length > 0) {
      console.log(`Found ${downloadActions.length} download actions, clicking last one...`);
      await downloadActions[downloadActions.length - 1].click();
    }

    // Wait for download to complete (up to 30 seconds)
    console.log('Waiting for file download to complete...');
    await page.waitForTimeout(5000);

    // Verify file was downloaded
    let attempts = 0;
    while (!downloadedFile && attempts < 12) {
      // Check if file exists in the download path
      const files = fs.readdirSync(downloadPath).filter(f => f.endsWith('.glb'));
      if (files.length > 0) {
        downloadedFile = path.join(downloadPath, files[0]);
        console.log(`Found downloaded file: ${downloadedFile}`);
        break;
      }
      await page.waitForTimeout(2000);
      attempts++;
    }

  } catch (error) {
    console.error('Error during automation:', error);
  } finally {
    await browser.close();
  }

  // Post-download verification and hashing
  if (downloadedFile && fs.existsSync(downloadedFile)) {
    const stats = fs.statSync(downloadedFile);
    console.log(`File size: ${stats.size} bytes`);

    if (stats.size > 1024 * 1024) {
      console.log('✓ File size check passed (> 1MB)');
    } else {
      console.warn('⚠ File size is less than 1MB');
    }

    // Calculate SHA256 hash
    const hash = crypto.createHash('sha256');
    const fileStream = fs.createReadStream(downloadedFile);

    return new Promise((resolve, reject) => {
      fileStream.on('data', (chunk) => hash.update(chunk));
      fileStream.on('end', () => {
        const sha256 = hash.digest('hex');
        console.log(`SHA256: ${sha256}`);

        // Generate manifest
        const manifest = {
          asset_id: 'sw_clone_trooper_sketchfab_001',
          source_url: 'https://sketchfab.com/3d-models/star-wars-clone-trooper-9b0d260e63174ac8873cb56919cc76f9',
          author_name: 'reizer',
          polycount_estimate: 4123,
          sha256: sha256,
          download_url: downloadedFile,
          acquired_at_utc: new Date().toISOString(),
          technical_status: 'downloaded',
          ip_status: 'fan_star_wars_private_only',
          file_size_bytes: stats.size
        };

        console.log('\n=== MANIFEST ===');
        console.log(JSON.stringify(manifest, null, 2));
        resolve(manifest);
      });
      fileStream.on('error', reject);
    });
  } else {
    console.error('Download failed: file not found');
    return null;
  }
}

downloadCloneTrooper().catch(console.error);
