#nullable enable

namespace DINOForge.Tests.GameLaunch;

/// <summary>
/// Canonical warfare-starwars unit IDs from <c>packs/warfare-starwars/units/*.yaml</c>
/// (14 Republic + 14 CIS). Used by GameLaunch pack tests; keep in sync with pack YAML.
/// </summary>
internal static class WarfareStarwarsPackUnits
{
    public const string PackId = "warfare-starwars";

    public static readonly string[] All =
    {
        // republic_units.yaml
        "rep_clone_militia",
        "rep_clone_trooper",
        "rep_clone_heavy",
        "rep_clone_sharpshooter",
        "rep_barc_speeder",
        "rep_atte_crew",
        "rep_clone_medic",
        "rep_arf_trooper",
        "rep_arc_trooper",
        "rep_jedi_knight",
        "rep_clone_wall_guard",
        "rep_clone_sniper",
        "rep_clone_commando",
        "rep_v19_torrent",
        // cis_units.yaml
        "cis_b1_battle_droid",
        "cis_b1_squad",
        "cis_b2_super_battle_droid",
        "cis_sniper_droid",
        "cis_stap_pilot",
        "cis_aat_crew",
        "cis_medical_droid",
        "cis_probe_droid",
        "cis_bx_commando_droid",
        "cis_general_grievous",
        "cis_droideka",
        "cis_dwarf_spider_droid",
        "cis_magnaguard",
        "cis_tri_fighter",
    };
}
