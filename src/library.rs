use std::{collections::HashMap, io::Write, sync::Arc};

use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::{card::Card, error::Error};
use std::fs::File;

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

    /// Saves the library to a file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the file where the library will be saved.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if there was an error writing the library to the file.
    pub fn save_to_file<P: AsRef<str>>(&self, file_path: P) -> Result<(), Error> {
        let serialized = serde_json::to_string(self)?;
        let mut file = File::create(file_path.as_ref())?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }

    /// Loads the library from a file.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the file from which the library will be loaded.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if there was an error reading the library from the file.
    pub fn from_file<P: AsRef<str>>(file_path: P) -> Result<Self, Error> {
        let file = File::open(file_path.as_ref())?;
        Ok(serde_json::from_reader(file)?)
    }

    pub fn add(&mut self, card_id: &str) {
        self.music.insert(card_id.into(), None);
    }

    pub fn remove(&mut self, card_id: &str) -> Option<Card> {
        self.music.remove_entry(card_id).and_then(|(_, v)| v)
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
