use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Instant;

use transcribe_rs::onnx::canary::{CanaryModel, CanaryParams};
use transcribe_rs::onnx::Quantization;
use transcribe_rs::audio;

#[cfg(unix)]
use std::os::unix::net::UnixListener;

fn get_audio_duration(path: &PathBuf) -> Result<f64, Box<dyn std::error::Error>> {
    let reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    let duration = reader.duration() as f64 / spec.sample_rate as f64;
    Ok(duration)
}

#[cfg(unix)]
fn transcribe_once(
    model: &mut CanaryModel,
    wav_path: &PathBuf,
    language_hint: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    let samples = audio::read_wav_samples(wav_path)?;
    let audio_duration = get_audio_duration(wav_path)?;

    eprintln!("Transcribing cached model with fresh audio");
    let transcribe_start = Instant::now();

    let mut params = CanaryParams::default();
    if let Some(lang) = language_hint {
        params.language = Some(lang.to_string());
    }
    let result = model.transcribe_with(&samples, &params)?;
    let transcribe_duration = transcribe_start.elapsed();
    let speedup_factor = audio_duration / transcribe_duration.as_secs_f64();

    eprintln!("Audio duration: {:.2}s", audio_duration);
    eprintln!("Transcription completed in {:.2?}", transcribe_duration);
    eprintln!(
        "Real-time speedup: {:.2}x faster than real-time",
        speedup_factor
    );

    Ok(result.text)
}

#[cfg(unix)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let home = std::env::var("HOME")?;
    let model_path = PathBuf::from(home).join(".local/share/models/canary-1b-v2-int8");
    let wav_path = PathBuf::from("/tmp/dictate.wav");
    let socket_path = "/tmp/transcribe.sock";

    eprintln!("Using Canary engine");
    eprintln!("Loading model: {:?}", model_path);

    let load_start = Instant::now();
    let mut model = CanaryModel::load(&model_path, &Quantization::Int8)?;
    let load_duration = load_start.elapsed();
    eprintln!("Model loaded in {:.2?}", load_duration);

    if fs::metadata(socket_path).is_ok() {
        fs::remove_file(socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    eprintln!("Server ready. Socket: {}", socket_path);
    eprintln!("WAV path (read fresh each request): {:?}", wav_path);

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(err) => {
                eprintln!("Accept error: {}", err);
                continue;
            }
        };

        let mut request = Vec::new();
        if let Err(err) = stream.read_to_end(&mut request) {
            let _ = writeln!(stream, "ERROR: failed to read request: {}", err);
            continue;
        }

        let request_text = String::from_utf8_lossy(&request);
        let language_hint = request_text.trim();
        let language_hint = if language_hint.is_empty() {
            None
        } else {
            Some(language_hint)
        };
        let effective_language = language_hint.unwrap_or("en");
        eprintln!("Language hint (request): {}", effective_language);

        match transcribe_once(&mut model, &wav_path, language_hint) {
            Ok(text) => {
                let _ = writeln!(stream, "{}", text);
            }
            Err(err) => {
                let _ = writeln!(stream, "ERROR: {}", err);
            }
        }
    }

    Ok(())
}

#[cfg(not(unix))]
fn main() {
    eprintln!("transcribe_server example is Unix-only (uses Unix domain sockets).");
}
