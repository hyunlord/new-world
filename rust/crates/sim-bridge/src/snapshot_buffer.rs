use sim_engine::AgentSnapshot;

/// Double-buffered agent snapshots for render interpolation.
///
/// `prev` and `curr` keep the previous and latest stable simulation snapshots so
/// Godot can render `lerp(prev, curr, alpha)` between simulation ticks.
#[derive(Debug, Clone, Default)]
pub struct SnapshotBuffer {
    prev: Vec<AgentSnapshot>,
    curr: Vec<AgentSnapshot>,
    agent_count: usize,
}

impl SnapshotBuffer {
    /// Creates an empty snapshot buffer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Swaps in a newly captured current snapshot set.
    ///
    /// On the first call, both buffers are initialized to the same data so
    /// interpolation has a valid previous frame immediately.
    pub fn swap(&mut self, new_curr: Vec<AgentSnapshot>) {
        if self.curr.is_empty() {
            self.prev = new_curr.clone();
            self.curr = new_curr;
        } else {
            self.prev = std::mem::replace(&mut self.curr, new_curr);
        }
        self.agent_count = self.curr.len();
    }

    /// Returns the previous snapshot slice.
    pub fn prev(&self) -> &[AgentSnapshot] {
        &self.prev
    }

    /// Returns the current snapshot slice.
    pub fn curr(&self) -> &[AgentSnapshot] {
        &self.curr
    }

    /// Returns the current number of alive agents represented in the buffer.
    pub fn agent_count(&self) -> usize {
        self.agent_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(entity_id: u32, x: f32) -> AgentSnapshot {
        AgentSnapshot {
            entity_id,
            x,
            ..AgentSnapshot::default()
        }
    }

    #[test]
    fn first_swap_initializes_prev_and_curr() {
        let mut buffer = SnapshotBuffer::new();
        buffer.swap(vec![snapshot(1, 10.0)]);

        assert_eq!(buffer.agent_count(), 1);
        assert_eq!(buffer.prev().len(), 1);
        assert_eq!(buffer.curr().len(), 1);
        let prev_x = buffer.prev()[0].x;
        let curr_x = buffer.curr()[0].x;
        assert_eq!(prev_x, 10.0);
        assert_eq!(curr_x, 10.0);
    }

    #[test]
    fn second_swap_rotates_curr_into_prev() {
        let mut buffer = SnapshotBuffer::new();
        buffer.swap(vec![snapshot(1, 10.0)]);
        buffer.swap(vec![snapshot(1, 20.0)]);

        assert_eq!(buffer.agent_count(), 1);
        let prev_x = buffer.prev()[0].x;
        let curr_x = buffer.curr()[0].x;
        assert_eq!(prev_x, 10.0);
        assert_eq!(curr_x, 20.0);
    }

    #[test]
    fn stage1_snapshot_buffer_double_swap() {
        let mut buffer = SnapshotBuffer::new();
        let first = vec![snapshot(1, 10.0)];
        let second = vec![snapshot(1, 20.0)];
        buffer.swap(first);
        let first_curr_x = buffer.curr()[0].x;
        assert_eq!(first_curr_x, 10.0);
        buffer.swap(second);
        let prev_x = buffer.prev()[0].x;
        let curr_x = buffer.curr()[0].x;
        assert_eq!(prev_x, 10.0);
        assert_eq!(curr_x, 20.0);
    }
}
