use std::{sync::Arc, time::Duration};

use crossbeam_channel::Sender;
use rusb::UsbContext;

use crate::error::Error;

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

/// Reads data from the card reader.
///
/// # Arguments
///
/// * `vid` - The vendor ID.
/// * `pid` - The product ID.
/// * `tx` - The sender channel for sending the card ID.
///
/// # Errors
///
/// Returns an `Error` if there was an error reading from the card reader or sending the card ID.
pub fn read(vid: u16, pid: u16, tx: &Sender<Arc<str>>) -> Result<(), Error> {
    let context = rusb::Context::new()?;
    let handle = context
        .open_device_with_vid_pid(vid, pid)
        .ok_or(Error::Rusb(rusb::Error::NotFound))?;

    #[cfg(target_os = "linux")]
    {
        if handle.kernel_driver_active(0)? {
            handle.detach_kernel_driver(0)?;
        }
    }

    handle.claim_interface(0)?;

    let mut buf = [0; 128];
    let mut last_scan = std::time::Instant::now();
    loop {
        match handle.read_interrupt(0x81, &mut buf, Duration::from_secs(1)) {
            Ok(_) => {
                if let Some(card_id) = extract_card_id(&buf) {
                    if last_scan.elapsed() < Duration::from_secs(2) {
                        continue;
                    }
                    tx.send(Arc::from(card_id)).map_err(Error::Send)?;
                    last_scan = std::time::Instant::now();
                }
            }
            Err(rusb::Error::Timeout) => (),
            Err(e) => eprintln!("Error: {e}"),
        }
    }
}
