//! Tests for FR-CIV-CORE-008
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-008: Multi-Client Command Ordering.
//! Commands from multiple clients are applied in deterministic order.

#[cfg(test)]
mod fr_fr_civ_core_008 {
    use civ_engine::command_queue::{Command, CommandError, CommandKind, CommandQueue};

    /// Commands pushed from different clients come out in FIFO order.
    #[test]
    fn commands_fifo_across_clients() {
        let mut q = CommandQueue::new(20);
        for i in 0..5u64 {
            q.push(Command {
                client_id: i % 3,
                seq: i,
                kind: CommandKind::Resume,
                tick_issued: i * 10,
            })
            .unwrap();
        }
        let drained = q.drain();
        assert_eq!(drained.len(), 5);
        for (idx, cmd) in drained.iter().enumerate() {
            assert_eq!(cmd.seq, idx as u64);
        }
    }

    /// Queue enforces capacity; excess commands are rejected.
    #[test]
    fn capacity_enforced() {
        let mut q = CommandQueue::new(3);
        for i in 0..3u64 {
            q.push(Command {
                client_id: i,
                seq: i,
                kind: CommandKind::Pause,
                tick_issued: 0,
            })
            .unwrap();
        }
        assert!(matches!(
            q.push(Command {
                client_id: 99,
                seq: 99,
                kind: CommandKind::Pause,
                tick_issued: 0,
            }),
            Err(CommandError::CapacityExceeded)
        ));
    }
}
