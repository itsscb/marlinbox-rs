use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};

use crossbeam_channel::Receiver;
use rodio::{Decoder, OutputStream, Sink};
use wifi_rs::{prelude::*, WiFi};

use crate::{card::Card, error::Error, library::Library};

fn toggle_hotspot(enable: bool) -> Result<(), Error> {
    let config = Config {
        interface: Some("wlp59s0"),
    };

    let mut hotspot = WiFi::new(Some(config));

    if enable {
        hotspot.create_hotspot("MARLIN", "M4rl!nB0x", None)?;
    } else {
        hotspot.stop_hotspot()?;
    }

    Ok(())
}

/// Runs the service.
///
/// # Arguments
///
/// * `rx` - The receiver channel.
/// * `library` - The library.
///
/// # Errors
///
/// Returns an error if there is an issue running the service.
pub fn run(rx: &Receiver<Arc<str>>, library: &Arc<Mutex<Library>>) -> Result<(), Error> {
    let (_stream, stream_handle) = OutputStream::try_default().map_err(Error::from)?;
    let sink = Sink::try_new(&stream_handle)?;

    let mut hotspot_enabled = false;

    loop {
        match rx.recv() {
            Ok(card_id) => {
                println!("Card ID: {card_id}");

                let library_lock = library.lock()?;
                let card = library_lock.get(&card_id);

                if let Some(music_file) = card {
                    println!("Playing: {music_file:?}");

                    match music_file {
                        Card::Play(music_file) => {
                            sink.stop();
                            let file = BufReader::new(File::open(music_file.as_ref())?);
                            let source = Decoder::new(file)?;
                            sink.append(source);
                        }
                        Card::Pause => sink.pause(),
                        Card::Resume => sink.play(),
                        Card::Next | Card::Previous => sink.stop(),
                        Card::Shuffle => {
                            let card = library_lock.get_random();
                            if let Some(Card::Play(music_file)) = card {
                                sink.stop();
                                let file = BufReader::new(File::open(music_file.as_ref())?);
                                let source = Decoder::new(file)?;
                                sink.append(source);
                            }
                        }
                        Card::ToggleHotspot => {
                            toggle_hotspot(!hotspot_enabled)?;
                            hotspot_enabled = !hotspot_enabled;
                        }
                        // TODO: Volume management. Currently the volume is set independetly from the OS which leads to a horrible quality decrease.
                        Card::VolumeUp => sink.set_volume(sink.volume() + 1.0),
                        Card::VolumeDown => sink.set_volume(sink.volume() - 1.0),
                    }
                } else {
                    println!("No music file found for this card");
                }
                drop(library_lock);
            }
            Err(e) => match e {
                crossbeam_channel::RecvError => break,
            },
        }
    }
    Ok(())
}
