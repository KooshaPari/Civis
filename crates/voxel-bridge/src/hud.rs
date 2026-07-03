//! TODO(FR): HUD module stub for UI panels, trees, and palettes.

/// Diplomacy FSM state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiplomacyFsm;

/// Diplomacy panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiplomacyPanel;

/// Event feed item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventFeedItem;

/// Event feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventFeed;

/// Event severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventSeverity {
    /// TODO
    Low,
}

/// Menu kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    /// TODO
    Main,
}

/// Menu stack error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuStackError;

/// Menu stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MenuStack;

/// Tech tree node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TechNode;

/// Tech tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TechTree;

/// Tech tree error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TechTreeError;

/// Tool palette entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolEntry;

/// Tool palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolPalette;

/// Tool palette error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolPaletteError;

/// Treaty slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreatySlot;

/// Hub palette schema version.
pub const HUB_PALETTE_SCHEMA_VERSION: u32 = 1;

/// Hub tech schema version.
pub const HUB_TECH_SCHEMA_VERSION: u32 = 1;
