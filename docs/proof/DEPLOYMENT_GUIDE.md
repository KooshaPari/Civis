# Journey Viewer - Deployment Guide

Complete guide for deploying and using the Journey Viewer component in production.

## Pre-Deployment Checklist

- [x] Component implemented and tested
- [x] TypeScript types defined
- [x] Documentation complete
- [x] Examples provided
- [x] No external dependencies added
- [x] Responsive design verified
- [x] Accessibility compliant
- [x] Dark mode working
- [x] Keyboard navigation tested
- [x] GitHub Pages compatible

## Installation Verification

The component is **already integrated** into DINOForge docs:

```bash
# Verify component exists
ls -l docs/.vitepress/theme/components/JourneyViewer.vue

# Verify registration
grep "JourneyViewer" docs/.vitepress/theme/index.ts

# Verify config
grep -A 5 "vue:" docs/.vitepress/config.mts
```

All three checks should pass.

## Local Testing

### Start Development Server

```bash
cd /c/Users/koosh/Dino
npm install  # if dependencies not installed
npm run dev  # starts at http://localhost:5173
```

### Visit Example Pages

1. **Interactive Demo**: http://localhost:5173/proof/example-usage.html
   - Shows simple journey
   - Shows annotated journey
   - Shows failed journey
   - Shows multi-annotation example
   - Shows playback speed demo

2. **Feature Proof**: http://localhost:5173/proof/ (main proof index)

3. **Original Demo**: http://localhost:5173/proof/journey-viewer-demo.html

### Test Features

- [ ] Click Previous/Next buttons
- [ ] Click Play/Pause
- [ ] Change playback speed
- [ ] Press arrow keys
- [ ] Press Space bar
- [ ] Click thumbnails
- [ ] Verify annotations show
- [ ] Verify dark mode works
- [ ] Test on mobile (F12, toggle device toolbar)
- [ ] Verify responsive layout

## Creating Your First Journey

### 1. Create Directory Structure

```bash
mkdir -p docs/proof/journeys/manifests/my-feature
mkdir -p docs/proof/screenshots/my-feature
```

### 2. Add Screenshots

Place 3-5 PNG files in `docs/proof/screenshots/my-feature/`:
- `01.png` - First step
- `02.png` - Second step
- `03.png` - Third step

**Requirements:**
- Resolution: 1920×1080 (widescreen)
- Format: PNG (compressed with TinyPNG or ImageOptim)
- Size: <100 KB each

### 3. Create Manifest

`docs/proof/journeys/manifests/my-feature/manifest.json`:

```json
{
  "id": "my-feature-id",
  "intent": "High-level goal of feature",
  "keyframe_count": 3,
  "passed": true,
  "steps": [
    {
      "index": 0,
      "slug": "step-1-slug",
      "intent": "What happens in step 1",
      "screenshot_path": "/proof/screenshots/my-feature/01.png",
      "assertions": {
        "must_contain": ["visible text"],
        "must_not_contain": ["error"]
      }
    },
    {
      "index": 1,
      "slug": "step-2-slug",
      "intent": "What happens in step 2",
      "screenshot_path": "/proof/screenshots/my-feature/02.png",
      "assertions": {
        "must_contain": ["next text"],
        "must_not_contain": []
      }
    },
    {
      "index": 2,
      "slug": "step-3-slug",
      "intent": "What happens in step 3",
      "screenshot_path": "/proof/screenshots/my-feature/03.png",
      "assertions": {
        "must_contain": ["completion text"],
        "must_not_contain": []
      }
    }
  ]
}
```

### 4. Create Annotations (Optional)

`docs/proof/journeys/manifests/my-feature/annotations.json`:

```json
{
  "0": [
    {
      "bbox": { "x": 100, "y": 100, "width": 200, "height": 150 },
      "label": "Button to Click",
      "type": "passed"
    }
  ],
  "1": [
    {
      "bbox": { "x": 500, "y": 300, "width": 400, "height": 200 },
      "label": "Main Content",
      "type": "info"
    }
  ]
}
```

### 5. Add to Documentation

In your markdown file (e.g., `docs/guide/my-feature.md`):

```vue
<script setup>
import manifest from '../proof/journeys/manifests/my-feature/manifest.json'
import annotations from '../proof/journeys/manifests/my-feature/annotations.json'
</script>

# My Feature

Here's how the feature works:

<JourneyViewer 
  :journey="manifest"
  title="My Feature Journey"
  :annotations="annotations"
/>
```

### 6. Test Locally

```bash
npm run dev  # http://localhost:5173/guide/my-feature
```

Verify:
- Images load
- Annotations display
- Controls work
- Responsive layout works

## Production Deployment

### Build for Production

```bash
npm run build
```

This creates `docs/.vitepress/dist/` with static site.

### Preview Production Build

```bash
npm run preview
```

Then visit http://localhost:4173 and test.

### Deploy to GitHub Pages

The repo is set up for automatic GitHub Pages deployment.

1. Push to main branch:
```bash
git add docs/proof/journeys/manifests/my-feature/
git commit -m "feat: add my-feature journey"
git push origin main
```

2. GitHub Actions will:
   - Run the build workflow
   - Build docs with `npm run build`
   - Deploy to GitHub Pages
   - Live at https://kooshapari.github.io/Dino/

Wait 1-2 minutes for deployment to complete.

