use crate::movegen::Move;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    Exact,
    Lower,
    Upper,
}

#[derive(Clone, Copy, Debug)]
pub struct TTEntry {
    pub key: u64,
    pub depth: i16, //-1 is empty
    pub score: i32, //stored score
    pub bound: Bound,
    pub best: Move,
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            key: 0,
            depth: -1,
            score: 0,
            bound: Bound::Exact,
            best: Move::NULL,
        }
    }
}

pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    mask: usize,
}

impl TranspositionTable {
    pub fn disabled() -> Self {
        Self {
            entries: Vec::new(),
            mask: 0,
        }
    }

    pub fn new_mb(size_mb: usize) -> Self {
        if size_mb == 0 {
            return Self::disabled();
        }

        let bytes = size_mb.saturating_mul(1024 * 1024);
        let entry_size = std::mem::size_of::<TTEntry>().max(1);
        let approx = (bytes / entry_size).max(1);

        let entry_count_pow2 = approx.next_power_of_two().max(1024);

        Self {
            entries: vec![TTEntry::default(); entry_count_pow2],
            mask: entry_count_pow2 - 1,
        }
    }

    fn idx(&self, key: u64) -> usize {
        (key as usize) & self.mask
    }

    pub fn probe(&self, key: u64) -> Option<TTEntry> {
        if self.entries.is_empty() {
            return None;
        }

        let slot_entry = self.entries[self.idx(key)];
        if slot_entry.depth >= 0 && slot_entry.key == key {
            Some(slot_entry)
        } else {
            None
        }
    }

    pub fn store(&mut self, key: u64, depth: i32, score: i32, bound: Bound, best: Move) {
        if self.entries.is_empty() {
            return;
        }
        let slot_idx = self.idx(key);
        let existing_entry = self.entries[slot_idx];

        let stored_depth = depth.clamp(0, i16::MAX as i32) as i16;

        let should_replace = existing_entry.depth < 0
            || (existing_entry.key == key && stored_depth >= existing_entry.depth)
            || (existing_entry.key != key && stored_depth > existing_entry.depth);

        if should_replace {
            self.entries[slot_idx] = TTEntry {
                key,
                depth: stored_depth,
                score,
                bound,
                best,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::movegen::Move;

    #[test]
    fn disabled_tt_never_hits_and_never_stores() {
        let mut tt = TranspositionTable::disabled();

        assert!(tt.probe(123).is_none());
        tt.store(123, 5, 42, Bound::Exact, Move::NULL);
        assert!(tt.probe(123).is_none());
    }

    #[test]
    fn store_then_probe_returns_entry() {
        let mut tt = TranspositionTable::new_mb(1);

        let key = 0xDEADBEEF_u64;
        tt.store(key, 7, 123, Bound::Exact, Move::NULL);

        let e = tt.probe(key).expect("entry should be present");
        assert_eq!(e.key, key);
        assert_eq!(e.depth, 7);
        assert_eq!(e.score, 123);
        assert_eq!(e.bound, Bound::Exact);
        assert_eq!(e.best, Move::NULL);
    }

    #[test]
    fn probe_miss_on_different_key_even_if_same_slot_possible() {
        //Can’t guarantee collision, but at least assert different key doesn't match.
        let mut tt = TranspositionTable::new_mb(1);

        let key1 = 1_u64;
        let key2 = 2_u64;
        tt.store(key1, 3, 11, Bound::Exact, Move::NULL);

        //key2 might collide- probe must return None unless key matches
        let hit = tt.probe(key2);
        assert!(hit.is_none() || hit.unwrap().key == key2);
        //stronger- ensure key1 is still retrievable
        assert!(tt.probe(key1).is_some());
    }

    #[test]
    fn replacement_prefers_deeper_entry() {
        let mut tt = TranspositionTable::new_mb(1);
        let key = 999_u64;

        tt.store(key, 4, 10, Bound::Upper, Move::NULL);
        tt.store(key, 2, 99, Bound::Lower, Move::NULL); // shall NOT replace (shallower)
        let e = tt.probe(key).unwrap();
        assert_eq!(e.depth, 4);
        assert_eq!(e.score, 10);
        assert_eq!(e.bound, Bound::Upper);

        tt.store(key, 6, 77, Bound::Exact, Move::NULL); // should replace (deeper)
        let e2 = tt.probe(key).unwrap();
        assert_eq!(e2.depth, 6);
        assert_eq!(e2.score, 77);
        assert_eq!(e2.bound, Bound::Exact);
    }

    #[test]
    fn depth_is_clamped_to_i16_max_and_non_negative() {
        let mut tt = TranspositionTable::new_mb(1);
        let key = 42_u64;

        tt.store(key, -10, 1, Bound::Exact, Move::NULL);
        let e = tt.probe(key).unwrap();
        assert_eq!(e.depth, 0);

        tt.store(key, i32::MAX, 2, Bound::Exact, Move::NULL);
        let e2 = tt.probe(key).unwrap();
        assert_eq!(e2.depth, i16::MAX);
    }
}
