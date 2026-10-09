fn main() {
	let version = std::fs::read_to_string("../../VERSION")
		.map(|v| v.trim().to_string())
		.unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());
	println!("cargo:rustc-env=IIW_SETUP_VERSION={version}");
	let name = std::fs::read_to_string("../../RELEASE_NAME").map(|v| v.trim().to_string()).unwrap_or_default();
	println!("cargo:rustc-env=IIW_RELEASE_NAME={name}");
	println!("cargo:rerun-if-changed=../../VERSION");
	println!("cargo:rerun-if-changed=../../RELEASE_NAME");
	println!("cargo:rerun-if-changed=../README.en.md");
	println!("cargo:rerun-if-changed=../README.pt-BR.md");
	println!("cargo:rerun-if-changed=../ui");
	tauri_build::build()
}
