//! The embedded READMEs, rendered to HTML for the UI.

use pulldown_cmark::{html, Options, Parser};

pub fn to_html(markdown: &str) -> String {
	let mut opts = Options::empty();
	opts.insert(Options::ENABLE_TABLES);
	opts.insert(Options::ENABLE_STRIKETHROUGH);
	let mut out = String::with_capacity(markdown.len() * 2);
	// Raw HTML in the README would be passed through as-is; there is none, and the UI's CSP
	// blocks scripts anyway.
	html::push_html(&mut out, Parser::new_ext(markdown, opts));
	out
}

#[cfg(test)]
mod tests {
	#[test]
	fn renders() {
		let h = super::to_html("# Title\n\n- a **b**\n- [link](https://github.com/x)\n");
		assert!(h.contains("<h1>Title</h1>"));
		assert!(h.contains("<strong>b</strong>"));
		assert!(h.contains("href=\"https://github.com/x\""));
	}
}
