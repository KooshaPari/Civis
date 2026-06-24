use std::collections::VecDeque;

/// A command issued by a connected client.
#[derive(Debug)]
pub struct Command {
    pub client_id: u64,
    pub seq: u64,
    pub kind: CommandKind,
    pub tick_issued: u64,
}

/// The type of command a client can issue.
#[derive(Debug)]
pub enum CommandKind {
    Pause,
    Resume,
    SetSpeed(u32),
    SaveReplay,
    LoadReplay(String),
    PolicyOverride { key: String, value: f64 },
}

/// A bounded multi-client command queue.
///
/// Commands are processed in FIFO order. The queue enforces a maximum
/// number of pending commands to prevent unbounded growth.
pub struct CommandQueue {
    commands: VecDeque<Command>,
    max_pending: usize,
}

/// Errors that can occur when pushing to a `CommandQueue`.
#[derive(Debug)]
pub enum CommandError {
    /// The queue has reached its maximum pending capacity.
    CapacityExceeded,
    /// The speed value in a `SetSpeed` command is invalid (e.g. zero).
    InvalidSpeed,
}

impl CommandQueue {
    /// Create a new queue with the given maximum pending capacity.
    pub fn new(max: usize) -> Self {
        CommandQueue {
            commands: VecDeque::new(),
            max_pending: max,
        }
    }

    /// Push a command onto the back of the queue.
    ///
    /// Returns `Err(CommandError::CapacityExceeded)` if the queue is full.
    /// Returns `Err(CommandError::InvalidSpeed)` if the command is `SetSpeed(0)`.
    pub fn push(&mut self, c: Command) -> Result<(), CommandError> {
        if self.commands.len() >= self.max_pending {
            return Err(CommandError::CapacityExceeded);
        }
        if let CommandKind::SetSpeed(speed) = c.kind {
            if speed == 0 {
                return Err(CommandError::InvalidSpeed);
            }
        }
        self.commands.push_back(c);
        Ok(())
    }

    /// Pop the front command from the queue, if any.
    pub fn pop(&mut self) -> Option<Command> {
        self.commands.pop_front()
    }

    /// Drain all commands from the queue.
    pub fn drain(&mut self) -> Vec<Command> {
        self.commands.drain(..).collect()
    }

    /// Return the number of pending commands.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Return true when there are no pending commands.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let mut q = CommandQueue::new(10);
        let cmd = Command {
            client_id: 1,
            seq: 1,
            kind: CommandKind::Pause,
            tick_issued: 100,
        };
        assert!(q.push(cmd).is_ok());

        let popped = q.pop().expect("should have a command");
        assert_eq!(popped.client_id, 1);
        assert_eq!(popped.seq, 1);
        assert!(matches!(popped.kind, CommandKind::Pause));
        assert_eq!(popped.tick_issued, 100);

        // Queue should now be empty
        assert!(q.pop().is_none());
    }

    #[test]
    fn capacity_enforcement() {
        let max = 3;
        let mut q = CommandQueue::new(max);

        // Fill up to capacity
        for i in 0..max {
            let cmd = Command {
                client_id: i as u64,
                seq: i as u64,
                kind: CommandKind::Resume,
                tick_issued: 0,
            };
            assert!(q.push(cmd).is_ok(), "push {} should succeed", i);
        }

        // Next push should exceed capacity
        let overflow = Command {
            client_id: 99,
            seq: 99,
            kind: CommandKind::Resume,
            tick_issued: 0,
        };
        match q.push(overflow) {
            Err(CommandError::CapacityExceeded) => { /* expected */ }
            other => panic!("expected CapacityExceeded, got {:?}", other),
        }
    }

    #[test]
    fn drain() {
        let mut q = CommandQueue::new(10);
        for i in 0..5 {
            let cmd = Command {
                client_id: i,
                seq: i,
                kind: CommandKind::Resume,
                tick_issued: i * 10,
            };
            q.push(cmd).unwrap();
        }

        let drained = q.drain();
        assert_eq!(drained.len(), 5);
        assert_eq!(q.len(), 0);
        // Verify order is preserved
        for (idx, cmd) in drained.iter().enumerate() {
            assert_eq!(cmd.client_id, idx as u64);
            assert_eq!(cmd.seq, idx as u64);
        }
    }

    #[test]
    fn len() {
        let mut q = CommandQueue::new(10);
        assert_eq!(q.len(), 0);

        q.push(Command { client_id: 1, seq: 1, kind: CommandKind::Pause, tick_issued: 0 }).unwrap();
        assert_eq!(q.len(), 1);

        q.push(Command { client_id: 2, seq: 2, kind: CommandKind::Resume, tick_issued: 0 }).unwrap();
        assert_eq!(q.len(), 2);

        q.pop();
        assert_eq!(q.len(), 1);

        q.pop();
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn invalid_speed_rejection() {
        let mut q = CommandQueue::new(10);

        // Zero speed is invalid
        let bad = Command {
            client_id: 0,
            seq: 0,
            kind: CommandKind::SetSpeed(0),
            tick_issued: 0,
        };
        match q.push(bad) {
            Err(CommandError::InvalidSpeed) => { /* expected */ }
            other => panic!("expected InvalidSpeed, got {:?}", other),
        }

        // Non-zero speed should succeed
        let good = Command {
            client_id: 0,
            seq: 0,
            kind: CommandKind::SetSpeed(1),
            tick_issued: 0,
        };
        assert!(q.push(good).is_ok());

        // And we can still pop it
        let popped = q.pop().expect("should have the SetSpeed command");
        match popped.kind {
            CommandKind::SetSpeed(v) => assert_eq!(v, 1),
            other => panic!("expected SetSpeed, got {:?}", other),
        }
    }
}
