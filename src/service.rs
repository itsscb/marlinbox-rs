use std::{
    fs::File,
    io::BufReader,
    sync::{Arc, Mutex},
};

use crossbeam_channel::{Receiver, Sender};
use rodio::{Decoder, OutputStream, Sink};
use tracing::{debug, error, info};
use wifi_rs::{prelude::*, WiFi};

use crate::{card::Card, error::Error, library::Library, manager};

static SUCCESS_SOUND: &str = "sounds/positive_confirmation.wav";
static FAILURE_SOUND: &str = "sounds/negative_confirmation.wav";

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

fn play_card(card: &Card, sink: &Sink) {
    match card {
        Card::Play(music_file) => {
            play_sound(sink, music_file);
        }
        Card::Pause => sink.pause(),
        Card::Resume => sink.play(),
        Card::Next | Card::Previous => sink.stop(),
        Card::ToggleHotspot | Card::Shuffle => {}

        // TODO: Volume management. Currently the volume is set independetly from the OS which leads to a horrible quality decrease.
        Card::VolumeUp => sink.set_volume(sink.volume() + 1.0),
        Card::VolumeDown => sink.set_volume(sink.volume() - 1.0),
    }
}

fn play_sound(sink: &Sink, file_path: &str) {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(err) => {
            error!("Failed to open file {file_path}: {err}");
            return;
        }
    };
    let file = BufReader::new(file);
    let source = match Decoder::new(file) {
        Ok(source) => source,
        Err(err) => {
            error!("Failed to decode file {file_path}: {err}");
            return;
        }
    };
    sink.stop();
    sink.append(source);
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
pub fn run(
    library: &Arc<Mutex<Library>>,
    tx_manager_shutdown: Sender<()>,
    rx_manager_shutdown: Receiver<()>,
    rx: &Receiver<Arc<str>>,
) -> Result<(), Error> {
    let tx_manager_shutdown = Arc::from(tx_manager_shutdown);
    let rx_manager_shutdown = Arc::from(rx_manager_shutdown);

    let (_stream, stream_handle) = OutputStream::try_default().map_err(Error::from)?;
    let sink = Sink::try_new(&stream_handle)?;

    let (tx_pairing, rx_pairing): (Sender<()>, Receiver<()>) = crossbeam_channel::bounded(1);
    let tx_pairing = Arc::from(tx_pairing);
    let mut pairing_cards: Vec<Arc<str>> = vec![];

    let mut hotspot_enabled = false;
    let mut is_pairing = false;

    loop {
        match rx_pairing.try_recv() {
            Ok(()) => {
                is_pairing = !is_pairing;
                if is_pairing {
                    info!("Pairing mode enabled");
                    pairing_cards.clear();
                }
            }
            Err(e) => match e {
                crossbeam_channel::TryRecvError::Empty => {}
                crossbeam_channel::TryRecvError::Disconnected => unreachable!(),
            },
        }
        match rx.try_recv() {
            Ok(card_id) => {
                debug!("Card ID: {card_id}");

                let mut library_lock = library.lock()?;
                let card = library_lock.get(&card_id);

                if let Some(music_file) = card {
                    info!("Playing: {music_file:?}");

                    match music_file {
                        Card::Shuffle => {
                            let card = library_lock.get_random();
                            if let Some(card) = card {
                                play_card(card, &sink);
                            }
                        }
                        Card::ToggleHotspot => {
                            let msg = if hotspot_enabled { "disable" } else { "enable" };
                            match toggle_hotspot(!hotspot_enabled) {
                                Ok(()) => info!("hotspot {msg}d"),
                                Err(err) => error!("Failed to {msg} hotspot: {err}"),
                            }
                            if hotspot_enabled {
                                match tx_manager_shutdown.send(()) {
                                    Ok(()) => info!("Sent shutdown message"),
                                    Err(err) => error!("Failed to send shutdown message: {err}"),
                                }
                            } else {
                                let tx_pairing_clone = tx_pairing.clone();
                                let rx_manager_shutdown_clone = rx_manager_shutdown.clone();
                                std::thread::spawn(move || {
                                    match manager::serve(
                                        "0.0.0.0:8080",
                                        tx_pairing_clone,
                                        rx_manager_shutdown_clone,
                                    ) {
                                        Ok(()) => info!("Manager started"),
                                        Err(err) => error!("Failed to start manager: {err}"),
                                    }
                                });
                            }
                            hotspot_enabled = !hotspot_enabled;
                        }
                        card => play_card(card, &sink),
                    }
                } else if is_pairing {
                    info!("Read card: {card_id}");
                    pairing_cards.push(card_id);
                    if let Some(most_common_card) = pairing_cards
                        .iter()
                        .find(|&card| pairing_cards.iter().filter(|&c| *c == *card).count() >= 3)
                    {
                        library_lock.add(most_common_card);
                        if let Err(err) = library_lock.save_to_file("music.json") {
                            library_lock.remove(most_common_card);
                            error!("Failed to save library: {err}");
                            play_sound(&sink, FAILURE_SOUND);
                        } else {
                            play_sound(&sink, SUCCESS_SOUND);
                            info!("Added card to library: {most_common_card}");
                        }
                        is_pairing = !is_pairing;
                    }
                } else {
                    info!("Unknown card");
                }
                drop(library_lock);
            }
            Err(e) => match e {
                crossbeam_channel::TryRecvError::Empty => continue,
                crossbeam_channel::TryRecvError::Disconnected => break,
            },
        }
    }
    Ok(())
}
