#!/usr/bin/env bash
set -euo pipefail
source_dir="$(cd "$(dirname "$0")" && pwd)"
scratch="$(mktemp -d)"
trap 'rm -rf -- "$scratch"' EXIT
mkdir -p "$scratch/repo/scripts/quality" "$scratch/repo/web/dashboard" "$scratch/bin"
cp "$source_dir/emit-quality-manifest.sh" "$source_dir/write-quality-manifest.py" "$source_dir/verify-quality-manifest.sh" "$scratch/repo/scripts/quality/"
for tool in just npm bun; do
    printf '#!/usr/bin/env bash\nexit "${MOCK_GATE_EXIT:-0}"\n' > "$scratch/bin/$tool"
    chmod +x "$scratch/bin/$tool"
done
cd "$scratch/repo"
git init -q
git add scripts
git -c user.name=Test -c user.email=test@example.invalid commit -qm initial
export PATH="$scratch/bin:$PATH"
export CIVIS_QUALITY_EXTRAS=0 CIVIS_QUALITY_UNREAL=0
bash scripts/quality/emit-quality-manifest.sh
bash scripts/quality/verify-quality-manifest.sh
cp .ci/quality-manifest.json "$scratch/previous-manifest"
if MOCK_GATE_EXIT=23 bash scripts/quality/emit-quality-manifest.sh; then
    echo 'failed gate was accepted' >&2; exit 1
fi
cmp .ci/quality-manifest.json "$scratch/previous-manifest"
if SKIP_CIVIS_3D_VERIFY=1 bash scripts/quality/emit-quality-manifest.sh; then
    echo 'required skip was accepted' >&2; exit 1
fi
printf dirty >> scripts/quality/write-quality-manifest.py
if bash scripts/quality/emit-quality-manifest.sh; then
    echo 'dirty tree was accepted' >&2; exit 1
fi
cmp .ci/quality-manifest.json "$scratch/previous-manifest"
echo 'synthetic shell emitter controls passed'
