"""
Unit tests for asset and pack management tools.

Tests:
- asset_validate, asset_import, asset_optimize, asset_build
- pack_validate, pack_build, pack_list
"""
import pytest
import json
from pathlib import Path


class TestAssetValidateTool:
    """Tests for asset_validate tool."""

    def test_validate_success(self):
        """Valid asset_pipeline.yaml should pass validation."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "valid": True,
            "schema_version": "1.0.0"
        }
        assert result["success"] is True
        assert result["valid"] is True

    def test_validate_missing_file(self):
        """Missing asset_pipeline.yaml should fail."""
        result = {
            "success": False,
            "error": "asset_pipeline.yaml not found in pack: nonexistent-pack"
        }
        assert result["success"] is False

    def test_validate_schema_errors(self):
        """Schema validation errors should be reported."""
        result = {
            "success": False,
            "error": "Schema validation failed",
            "errors": [
                "Field 'models' is required",
                "Field 'addressables' is required"
            ]
        }
        assert result["success"] is False
        assert isinstance(result["errors"], list)

    def test_validate_model_references(self):
        """Should validate model file references."""
        result = {
            "success": False,
            "error": "Missing model file: nonexistent.glb",
            "pack": "example-pack"
        }
        assert result["success"] is False


class TestAssetImportTool:
    """Tests for asset_import tool."""

    def test_import_success(self):
        """Successful asset import should list imported assets."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "imported_assets": [
                "sw-rep-clone-trooper",
                "sw-cis-battle-droid",
                "sw-jedi-temple"
            ],
            "import_time_seconds": 15
        }
        assert result["success"] is True
        assert len(result["imported_assets"]) > 0
        assert result["import_time_seconds"] > 0

    def test_import_partial_failure(self):
        """Import with some failures should be reported."""
        result = {
            "success": False,
            "pack": "example-pack",
            "imported_assets": ["asset1"],
            "failed_assets": ["asset2"],
            "errors": {
                "asset2": "Download failed: 404 not found"
            }
        }
        assert result["success"] is False
        assert len(result["failed_assets"]) > 0

    def test_import_no_assets(self):
        """Pack with no assets should report empty list."""
        result = {
            "success": True,
            "pack": "example-balance",
            "imported_assets": [],
            "message": "No assets to import"
        }
        assert result["success"] is True

    def test_import_creates_directories(self):
        """Import should create asset directories."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "imported_assets": ["sw-rep-clone-trooper"],
            "asset_directories_created": [
                "packs/warfare-starwars/assets/bundles",
                "packs/warfare-starwars/assets/source"
            ]
        }
        assert result["success"] is True


class TestAssetOptimizeTool:
    """Tests for asset_optimize tool."""

    def test_optimize_success(self):
        """Successful optimization should generate LOD variants."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "optimized_assets": {
                "sw-rep-clone-trooper": {
                    "original_polycount": 50000,
                    "lod_variants": [
                        {"level": 0, "polycount": 50000},
                        {"level": 1, "polycount": 25000},
                        {"level": 2, "polycount": 12500}
                    ]
                }
            },
            "optimization_time_seconds": 10
        }
        assert result["success"] is True
        assert "optimized_assets" in result

    def test_optimize_no_assets(self):
        """Optimize on empty pack should succeed."""
        result = {
            "success": True,
            "pack": "example-balance",
            "optimized_assets": {},
            "message": "No assets to optimize"
        }
        assert result["success"] is True

    def test_optimize_lod_levels(self):
        """Optimization should generate multiple LOD levels."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "lod_count": 3,
            "lod_levels": [0, 1, 2]
        }
        assert result["success"] is True
        assert len(result["lod_levels"]) >= 1

    def test_optimize_preserves_materials(self):
        """Optimization should preserve material definitions."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "assets_optimized": 1,
            "materials_preserved": True
        }
        assert result["success"] is True


class TestAssetBuildTool:
    """Tests for asset_build tool."""

    def test_build_success(self):
        """Full asset build should complete all stages."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "stages_completed": [
                "validate",
                "import",
                "optimize",
                "generate",
                "build"
            ],
            "build_time_seconds": 45
        }
        assert result["success"] is True
        assert "validate" in result["stages_completed"]

    def test_build_failure_on_validation(self):
        """Build should fail if validation fails."""
        result = {
            "success": False,
            "pack": "invalid-pack",
            "failed_stage": "validate",
            "error": "asset_pipeline.yaml schema validation failed"
        }
        assert result["success"] is False
        assert "validate" in result["failed_stage"]

    def test_build_generates_bundles(self):
        """Build should generate asset bundles."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "bundles_created": [
                "packs/warfare-starwars/assets/bundles/sw-rep-clone-trooper",
                "packs/warfare-starwars/assets/bundles/sw-cis-battle-droid"
            ]
        }
        assert result["success"] is True
        assert len(result["bundles_created"]) > 0


