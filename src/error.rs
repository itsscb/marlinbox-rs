use std::{fmt, sync::Arc};

pub enum Error {
    Io(std::io::Error),
    Wifi(wifi_rs::prelude::WifiHotspotError),
    Decoder(rodio::decoder::DecoderError),
    Play(rodio::PlayError),
    Stream(rodio::StreamError),
    Recv(crossbeam_channel::RecvError),
    File(std::io::Error),
    Rusb(rusb::Error),
    Send(crossbeam_channel::SendError<Arc<str>>),
    MutexPoison,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::Wifi(err) => write!(f, "WiFi error: {err:?}"),
            Self::Decoder(err) => write!(f, "Decoder error: {err:?}"),
            Self::Play(err) => write!(f, "Play error: {err:?}"),
            Self::Stream(err) => write!(f, "Stream error: {err:?}"),
            Self::Recv(_) => write!(f, "Receiver channel error"),
            Self::File(err) => write!(f, "File error: {err}"),
            Self::Rusb(err) => write!(f, "Rusb error: {err}"),
            Self::Send(err) => write!(f, "Send error: {err}"),
            Self::MutexPoison => write!(f, "Mutex poison error"),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::Wifi(err) => write!(f, "WiFi error: {err:?}"),
            Self::Decoder(err) => write!(f, "Decoder error: {err:?}"),
            Self::Play(err) => write!(f, "Play error: {err:?}"),
            Self::Stream(err) => write!(f, "Stream error: {err:?}"),
            Self::Recv(_) => write!(f, "Receiver channel error"),
            Self::File(err) => write!(f, "File error: {err}"),
            Self::Rusb(err) => write!(f, "Rusb error: {err}"),
            Self::Send(err) => write!(f, "Send error: {err}"),
            Self::MutexPoison => write!(f, "Mutex poison error"),
        }
    }
}

impl From<rusb::Error> for Error {
    fn from(err: rusb::Error) -> Self {
        Self::Rusb(err)
    }
}

impl From<rodio::StreamError> for Error {
    fn from(err: rodio::StreamError) -> Self {
        Self::Stream(err)
    }
}

impl From<rodio::decoder::DecoderError> for Error {
    fn from(err: rodio::decoder::DecoderError) -> Self {
        Self::Decoder(err)
    }
}

impl From<rodio::PlayError> for Error {
    fn from(err: rodio::PlayError) -> Self {
        Self::Play(err)
    }
}

impl From<crossbeam_channel::RecvError> for Error {
    fn from(_: crossbeam_channel::RecvError) -> Self {
        Self::Recv(crossbeam_channel::RecvError)
    }
}

impl From<wifi_rs::prelude::WifiHotspotError> for Error {
    fn from(err: wifi_rs::prelude::WifiHotspotError) -> Self {
        Self::Wifi(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl<T> From<std::sync::PoisonError<std::sync::MutexGuard<'_, T>>> for Error {
    fn from(_: std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> Self {
        Self::MutexPoison
    }
}

impl std::error::Error for Error {}