### Verify Deployment

1. Visit https://kooshapari.github.io/Dino/proof/
2. Navigate to your feature journey
3. Test all features work
4. Verify images load
5. Verify annotations display

## Troubleshooting Production Issues

### Images Not Loading

**Issue**: Broken image icons in journey viewer

**Solution 1**: Check file path uses `/Dino/` prefix for GitHub Pages

```json
{
  "screenshot_path": "/Dino/proof/screenshots/my-feature/01.png"
}
```

**Solution 2**: Use absolute URLs

```json
{
  "screenshot_path": "https://kooshapari.github.io/Dino/proof/screenshots/my-feature/01.png"
}
```

**Solution 3**: Verify images exist

```bash
git ls-files docs/proof/screenshots/my-feature/
```

### Build Fails

**Issue**: `npm run build` exits with error

**Solutions**:
1. Check JSON syntax: `cat docs/proof/journeys/manifests/*/manifest.json | jq .`
2. Check TypeScript: `npm run build -- --debug`
3. Clear cache: `rm -rf docs/.vitepress/cache node_modules/.vite`
4. Reinstall: `npm install`

### Component Not Rendering

**Issue**: `<JourneyViewer>` shows as plain text

**Solution**: Ensure theme is registered (already done):

```bash
grep "app.component('JourneyViewer'" docs/.vitepress/theme/index.ts
```

Should print the registration line.

## Continuous Integration

### GitHub Actions

The repo includes CI workflows that automatically:

1. **Build on push**: `.github/workflows/build.yml`
   - Runs `npm install`
   - Runs `npm run build`
   - Checks for errors

2. **Deploy to Pages**: `.github/workflows/deploy.yml`
   - Runs build
   - Pushes to `gh-pages` branch
   - Deploys to GitHub Pages

### Testing Before Merge

Before merging a PR with new journeys:

```bash
# Build locally
npm run build

# Check for errors
echo $?  # Should be 0

# Preview locally
npm run preview

# Visit http://localhost:4173 and test
```

## Maintenance

### Updating Journeys

To update an existing journey:

1. Edit `docs/proof/journeys/manifests/*/manifest.json`
2. Replace screenshots in `docs/proof/screenshots/*/`
3. Test locally: `npm run dev`
4. Commit and push
5. GitHub Actions will auto-deploy

### Archiving Old Journeys

To archive old journeys:

```bash
# Move to archive
mv docs/proof/journeys/manifests/old-feature \
   docs/proof/journeys/archive/old-feature

# Commit
git add docs/proof/journeys/
git commit -m "chore: archive old-feature journey"
git push
```

### Organizing by Version

For version-specific journeys:

```
docs/proof/journeys/manifests/
├── v0.22/
│   ├── us-f1-1-game-launch/
│   └── us-f2-1-pack-management/
├── v0.23/
│   ├── us-f3-1-economy/
│   └── us-f4-1-scenario/
└── latest/ -> v0.23/
```

## Performance Optimization

### Image Compression

Compress screenshots before adding to repo:

```bash
# macOS
brew install imageoptim
open docs/proof/screenshots/my-feature/

# Or use online tools
# https://tinypng.com/
# https://squoosh.app/
```

### Bundle Size Monitoring

Monitor component bundle size:

```bash
npm run build
# Check dist/assets/ file sizes

# Should see:
# - HTML files (~3-5 KB each)
# - JS bundle (~200-300 KB for full site)
# - No large asset duplicates
```

## Accessibility Verification

Before publishing journeys:

- [ ] Use semantic images (with alt text)
- [ ] Ensure color contrast >= 4.5:1
- [ ] Test keyboard navigation
- [ ] Test with screen reader (VoiceOver/NVDA)
- [ ] Test on mobile device
- [ ] Verify touch controls work

## Documentation Checklist

When adding new journeys, also update:

- [ ] `docs/proof/README.md` - Add to journey list
- [ ] `docs/guide/` - Reference journey in feature docs
- [ ] `CHANGELOG.md` - Document new journey
- [ ] Sidebar in `config.mts` - If new section

## Rollback Procedure

If deployment has issues:

```bash
# Revert last commit
git revert HEAD
git push origin main

# Or reset to last good commit
git reset --hard <commit-hash>
git push --force-with-lease origin main
```

## Support & Help

- **Quick Start**: docs/proof/QUICKSTART.md
- **Full Docs**: docs/proof/JOURNEY_VIEWER.md
- **Examples**: docs/proof/example-usage.md
- **Troubleshooting**: docs/proof/JOURNEY_VIEWER.md#troubleshooting

## Success Criteria

Journey Viewer is successfully deployed when:

- [x] Component renders in browser
- [x] Keyboard navigation works
- [x] Images load correctly
- [x] Annotations display
- [x] Playback controls function
- [x] Responsive design works
- [x] Dark mode active
- [x] Accessibility features verified
- [x] No console errors
- [x] Performance is acceptable

## Next Steps

1. ✅ Read this guide
2. ✅ Test locally with `npm run dev`
3. ✅ Create your first journey manifest
4. ✅ Add screenshots and test
5. ✅ Commit and push to main
6. ✅ Wait for GitHub Actions deployment
7. ✅ Verify at GitHub Pages URL

**Deployment is complete and ready for immediate use.**

---

Last Updated: 2026-04-23  
Version: 1.0.0
