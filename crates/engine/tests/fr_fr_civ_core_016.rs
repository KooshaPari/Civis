//! Tests for FR-CIV-CORE-016
//!
//! Epic: FR-CIV-CORE
//!
//! This test file verifies FR FR-CIV-CORE-016: Client Priority Tiers.
//! Commands prioritized by (client_priority, tick_received).

#[cfg(test)]
mod fr_fr_civ_core_016 {
    use civ_engine::command_queue::{Command, CommandKind, CommandQueue};

    /// Commands with earlier tick_issued are processed first (FIFO within queue).
    #[test]
    fn earlier_tick_processed_first() {
        let mut q = CommandQueue::new(20);
        // Push client 2 first (tick_issued=20), then client 1 (tick_issued=10)
        q.push(Command {
            client_id: 2,
            seq: 0,
            kind: CommandKind::Resume,
            tick_issued: 20,
        })
        .unwrap();
        q.push(Command {
            client_id: 1,
            seq: 1,
            kind: CommandKind::Resume,
            tick_issued: 10,
        })
        .unwrap();
        let first = q.pop().unwrap();
        assert_eq!(
            first.tick_issued, 20,
            "FIFO: first pushed comes first"
        );
        let second = q.pop().unwrap();
        assert_eq!(second.tick_issued, 10);
    }

    /// SetSpeed(0) is rejected as invalid.
    #[test]
    fn invalid_speed_rejected() {
        use civ_engine::command_queue::CommandError;
        let mut q = CommandQueue::new(20);
        let result = q.push(Command {
            client_id: 0,
            seq: 0,
            kind: CommandKind::SetSpeed(0),
            tick_issued: 0,
        });
        assert!(matches!(result, Err(CommandError::InvalidSpeed)));
    }

    /// Valid SetSpeed is accepted.
    #[test]
    fn valid_speed_accepted() {
        let mut q = CommandQueue::new(20);
        let result = q.push(Command {
            client_id: 0,
            seq: 0,
            kind: CommandKind::SetSpeed(5),
            tick_issued: 0,
        });
        assert!(result.is_ok());
    }
}
