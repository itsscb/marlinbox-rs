use std::{collections::HashMap, sync::Arc};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::card::Card;

#[derive(Debug, Serialize, Deserialize)]
pub struct Library {
    music: HashMap<Arc<str>, Option<Card>>,
}

impl Library {
    #[must_use]
    pub fn new() -> Self {
        Self {
            music: HashMap::new(),
        }
    }

    pub fn add(&mut self, card_id: &str) {
        self.music.insert(card_id.into(), None);
    }

    pub fn update(&mut self, card_id: &str, music_file: Option<Card>) {
        self.music
            .insert(card_id.into(), music_file.map(Into::into));
    }

    #[must_use]
    pub fn get(&self, card_id: &str) -> Option<&Card> {
        self.music.get::<str>(card_id).unwrap_or(&None).as_ref()
    }

    #[must_use]
    pub fn get_random(&self) -> Option<&Card> {
        let play_cards: Vec<&Card> = self.music.values().flatten().collect();

        play_cards.choose(&mut rand::thread_rng()).copied()
    }
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}
