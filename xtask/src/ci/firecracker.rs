use std::env;
use std::path::Path;

use anyhow::Result;
use clap::Args;
use xshell::cmd;

use crate::build::Build;

/// Run hermit-rs images on Firecracker.
#[derive(Args)]
pub struct Firecracker {
	/// Run Firecracker using `sudo`.
	#[arg(long)]
	sudo: bool,

	#[command(flatten)]
	build: Build,

	#[arg(long, default_value_t = String::from("hello_world-microvm"))]
	image: String,
}

impl Firecracker {
	pub fn run(mut self) -> Result<()> {
		self.build.cargo_build.features.push("fc".to_string());

		self.build.run()?;

		let sh = crate::sh()?;

		let config = format!(
			include_str!("firecracker_vm_config.json"),
			kernel_image_path = self.build.dist_object().display(),
			initrd_path = self.build.ci_image(&self.image).display(),
		);
		eprintln!("firecracker config");
		eprintln!("{config}");
		let config_path = Path::new("firecracker_vm_config.json");
		sh.write_file(config_path, config)?;

		let firecracker = env::var("FIRECRACKER").unwrap_or_else(|_| "firecracker".to_string());
		let program = if self.sudo {
			"sudo"
		} else {
			firecracker.as_str()
		};
		let arg = self.sudo.then_some(firecracker.as_str());

		let log_path = Path::new("firecracker.log");
		sh.write_file(log_path, "")?;
		let res = cmd!(sh, "{program} {arg...} --no-api --config-file {config_path} --log-path {log_path} --level Info --show-level --show-log-origin").run();
		let log = sh.read_file(log_path)?;

		eprintln!("firecracker log");
		eprintln!("{log}");
		res?;

		Ok(())
	}
}
