//! songrec for Windows: SongRec's command-line recognizer without GLib.
//!
//! The fingerprinting (algorithm.rs, signature_format.rs, hanning.rs, user_agent.rs) is
//! SongRec's own code, compiled from the upstream checkout as is. What's here replaces the
//! parts that need GLib/libsoup: the HTTP request to Shazam (communication.rs) and the
//! listening loop (listen.rs), which records with cpal so it can also take the default
//! output device through WASAPI loopback - upstream only lists capture devices.
//!
//! Subcommands keep upstream's names and output:
//!   audio-file-to-recognized-song FILE   Shazam's JSON answer, pretty-printed
//!   audio-file-to-fingerprint FILE       the data-URI signature
//!   fingerprint-to-recognized-song URI   Shazam's JSON answer, pretty-printed
//!   listen [options]                     one line per newly recognized song
//!   recognize [options]                  listen until the first match
//! listen/recognize options: --audio-device ID, --loopback (default output device),
//! --request-interval SECS, --json, --timeout SECS, --list-devices.
//!
//! Exit codes: 0 done (a timeout without a match included), 1 bad usage or other error,
//! 2 no usable audio device, 3 Shazam unreachable, 4 rate-limited by Shazam.

use std::process::ExitCode;

// Its ffmpeg fallback import is unused without the "ffmpeg" feature.
#[path = "../../upstream/src/core/fingerprinting/algorithm.rs"]
#[allow(unused_imports)]
mod algorithm;
#[path = "../../upstream/src/core/fingerprinting/hanning.rs"]
mod hanning;
#[path = "../../upstream/src/core/fingerprinting/signature_format.rs"]
mod signature_format;
#[path = "../../upstream/src/core/fingerprinting/user_agent.rs"]
mod user_agent;

mod communication;
mod listen;

// The module paths SongRec's files expect (crate::core::fingerprinting::..., crate::plugins::...).
mod core {
    pub mod fingerprinting {
        pub(crate) use crate::hanning;
        pub(crate) use crate::signature_format;
    }
}
mod plugins {
    /// algorithm.rs imports this unconditionally but only calls it with the "ffmpeg"
    /// feature, which this build leaves off (upstream's version needs the tempfile crate).
    pub mod ffmpeg_wrapper {
        #[allow(dead_code)]
        pub fn decode_with_ffmpeg(
            _file_path: &str,
        ) -> Option<rodio::Decoder<std::io::BufReader<std::fs::File>>> {
            None
        }
    }
}

use algorithm::SignatureGenerator;
use communication::{RecognizeError, recognize_song_from_signature};
use signature_format::DecodedSignature;

pub const EXIT_USAGE: u8 = 1;
pub const EXIT_AUDIO: u8 = 2;
pub const EXIT_NETWORK: u8 = 3;
pub const EXIT_RATE_LIMITED: u8 = 4;

fn usage() -> ExitCode {
    eprintln!(
        "songrec {} - SongRec's recognizer for Windows\n\n\
        Usage:\n  \
        songrec audio-file-to-recognized-song FILE\n  \
        songrec audio-file-to-fingerprint FILE\n  \
        songrec fingerprint-to-recognized-song URI\n  \
        songrec listen|recognize [--audio-device ID | --loopback] [--request-interval SECS]\n                           \
        [--timeout SECS] [--json] [--list-devices]\n\n\
        Source: https://github.com/marin-m/SongRec (GPL-3.0-or-later)",
        env!("CARGO_PKG_VERSION")
    );
    ExitCode::from(EXIT_USAGE)
}

fn exit_for(error: &RecognizeError) -> ExitCode {
    eprintln!("Error: {error}");
    ExitCode::from(match error {
        RecognizeError::RateLimited => EXIT_RATE_LIMITED,
        RecognizeError::Network(_) => EXIT_NETWORK,
        RecognizeError::Other(_) => EXIT_USAGE,
    })
}

fn print_answer(signature: Result<DecodedSignature, Box<dyn std::error::Error>>) -> ExitCode {
    let signature = match signature {
        Ok(signature) => signature,
        Err(error) => {
            eprintln!("Error: {error}");
            return ExitCode::from(EXIT_USAGE);
        }
    };

    match recognize_song_from_signature(&signature) {
        Ok(json) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&json).unwrap_or_default()
            );
            ExitCode::SUCCESS
        }
        Err(error) => exit_for(&error),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(subcommand) = args.first() else {
        return usage();
    };

    match (subcommand.as_str(), args.get(1)) {
        ("audio-file-to-recognized-song", Some(file)) => {
            print_answer(SignatureGenerator::make_signature_from_file(file))
        }
        ("fingerprint-to-recognized-song", Some(uri)) => {
            print_answer(DecodedSignature::decode_from_uri(uri))
        }
        ("audio-file-to-fingerprint", Some(file)) => {
            match SignatureGenerator::make_signature_from_file(file).and_then(|s| s.encode_to_uri())
            {
                Ok(uri) => {
                    println!("{uri}");
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("Error: {error}");
                    ExitCode::from(EXIT_USAGE)
                }
            }
        }
        ("listen" | "recognize", _) => match listen::Options::parse(&args[1..]) {
            Some(mut options) => {
                options.once = subcommand == "recognize";
                listen::run(&options)
            }
            None => usage(),
        },
        ("--version" | "-V", _) => {
            println!("songrec {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        _ => usage(),
    }
}
