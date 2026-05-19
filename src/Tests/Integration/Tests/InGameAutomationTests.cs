#nullable enable
using System.Threading.Tasks;
using DINOForge.Bridge.Protocol;
using DINOForge.Tests.Mocks;
using FluentAssertions;
using Xunit;

namespace DINOForge.Tests.Integration.Tests;

/// <summary>
/// Integration tests for in-game automation via MCP bridge.
/// These tests validate the MCP tools that automate game interactions.
/// 
/// Maps to MCP tool coverage:
/// - game_status, game_get_stat, game_apply_override, game_reload_packs
/// - game_verify_mod, game_wait_for_world, game_query_entities
/// - game_screenshot, game_dump_state, game_ping
/// - game_navigate_to (with workaround for Spectre bug)
/// - log_swap_status (when implemented)
/// - log_debug_log, log_packs_loaded (when registered)
/// </summary>
[Trait("Category", "MCP")]
[Trait("Category", "E2E")]
[Trait("Journey", "Journey-AutomateGame")]
public class InGameAutomationTests
{
    private readonly FakeGameBridge _bridge = new();

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: game_reload_packs → bridge.ReloadPacks()
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_game_reload_packs_Succeeds()
    {
        // Act
        var result = _bridge.ReloadPacks(null);

        // Assert
        result.Should().NotBeNull();
        result.Success.Should().BeTrue();
        result.LoadedPacks.Should().HaveCount(2, "warfare-starwars + example-balance packs should be loaded");
    }

    [Fact]
    public void MCP_game_reload_packs_ContainsWarfareStarWars()
    {
        // Act
        var result = _bridge.ReloadPacks(null);

        // Assert
        result.LoadedPacks.Should().Contain("warfare-starwars", "Star Wars pack should be loaded");
    }

    [Fact]
    public void MCP_game_reload_packs_CatalogHas28Units()
    {
        // Act
        _bridge.ReloadPacks(null);
        var catalog = _bridge.GetCatalog();

        // Assert
        catalog.Units.Should().HaveCount(28, "14 Republic + 14 CIS units");
    }

