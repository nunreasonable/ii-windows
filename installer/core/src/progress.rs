use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
	Pending,
	Running,
	Done,
	Skipped,
	Warning,
	Failed,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Msg {
	pub key: String,
	pub params: BTreeMap<String, String>,
	pub text: String,
}

impl Msg {
	pub fn new(key: &str, text: impl Into<String>) -> Msg {
		Msg { key: key.into(), params: BTreeMap::new(), text: text.into() }
	}
	pub fn with(mut self, k: &str, v: impl Into<String>) -> Msg {
		self.params.insert(k.into(), v.into());
		self
	}
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
	Plan {
		steps: Vec<String>,
	},
	Step {
		id: String,
		status: Status,
		detail: Option<Msg>,
	},
	Progress {
		id: String,
		fraction: f64,
		text: Option<String>,
	},
	Log {
		line: String,
	},
	Finished {
		ok: bool,
		error: Option<Msg>,
		warnings: Vec<Msg>,
		notes: Vec<Msg>,
		can_launch: bool,
	},
}

pub trait Reporter: Send + Sync {
	fn emit(&self, event: Event);
	fn cancelled(&self) -> bool {
		false
	}
}
