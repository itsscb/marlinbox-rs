use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Card {
    Play(Arc<str>),
    Pause,
    Resume,
    Next,
    Previous,
    Shuffle,
}

impl From<&Card> for Option<Card> {
    fn from(card: &Card) -> Self {
        Some(card.clone())
    }
}

impl<T: AsRef<str>> From<T> for Card {
    fn from(music_file: T) -> Self {
        Self::Play(Arc::from(music_file.as_ref()))
    }
}