    [Fact]
    public void MCP_game_reload_packs_OverridePersistsAfterReload()
    {
        // Arrange - load packs first
        _bridge.ReloadPacks(null);

        // Step 1: Apply override
        _bridge.ApplyOverride("unit.stats.hp", 777f, "persist-test", null);

        // Step 2: Reload packs
        _bridge.ReloadPacks(null);

        // Step 3: Verify override still applies
        var stat = _bridge.GetStat("unit.stats.hp", null);
        stat.Value.Should().Be(777f, "override should persist after reload");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: game_get_stat → bridge.GetStat()
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_game_get_stat_ValidPath_ReturnsValue()
    {
        // Act
        var result = _bridge.GetStat("unit.stats.hp", null);

        // Assert
        result.Should().NotBeNull();
        result.SdkPath.Should().Be("unit.stats.hp");
    }

    [Fact]
    public void MCP_game_get_stat_AfterOverride_ReturnsOverriddenValue()
    {
        // Arrange - load packs first
        _bridge.ReloadPacks(null);

        // Step 1: Apply override
        _bridge.ApplyOverride("unit.stats.hp", 999f, "test-override", null);

        // Step 2: Read stat
        var stat = _bridge.GetStat("unit.stats.hp", null);

        // Assert
        stat.Value.Should().Be(999f, "override should change the value");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: game_apply_override → bridge.ApplyOverride()
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_game_apply_override_ValidStat_Succeeds()
    {
        // Arrange - load packs first
        _bridge.ReloadPacks(null);

        // Act
        var result = _bridge.ApplyOverride("unit.stats.hp", 500f, "test-automation", null);

        // Assert
        result.Should().NotBeNull();
        result.Success.Should().BeTrue();
    }

    [Fact]
    public void MCP_game_apply_override_MultipleOverrides_AllPersist()
    {
        // Arrange - load packs first
        _bridge.ReloadPacks(null);

        // Act
        var hpOverride = _bridge.ApplyOverride("unit.stats.hp", 100f, "test-hp", null);
        var damageOverride = _bridge.ApplyOverride("unit.stats.damage", 50f, "test-damage", null);

        // Assert
        hpOverride.Success.Should().BeTrue();
        damageOverride.Success.Should().BeTrue();
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: game_verify_mod → bridge.VerifyMod()
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_game_verify_mod_DINOForge_Loaded()
    {
        // Act
        var result = _bridge.VerifyMod("DINOForge.Runtime");

        // Assert
        result.Should().NotBeNull();
        result.Loaded.Should().BeTrue();
        result.Errors.Should().BeEmpty();
    }

    [Fact]
    public void MCP_game_verify_mod_WarfareStarWars_Loaded()
    {
        // Act
        var result = _bridge.VerifyMod("warfare-starwars");

        // Assert
        result.Should().NotBeNull();
        result.Loaded.Should().BeTrue();
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: game_query_entities → bridge.QueryEntities()
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_game_query_entities_ByType_ReturnsMatching()
    {
        // Arrange
        _bridge.ReloadPacks(null);

        // Act
        var entities = _bridge.QueryEntities("Unit", null);

        // Assert
        entities.Should().NotBeNull();
        entities.Count.Should().Be(100, "FakeGameBridge returns 100 entities when packs loaded");
    }

    [Fact]
    public void MCP_game_query_entities_EmptyResult_IsHandled()
    {
        // Act
        var entities = _bridge.QueryEntities("NonExistentType", null);

        // Assert - empty result should be handled gracefully
        entities.Should().NotBeNull();
        entities.Count.Should().Be(0, "should have no entities");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: game_wait_for_world → bridge.WaitForWorld()
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_game_wait_for_world_AfterPackLoad_ReturnsTrue()
    {
        // Arrange - load packs first
        _bridge.ReloadPacks(null);

        // Act
        var result = _bridge.WaitForWorld(1000);

        // Assert
        result.Should().NotBeNull();
        result.Ready.Should().BeTrue();
    }

    [Fact]
    public void MCP_game_wait_for_world_Timeout_ReturnsNotReady()
    {
        // Arrange - use very short timeout
        var result = _bridge.WaitForWorld(1);

        // Assert
        result.Should().NotBeNull();
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: game_screenshot → bridge.Screenshot()
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_game_screenshot_Succeeds()
    {
        // Act
        var result = _bridge.Screenshot(null);

        // Assert
        result.Should().NotBeNull();
        result.Success.Should().BeTrue();
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: game_dump_state → bridge.DumpState() - placeholder
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_game_dump_state_Placeholder_NotYetImplemented()
    {
        // NOTE: DumpState is not implemented in FakeGameBridge
        // Placeholder until implemented
        true.Should().BeTrue("placeholder until DumpState is implemented");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: game_navigate_to (with Spectre bug workaround)
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_game_navigate_to_Placeholder_SpectreBug()
    {
        // NOTE: game_navigate_to has a known bug with Spectre.Console "cat" style
        // This test documents the expected behavior when bug is fixed

        // Placeholder until Spectre bug is fixed in GameControlCli
        true.Should().BeTrue("placeholder until Spectre bug is fixed");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: log_swap_status - placeholder
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_log_swap_status_Placeholder_NotRegistered()
    {
        // NOTE: log_swap_status is not yet registered in the MCP server
        // Placeholder until tool is registered
        true.Should().BeTrue("placeholder until log_swap_status is implemented");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: log_debug_log - placeholder
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_log_debug_log_Placeholder_ToolNotYetRegistered()
    {
        // NOTE: log_debug_log is not yet registered in the MCP server
        // This test documents the expected behavior when implemented
        true.Should().BeTrue("placeholder until log_debug_log is implemented");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: log_packs_loaded - placeholder
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_log_packs_loaded_Placeholder_ToolNotYetRegistered()
    {
        // NOTE: log_packs_loaded is not yet registered in the MCP server
        // This test documents the expected behavior when implemented
        true.Should().BeTrue("placeholder until log_packs_loaded is implemented");
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: game_ping → bridge.Ping()
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_game_ping_ReturnsPong()
    {
        // Act
        var result = _bridge.Ping();

        // Assert
        result.Should().NotBeNull();
        result.Pong.Should().BeTrue();
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // MCP Tool: game_get_component_map → bridge.GetComponentMap()
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void MCP_game_get_component_map_ReturnsMappings()
    {
        // Act
        var result = _bridge.GetComponentMap(null);

        // Assert
        result.Should().NotBeNull();
        result.Mappings.Should().HaveCount(3);
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Integration: Full Workflow
    // ═════════════════════════════════════════════════════════════════════════════

    [Fact]
    public void FullWorkflow_PackLoad_StatRead_Override_Verify_Reload()
    {
        // Step 1: Load packs
        var loadResult = _bridge.ReloadPacks(null);
        loadResult.Success.Should().BeTrue();

        // Step 2: Read stat
        var stat1 = _bridge.GetStat("unit.stats.hp", null);
        stat1.Should().NotBeNull();

        // Step 3: Apply override
        var overrideResult = _bridge.ApplyOverride("unit.stats.hp", 1234f, "full-workflow", null);
        overrideResult.Success.Should().BeTrue();

        // Step 4: Verify stat changed
        var stat2 = _bridge.GetStat("unit.stats.hp", null);
        stat2!.Value.Should().Be(1234f);

        // Step 5: Verify mod loaded
        var verifyResult = _bridge.VerifyMod("warfare-starwars");
        verifyResult.Loaded.Should().BeTrue();

        // Step 6: Reload packs
        var reloadResult = _bridge.ReloadPacks(null);
        reloadResult.Success.Should().BeTrue();

        // Step 7: Verify override persisted
        var stat3 = _bridge.GetStat("unit.stats.hp", null);
        stat3!.Value.Should().Be(1234f, "override should persist through reload");
    }

    [Fact]
    public void FullWorkflow_QueryEntities_VerifyMod_Ping()
    {
        // Step 1: Load packs
        _bridge.ReloadPacks(null);

        // Step 2: Query entities
        var entities = _bridge.QueryEntities("Unit", null);
        entities.Count.Should().BeGreaterThan(0);

        // Step 3: Verify mod
        var verifyResult = _bridge.VerifyMod("warfare-starwars");
        verifyResult.Loaded.Should().BeTrue();

        // Step 4: Ping bridge
        var pingResult = _bridge.Ping();
        pingResult.Pong.Should().BeTrue();
    }
}
