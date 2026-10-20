use serde::{Deserialize, Deserializer, Serialize};

pub const TASK_PREFIX: &str = "illogical-impulse";
pub const TASK_DESCRIPTION: &str = "Starts illogical-impulse (ii-windows) when you sign in.";
pub const SHELL_ARGS: &str = "-n -c ii";

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StartupMethod {
	#[default]
	Task,
	Run,
}

impl StartupMethod {
	pub fn parse(s: &str) -> Option<StartupMethod> {
		match s.trim().to_ascii_lowercase().as_str() {
			"task" => Some(StartupMethod::Task),
			"run" => Some(StartupMethod::Run),
			_ => None,
		}
	}

	pub fn as_str(self) -> &'static str {
		match self {
			StartupMethod::Task => "task",
			StartupMethod::Run => "run",
		}
	}
}

pub fn lenient_method<'de, D: Deserializer<'de>>(d: D) -> Result<Option<StartupMethod>, D::Error> {
	let v = Option::<serde_json::Value>::deserialize(d)?;
	Ok(v.as_ref().and_then(|v| v.as_str()).and_then(StartupMethod::parse))
}

pub fn task_name(sid: &str) -> String {
	format!("{TASK_PREFIX}-{}", sid.trim())
}

pub fn user_id(domain: &str, name: &str, sid: &str) -> String {
	let (domain, name) = (domain.trim(), name.trim());
	match (domain.is_empty(), name.is_empty()) {
		(_, true) => sid.trim().to_string(),
		(true, false) => name.to_string(),
		(false, false) => format!("{domain}\\{name}"),
	}
}

pub fn xml_escape(s: &str) -> String {
	let mut out = String::with_capacity(s.len());
	for c in s.chars() {
		match c {
			'&' => out.push_str("&amp;"),
			'<' => out.push_str("&lt;"),
			'>' => out.push_str("&gt;"),
			'"' => out.push_str("&quot;"),
			'\'' => out.push_str("&apos;"),
			c => out.push(c),
		}
	}
	out
}

pub fn task_command(exe: &str) -> String {
	if exe.contains(char::is_whitespace) && !exe.starts_with('"') {
		format!("\"{exe}\"")
	} else {
		exe.to_string()
	}
}

pub fn run_command(exe: &str) -> String {
	format!("\"{exe}\" {SHELL_ARGS}")
}

