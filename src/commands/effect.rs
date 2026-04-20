use std::path::Path;

use crate::cli::ResolvedArgs;
use crate::error::GenerateError;
use crate::generator::{GenerateContext, Generator, TemplateContext};
use crate::naming::validate_crate_name;

/// Effect generator :
/// creates a PrGPU/VEKL-based Adobe effect project.
pub struct EffectGenerator;

impl Generator for EffectGenerator {
	fn validate(args: &ResolvedArgs) -> Result<(), GenerateError> {
		validate_crate_name(&args.name)?;

		if args.app.is_empty() {
			return Err(GenerateError::NoAppSpecified);
		}

		if let Some(ref prefix) = args.prefix {
			crate::naming::validate_prefix(prefix)?;
		}

		Ok(())
	}

	fn generate(args: &ResolvedArgs, ctx: &GenerateContext) -> Result<(), GenerateError> {
		let tpl_ctx = TemplateContext::from_args(args);
		let tera_ctx = tpl_ctx.to_tera_context();

		// Walk the template directory and render/copy all files
		let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
		let template_base = Path::new(&manifest_dir).join("templates").join("effect");

		let mode_dir = match args.mode {
			crate::cli::PassMode::SinglePass => "single-pass",
			crate::cli::PassMode::MultiPass => "multi-pass",
		};

		let template_dir = template_base.join(mode_dir);
		walk_and_render(&template_dir, "", ctx, &tera_ctx)?;

		// Rename .vekl files to match kernel names.
		// `declare_kernel!` looks for `<kernel_name>.vekl` in the shaders directory,
		// so the generic template filenames must be renamed after rendering.
		let shaders_dir = ctx.output_dir.join("shaders");
		match args.mode {
			crate::cli::PassMode::SinglePass => {
				let from = shaders_dir.join("effect.vekl");
				let to = shaders_dir.join(format!("{}.vekl", tpl_ctx.kernel_name));
				if from.exists() {
					std::fs::rename(&from, &to)?;
					log::info!("Renamed {} → {}", from.display(), to.display());
				}
			}
			crate::cli::PassMode::MultiPass => {
				let from1 = shaders_dir.join("edge_detect.vekl");
				let to1 = shaders_dir.join(format!("{}.vekl", tpl_ctx.pass1_kernel_name));
				if from1.exists() {
					std::fs::rename(&from1, &to1)?;
					log::info!("Renamed {} → {}", from1.display(), to1.display());
				}
				let from2 = shaders_dir.join("composite.vekl");
				let to2 = shaders_dir.join(format!("{}.vekl", tpl_ctx.pass2_kernel_name));
				if from2.exists() {
					std::fs::rename(&from2, &to2)?;
					log::info!("Renamed {} → {}", from2.display(), to2.display());
				}
			}
		}

		// Update workspace if detected
		if ctx.in_workspace
			&& let Some(ref workspace_toml) = ctx.workspace_cargo_toml {
				crate::workspace::add_workspace_member(workspace_toml, &args.name)?;
				log::info!("Added '{}' to workspace members", args.name);
			}

		Ok(())
	}

	fn post_generate(args: &ResolvedArgs, ctx: &GenerateContext) -> Result<(), GenerateError> {
		log::info!("Running cargo check on {}...", ctx.output_dir.display());

		let output = std::process::Command::new("cargo")
			.args(["check"])
			.current_dir(&ctx.output_dir)
			.output();

		match output {
			Ok(output) if output.status.success() => {
				log::info!("✓ cargo check passed for '{}'", args.name);
				Ok(())
			}
			Ok(output) => {
				let stdout = String::from_utf8_lossy(&output.stdout);
				let stderr = String::from_utf8_lossy(&output.stderr);
				log::warn!("⚠ cargo check failed for '{}'. Project was created but may need fixes.", args.name);
				if !stderr.is_empty() {
					eprintln!("{stderr}");
				}
				if !stdout.is_empty() {
					eprintln!("{stdout}");
				}
				Err(GenerateError::CargoCheckFailed)
			}
			Err(e) => {
				log::warn!("⚠ Could not run cargo check: {e}. Project was created successfully.");
				Ok(())
			}
		}
	}
}

/// Recursively walk a template directory, rendering `.tera` files and copying others.
fn walk_and_render(
	dir: &Path,
	relative_prefix: &str,
	ctx: &GenerateContext,
	tera_ctx: &tera::Context,
) -> Result<(), GenerateError> {
	if !dir.exists() {
		log::warn!("Template directory does not exist: {}", dir.display());
		return Ok(());
	}

	for entry in std::fs::read_dir(dir)? {
		let entry = entry?;
		let file_name = entry.file_name().to_string_lossy().to_string();
		let entry_path = entry.path();
		let relative = if relative_prefix.is_empty() {
			file_name.clone()
		} else {
			format!("{relative_prefix}/{file_name}")
		};

		if entry_path.is_dir() {
			walk_and_render(&entry_path, &relative, ctx, tera_ctx)?;
		} else if file_name.ends_with(".tera") {
			// Render Tera template — output path strips the .tera extension
			let template_path = &relative;
			let output_path = &relative[..relative.len() - 5]; // strip ".tera"
			ctx.render_to_file(template_path, output_path, tera_ctx)?;
		} else {
			// Copy static file as-is
			ctx.copy_static(&entry_path, &relative)?;
		}
	}

	Ok(())
}
