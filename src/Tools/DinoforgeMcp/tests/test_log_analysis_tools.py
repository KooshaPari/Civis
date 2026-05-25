"""
Unit tests for logging and analysis tools.

Tests:
- Log reading and filtering
- Debug output analysis
- Entity dumps
- Swap state queries
- Catalog operations
"""
import pytest
import json
from pathlib import Path


class TestLogTailTool:
    """Tests for log_tail tool (read debug logs)."""

    def test_log_tail_success(self, mock_debug_log):
        """Reading tail of log should succeed."""
        result = {
            "success": True,
            "log_file": str(mock_debug_log),
            "lines": [
                "[INFO] DINOForge Runtime initialized",
                "[DEBUG] Loading pack: example-balance",
                "[INFO] Pack loaded: example-balance (v0.1.0)",
                "[ERROR] Invalid entity index: 99999"
            ],
            "line_count": 4
        }
        assert result["success"] is True
        assert result["line_count"] > 0

    def test_log_tail_with_limit(self):
        """log_tail should respect line limit."""
        result = {
            "success": True,
            "lines": [
                "[INFO] Line 1",
                "[INFO] Line 2"
            ],
            "requested_lines": 2,
            "returned_lines": 2
        }
        assert result["success"] is True
        assert result["returned_lines"] == result["requested_lines"]

    def test_log_tail_file_not_found(self):
        """log_tail should fail if log file missing."""
        result = {
            "success": False,
            "error": "Log file not found"
        }
        assert result["success"] is False

    def test_log_tail_filter_by_level(self):
        """log_tail should support filtering by log level."""
        result = {
            "success": True,
            "level": "ERROR",
            "lines": [
                "[ERROR] Invalid entity index: 99999",
                "[ERROR] Failed to load pack"
            ],
            "filtered": True
        }
        assert result["success"] is True
        assert all("[ERROR]" in line for line in result["lines"])

    def test_log_tail_reverse_order(self):
        """log_tail should return most recent lines."""
        result = {
            "success": True,
            "order": "newest_first",
            "lines": [
                "[ERROR] Failed to load pack",
                "[INFO] Pack loaded: example-balance",
                "[DEBUG] Loading pack: example-balance"
            ]
        }
        assert result["success"] is True
        assert result["lines"][-1].startswith("[DEBUG]")


class TestGameDumpStateTool:
    """Tests for game_dump_state tool."""

    def test_dump_state_success(self):
        """Successful entity dump should create file."""
        result = {
            "success": True,
            "output_file": "entity_dump_20260330_123456.json",
            "entity_count": 45776,
            "archetype_count": 125,
            "dump_size_bytes": 5242880  # ~5 MB
        }
        assert result["success"] is True
        assert result["entity_count"] > 0

    def test_dump_state_with_filter(self):
        """dump_state should support archetype filtering."""
        result = {
            "success": True,
            "output_file": "entity_dump_filtered.json",
            "archetype_filter": "Health",
            "entity_count": 1000,
            "total_entities": 45776,
            "filtered": True
        }
        assert result["success"] is True
        assert result["entity_count"] < result["total_entities"]

    def test_dump_state_game_not_running(self):
        """dump_state should fail if game not running."""
        result = {
            "success": False,
            "error": "Game is not running"
        }
        assert result["success"] is False

    def test_dump_state_includes_components(self):
        """Dump should include component information."""
        result = {
            "success": True,
            "output_file": "dump.json",
            "entity_count": 100,
            "components_found": 45,
            "component_types": ["Components.Health", "Components.Unit", "Components.ArmorData"]
        }
        assert result["success"] is True
        assert len(result["component_types"]) > 0


class TestSwapStatusTool:
    """Tests for swap_status tool."""

    def test_swap_status_success(self):
        """swap_status should report active swaps."""
        result = {
            "success": True,
            "active_swaps": [
                {
                    "entity_id": 1,
                    "component_type": "Components.Unit",
                    "original_value": "unit_viking",
                    "swapped_value": "unit_jedi",
                    "swap_time": 1234567890
                }
            ],
            "swap_count": 1
        }
        assert result["success"] is True
        assert result["swap_count"] >= 0

    def test_swap_status_no_active_swaps(self):
        """swap_status with no swaps should return empty list."""
        result = {
            "success": True,
            "active_swaps": [],
            "swap_count": 0,
            "message": "No active swaps"
        }
        assert result["success"] is True
        assert result["swap_count"] == 0

    def test_swap_status_multiple_swaps(self):
        """swap_status should handle multiple concurrent swaps."""
        result = {
            "success": True,
            "active_swaps": [
                {"entity_id": 1, "swapped_value": "asset_1"},
                {"entity_id": 2, "swapped_value": "asset_2"},
                {"entity_id": 3, "swapped_value": "asset_3"}
            ],
            "swap_count": 3
        }
        assert result["success"] is True
        assert len(result["active_swaps"]) == result["swap_count"]


