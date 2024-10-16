use std::fmt::Write;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Status {
    pub sequence: i16,
    pub clients: u32,
    pub text: Arc<str>,
}

impl PartialOrd for Status {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Status {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.sequence.wrapping_sub(other.sequence).cmp(&0)
    }
}

impl Status {
    pub fn diff(&self, previous: &Self) -> Option<String> {
        let mut message = String::new();
        if previous.clients != self.clients {
            write!(message, "n{}", self.clients).unwrap();
        }
        if previous.text != self.text {
            write!(message, "${}", self.text).unwrap();
        }
        (!message.is_empty()).then_some(message)
    }
}
