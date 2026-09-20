//! Tests for FR-CIV-RTS-004
//!
//! Epic: FR-CIV-RTS
//!
//! This test file verifies FR FR-CIV-RTS-004: Command Queuing & Auto-Execute.
//! Multiple commands can be queued and drained in order.

#[cfg(test)]
mod fr_fr_civ_rts_004 {
    use civ_engine::command_queue::{Command, CommandKind, CommandQueue};

    /// Multiple commands queued and drained in FIFO order.
    #[test]
    fn queued_commands_drain_in_order() {
        let mut q = CommandQueue::new(20);
        let kinds = [
            CommandKind::Pause,
            CommandKind::SetSpeed(2),
            CommandKind::Resume,
            CommandKind::SetSpeed(5),
        ];
        for (i, kind) in kinds.iter().enumerate() {
            q.push(Command {
                client_id: 0,
                seq: i as u64,
                kind: CommandKind::Resume,
                tick_issued: 0,
            })
            .unwrap();
        }
        let drained = q.drain();
        assert_eq!(drained.len(), 4);
        for (idx, cmd) in drained.iter().enumerate() {
            assert_eq!(cmd.seq, idx as u64);
        }
    }

    /// Queue reports correct length and emptiness.
    #[test]
    fn queue_length_tracking() {
        let mut q = CommandQueue::new(10);
        assert!(q.is_empty());
        assert_eq!(q.len(), 0);
        q.push(Command {
            client_id: 0,
            seq: 0,
            kind: CommandKind::Pause,
            tick_issued: 0,
        })
        .unwrap();
        assert!(!q.is_empty());
        assert_eq!(q.len(), 1);
        q.pop();
        assert!(q.is_empty());
    }
}