pub fn task_xml(user: &str, exe: &str, work_dir: &str) -> String {
	let user = xml_escape(user);
	let mut x = String::new();
	x.push_str("<Task version=\"1.2\" xmlns=\"http://schemas.microsoft.com/windows/2004/02/mit/task\">");
	x.push_str("<RegistrationInfo><Description>");
	x.push_str(&xml_escape(TASK_DESCRIPTION));
	x.push_str("</Description></RegistrationInfo>");
	x.push_str("<Triggers><LogonTrigger><Enabled>true</Enabled><UserId>");
	x.push_str(&user);
	x.push_str("</UserId></LogonTrigger></Triggers>");
	x.push_str("<Principals><Principal id=\"Author\"><UserId>");
	x.push_str(&user);
	x.push_str(
		"</UserId><LogonType>InteractiveToken</LogonType><RunLevel>LeastPrivilege</RunLevel></Principal></Principals>",
	);
	x.push_str("<Settings><MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>");
	x.push_str("<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>");
	x.push_str("<StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>");
	x.push_str("<ExecutionTimeLimit>PT0S</ExecutionTimeLimit><Priority>4</Priority><Enabled>true</Enabled></Settings>");
	x.push_str("<Actions Context=\"Author\"><Exec><Command>");
	x.push_str(&xml_escape(&task_command(exe)));
	x.push_str("</Command><Arguments>");
	x.push_str(&xml_escape(SHELL_ARGS));
	x.push_str("</Arguments><WorkingDirectory>");
	x.push_str(&xml_escape(work_dir));
	x.push_str("</WorkingDirectory></Exec></Actions></Task>");
	x
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Live {
	pub task: Option<bool>,
	pub run: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refresh {
	Task { enabled: bool },
	Run { enabled: bool },
	Migrate,
	Off,
}

pub fn refresh_plan(recorded: Option<StartupMethod>, live: Live) -> Refresh {
	match (live.task, live.run) {
		(Some(enabled), _) => Refresh::Task { enabled },
		(None, Some(true)) if recorded.is_none() => Refresh::Migrate,
		(None, Some(enabled)) => Refresh::Run { enabled },
		(None, None) => Refresh::Off,
	}
}

pub fn current(recorded: Option<StartupMethod>, live: Live) -> (bool, StartupMethod) {
	match refresh_plan(recorded, live) {
		Refresh::Task { enabled } => (enabled, StartupMethod::Task),
		Refresh::Run { enabled } => {
			(enabled, if recorded.is_none() { StartupMethod::Task } else { StartupMethod::Run })
		}
		Refresh::Migrate => (true, StartupMethod::Task),
		Refresh::Off => (false, recorded.unwrap_or_default()),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn between<'a>(xml: &'a str, open: &str, close: &str) -> Vec<&'a str> {
		let mut out = Vec::new();
		let mut rest = xml;
		while let Some(a) = rest.find(open) {
			let after = &rest[a + open.len()..];
			let b = after.find(close).unwrap();
			out.push(&after[..b]);
			rest = &after[b + close.len()..];
		}
		out
	}

	#[test]
	fn escapes_xml_text() {
		assert_eq!(xml_escape(r#"a&b<c>d"e'f"#), "a&amp;b&lt;c&gt;d&quot;e&apos;f");
		assert_eq!(xml_escape("José Ünïcode 用户"), "José Ünïcode 用户");
		assert_eq!(xml_escape(""), "");
	}

	#[test]
	fn formats_user_ids() {
		let sid = "S-1-5-21-1-2-3-1001";
		assert_eq!(user_id("DESKTOP-ABC", "rafab", sid), r"DESKTOP-ABC\rafab");
		assert_eq!(user_id("AzureAD", "RafaelPasquarella", sid), r"AzureAD\RafaelPasquarella");
		assert_eq!(user_id(" PC ", " João Silva ", sid), r"PC\João Silva");
		assert_eq!(user_id("", "rafab", sid), "rafab");
		assert_eq!(user_id("PC", "", sid), sid);
		assert_eq!(user_id("", "  ", sid), sid);
		assert_eq!(task_name(sid), "illogical-impulse-S-1-5-21-1-2-3-1001");
	}

	#[test]
	fn quotes_commands() {
		assert_eq!(
			task_command(r"C:\Users\rafab\AppData\Local\ii-windows\qsw.exe"),
			r"C:\Users\rafab\AppData\Local\ii-windows\qsw.exe"
		);
		assert_eq!(
			task_command(r"C:\Users\João Silva\AppData\Local\ii-windows\qsw.exe"),
			r#""C:\Users\João Silva\AppData\Local\ii-windows\qsw.exe""#
		);
		assert_eq!(task_command(r#""C:\a b\qsw.exe""#), r#""C:\a b\qsw.exe""#);
		assert_eq!(run_command(r"C:\a b\qsw.exe"), r#""C:\a b\qsw.exe" -n -c ii"#);
		assert_eq!(run_command(r"C:\ab\qsw.exe"), r#""C:\ab\qsw.exe" -n -c ii"#);
	}

	#[test]
	fn builds_the_task() {
		let xml = task_xml(
			r"PC\rafab",
			r"C:\Users\rafab\AppData\Local\ii-windows\qsw.exe",
			r"C:\Users\rafab\AppData\Local\ii-windows",
		);
		assert!(
			xml.starts_with("<Task version=\"1.2\" xmlns=\"http://schemas.microsoft.com/windows/2004/02/mit/task\">")
		);
		assert!(xml.ends_with("</Task>"));
		assert_eq!(between(&xml, "<UserId>", "</UserId>"), vec![r"PC\rafab", r"PC\rafab"]);
		assert_eq!(between(&xml, "<Command>", "</Command>"), vec![r"C:\Users\rafab\AppData\Local\ii-windows\qsw.exe"]);
		assert_eq!(between(&xml, "<Arguments>", "</Arguments>"), vec!["-n -c ii"]);
		assert_eq!(
			between(&xml, "<WorkingDirectory>", "</WorkingDirectory>"),
			vec![r"C:\Users\rafab\AppData\Local\ii-windows"]
		);
		for part in [
			"<LogonTrigger><Enabled>true</Enabled>",
			"<LogonType>InteractiveToken</LogonType>",
			"<RunLevel>LeastPrivilege</RunLevel>",
			"<MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>",
			"<DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>",
			"<StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>",
			"<ExecutionTimeLimit>PT0S</ExecutionTimeLimit>",
			"<Priority>4</Priority>",
			"<Actions Context=\"Author\">",
			"<Principal id=\"Author\">",
			"<Description>Starts illogical-impulse (ii-windows) when you sign in.</Description>",
		] {
			assert!(xml.contains(part), "missing {part}");
		}
		assert!(!xml.contains("<?xml"));
	}

	#[test]
	fn escapes_paths_and_users_in_the_task() {
		let xml = task_xml(
			r"CORP\Tom & Jerry",
			r"C:\Users\Tom & Jerry\AppData\Local\ii-windows\qsw.exe",
			r"C:\Users\Tom & Jerry\<x>",
		);
		assert_eq!(between(&xml, "<UserId>", "</UserId>"), vec![r"CORP\Tom &amp; Jerry", r"CORP\Tom &amp; Jerry"]);
		assert_eq!(
			between(&xml, "<Command>", "</Command>"),
			vec![r"&quot;C:\Users\Tom &amp; Jerry\AppData\Local\ii-windows\qsw.exe&quot;"]
		);
		assert_eq!(
			between(&xml, "<WorkingDirectory>", "</WorkingDirectory>"),
			vec![r"C:\Users\Tom &amp; Jerry\&lt;x&gt;"]
		);
		let unicode = task_xml(
			r"MÁQUINA\Conceição",
			r"C:\Users\Conceição\AppData\Local\ii-windows\qsw.exe",
			r"C:\Users\Conceição\AppData\Local\ii-windows",
		);
		assert_eq!(between(&unicode, "<UserId>", "</UserId>"), vec![r"MÁQUINA\Conceição", r"MÁQUINA\Conceição"]);
		assert_eq!(
			between(&unicode, "<Command>", "</Command>"),
			vec![r"C:\Users\Conceição\AppData\Local\ii-windows\qsw.exe"]
		);
		let lt = xml.matches('<').count();
		let gt = xml.matches('>').count();
		assert_eq!(lt, gt);
	}

	#[test]
	fn parses_methods_leniently() {
		#[derive(Deserialize)]
		struct O {
			#[serde(default, deserialize_with = "lenient_method")]
			m: Option<StartupMethod>,
		}
		let p = |s: &str| serde_json::from_str::<O>(s).unwrap().m;
		assert_eq!(p(r#"{"m":"task"}"#), Some(StartupMethod::Task));
		assert_eq!(p(r#"{"m":"run"}"#), Some(StartupMethod::Run));
		assert_eq!(p(r#"{"m":"RUN"}"#), Some(StartupMethod::Run));
		assert_eq!(p(r#"{"m":"service"}"#), None);
		assert_eq!(p(r#"{"m":null}"#), None);
		assert_eq!(p(r#"{"m":3}"#), None);
		assert_eq!(p(r#"{}"#), None);
		assert_eq!(serde_json::to_string(&StartupMethod::Run).unwrap(), r#""run""#);
	}

	#[test]
	fn plans_refreshes() {
		use StartupMethod::*;
		let live = |task, run| Live { task, run };
		assert_eq!(refresh_plan(None, live(None, Some(true))), Refresh::Migrate);
		assert_eq!(refresh_plan(None, live(None, Some(false))), Refresh::Run { enabled: false });
		assert_eq!(refresh_plan(Some(Task), live(None, Some(true))), Refresh::Run { enabled: true });
		assert_eq!(refresh_plan(Some(Run), live(None, Some(true))), Refresh::Run { enabled: true });
		assert_eq!(refresh_plan(Some(Run), live(Some(true), None)), Refresh::Task { enabled: true });
		assert_eq!(refresh_plan(Some(Task), live(Some(false), None)), Refresh::Task { enabled: false });
		assert_eq!(refresh_plan(None, live(Some(true), Some(true))), Refresh::Task { enabled: true });
		assert_eq!(refresh_plan(Some(Task), live(None, None)), Refresh::Off);
		assert_eq!(refresh_plan(None, live(None, None)), Refresh::Off);
	}

	#[test]
	fn reports_the_current_choice() {
		use StartupMethod::*;
		let live = |task, run| Live { task, run };
		assert_eq!(current(None, live(None, Some(true))), (true, Task));
		assert_eq!(current(None, live(None, Some(false))), (false, Task));
		assert_eq!(current(Some(Task), live(None, Some(true))), (true, Run));
		assert_eq!(current(Some(Run), live(None, Some(false))), (false, Run));
		assert_eq!(current(Some(Run), live(Some(true), None)), (true, Task));
		assert_eq!(current(Some(Run), live(None, None)), (false, Run));
		assert_eq!(current(None, live(None, None)), (false, Task));
	}
}
