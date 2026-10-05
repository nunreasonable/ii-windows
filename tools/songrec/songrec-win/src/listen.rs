use std::collections::VecDeque;
use std::num::NonZero;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};
use rodio::nz;

use crate::algorithm::SignatureGenerator;
use crate::communication::{RecognizeError, has_match, recognize_song_from_signature};
use crate::{EXIT_AUDIO, EXIT_NETWORK, EXIT_RATE_LIMITED};

const BUFFER_SIZE_SECS: usize = 12;
const WINDOW: usize = 16000 * BUFFER_SIZE_SECS;

pub struct Options {
    pub audio_device: Option<String>,
    pub loopback: bool,
    pub request_interval: u64,
    pub timeout: Option<u64>,
    pub json: bool,
    pub list_devices: bool,
    pub once: bool,
}

impl Options {
    pub fn parse(args: &[String]) -> Option<Self> {
        let mut options = Options {
            audio_device: None,
            loopback: false,
            request_interval: 10,
            timeout: None,
            json: false,
            list_devices: false,
            once: false,
        };

        let mut args = args.iter();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-d" | "--audio-device" => options.audio_device = Some(args.next()?.clone()),
                "--loopback" => options.loopback = true,
                "-i" | "--request-interval" => {
                    options.request_interval = args.next()?.parse().ok().filter(|&s| s > 0)?
                }
                "--timeout" => options.timeout = Some(args.next()?.parse().ok()?),
                "-j" | "--json" => options.json = true,
                "-l" | "--list-devices" => options.list_devices = true,
                "--disable-mpris" => {}
                _ => {
                    eprintln!("Unknown argument: {arg}");
                    return None;
                }
            }
        }

        Some(options)
    }
}

#[derive(Default)]
struct Window {
    samples: VecDeque<f32>,
    unprocessed: usize,
    error: Option<String>,
}

fn device_label(device: &cpal::Device) -> String {
    let name = device
        .description()
        .map(|d| d.name().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let direction = if device.supports_input() {
        "input"
    } else {
        "output"
    };
    format!("{name} ({direction})")
}

fn list_devices(host: &cpal::Host) -> ExitCode {
    let Ok(devices) = host.devices() else {
        eprintln!("Error: could not enumerate audio devices");
        return ExitCode::from(EXIT_AUDIO);
    };
    for device in devices {
        if let Ok(id) = device.id() {
            println!("{id}\t{}", device_label(&device));
        }
    }
    ExitCode::SUCCESS
}

fn pick_device(host: &cpal::Host, options: &Options) -> Option<cpal::Device> {
    if let Some(wanted) = &options.audio_device {
        return host.devices().ok()?.find(|device| {
            device.id().is_ok_and(|id| id.to_string() == *wanted)
                || device.description().is_ok_and(|d| d.name() == wanted)
        });
    }
    if options.loopback {
        host.default_output_device()
    } else {
        host.default_input_device()
    }
}

fn push_chunk<T>(window: &Mutex<Window>, data: &[T], channels: u16, sample_rate: u32)
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let (Some(channels), Some(sample_rate)) = (NonZero::new(channels), NonZero::new(sample_rate))
    else {
        return;
    };
    let input = rodio::buffer::SamplesBuffer::new(
        channels,
        sample_rate,
        data.iter()
            .map(|s| s.to_sample::<f32>())
            .collect::<Vec<f32>>(),
    );
    let resampled: Vec<f32> =
        rodio::source::UniformSourceIterator::new(input, nz!(1), nz!(16000)).collect();

    let Ok(mut window) = window.lock() else {
        return;
    };
    window.unprocessed += resampled.len();
    window.samples.extend(resampled);
    let excess = window.samples.len().saturating_sub(WINDOW);
    window.samples.drain(..excess);
}

