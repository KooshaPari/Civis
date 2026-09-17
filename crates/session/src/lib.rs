//! civ-session — Session management for PvE, hot-seat, observer, and challenge modes (CIV-0900).
//!
//! This crate provides session-level abstractions that wrap the simulation engine:
//! - **PvE mode** (FR-SESS-001): human player vs AI civilizations.
//! - **Hot-seat multiplayer** (FR-SESS-002): multiple human players, one active at a time.
//! - **Observer mode** (FR-SESS-003): read-only session access without influencing simulation.
//! - **Challenge mode** (FR-SESS-004): async seed submission for competitive scoring.
//! - **Turn boundary events** (FR-SESS-006): `session.turn.start.v1` / `session.turn.end.v1` in hot-seat.
//!
//! Speed control (FR-SESS-005) lives in the server crate (`sim.set_speed`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Unique session identifier.
pub type SessionId = Uuid;

/// A player can be human, AI, or an observer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlayerKind {
    /// Human-controlled player.
    Human,
    /// AI-controlled civilization.
    Ai,
    /// Read-only observer (FR-SESS-003).
    Observer,
}

/// Speed multiplier for the session (FR-SESS-005 reference).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionSpeed {
    /// Simulation is paused.
    Paused,
    /// Normal speed (1 tick per interval).
    Normal,
    /// Double speed.
    Fast,
    /// Quadruple speed.
    Turbo,
}

/// Game modes supported by the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameMode {
    /// PvE: human vs AI (FR-SESS-001).
    PvE,
    /// Hot-seat multiplayer (FR-SESS-002).
    HotSeat,
    /// Observer only (FR-SESS-003).
    Observer,
    /// Challenge mode with async seed submission (FR-SESS-004).
    Challenge,
}

/// A player entry in the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    /// Unique player ID within the session.
    pub id: PlayerId,
    /// Display name.
    pub name: String,
    /// Whether this player is human, AI, or observer.
    pub kind: PlayerKind,
    /// Civilization index this player controls (if any).
    pub civ_index: Option<u32>,
}

/// Player identifier (UUID).
pub type PlayerId = Uuid;

/// Turn boundary event emitted during hot-seat mode (FR-SESS-006).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnEvent {
    /// Session this event belongs to.
    pub session_id: SessionId,
    /// Tick at which the turn boundary occurs.
    pub tick: u64,
    /// Player whose turn is starting or ending.
    pub player_id: PlayerId,
    /// Whether this is a turn start or turn end.
    pub kind: TurnEventKind,
}

/// Kind of turn boundary event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TurnEventKind {
    /// `session.turn.start.v1`
    Start,
    /// `session.turn.end.v1`
    End,
}

/// Challenge submission for async scoring (FR-SESS-004).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeSubmission {
    /// Submission ID.
    pub id: Uuid,
    /// The civilization seed (serialized world-state hash or seed u64).
    pub seed: u64,
    /// Submission timestamp (epoch millis).
    pub submitted_at: u64,
    /// Optional player name.
    pub player_name: Option<String>,
    /// Score computed by the challenge evaluator (filled after evaluation).
    pub score: Option<i64>,
}

/// Session errors.
#[derive(Debug, Error)]
pub enum SessionError {
    /// Session not found.
    #[error("session not found: {0}")]
    NotFound(String),

    /// Player not in session.
    #[error("player {0} not in session {1}")]
    PlayerNotFound(PlayerId, SessionId),

    /// Mode mismatch for the requested operation.
    #[error("operation not available in {mode:?} mode")]
    ModeMismatch {
        /// The current session mode.
        mode: GameMode,
    },

    /// Observer attempted a mutating action.
    #[error("observers cannot mutate simulation state")]
    ObserverReadOnly,

    /// Turn not active for this player.
    #[error("not your turn (player {0})")]
    NotYourTurn(PlayerId),

    /// Session already started.
    #[error("session already started")]
    AlreadyStarted,

