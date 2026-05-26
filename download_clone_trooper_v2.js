const { chromium } = require('playwright');
const fs = require('fs');
const path = require('path');
const crypto = require('crypto');

async function downloadCloneTrooper() {
  const downloadDir = 'C:\\Users\\koosh\\Downloads\\sketchfab_temp';
  const targetDir = 'C:\\Users\\koosh\\Dino\\packs\\warfare-starwars\\assets\\raw\\sw_clone_trooper_sketchfab_001';

  // Create temp directory
  if (!fs.existsSync(downloadDir)) {
    fs.mkdirSync(downloadDir, { recursive: true });
  }

  const browser = await chromium.launch({
    headless: false,
    downloadsPath: downloadDir
  });

  const context = await browser.newContext({
    acceptDownloads: true
  });

  const page = await context.newPage();

  try {
    console.log('Step 1: Navigating to Sketchfab model page...');
    await page.goto('https://sketchfab.com/3d-models/star-wars-clone-trooper-9b0d260e63174ac8873cb56919cc76f9', {
      waitUntil: 'domcontentloaded',
      timeout: 60000
    });

    console.log('Step 2: Waiting for page content to load...');
    await page.waitForTimeout(5000); // Wait for interactive elements

    // Try to find and click the Download button (typically in upper right or model controls)
    console.log('Step 3: Attempting to find and click Download button...');

    // Try multiple selectors for download button
    const selectors = [
      'button[title*="Download"]',
      'button:has-text("Download")',
      'a[href*="download"]',
      '[class*="download"] button',
      '[aria-label*="download"]'
    ];

    let foundDownload = false;

    for (const selector of selectors) {
      try {
        const element = await page.$(selector);
        if (element) {
          console.log(`Found element with selector: ${selector}`);
          const download = await page.waitForEvent('download', async () => {
            await element.click();
            console.log('Clicked download button');
          });

          const filePath = await download.path();
          const fileName = download.suggestedFilename();
          console.log(`Download started: ${fileName}`);
          console.log(`Temporary path: ${filePath}`);

          // Wait for download to complete
          await download.saveAs(path.join(targetDir, 'source_download.glb'));
          console.log(`Saved to: ${path.join(targetDir, 'source_download.glb')}`);

          foundDownload = true;
          break;
        }
      } catch (e) {
        // Continue to next selector
      }
    }

    if (!foundDownload) {
      console.log('Step 3b: Trying to interact with page to reveal download...');

      // Scroll to ensure buttons are visible
      await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
      await page.waitForTimeout(2000);

      // Look for any button/link with "download" text (case-insensitive)
      const buttons = await page.$$('button, a');
      for (const button of buttons) {
        const text = await button.textContent();
        if (text && text.toLowerCase().includes('download')) {
          console.log(`Found button: ${text.trim()}`);

          try {
            const download = await page.waitForEvent('download', async () => {
              await button.click();
            });

            const filePath = await download.path();
            console.log(`Download started: ${download.suggestedFilename()}`);
            await download.saveAs(path.join(targetDir, 'source_download.glb'));
            foundDownload = true;
            break;
          } catch (e) {
            console.error(`Error with button: ${e.message}`);
          }
        }
      }
    }

    if (!foundDownload) {
      console.warn('⚠ Could not locate download button through automation');
      console.warn('The page may require manual intervention or JavaScript that we cannot automate');
      console.log('Please manually click the Download button and select GLB format');
    }

  } catch (error) {
    console.error('Error during automation:', error);
  } finally {
    await browser.close();
  }

  // Post-download verification and hashing
  console.log('\nStep 4: Verifying downloaded file...');
  const targetFile = path.join(targetDir, 'source_download.glb');

  if (fs.existsSync(targetFile)) {
    const stats = fs.statSync(targetFile);
    console.log(`✓ File found: ${targetFile}`);
    console.log(`File size: ${stats.size} bytes (${(stats.size / 1024 / 1024).toFixed(2)} MB)`);

    if (stats.size > 1024 * 1024) {
      console.log('✓ File size check passed (> 1MB)');
    } else {
      console.warn('⚠ File size is less than 1MB');
    }

    // Calculate SHA256 hash
    console.log('Step 5: Calculating SHA256 hash...');
    const hash = crypto.createHash('sha256');
    const fileContent = fs.readFileSync(targetFile);
    hash.update(fileContent);
    const sha256 = hash.digest('hex');

    console.log(`SHA256: ${sha256}`);

    // Generate and save manifest
    const manifest = {
      asset_id: 'sw_clone_trooper_sketchfab_001',
      source_url: 'https://sketchfab.com/3d-models/star-wars-clone-trooper-9b0d260e63174ac8873cb56919cc76f9',
      author_name: 'reizer',
      polycount_estimate: 4123,
      sha256: sha256,
      download_url: targetFile,
      acquired_at_utc: new Date().toISOString(),
      technical_status: 'downloaded',
      ip_status: 'fan_star_wars_private_only',
      file_size_bytes: stats.size
    };

    console.log('\n=== MANIFEST ===');
    console.log(JSON.stringify(manifest, null, 2));

    // Save manifest to file
    const manifestPath = path.join(targetDir, 'download_manifest.json');
    fs.writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));
    console.log(`\nManifest saved to: ${manifestPath}`);

    return manifest;
  } else {
    console.error('✗ Download failed: source_download.glb not found');
    console.error(`Expected location: ${targetFile}`);
    console.error(`Directory contents: ${fs.readdirSync(targetDir)}`);
    return null;
  }
}

downloadCloneTrooper().catch(console.error);
