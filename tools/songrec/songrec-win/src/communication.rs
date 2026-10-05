//! Shazam's recognition request, as upstream's core/fingerprinting/communication.rs sends it
//! (same endpoint, body and headers), over ureq instead of libsoup.

use std::fmt;
use std::time::{Duration, SystemTime};

use rand::prelude::IndexedRandom;
use serde_json::{Value, json};
use ureq::tls::{TlsConfig, TlsProvider};
use uuid::Uuid;

use crate::signature_format::DecodedSignature;
use crate::user_agent::USER_AGENTS;

#[derive(Debug)]
pub enum RecognizeError {
    /// HTTP 429: upstream tells the user to raise the request interval.
    RateLimited,
    /// No connection, TLS failure, timeout, or a non-JSON answer.
    Network(String),
    Other(String),
}

impl fmt::Display for RecognizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RateLimited => write!(f, "Your IP has been rate-limited"),
            Self::Network(message) => write!(f, "Network unreachable: {message}"),
            Self::Other(message) => write!(f, "{message}"),
        }
    }
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        // Upstream's soup session timeout.
        .timeout_global(Some(Duration::from_secs(20)))
        // A 429 must reach us as a status, not as an error with the body thrown away.
        .http_status_as_error(false)
        // SChannel: Windows' own TLS and certificate store, nothing to ship.
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .build(),
        )
        .build()
        .into()
}

pub fn recognize_song_from_signature(
    signature: &DecodedSignature,
) -> Result<Value, RecognizeError> {
    let timestamp_ms = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| RecognizeError::Other(e.to_string()))?
        .as_millis();

    let uri = signature
        .encode_to_uri()
        .map_err(|e| RecognizeError::Other(e.to_string()))?;

    let post_data = json!({
        "geolocation": {
            "altitude": 300,
            "latitude": 45,
            "longitude": 2
        },
        "signature": {
            "samplems": (signature.number_samples as f32 / signature.sample_rate_hz as f32 * 1000.) as u32,
            "timestamp": timestamp_ms as u32,
            "uri": uri
        },
        "timestamp": timestamp_ms as u32,
        "timezone": "Europe/Paris"
    })
    .to_string();

    let uuid_1 = Uuid::new_v4().hyphenated().to_string().to_uppercase();
    let uuid_2 = Uuid::new_v4().hyphenated().to_string();

    let url = format!(
        "https://amp.shazam.com/discovery/v5/en/US/android/-/tag/{}/{}\
?sync=true\
&webv3=true\
&sampling=true\
&connected=\
&shazamapiversion=v3\
&sharehub=true\
&video=v3",
        uuid_1, uuid_2
    );

    let user_agent = USER_AGENTS
        .choose(&mut rand::rng())
        .copied()
        .unwrap_or_default();

    let mut response = agent()
        .post(&url)
        .header("User-Agent", user_agent)
        .header("Content-Language", "en_US")
        .header("Content-Type", "application/json")
        .send(post_data.as_str())
        .map_err(|e| RecognizeError::Network(e.to_string()))?;

    if response.status().as_u16() == 429 {
        return Err(RecognizeError::RateLimited);
    }

    let body = response
        .body_mut()
        .read_to_vec()
        .map_err(|e| RecognizeError::Network(e.to_string()))?;

    serde_json::from_slice(&body).map_err(|e| {
        RecognizeError::Network(format!(
            "unexpected answer (HTTP {}): {e}",
            response.status()
        ))
    })
}

/// Shazam answers 200 with an empty "matches" list (and no "track") when nothing matched.
pub fn has_match(json: &Value) -> bool {
    json["track"]["title"].is_string() && json["track"]["subtitle"].is_string()
}