    /// Duplicate player.
    #[error("player {0} already in session")]
    DuplicatePlayer(PlayerId),

    /// Session is at capacity.
    #[error("session is full ({current}/{max})")]
    Full {
        /// Current player count.
        current: usize,
        /// Maximum capacity.
        max: usize,
    },
}

/// Result alias.
pub type SessionResult<T> = Result<T, SessionError>;

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

/// A game session wrapping the simulation engine (FR-SESS-001 through FR-SESS-006).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Session UUID.
    pub id: SessionId,
    /// Game mode.
    pub mode: GameMode,
    /// Registered players.
    pub players: Vec<Player>,
    /// Whether the session has been started.
    pub started: bool,
    /// Current tick of the simulation.
    pub tick: u64,
    /// Current speed setting.
    pub speed: SessionSpeed,
    /// Active player index (for hot-seat: index into `players`).
    pub active_player_index: Option<usize>,
    /// Turn events emitted during the session.
    pub turn_events: Vec<TurnEvent>,
    /// Challenge submissions (only for Challenge mode).
    pub challenge_submissions: Vec<ChallengeSubmission>,
    /// Maximum number of players for this session.
    pub max_players: usize,
    /// RNG seed for the simulation.
    pub seed: u64,
}

impl Session {
    /// Create a new PvE session with the given seed (FR-SESS-001).
    #[must_use]
    pub fn new_pve(seed: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            mode: GameMode::PvE,
            players: Vec::new(),
            started: false,
            tick: 0,
            speed: SessionSpeed::Normal,
            active_player_index: None,
            turn_events: Vec::new(),
            challenge_submissions: Vec::new(),
            max_players: 8,
            seed,
        }
    }

    /// Create a new hot-seat multiplayer session (FR-SESS-002).
    #[must_use]
    pub fn new_hotseat(seed: u64, max_players: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            mode: GameMode::HotSeat,
            players: Vec::new(),
            started: false,
            tick: 0,
            speed: SessionSpeed::Normal,
            active_player_index: None,
            turn_events: Vec::new(),
            challenge_submissions: Vec::new(),
            max_players,
            seed,
        }
    }

    /// Create a new observer session (FR-SESS-003).
    #[must_use]
    pub fn new_observer() -> Self {
        Self {
            id: Uuid::new_v4(),
            mode: GameMode::Observer,
            players: Vec::new(),
            started: false,
            tick: 0,
            speed: SessionSpeed::Normal,
            active_player_index: None,
            turn_events: Vec::new(),
            challenge_submissions: Vec::new(),
            max_players: 16,
            seed: 0,
        }
    }

    /// Create a new challenge mode session (FR-SESS-004).
    #[must_use]
    pub fn new_challenge(seed: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            mode: GameMode::Challenge,
            players: Vec::new(),
            started: false,
            tick: 0,
            speed: SessionSpeed::Normal,
            active_player_index: None,
            turn_events: Vec::new(),
            challenge_submissions: Vec::new(),
            max_players: 1,
            seed,
        }
    }

    /// Add a human player to the session.
    pub fn add_human_player(&mut self, name: impl Into<String>) -> SessionResult<PlayerId> {
        self.add_player(name, PlayerKind::Human)
    }

    /// Add an AI player to the session.
    pub fn add_ai_player(&mut self, name: impl Into<String>) -> SessionResult<PlayerId> {
        self.add_player(name, PlayerKind::Ai)
    }

    /// Add an observer to the session (FR-SESS-003).
    pub fn add_observer(&mut self, name: impl Into<String>) -> SessionResult<PlayerId> {
        if self.mode == GameMode::Challenge {
            return Err(SessionError::ModeMismatch {
                mode: self.mode,
            });
        }
        self.add_player(name, PlayerKind::Observer)
    }

    fn add_player(&mut self, name: impl Into<String>, kind: PlayerKind) -> SessionResult<PlayerId> {
        if self.started {
            return Err(SessionError::AlreadyStarted);
        }
        if self.players.len() >= self.max_players {
            return Err(SessionError::Full {
                current: self.players.len(),
                max: self.max_players,
            });
        }
        let id = Uuid::new_v4();
        let civ_index = if matches!(kind, PlayerKind::Human | PlayerKind::Ai) {
            let idx = self.players.iter().filter(|p| p.kind != PlayerKind::Observer).count() as u32;
            Some(idx)
        } else {
            None
        };
        self.players.push(Player {
            id,
            name: name.into(),
            kind,
            civ_index,
        });
        Ok(id)
    }

    /// Start the session. Sets the active player for hot-seat mode.
    pub fn start(&mut self) -> SessionResult<()> {
        if self.started {
            return Err(SessionError::AlreadyStarted);
        }
        self.started = true;
        if self.mode == GameMode::HotSeat && !self.players.is_empty() {
            self.active_player_index = Some(0);
        }
        Ok(())
    }

    /// Advance the tick. For hot-seat mode, emits turn boundary events (FR-SESS-006).
    pub fn advance_tick(&mut self) -> Vec<TurnEvent> {
        let mut events = Vec::new();
        self.tick += 1;

        if self.mode == GameMode::HotSeat {
            if let Some(active_idx) = self.active_player_index {
                // Emit turn.end for the current player
                let player = &self.players[active_idx];
                let end_event = TurnEvent {
                    session_id: self.id,
                    tick: self.tick,
                    player_id: player.id,
                    kind: TurnEventKind::End,
                };
                self.turn_events.push(end_event.clone());
                events.push(end_event);

                // Advance to next human player
                let next_human = self.find_next_human_player(active_idx);
                self.active_player_index = Some(next_human);

                // Emit turn.start for the next player
                let player = &self.players[next_human];
                let start_event = TurnEvent {
                    session_id: self.id,
                    tick: self.tick,
                    player_id: player.id,
                    kind: TurnEventKind::Start,
                };
                self.turn_events.push(start_event.clone());
                events.push(start_event);
            }
        }
        events
    }

    /// Find the next human player index after the given index (wrapping).
    fn find_next_human_player(&self, after: usize) -> usize {
        let len = self.players.len();
        for offset in 1..=len {
            let idx = (after + offset) % len;
            if self.players[idx].kind == PlayerKind::Human {
                return idx;
            }
        }
        // Fallback: no human players; return after+1
        (after + 1) % len
    }

    /// Submit a challenge seed (FR-SESS-004).
    pub fn submit_challenge(
        &mut self,
        seed: u64,
        player_name: Option<String>,
        submitted_at: u64,
    ) -> SessionResult<Uuid> {
        if self.mode != GameMode::Challenge {
            return Err(SessionError::ModeMismatch {
                mode: self.mode,
            });
        }
        let id = Uuid::new_v4();
        self.challenge_submissions.push(ChallengeSubmission {
            id,
            seed,
            submitted_at,
            player_name,
            score: None,
        });
        Ok(id)
    }

    /// Get the current active player (hot-seat mode).
    #[must_use]
    pub fn active_player(&self) -> Option<&Player> {
        self.active_player_index.and_then(|idx| self.players.get(idx))
    }

    /// Check whether the given player is an observer (FR-SESS-003).
    #[must_use]
    pub fn is_observer(&self, player_id: PlayerId) -> bool {
        self.players.iter().any(|p| p.id == player_id && p.kind == PlayerKind::Observer)
    }

    /// Get human players in the session.
    #[must_use]
    pub fn human_players(&self) -> Vec<&Player> {
        self.players.iter().filter(|p| p.kind == PlayerKind::Human).collect()
    }

    /// Get AI players in the session.
    #[must_use]
    pub fn ai_players(&self) -> Vec<&Player> {
        self.players.iter().filter(|p| p.kind == PlayerKind::Ai).collect()
    }

    /// Serialize the session to JSON for persistence.
    #[must_use]
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("session serialization should not fail")
    }

    /// Deserialize a session from JSON.
    pub fn from_json(json: &str) -> SessionResult<Self> {
        serde_json::from_str(json).map_err(|e| SessionError::NotFound(e.to_string()))
    }
}