fn build_stream<T>(
    device: &cpal::Device,
    config: cpal::StreamConfig,
    window: Arc<Mutex<Window>>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let channels = config.channels;
    let sample_rate = config.sample_rate;
    let error_window = window.clone();
    device.build_input_stream(
        config,
        move |data: &[T], _: &_| push_chunk(&window, data, channels, sample_rate),
        move |error: cpal::Error| {
            if !matches!(
                error.kind(),
                cpal::ErrorKind::Xrun | cpal::ErrorKind::RealtimeDenied
            ) && let Ok(mut window) = error_window.lock()
            {
                window.error = Some(error.to_string());
            }
        },
        None,
    )
}

fn start_capture(
    device: &cpal::Device,
    window: Arc<Mutex<Window>>,
) -> Result<cpal::Stream, String> {
    let config = if device.supports_input() {
        device.default_input_config()
    } else {
        device.default_output_config()
    }
    .map_err(|e| e.to_string())?;

    let stream_config: cpal::StreamConfig = config.config();
    let stream = match config.sample_format() {
        SampleFormat::F32 => build_stream::<f32>(device, stream_config, window),
        SampleFormat::I16 => build_stream::<i16>(device, stream_config, window),
        SampleFormat::I32 => build_stream::<i32>(device, stream_config, window),
        SampleFormat::U16 => build_stream::<u16>(device, stream_config, window),
        SampleFormat::U8 => build_stream::<u8>(device, stream_config, window),
        SampleFormat::F64 => build_stream::<f64>(device, stream_config, window),
        format => return Err(format!("unsupported sample format {format}")),
    }
    .map_err(|e| e.to_string())?;

    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}

pub fn run(options: &Options) -> ExitCode {
    let host = cpal::default_host();

    if options.list_devices {
        return list_devices(&host);
    }

    let Some(device) = pick_device(&host, options) else {
        eprintln!("Error: no audio device available");
        return ExitCode::from(EXIT_AUDIO);
    };

    let window = Arc::new(Mutex::new(Window::default()));
    let _stream = match start_capture(&device, window.clone()) {
        Ok(stream) => stream,
        Err(error) => {
            eprintln!(
                "Error: could not record from {}: {error}",
                device_label(&device)
            );
            return ExitCode::from(EXIT_AUDIO);
        }
    };

    let started = Instant::now();
    let interval_samples = 16000 * options.request_interval as usize;
    let mut last_track: Option<String> = None;
    let mut last_error: Option<RecognizeError> = None;

    loop {
        if options
            .timeout
            .is_some_and(|t| started.elapsed() >= Duration::from_secs(t))
        {
            break;
        }

        std::thread::sleep(Duration::from_millis(100));

        let samples = {
            let Ok(mut window) = window.lock() else { break };
            if let Some(error) = window.error.take() {
                eprintln!("Error: audio stream: {error}");
                return ExitCode::from(EXIT_AUDIO);
            }
            if window.unprocessed < interval_samples {
                continue;
            }
            window.unprocessed = 0;
            window.samples.iter().copied().collect::<Vec<f32>>()
        };

        if samples.iter().all(|&s| s == 0.0) {
            continue;
        }

        let mut padded = vec![0.0f32; WINDOW - samples.len()];
        padded.extend(samples);

        let signature = SignatureGenerator::make_signature_from_buffer(&padded);
        match recognize_song_from_signature(&signature) {
            Ok(json) if has_match(&json) => {
                last_error = None;
                let key = json["track"]["key"].as_str().map(str::to_string);
                if key.is_none() || key != last_track {
                    if options.json {
                        println!("{}", serde_json::to_string(&json).unwrap_or_default());
                    } else {
                        println!(
                            "{} - {}",
                            json["track"]["subtitle"].as_str().unwrap_or_default(),
                            json["track"]["title"].as_str().unwrap_or_default()
                        );
                    }
                    last_track = key;
                }
                if options.once {
                    return ExitCode::SUCCESS;
                }
            }
            Ok(_) => last_error = None,
            Err(error) => {
                eprintln!("Warning: {error}");
                last_error = Some(error);
            }
        }
    }

    match last_error {
        Some(RecognizeError::RateLimited) => ExitCode::from(EXIT_RATE_LIMITED),
        Some(RecognizeError::Network(_)) => ExitCode::from(EXIT_NETWORK),
        _ => ExitCode::SUCCESS,
    }
}
