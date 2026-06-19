//! Celebration effects triggered at the end of a game.
//!
//! Both functions are intentionally non-fatal: failures (missing audio device,
//! Raycast not installed, etc.) are silently swallowed so that the core game
//! logic is never interrupted by optional meme features.

use std::io::{BufReader, Cursor};

use rodio::{source::Source, Decoder, OutputStream};

/// Triggers the Raycast confetti animation on macOS. Silently does nothing if
/// Raycast/`open` is unavailable or the spawn fails.
pub fn confetti() {
    let _ = std::process::Command::new("open")
        .arg("raycast://confetti")
        .stderr(std::process::Stdio::null())
        .spawn()
        .and_then(|mut child| child.wait());
}

/// Plays the bundled loss sound via rodio. Silently does nothing if no audio
/// device is available or any step fails. The 1s drain sleep only runs when
/// playback actually started.
pub fn play_sad_sound() {
    let audio_bytes: &[u8] = include_bytes!("../assets/stupid.mp3");
    let play = || -> Result<(), Box<dyn std::error::Error>> {
        let (_stream, stream_handle) = OutputStream::try_default()?;
        let source = Decoder::new(BufReader::new(Cursor::new(audio_bytes)))?;
        stream_handle.play_raw(source.convert_samples())?;
        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok(())
    };
    let _ = play();
}
