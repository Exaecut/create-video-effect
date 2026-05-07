use include_dir::{Dir, DirEntry};

use crate::cli::ResolvedArgs;
use crate::error::GenerateError;
use crate::generator::{embedded_template_dir, embedded_template_prefix, GenerateContext, Generator, TemplateContext};
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

		// Walk the EMBEDDED templates (baked in at compile time) for the
		// requested pass mode and render / copy each file.
		let prefix = embedded_template_prefix(args);
		let root = embedded_template_dir(args).ok_or_else(|| {
			GenerateError::Workspace(format!("Embedded template dir '{prefix}' not found in binary"))
		})?;
		walk_embedded_and_render(root, prefix, ctx, &tera_ctx)?;

		// Rename .slang shader files to match kernel names.
		// `prgpu::build::compile_shaders` globs `shaders/*.slang` and the
		// `declare_kernel!` macro looks up a `<kernel_name>_CPU_DISPATCH`
		// symbol that was generated from `<kernel_name>.slang`, so each
		// shader file must be renamed to its kernel's identifier after
		// Tera rendering.
		let shaders_dir = ctx.output_dir.join("shaders");
		match args.mode {
			crate::cli::PassMode::SinglePass => {
				let from = shaders_dir.join("effect.slang");
				let to = shaders_dir.join(format!("{}.slang", tpl_ctx.kernel_name));
				if from.exists() {
					std::fs::rename(&from, &to)?;
					log::info!("Renamed {} → {}", from.display(), to.display());
				}
			}
			crate::cli::PassMode::MultiPass => {
				let from1 = shaders_dir.join("edge_detect.slang");
				let to1 = shaders_dir.join(format!("{}.slang", tpl_ctx.pass1_kernel_name));
				if from1.exists() {
					std::fs::rename(&from1, &to1)?;
					log::info!("Renamed {} → {}", from1.display(), to1.display());
				}
				let from2 = shaders_dir.join("composite.slang");
				let to2 = shaders_dir.join(format!("{}.slang", tpl_ctx.pass2_kernel_name));
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

/// Walk the embedded `Dir` for the selected pass mode and render every
/// `.tera` template / copy every static file to the output directory.
///
/// `root_prefix` is the include_dir path of `dir` (e.g. `effect/single-pass`);
/// we strip it from every entry path so the file lands at the right place
/// in the user's crate (src/lib.rs, benches/effect_cpu.rs, etc.).
fn walk_embedded_and_render(
	dir: &Dir<'_>,
	root_prefix: &str,
	ctx: &GenerateContext,
	tera_ctx: &tera::Context,
) -> Result<(), GenerateError> {
	for entry in dir.entries() {
		match entry {
			DirEntry::Dir(sub) => {
				walk_embedded_and_render(sub, root_prefix, ctx, tera_ctx)?;
			}
			DirEntry::File(file) => {
				let full = file.path().to_string_lossy().to_string();
				// Strip the pass-mode prefix so the output path is relative
				// to the generated crate root.
				let rel = full.strip_prefix(&format!("{root_prefix}/")).unwrap_or(&full).to_string();
				let file_name = std::path::Path::new(&rel)
					.file_name()
					.and_then(|s| s.to_str())
					.unwrap_or("");

				if file_name.ends_with(".tera") {
					let output_path = &rel[..rel.len() - 5]; // strip ".tera"
					ctx.render_to_file(&rel, output_path, tera_ctx)?;
				} else {
					// Static file — write the embedded bytes verbatim.
					ctx.write_file_bytes(&rel, file.contents())?;
				}
			}
		}
	}
	Ok(())
}
