use marlinbox_rs::{card::Card, library::Library};
use rodio::{Decoder, OutputStream, Sink};
use rusb::{Context, DeviceHandle, UsbContext};
use std::{fs::File, io::BufReader, time::Duration};

const VID: u16 = 0xffff; // Replace with your device's Vendor ID
const PID: u16 = 0x0035; // Replace with your device's Product ID

fn extract_card_id(buf: &[u8]) -> Option<String> {
    let significant_indices = [2, 18, 34, 50, 66, 82, 98, 114];
    let extracted: Vec<u8> = significant_indices
        .iter()
        .filter_map(|&i| buf.get(i).copied())
        .collect();

    if extracted.len() == 8 && extracted.iter().all(|&b| b != 0) {
        Some(extracted.iter().fold(String::new(), |mut acc, &b| {
            acc.push_str(&format!("{b:02X}"));
            acc
        }))
    } else {
        None
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Sink::try_new(&stream_handle).unwrap();

    let music: Library = serde_json::from_reader(File::open("music.json")?)?;
    // let mut music = Library::new();
    // music.update(
    //     "27271E24211E1E26",
    //     Some(Card::Play("02 - Disturbed - Immortalized.mp3".into())),
    // );
    // music.update(
    //     "27271E242121221F",
    //     Some(Card::Play(
    //         "11 - Disturbed - The Sound Of Silence.mp3".into(),
    //     )),
    // );
    // music.update("27271E2420222321", Card::Shuffle.into());

    // let music_json = serde_json::to_string_pretty(&music)?;
    // fs::write("music.json", music_json)?;

    let context = Context::new()?;
    let handle: DeviceHandle<Context> = context
        .open_device_with_vid_pid(VID, PID)
        .ok_or("Device not found")?;

    #[cfg(target_os = "linux")]
    {
        if handle.kernel_driver_active(0)? {
            handle.detach_kernel_driver(0)?;
        }
    }

    handle.claim_interface(0)?;

    println!("Starting to read RFID cards...");

    let mut buf = [0u8; 120];
    // let mut last_id: Option<String> = None;

    loop {
        match handle.read_interrupt(0x81, &mut buf, Duration::from_millis(1000)) {
            Ok(len) => {
                if len == 120 {
                    if let Some(card_id) = extract_card_id(&buf) {
                        // if last_id.as_ref() != Some(&card_id) {
                        println!("Card ID: {card_id}");
                        if let Some(music_file) = music.get(card_id.as_ref()) {
                            println!("Playing: {music_file:?}");

                            match music_file {
                                Card::Play(music_file) => {
                                    sink.stop();
                                    let file =
                                        BufReader::new(File::open(music_file.as_ref()).unwrap());
                                    let source = Decoder::new(file).unwrap();
                                    sink.append(source);
                                }
                                Card::Pause => sink.pause(),
                                Card::Resume => sink.play(),
                                Card::Next | Card::Previous => sink.stop(),
                                Card::Shuffle => {
                                    if let Some(Card::Play(music_file)) = music.get_random() {
                                        sink.stop();
                                        let file = BufReader::new(
                                            File::open(music_file.as_ref()).unwrap(),
                                        );
                                        let source = Decoder::new(file).unwrap();
                                        sink.append(source);
                                    }
                                }
                            }
                        } else {
                            println!("No music file found for this card");
                        }
                    }
                }
            }
            Err(rusb::Error::Timeout) => {
                // last_id = None;
            }
            Err(e) => eprintln!("Error: {e:?}"),
        }
    }
}
