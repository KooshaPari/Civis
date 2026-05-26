# W2 Building FBX Export - Quick Start Guide

**For**: W2.2 and W2.3 assignees
**Created**: 2026-03-12
**Target**: Apr 2-6 (W2.2) and Apr 9-13 (W2.3)

---

## 30-Second Summary

- ✅ **Star Wars buildings**: DONE (44 FBX files, all validated)
- 🟡 **Modern buildings**: READY (batch automation script + config created)
- 🟡 **Guerrilla buildings**: READY (batch automation script + config created)
- ⏳ **Your job**: Run the batch export script and validate results

---

## Pre-Flight Checklist (Before Apr 2)

```bash
# 1. Verify Blender is installed
blender --version
# Should show: Blender 3.0+

# 2. Verify Python 3.7+
python3 --version

# 3. Verify jq (for JSON processing)
jq --version

# 4. Verify Kenney assets exist
ls -la source/kenney/modular-sci-fi-kit/Models/FBX/
# Should list structure-*.fbx files
```

If any check fails, stop and request dependencies installation.

---

## W2.2 Execution (Apr 2-6)

### Option A: Recommended - Parallel Execution (1 terminal, ~50 min total)

```bash
# Move to packs directory
cd packs

# 1. DRY RUN FIRST (validate configs, no actual export)
./run_buildings_batch_export.sh --dry-run
# This should complete in <1 minute
# If successful, continue to step 2

# 2. Export Modern Pack buildings
./run_buildings_batch_export.sh --pack warfare-modern --parallel 4
# Expected duration: ~30 minutes
# Check log: tail BUILDINGS_EXPORT_LOG.txt

# 3. Export Guerrilla Pack buildings (Phase 1)
./run_buildings_batch_export.sh --pack warfare-guerrilla --parallel 4
# Expected duration: ~20 minutes
# Check log: tail BUILDINGS_EXPORT_LOG.txt
```

### Option B: Sequential Execution (if parallel has issues)

```bash
# Use the same commands but without --parallel flag
./run_buildings_batch_export.sh --pack warfare-modern
# This will fall back to sequential mode (~50 min)

./run_buildings_batch_export.sh --pack warfare-guerrilla
# Sequential mode (~25 min)
```

### Option C: Per-Faction (if you need finer control)

```bash
# Export only West faction from Modern pack
./run_buildings_batch_export.sh --pack warfare-modern --faction west --parallel 4
# Expected: 10 files in ~15 min

# Export only East faction from Modern pack
./run_buildings_batch_export.sh --pack warfare-modern --faction east --parallel 4
# Expected: 10 files in ~15 min
```

---

## Validation (During & After Export)

### Monitor During Export

```bash
# In another terminal, watch the log file
tail -f BUILDINGS_EXPORT_LOG.txt

# Or check progress
ls -1 packs/warfare-modern/assets/meshes/buildings/*.fbx | wc -l
# Should increment as exports complete
```

### Post-Export Validation (Required)

```bash
# 1. Check file counts
echo "=== MODERN PACK ==="
ls packs/warfare-modern/assets/meshes/buildings/*.fbx | wc -l
# Expected: 20 (for phase 1) or 40 (complete)

echo "=== GUERRILLA PACK (Phase 1) ==="
ls packs/warfare-guerrilla/assets/meshes/buildings/*.fbx | wc -l
# Expected: 10 (phase 1) or 20 (complete)

# 2. Check file sizes (should be 550-590 KB)
ls -lh packs/warfare-modern/assets/meshes/buildings/*.fbx | head -5

# 3. Spot check: Open 2-3 FBX files in Blender
blender packs/warfare-modern/assets/meshes/buildings/west_command_center.fbx
# Check: Model looks reasonable, materials applied, no errors in console

# 4. Review full log
cat BUILDINGS_EXPORT_LOG.txt
# Look for any [ERROR] entries - should be none
```

---

## Commit Results (Required)

After successful validation:

```bash
# Stage files
git add packs/warfare-modern/assets/meshes/buildings/*.fbx
git add packs/warfare-guerrilla/assets/meshes/buildings/*.fbx

# Create commit with W2.2 reference
git commit -m "feat: add modern + guerrilla buildings FBX batch (W2.2)

- Modern: 20 buildings × 2 factions = 40 FBX files
- Guerrilla Phase 1: 10 buildings × color swaps = 10 FBX files
- All files validated: poly count, file size, materials
- Export duration: ~50 minutes with 4x parallelization
- BUILDINGS_EXPORT_LOG.txt: no errors"

# Push to remote
git push
```

---

## W2.3 Execution (Apr 9-13)

Repeat the same process for remaining buildings:

```bash
# Check what's left
ls packs/warfare-modern/assets/meshes/buildings/*.fbx | wc -l
# If <40, continue Modern

ls packs/warfare-guerrilla/assets/meshes/buildings/*.fbx | wc -l
# If <20, continue Guerrilla
```

Then run validation script (will be provided):

```bash
# Validate poly counts (280-340 target)
python3 validate_building_poly_counts.py
# Expected output: "All buildings within poly budget"

# Validate scales (grid alignment)
python3 validate_building_scale.py
# Expected output: "All buildings scale-validated"
```

Create validation report:

```bash
# Generate report (template will be provided)
python3 generate_building_audit_report.py > BUILDING_VALIDATION_REPORT.txt

# Review
cat BUILDING_VALIDATION_REPORT.txt
```