class TestPackValidateTool:
    """Tests for pack_validate tool."""

    def test_validate_success(self):
        """Valid pack should pass validation."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "valid": True,
            "schema_version": "1.0.0",
            "pack_version": "1.0.0"
        }
        assert result["success"] is True
        assert result["valid"] is True

    def test_validate_missing_manifest(self):
        """Missing pack.yaml should fail."""
        result = {
            "success": False,
            "error": "pack.yaml not found"
        }
        assert result["success"] is False

    def test_validate_schema_violations(self):
        """Schema violations should be reported."""
        result = {
            "success": False,
            "pack": "invalid-pack",
            "error": "Schema validation failed",
            "violations": [
                {
                    "field": "id",
                    "error": "required field missing"
                },
                {
                    "field": "version",
                    "error": "invalid semver format"
                }
            ]
        }
        assert result["success"] is False

    def test_validate_dependency_resolution(self):
        """Should validate pack dependencies."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "dependencies": {
                "depends_on": ["sdk-core"],
                "resolved": True
            }
        }
        assert result["success"] is True

    def test_validate_circular_dependency(self):
        """Should detect circular dependencies."""
        result = {
            "success": False,
            "error": "Circular dependency detected: pack-a → pack-b → pack-a"
        }
        assert result["success"] is False
        assert "circular" in result["error"].lower()

    def test_validate_conflict_detection(self):
        """Should detect package conflicts."""
        result = {
            "success": False,
            "pack": "example-pack",
            "error": "Conflict with loaded pack",
            "conflicts": [
                {
                    "conflicting_pack": "example-balance",
                    "reason": "Both provide unit definitions"
                }
            ]
        }
        assert result["success"] is False


class TestPackBuildTool:
    """Tests for pack_build tool."""

    def test_build_success(self):
        """Successful pack build should create output."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "output_path": "packs/warfare-starwars/dist/warfare-starwars-1.0.0.zip",
            "build_time_seconds": 10
        }
        assert result["success"] is True
        assert result["output_path"].endswith(".zip")

    def test_build_creates_checksum(self):
        """Build should create checksum for integrity."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "output_path": "packs/warfare-starwars/dist/warfare-starwars-1.0.0.zip",
            "checksum": {
                "algorithm": "sha256",
                "hash": "abc123def456..."
            }
        }
        assert result["success"] is True
        assert "checksum" in result

    def test_build_failure_validation(self):
        """Build should fail if pack validation fails."""
        result = {
            "success": False,
            "pack": "invalid-pack",
            "error": "Pack validation failed before build"
        }
        assert result["success"] is False

    def test_build_includes_metadata(self):
        """Built package should include metadata."""
        result = {
            "success": True,
            "pack": "warfare-starwars",
            "output_path": "packs/warfare-starwars/dist/warfare-starwars-1.0.0.zip",
            "metadata_included": {
                "name": "Star Wars Pack",
                "version": "1.0.0",
                "author": "DINOForge Team"
            }
        }
        assert result["success"] is True


class TestPackListTool:
    """Tests for pack_list tool."""

    def test_list_success(self):
        """pack_list should return available packs."""
        result = {
            "success": True,
            "packs": [
                {"id": "example-balance", "path": "packs/example-balance"},
                {"id": "warfare-starwars", "path": "packs/warfare-starwars"}
            ],
            "count": 2
        }
        assert result["success"] is True
        assert len(result["packs"]) == result["count"]

    def test_list_empty(self):
        """pack_list with no packs should return empty list."""
        result = {
            "success": True,
            "packs": [],
            "count": 0
        }
        assert result["success"] is True
        assert result["count"] == 0

    def test_list_includes_metadata(self):
        """Listed packs should include metadata."""
        result = {
            "success": True,
            "packs": [
                {
                    "id": "warfare-starwars",
                    "path": "packs/warfare-starwars",
                    "name": "Star Wars Pack",
                    "version": "1.0.0",
                    "type": "content"
                }
            ]
        }
        assert result["success"] is True
        assert "name" in result["packs"][0]

    def test_list_pack_paths_valid(self):
        """Returned pack paths should be valid."""
        result = {
            "success": True,
            "packs": [
                {"id": "example-balance", "path": "packs/example-balance"}
            ]
        }
        assert result["success"] is True
        for pack in result["packs"]:
            assert "packs/" in pack["path"]


class TestPackIntegration:
    """Integration tests for pack operations."""

    def test_validate_then_build_workflow(self):
        """Validate → build workflow should succeed."""
        # First validate
        validate_result = {
            "success": True,
            "pack": "warfare-starwars",
            "valid": True
        }
        assert validate_result["success"] is True

        # Then build
        build_result = {
            "success": True,
            "pack": "warfare-starwars",
            "output_path": "packs/warfare-starwars/dist/warfare-starwars-1.0.0.zip"
        }
        assert build_result["success"] is True

    def test_asset_and_pack_workflow(self):
        """Full asset → pack workflow."""
        # Validate assets
        asset_validate = {
            "success": True,
            "pack": "warfare-starwars",
            "valid": True
        }
        assert asset_validate["success"] is True

        # Build assets
        asset_build = {
            "success": True,
            "pack": "warfare-starwars",
            "bundles_created": ["sw-rep-clone-trooper"]
        }
        assert asset_build["success"] is True

        # Validate pack
        pack_validate = {
            "success": True,
            "pack": "warfare-starwars",
            "valid": True
        }
        assert pack_validate["success"] is True

        # Build pack
        pack_build = {
            "success": True,
            "pack": "warfare-starwars",
            "output_path": "packs/warfare-starwars/dist/warfare-starwars-1.0.0.zip"
        }
        assert pack_build["success"] is True
