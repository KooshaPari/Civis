"""One-shot YAML schema drift audit. Pure read-only inspection."""
import json, yaml, jsonschema
from pathlib import Path

schema_dir = Path('schemas')
pack_dir = Path('packs')

schemas = {p.stem.replace('.schema', ''): json.loads(p.read_text(encoding='utf-8'))
           for p in schema_dir.glob('*.schema.json')}

DIR_TO_SCHEMA = {
    'units': 'unit',
    'buildings': 'building',
    'factions': 'faction',
    'weapons': 'weapon',
    'projectiles': 'projectile',
    'doctrines': 'doctrine',
    'skills': 'skill',
    'squads': 'squad',
    'waves': 'wave',
    'scenarios': 'scenario',
    'economy_profiles': 'economy-profile',
    'hud_elements': 'ui-overlay',
    'menus': 'ui-overlay',
    'themes': 'ui-overlay',
}

issues = []
validated = []
skipped = []

# Validate pack manifests
for pack in sorted(pack_dir.iterdir()):
    if not pack.is_dir() or pack.name.startswith('_'):
        continue
    for manifest_name in ['pack.yaml', 'manifest.yaml']:
        manifest = pack / manifest_name
        if manifest.exists():
            try:
                doc = yaml.safe_load(manifest.read_text(encoding='utf-8'))
                jsonschema.validate(doc, schemas['pack-manifest'])
                validated.append(str(manifest))
            except jsonschema.ValidationError as e:
                path = '/'.join(str(p) for p in e.absolute_path)
                issues.append((str(manifest), 'pack-manifest', path + ': ' + e.message[:150]))
            except Exception as e:
                issues.append((str(manifest), 'pack-manifest', 'PARSE: ' + str(e)[:150]))

# Validate domain content directories
for pack in sorted(pack_dir.iterdir()):
    if not pack.is_dir() or pack.name.startswith('_') or pack.name.startswith('test-'):
        continue
    for subdir_name, schema_key in DIR_TO_SCHEMA.items():
        subdir = pack / subdir_name
        if not subdir.exists():
            continue
        if schema_key not in schemas:
            skipped.append(str(subdir) + ": schema '" + schema_key + "' missing")
            continue
        for yf in subdir.glob('*.yaml'):
            try:
                doc = yaml.safe_load(yf.read_text(encoding='utf-8'))
            except Exception as e:
                issues.append((str(yf), schema_key, 'YAML PARSE: ' + str(e)[:150]))
                continue

            # Direct
            try:
                jsonschema.validate(doc, schemas[schema_key])
                validated.append(str(yf) + ' [direct]')
                continue
            except jsonschema.ValidationError:
                pass

            # List wrapper
            if isinstance(doc, dict) and subdir_name in doc and isinstance(doc[subdir_name], list):
                ok = True
                first_err = None
                for i, item in enumerate(doc[subdir_name]):
                    try:
                        jsonschema.validate(item, schemas[schema_key])
                    except jsonschema.ValidationError as e:
                        ok = False
                        if first_err is None:
                            path = '/'.join(str(p) for p in e.absolute_path)
                            first_err = 'item[' + str(i) + '].' + path + ': ' + e.message[:120]
                        break
                if ok:
                    validated.append(str(yf) + ' [list-' + subdir_name + ']')
                else:
                    issues.append((str(yf), schema_key, first_err or 'list validation failed'))
                continue

            # Re-throw direct error to capture
            try:
                jsonschema.validate(doc, schemas[schema_key])
            except jsonschema.ValidationError as e:
                path = '/'.join(str(p) for p in e.absolute_path)
                issues.append((str(yf), schema_key, 'direct ' + path + ': ' + e.message[:150]))

print('=== VALIDATED:', len(validated), '===')
for v in validated:
    print('  OK ' + v)

print()
print('=== ISSUES:', len(issues), '===')
for f, s, msg in issues:
    print('  FAIL [' + s + '] ' + f)
    print('        -> ' + msg)

print()
print('=== SKIPPED:', len(skipped), '===')
for s in skipped:
    print('  SKIP ' + s)