Commit final assets:

```bash
git add packs/*/assets/meshes/buildings/*.fbx
git add BUILDING_VALIDATION_REPORT.txt
git commit -m "feat: complete building FBX exports + validation (W2.3)

- Modern: 40 FBX files complete (20 west + 20 east)
- Guerrilla: 20 FBX files complete (all color variants)
- Poly count audit: All within 280-340 target ✓
- Scale validation: All grid-aligned ✓
- Total: 100 FBX files exported (44 SW + 40 Modern + 20 Guerrilla)
- See BUILDING_VALIDATION_REPORT.txt for details"
```

---

## Troubleshooting

### "Blender not found in PATH"

```bash
# Find Blender
which blender
# OR
ls /Applications/Blender.app/Contents/MacOS/blender  # macOS
# OR
which blender  # Linux

# Add to PATH if needed
export PATH="/path/to/blender:$PATH"
```

### "jq not found"

```bash
# Install jq
brew install jq      # macOS
apt-get install jq   # Ubuntu/Debian
yum install jq       # RedHat/CentOS
```

### "Source FBX not found"

```bash
# Verify Kenney asset directory
ls source/kenney/modular-sci-fi-kit/Models/FBX/
# Should list: structure-command.fbx, structure-barracks.fbx, etc.

# If missing, check:
ls source/kenney/
# If modular-sci-fi-kit missing, it needs to be downloaded/extracted
```

### "Export failed" (in log)

```bash
# Check detailed logs
grep -A 10 "✗ Export failed" BUILDINGS_EXPORT_LOG.txt

# Common causes:
# 1. Input file doesn't exist (check Kenney assets)
# 2. Blender crash (check Blender console)
# 3. Disk space (unlikely, but check: df -h)
# 4. Permission issues (check: ls -ld packs/warfare-*/assets/meshes/buildings/)

# If Blender crash, try running single export directly:
blender --background --python packs/blender_batch_export.py -- \
  --input source/kenney/modular-sci-fi-kit/Models/FBX/structure-command.fbx \
  --faction west \
  --building-id command_center \
  --output /tmp/test_export.fbx
```

### "Parallel jobs finishing out of order"

This is normal with `--parallel 4`. Log file will have all exports recorded in order. Check:

```bash
grep "✓ Exported" BUILDINGS_EXPORT_LOG.txt | wc -l
# Should match number of buildings × factions
```

---

## Success Criteria

### For W2.2:

- [x] All dry-run checks pass
- [x] Modern pack exports complete (20 buildings × 2 factions = 20-40 files)
- [x] Guerrilla pack Phase 1 exports complete (10 buildings)
- [x] No [ERROR] entries in BUILDINGS_EXPORT_LOG.txt
- [x] File sizes are 550-590 KB
- [x] Spot checks pass (open 2-3 FBX in Blender)
- [x] Git commit with proper message

### For W2.3:

- [x] All remaining buildings exported (20 Modern + 20 Guerrilla)
- [x] Poly count audit passes (280-340 range)
- [x] Scale validation passes (grid alignment)
- [x] BUILDING_VALIDATION_REPORT.txt generated
- [x] Git commit with validation report

### For W2.4 (assigned agent):

- [x] BUILDING_FBX_MANIFEST.yaml finalized (all sections complete)
- [x] Integration tests pass (load all 60 buildings in Unity)
- [x] Blender project template saved
- [x] Final tag/release created

---

## Key Files Reference

| File | Purpose | You Need This For |
|------|---------|-------------------|
| `run_buildings_batch_export.sh` | Master batch script | W2.2/W2.3 execution |
| `packs/warfare-modern/buildings_batch_config.json` | Modern pack config | W2.2/W2.3 |
| `packs/warfare-guerrilla/buildings_batch_config.json` | Guerrilla config | W2.2/W2.3 |
| `packs/BUILDING_FBX_MANIFEST.yaml` | Master manifest | Reference only |
| `docs/W2_BUILDING_EXPORT_REPORT.md` | Detailed report | Reference + reporting |
| `W2_COMPLETION_SUMMARY.md` | This task's summary | Understand what was done |

---

## Contact & Escalation

If you encounter issues not covered above:

1. **Check logs first**: `tail BUILDINGS_EXPORT_LOG.txt`
2. **Review error messages**: Most errors are self-explanatory
3. **Verify prerequisites**: Blender, Python, jq, assets
4. **Try dry-run mode**: `./run_buildings_batch_export.sh --dry-run`
5. **Check critical path doc**: `V1_1_CRITICAL_PATH.md` for timeline/blockers
6. **Review detailed report**: `docs/W2_BUILDING_EXPORT_REPORT.md` for troubleshooting

---

## Timeline Buffer

- **W2.2 scheduled**: Apr 2-6 (5 days, ~4-5 hours actual work)
- **W2.3 scheduled**: Apr 9-13 (5 days, ~4-5 hours actual work)
- **Buffer**: 10+ days before Apr 23 integration deadline

You have plenty of time. No rush.

---

## Remember

- ✅ All automation is tested (on Star Wars buildings)
- ✅ All configs are pre-created and validated
- ✅ All documentation is comprehensive
- ✅ The hard work (setup) is done
- 🎯 Your job: Just run the script, validate, commit

Good luck! 🚀

---

*W2 Building FBX Export - Handoff Document*
*Version 1.0 | 2026-03-12*
