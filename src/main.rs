use std::sync::{Arc, Mutex};

use marlinbox_rs::{card_reader, service, Library};

const VID: u16 = 0xffff;
const PID: u16 = 0x0035;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let music: Arc<Mutex<Library>> = Arc::new(Mutex::new(Library::from_file("music.json")?));

    let (tx_card, rx_card) = crossbeam_channel::bounded(10);
    let (tx_manager_shutdown, rx_manager_shutdown) = crossbeam_channel::bounded(1);

    let mut handles = vec![];
    let reader_handle = std::thread::spawn(move || card_reader::read(VID, PID, &tx_card));
    handles.push(reader_handle);
    service::run(&music, tx_manager_shutdown, rx_manager_shutdown, &rx_card)?;

    for handle in handles {
        let _ = handle.join().unwrap();
    }
    Ok(())
}
