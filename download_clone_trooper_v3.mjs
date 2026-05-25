import { chromium } from 'playwright';
import fs from 'fs';
import path from 'path';
import crypto from 'crypto';

async function downloadCloneTrooper() {
  const downloadDir = '/tmp/sketchfab_temp';
  const targetDir = '/c/Users/koosh/Dino/packs/warfare-starwars/assets/raw/sw_clone_trooper_sketchfab_001';

  // Create directories
  [downloadDir, targetDir].forEach(dir => {
    if (!fs.existsSync(dir)) {
      fs.mkdirSync(dir, { recursive: true });
    }
  });

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
    await page.waitForTimeout(5000);

    console.log('Step 3: Looking for Download button...');

    // Try to find download button by various methods
    let downloadPromise = null;

    try {
      // Method 1: Look for button with download text
      const downloadBtn = page.locator('button:has-text("Download")').first();
      if (await downloadBtn.isVisible({ timeout: 5000 })) {
        console.log('Found Download button, setting up download listener...');
        downloadPromise = page.waitForEvent('download');
        await downloadBtn.click();
      }
    } catch (e) {
      console.log('Method 1 failed, trying alternative approaches...');
    }

    if (!downloadPromise) {
      try {
        // Method 2: Look for any link/button containing download in title or aria-label
        const dlElements = await page.locator('[title*="download" i], [aria-label*="download" i]').all();
        if (dlElements.length > 0) {
          console.log(`Found ${dlElements.length} download-related elements`);
          downloadPromise = page.waitForEvent('download');
          await dlElements[0].click();
        }
      } catch (e) {
        console.log('Method 2 failed...');
      }
    }

    if (downloadPromise) {
      const download = await downloadPromise;
      console.log(`Download started: ${download.suggestedFilename()}`);

      const targetFile = path.join(targetDir, 'source_download.glb');
      await download.saveAs(targetFile);
      console.log(`✓ File saved to: ${targetFile}`);
    } else {
      console.warn('⚠ Could not locate and click download button');
      console.log('Browser is still open - you can manually interact with it');
      console.log('Click Download > GLB format, and I will proceed with verification');
      await page.waitForTimeout(30000); // Wait 30 seconds for manual action
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
      console.warn('⚠ File size is less than 1MB - may be incomplete');
    }

    // Calculate SHA256 hash
    console.log('\nStep 5: Calculating SHA256 hash...');
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
    if (fs.existsSync(targetDir)) {
      console.error(`Directory contents: ${fs.readdirSync(targetDir)}`);
    }
    return null;
  }
}

downloadCloneTrooper().catch(console.error);
