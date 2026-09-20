//! Tests for FR-CIV-RTS-012
//!
//! Epic: FR-CIV-RTS
//!
//! This test file verifies FR FR-CIV-RTS-012: Turn-Based vs Real-Time.
//! Pause/Resume/SetSpeed commands exist for speed control.

#[cfg(test)]
mod fr_fr_civ_rts_012 {
    use civ_engine::command_queue::{Command, CommandKind, CommandQueue};

    /// Pause command can be issued.
    #[test]
    fn pause_command() {
        let mut q = CommandQueue::new(10);
        q.push(Command {
            client_id: 0,
            seq: 0,
            kind: CommandKind::Pause,
            tick_issued: 0,
        })
        .unwrap();
        let cmd = q.pop().unwrap();
        assert!(matches!(cmd.kind, CommandKind::Pause));
    }

    /// Resume command can be issued.
    #[test]
    fn resume_command() {
        let mut q = CommandQueue::new(10);
        q.push(Command {
            client_id: 0,
            seq: 0,
            kind: CommandKind::Resume,
            tick_issued: 0,
        })
        .unwrap();
        let cmd = q.pop().unwrap();
        assert!(matches!(cmd.kind, CommandKind::Resume));
    }

    /// SetSpeed command with valid speed.
    #[test]
    fn set_speed_command() {
        let mut q = CommandQueue::new(10);
        q.push(Command {
            client_id: 0,
            seq: 0,
            kind: CommandKind::SetSpeed(5),
            tick_issued: 0,
        })
        .unwrap();
        let cmd = q.pop().unwrap();
        assert!(matches!(cmd.kind, CommandKind::SetSpeed(5)));
    }
}
