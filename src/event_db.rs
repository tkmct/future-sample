use std::collections::HashMap;

pub trait EventDB {
    fn get_last_logged_block(&self, topic_hash: String) -> Option<usize>;
    fn set_last_logged_block(&mut self, topic_hash: String, block_number: usize);
    fn get_event_seen(&self, event_hash: String) -> bool;
    fn set_event_seen(&mut self, event_hash: String);
}

#[derive(Default)]
pub struct DefaultEventDB {
    last_logged_blocks: HashMap<String, usize>,
    seen_events: HashMap<String, bool>,
}

impl DefaultEventDB {
    pub fn new() -> DefaultEventDB {
        Default::default()
    }
}

impl EventDB for DefaultEventDB {
    fn get_last_logged_block(&self, topic_hash: String) -> Option<usize> {
        match self.last_logged_blocks.get(&topic_hash) {
            Some(block_number) => Some(*block_number),
            None => None,
        }
    }

    fn set_last_logged_block(&mut self, topic_hash: String, block_number: usize) {
        self.last_logged_blocks.insert(topic_hash, block_number);
    }

    fn get_event_seen(&self, event_hash: String) -> bool {
        match self.seen_events.get(&event_hash) {
            Some(seen) => *seen,
            None => false,
        }
    }

    fn set_event_seen(&mut self, event_hash: String) {
        self.seen_events.insert(event_hash, true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_last_logged_block() {
        let mut db = DefaultEventDB::new();
        assert_eq!(db.get_last_logged_block("hello".to_string()), None);
        db.set_last_logged_block("hello".to_string(), 1);
        assert_eq!(db.get_last_logged_block("hello".to_string()), Some(1));
    }

    #[test]
    fn test_event_seen() {
        let mut db = DefaultEventDB::new();
        assert_eq!(db.get_event_seen("hello".to_string()), false);
        db.set_event_seen("hello".to_string());
        assert_eq!(db.get_event_seen("hello".to_string()), true);
    }

}
