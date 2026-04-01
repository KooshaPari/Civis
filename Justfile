# DINOForge Dev CLI
# Install: https://github.com/casey/just
# Usage: just <recipe>

# Game directory (from Directory.Build.props)
game_dir := `grep '<GameDir>' Directory.Build.props | sed 's/.*<GameDir>//' | sed 's/<.*//'`
plugins_dir := game_dir / "BepInEx" / "plugins"
packs_dir := game_dir / "packs"

# Default: show available commands
default:
    @just --list

# ── Build ─────────────────────────────────────────────

# Build everything (requires game DLLs)
build:
    dotnet build src/DINOForge.sln

# Build CI-safe solution (no game DLL deps)
build-ci:
    dotnet build src/DINOForge.CI.sln

# Build Runtime only (auto-deploys to game)
build-runtime:
    dotnet build src/Runtime/DINOForge.Runtime.csproj

# ── Test ──────────────────────────────────────────────

# Run all tests
test:
    dotnet test src/DINOForge.CI.sln --no-build

# Run tests with verbose output
test-verbose:
    dotnet test src/Tests/DINOForge.Tests.csproj --no-build --logger "console;verbosity=detailed"

# Run a specific test class
test-class class:
    dotnet test src/Tests/DINOForge.Tests.csproj --no-build --filter "FullyQualifiedName~{{class}}"

# Run performance regression tests only
test-performance:
    dotnet test src/Tests/DINOForge.Tests.csproj --no-build --filter "Category=Performance"

# ── Code Quality ──────────────────────────────────────

# Check code formatting
lint:
    dotnet format src/DINOForge.CI.sln --verify-no-changes

# Auto-fix formatting
format:
    dotnet format src/DINOForge.CI.sln

# Full quality check (build + test + lint)
check: build-ci test lint
    @echo "All checks passed."

# ── Deploy ────────────────────────────────────────────

# Build & deploy Runtime + packs to game directory
deploy: deploy-runtime deploy-packs
    @echo ""
    @echo "Deploy complete. Restart DINO to load changes."
    @echo "  F9  = Debug Overlay"
    @echo "  F10 = Mod Menu"

# Build & deploy Runtime DLL to BepInEx/plugins/
deploy-runtime:
    @echo "Building Runtime (Release)..."
    dotnet build src/Runtime/DINOForge.Runtime.csproj -c Release
    @echo "Runtime deployed to: {{plugins_dir}}"

# Sync packs to game directory
deploy-packs:
    @echo "Syncing packs..."
    @mkdir -p "{{packs_dir}}"
    @cp -r packs/* "{{packs_dir}}/" 2>/dev/null || true
    @echo "Packs synced to: {{packs_dir}}"

# Verify deployment status
deploy-status:
    @echo "=== Deployment Status ==="
    @echo "Game: {{game_dir}}"
    @test -f "{{plugins_dir}}/DINOForge.Runtime.dll" && echo "Runtime DLL: OK" || echo "Runtime DLL: MISSING"
    @test -f "{{plugins_dir}}/DINOForge.SDK.dll" && echo "SDK DLL: OK" || echo "SDK DLL: MISSING"
    @test -d "{{packs_dir}}" && echo "Packs dir: OK ($(find '{{packs_dir}}' -name 'pack.yaml' 2>/dev/null | wc -l) packs)" || echo "Packs dir: MISSING"
    @test -f "{{game_dir}}/BepInEx/LogOutput.log" && echo "BepInEx log: EXISTS" || echo "BepInEx log: NOT FOUND"

# ── Pack Management ───────────────────────────────────

# Validate all packs
validate-packs:
    dotnet run --project src/Tools/PackCompiler -- validate packs/example-balance

# Validate a specific pack
validate pack_path:
    dotnet run --project src/Tools/PackCompiler -- validate {{pack_path}}

# Build a pack for distribution
build-pack pack_path output="":
    dotnet run --project src/Tools/PackCompiler -- build {{pack_path}} {{if output != "" { "-o " + output } else { "" } }}

# ── Documentation ─────────────────────────────────────

# Start docs dev server
docs-dev:
    cd docs && npx vitepress dev

# Build docs for production
docs-build:
    cd docs && npx vitepress build

# ── Utilities ─────────────────────────────────────────

# Clean all build artifacts
clean:
    dotnet clean src/DINOForge.sln
    @find src -name "bin" -type d -exec rm -rf {} + 2>/dev/null || true
    @find src -name "obj" -type d -exec rm -rf {} + 2>/dev/null || true
    @echo "Cleaned."

# Analyze game entity dumps
dump-analyze:
    dotnet run --project src/Tools/DumpTools -- analyze "{{game_dir}}/BepInEx/dinoforge_dumps"

# Show latest BepInEx log
log:
    @cat "{{game_dir}}/BepInEx/LogOutput.log" 2>/dev/null | tail -50 || echo "No log found"

# Show DINOForge debug log
debug-log:
    @cat "{{game_dir}}/BepInEx/dinoforge_debug.log" 2>/dev/null | tail -30 || echo "No debug log found"

# ── Mutation Testing ──────────────────────────────────

# Run Stryker.NET mutation testing
# Fails if mutation score < 70%
mutation-test:
    ./scripts/mutation-test.ps1

# Run mutation testing on a specific project
mutation-test-project project:
    ./scripts/mutation-test.ps1 -Project {{project}}
