use rodio::{Decoder, OutputStreamBuilder, Sink};
use std::io::Cursor;

const DONE_SOUND: &[u8] = include_bytes!("../assets/done.wav");

pub fn init_audio() {
    match OutputStreamBuilder::open_default_stream() {
        Ok(_stream_handle) => {
            println!("Audio stream initialized successfully");
        }
        Err(e) => {
            eprintln!("Failed to initialize audio stream: {:?}", e);
        }
    }
}

pub fn play_done_sound() {
    std::thread::spawn(|| {
        let wav_data = DONE_SOUND;
        match OutputStreamBuilder::open_default_stream() {
            Ok(stream_handler) => {
                let sink = Sink::connect_new(&stream_handler.mixer());
                let cursor = Cursor::new(wav_data.as_ref());
                match Decoder::new(cursor) {
                    Ok(source) => {
                        sink.append(source);
                        sink.sleep_until_end();
                    }
                    Err(e) => eprintln!("Failed to decode audio: {}", e),
                }
            }
            Err(e) => eprintln!("Failed to open audio stream: {}", e),
        }
    });
}