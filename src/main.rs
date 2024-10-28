use std::{
    fs::File,
    sync::{Arc, Mutex},
};

use marlinbox_rs::{card_reader, library::Library, service};

const VID: u16 = 0xffff;
const PID: u16 = 0x0035;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let music: Arc<Mutex<Library>> = Arc::new(Mutex::new(serde_json::from_reader(File::open(
        "music.json",
    )?)?));

    let (tx, rx) = crossbeam_channel::bounded(10);

    let mut handles = vec![];
    let reader_handle = std::thread::spawn(move || card_reader::read(VID, PID, &tx));
    handles.push(reader_handle);
    service::run(&rx, &music)?;

    for handle in handles {
        let _ = handle.join().unwrap();
    }
    Ok(())
}