// ---------------------------------------------------------------------------
// Session event types (for the event bus)
// ---------------------------------------------------------------------------

/// Session-level events emitted to the event bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionEvent {
    /// `session.turn.start.v1` — hot-seat turn started (FR-SESS-006).
    TurnStart(TurnEvent),
    /// `session.turn.end.v1` — hot-seat turn ended (FR-SESS-006).
    TurnEnd(TurnEvent),
    /// `session.speed_changed.v1` — speed changed (FR-SESS-005 reference).
    SpeedChanged {
        /// Session ID.
        session_id: SessionId,
        /// New speed.
        speed: SessionSpeed,
        /// Tick at which the change occurred.
        tick: u64,
    },
    /// `session.challenge.submitted.v1` — challenge seed submitted (FR-SESS-004).
    ChallengeSubmitted {
        /// Submission ID.
        submission_id: Uuid,
        /// Session ID.
        session_id: SessionId,
        /// Tick.
        tick: u64,
    },
}

/// Format a turn event as JSON for the event bus.
#[must_use]
pub fn format_turn_event(event: &TurnEvent) -> String {
    let event_type = match event.kind {
        TurnEventKind::Start => "session.turn.start.v1",
        TurnEventKind::End => "session.turn.end.v1",
    };
    serde_json::json!({
        "event": event_type,
        "session_id": event.session_id,
        "tick": event.tick,
        "player_id": event.player_id,
    })
    .to_string()
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // FR-SESS-001: PvE mode
    // -------------------------------------------------------------------------

    #[test]
    fn session_pve_mode_supported() {
        let mut session = Session::new_pve(42);
        assert_eq!(session.mode, GameMode::PvE);
        assert_eq!(session.seed, 42);

        let human_id = session.add_human_player("Alice").expect("add human");
        let ai_id = session.add_ai_player("Bot").expect("add ai");

        session.start().expect("start");

        assert!(session.human_players().iter().any(|p| p.id == human_id));
        assert!(session.ai_players().iter().any(|p| p.id == ai_id));
    }

    #[test]
    fn pve_multiple_ai_opponents() {
        let mut session = Session::new_pve(1);
        let _h = session.add_human_player("Human").expect("h");
        let _a1 = session.add_ai_player("AI-1").expect("a1");
        let _a2 = session.add_ai_player("AI-2").expect("a2");
        let _a3 = session.add_ai_player("AI-3").expect("a3");
        session.start().expect("start");

        assert_eq!(session.ai_players().len(), 3);
        assert_eq!(session.human_players().len(), 1);
    }

    // -------------------------------------------------------------------------
    // FR-SESS-002: Hot-seat multiplayer
    // -------------------------------------------------------------------------

    #[test]
    fn hotseat_multi_human() {
        let mut session = Session::new_hotseat(99, 4);
        let p1 = session.add_human_player("Player 1").expect("p1");
        let p2 = session.add_human_player("Player 2").expect("p2");
        session.start().expect("start");

        // First active player is index 0
        assert_eq!(session.active_player().unwrap().id, p1);

        // Advance tick — emits turn.end for p1, turn.start for p2
        let events = session.advance_tick();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].kind, TurnEventKind::End);
        assert_eq!(events[0].player_id, p1);
        assert_eq!(events[1].kind, TurnEventKind::Start);
        assert_eq!(events[1].player_id, p2);

        // Now p2 is active
        assert_eq!(session.active_player().unwrap().id, p2);

        // Advance again — cycles back to p1
        let events = session.advance_tick();
        assert_eq!(events[0].player_id, p2);
        assert_eq!(events[1].player_id, p1);
        assert_eq!(session.active_player().unwrap().id, p1);
    }

    #[test]
    fn hotseat_with_ai_players_skips_to_next_human() {
        let mut session = Session::new_hotseat(1, 4);
        let p1 = session.add_human_player("H1").expect("h1");
        let _ai = session.add_ai_player("AI").expect("ai");
        let p2 = session.add_human_player("H2").expect("h2");
        session.start().expect("start");

        assert_eq!(session.active_player().unwrap().id, p1);
        let events = session.advance_tick();
        // Should skip the AI and go directly to p2
        assert_eq!(events[1].player_id, p2);
    }

    #[test]
    fn hotseat_respects_max_capacity() {
        let mut session = Session::new_hotseat(1, 2);
        let _p1 = session.add_human_player("P1").expect("p1");
        let _p2 = session.add_human_player("P2").expect("p2");
        let err = session.add_human_player("P3").expect_err("should be full");
        assert!(matches!(err, SessionError::Full { .. }));
    }

    // -------------------------------------------------------------------------
    // FR-SESS-003: Observer mode
    // -------------------------------------------------------------------------

    #[test]
    fn observer_read_only() {
        let mut session = Session::new_pve(42);
        let _h = session.add_human_player("Player").expect("h");
        let obs_id = session.add_observer("Watcher").expect("obs");
        session.start().expect("start");

        assert!(session.is_observer(obs_id));
        assert!(!session.is_observer(Uuid::new_v4())); // random ID is not an observer
    }

    #[test]
    fn observer_session_allows_many_observers() {
        let mut session = Session::new_observer();
        let _o1 = session.add_observer("O1").expect("o1");
        let _o2 = session.add_observer("O2").expect("o2");
        let _o3 = session.add_observer("O3").expect("o3");
        session.start().expect("start");
        assert_eq!(session.players.len(), 3);
        assert!(session.players.iter().all(|p| p.kind == PlayerKind::Observer));
    }

    #[test]
    fn observer_no_civ_index() {
        let mut session = Session::new_pve(1);
        let obs_id = session.add_observer("Watcher").expect("obs");
        let player = session.players.iter().find(|p| p.id == obs_id).unwrap();
        assert!(player.civ_index.is_none());
    }

    #[test]
    fn human_players_get_civ_index() {
        let mut session = Session::new_pve(1);
        let p1 = session.add_human_player("P1").expect("p1");
        let p2 = session.add_human_player("P2").expect("p2");
        let h1 = session.players.iter().find(|p| p.id == p1).unwrap();
        let h2 = session.players.iter().find(|p| p.id == p2).unwrap();
        assert_eq!(h1.civ_index, Some(0));
        assert_eq!(h2.civ_index, Some(1));
    }

    // -------------------------------------------------------------------------
    // FR-SESS-004: Challenge mode
    // -------------------------------------------------------------------------

    #[test]
    fn async_submission_accepted() {
        let mut session = Session::new_challenge(12345);
        let sub_id = session
            .submit_challenge(999, Some("Alice".into()), 1_000_000)
            .expect("submit");
        assert_eq!(session.challenge_submissions.len(), 1);
        assert_eq!(session.challenge_submissions[0].seed, 999);
        assert_eq!(session.challenge_submissions[0].id, sub_id);
    }

    #[test]
    fn challenge_multiple_submissions() {
        let mut session = Session::new_challenge(1);
        let _s1 = session.submit_challenge(10, None, 100).expect("s1");
        let _s2 = session.submit_challenge(20, Some("Bob".into()), 200).expect("s2");
        let _s3 = session.submit_challenge(30, None, 300).expect("s3");
        assert_eq!(session.challenge_submissions.len(), 3);
    }

    #[test]
    fn challenge_rejects_observer_add() {
        let mut session = Session::new_challenge(1);
        let err = session.add_observer("Watcher").expect_err("observers not allowed in challenge");
        assert!(matches!(err, SessionError::ModeMismatch { .. }));
    }

    #[test]
    fn challenge_rejects_non_challenge_submit() {
        let mut session = Session::new_pve(1);
        let err = session
            .submit_challenge(42, None, 0)
            .expect_err("submit should fail in PvE");
        assert!(matches!(err, SessionError::ModeMismatch { .. }));
    }

    // -------------------------------------------------------------------------
    // FR-SESS-006: Turn boundary events
    // -------------------------------------------------------------------------

    #[test]
    fn turn_events_emitted() {
        let mut session = Session::new_hotseat(1, 4);
        let p1 = session.add_human_player("P1").expect("p1");
        let p2 = session.add_human_player("P2").expect("p2");
        session.start().expect("start");

        let events = session.advance_tick();
        assert_eq!(events.len(), 2);

        // Verify turn.end for p1
        assert_eq!(events[0].kind, TurnEventKind::End);
        assert_eq!(events[0].player_id, p1);
        assert_eq!(events[0].tick, 1);
        assert_eq!(events[0].session_id, session.id);

        // Verify turn.start for p2
        assert_eq!(events[1].kind, TurnEventKind::Start);
        assert_eq!(events[1].player_id, p2);
        assert_eq!(events[1].tick, 1);
    }

    #[test]
    fn turn_events_recorded_in_session() {
        let mut session = Session::new_hotseat(1, 4);
        let _p1 = session.add_human_player("P1").expect("p1");
        let _p2 = session.add_human_player("P2").expect("p2");
        session.start().expect("start");

        session.advance_tick();
        session.advance_tick();

        // 2 ticks x 2 events (start+end) = 4 turn events
        assert_eq!(session.turn_events.len(), 4);
    }

    #[test]
    fn non_hotseat_no_turn_events() {
        let mut session = Session::new_pve(1);
        let _h = session.add_human_player("H").expect("h");
        session.start().expect("start");
        let events = session.advance_tick();
        assert!(events.is_empty());
        assert!(session.turn_events.is_empty());
    }

    // -------------------------------------------------------------------------
    // Session start / lifecycle
    // -------------------------------------------------------------------------

    #[test]
    fn cannot_start_twice() {
        let mut session = Session::new_pve(1);
        session.start().expect("start");
        let err = session.start().expect_err("double start");
        assert!(matches!(err, SessionError::AlreadyStarted));
    }

    #[test]
    fn cannot_add_player_after_start() {
        let mut session = Session::new_pve(1);
        session.start().expect("start");
        let err = session.add_human_player("Late").expect_err("late");
        assert!(matches!(err, SessionError::AlreadyStarted));
    }

    // -------------------------------------------------------------------------
    // Serialization roundtrip
    // -------------------------------------------------------------------------

    #[test]
    fn session_json_roundtrip() {
        let mut session = Session::new_hotseat(42, 4);
        let _p1 = session.add_human_player("Alice").expect("p1");
        let _p2 = session.add_human_player("Bob").expect("p2");
        session.start().expect("start");
        session.advance_tick();

        let json = session.to_json();
        let restored = Session::from_json(&json).expect("from_json");
        assert_eq!(restored.id, session.id);
        assert_eq!(restored.mode, session.mode);
        assert_eq!(restored.tick, session.tick);
        assert_eq!(restored.players.len(), session.players.len());
        assert_eq!(restored.turn_events.len(), session.turn_events.len());
    }

    // -------------------------------------------------------------------------
    // Turn event formatting
    // -------------------------------------------------------------------------

    #[test]
    fn turn_event_json_format() {
        let event = TurnEvent {
            session_id: Uuid::nil(),
            tick: 5,
            player_id: Uuid::nil(),
            kind: TurnEventKind::Start,
        };
        let json = format_turn_event(&event);
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse");
        assert_eq!(v["event"], "session.turn.start.v1");
        assert_eq!(v["tick"], 5);
    }
}