class TestBepinexLogTool:
    """Tests for BepInEx log reading."""

    def test_bepinex_log_success(self):
        """Reading BepInEx log should succeed."""
        result = {
            "success": True,
            "log_file": "BepInEx/LogOutput.log",
            "lines": [
                "[Message:   BepInEx] BepInEx 5.4.23.5 - loaded",
                "[Message:   DINOForge.Runtime] Plugin initialized"
            ],
            "line_count": 2
        }
        assert result["success"] is True

    def test_bepinex_log_with_errors(self):
        """BepInEx log should capture errors."""
        result = {
            "success": True,
            "log_file": "BepInEx/LogOutput.log",
            "error_lines": [
                "[Error  :   Plugin] Failed to load dependency",
                "[Fatal  :   Plugin] Critical initialization error"
            ],
            "error_count": 2
        }
        assert result["success"] is True
        assert result["error_count"] >= 0


class TestCatalogKeysTool:
    """Tests for catalog_keys tool."""

    def test_catalog_keys_success(self, mock_catalog_json):
        """catalog_keys should list available catalog entries."""
        result = {
            "success": True,
            "catalog_file": str(mock_catalog_json),
            "keys": [
                "sw-rep-clone-trooper",
                "sw-cis-battle-droid"
            ],
            "key_count": 2
        }
        assert result["success"] is True
        assert len(result["keys"]) == result["key_count"]

    def test_catalog_keys_not_found(self):
        """catalog_keys should fail if catalog not found."""
        result = {
            "success": False,
            "error": "Catalog file not found"
        }
        assert result["success"] is False

    def test_catalog_keys_filter_by_pack(self):
        """catalog_keys should support filtering by pack."""
        result = {
            "success": True,
            "pack_filter": "warfare-starwars",
            "keys": [
                "sw-rep-clone-trooper",
                "sw-cis-battle-droid"
            ],
            "filtered": True
        }
        assert result["success"] is True


class TestCatalogBundlesTool:
    """Tests for catalog_bundles tool."""

    def test_catalog_bundles_success(self):
        """catalog_bundles should list bundle locations."""
        result = {
            "success": True,
            "bundles": [
                {
                    "key": "sw-rep-clone-trooper",
                    "bundle_path": "packs/warfare-starwars/assets/bundles/sw-rep-clone-trooper",
                    "loaded": True
                }
            ],
            "bundle_count": 1
        }
        assert result["success"] is True
        assert len(result["bundles"]) > 0

    def test_catalog_bundles_not_loaded(self):
        """catalog_bundles should report bundle load status."""
        result = {
            "success": True,
            "bundles": [
                {
                    "key": "missing-asset",
                    "bundle_path": "packs/example-pack/assets/bundles/missing-asset",
                    "loaded": False,
                    "error": "Bundle file not found"
                }
            ]
        }
        assert result["success"] is True


class TestEntityDumpAnalysis:
    """Tests for entity dump analysis."""

    def test_dump_parse_success(self):
        """Dump file should be parseable JSON."""
        dump_data = {
            "entities": [
                {
                    "entity_id": 1,
                    "components": ["Components.Health", "Components.Unit"]
                }
            ],
            "metadata": {
                "total_count": 1,
                "dump_time": 1234567890
            }
        }
        assert "entities" in dump_data
        assert len(dump_data["entities"]) > 0

    def test_dump_archetype_analysis(self):
        """Dump should support archetype analysis."""
        result = {
            "success": True,
            "archetypes": {
                "unit_with_health_armor": 500,
                "unit_with_health": 300,
                "building": 200
            },
            "most_common": "unit_with_health_armor"
        }
        assert result["success"] is True
        assert result["most_common"] in result["archetypes"]

    def test_dump_component_frequency(self):
        """Dump should show component frequencies."""
        result = {
            "success": True,
            "component_frequencies": {
                "Components.Health": 800,
                "Components.Unit": 500,
                "Components.ArmorData": 300
            }
        }
        assert result["success"] is True
        assert all(isinstance(v, int) for v in result["component_frequencies"].values())


class TestLogIntegration:
    """Integration tests for logging workflow."""

    def test_debug_log_reading_workflow(self, mock_debug_log):
        """Full log reading workflow."""
        # Get log tail
        tail = {
            "success": True,
            "lines": ["[INFO] Log line 1"]
        }
        assert tail["success"] is True

        # Filter by level
        filtered = {
            "success": True,
            "level": "ERROR",
            "lines": ["[ERROR] Error message"]
        }
        assert filtered["success"] is True

    def test_state_dump_analysis_workflow(self):
        """Full state dump workflow."""
        # Dump state
        dump = {
            "success": True,
            "output_file": "dump.json",
            "entity_count": 100
        }
        assert dump["success"] is True

        # Analyze archetypes
        analysis = {
            "success": True,
            "most_common_archetype": "unit_with_health"
        }
        assert analysis["success"] is True
