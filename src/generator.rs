use std::fs;
use std::path::Path;

use serde::Serialize;
use tera::Tera;

use crate::cli::{AppTarget, PassMode, ResolvedArgs};
use crate::error::GenerateError;
use crate::naming::{derive_display_name, derive_pipl_names};

/// Core trait for all subcommands — effect, transition, future types.
pub trait Generator {
	/// Validate arguments before generation.
	fn validate(args: &ResolvedArgs) -> Result<(), GenerateError>;

	/// Generate the project files.
	fn generate(args: &ResolvedArgs, ctx: &GenerateContext) -> Result<(), GenerateError>;

	/// Post-generation steps — cargo check.
	fn post_generate(args: &ResolvedArgs, ctx: &GenerateContext) -> Result<(), GenerateError>;
}

/// Shared generation context passed to all generators.
pub struct GenerateContext {
	pub tera: Tera,
	pub output_dir: std::path::PathBuf,
	pub in_workspace: bool,
	pub workspace_cargo_toml: Option<std::path::PathBuf>,
}

impl GenerateContext {
	/// Build a GenerateContext: detect workspace, initialize Tera with the correct template dir.
	pub fn new(args: &ResolvedArgs) -> Result<Self, GenerateError> {
		let output_dir = match &args.dir {
			Some(d) => d.clone(),
			None => std::path::PathBuf::from(&args.name),
		};

		let workspace_cargo_toml = crate::workspace::detect_workspace(&output_dir);
		let in_workspace = workspace_cargo_toml.is_some();

		let template_dir = template_dir_for(args)?;
		let tera = Tera::new(&format!("{}/**/*", template_dir.display()))?;

		Ok(Self {
			tera,
			output_dir,
			in_workspace,
			workspace_cargo_toml,
		})
	}

	/// Render a Tera template and write it to the output directory.
	pub fn render_to_file(
		&self,
		template_path: &str,
		output_relative: &str,
		context: &tera::Context,
	) -> Result<(), GenerateError> {
		let rendered = self.tera.render(template_path, context)?;
		let output_path = self.output_dir.join(output_relative);

		if let Some(parent) = output_path.parent() {
			fs::create_dir_all(parent)?;
		}

		fs::write(&output_path, rendered)?;
		Ok(())
	}

	/// Copy a static file (non-template) to the output directory.
	pub fn copy_static(&self, source: &Path, output_relative: &str) -> Result<(), GenerateError> {
		let output_path = self.output_dir.join(output_relative);
		if let Some(parent) = output_path.parent() {
			fs::create_dir_all(parent)?;
		}
		fs::copy(source, &output_path)?;
		Ok(())
	}

	/// Create an empty directory in the output.
	#[allow(dead_code)]
	pub fn create_dir(&self, output_relative: &str) -> Result<(), GenerateError> {
		let dir_path = self.output_dir.join(output_relative);
		fs::create_dir_all(&dir_path)?;
		Ok(())
	}

	/// Create a file with the given content at the output relative path.
	#[allow(dead_code)]
	pub fn write_file(&self, output_relative: &str, content: &str) -> Result<(), GenerateError> {
		let output_path = self.output_dir.join(output_relative);
		if let Some(parent) = output_path.parent() {
			fs::create_dir_all(parent)?;
		}
		fs::write(&output_path, content)?;
		Ok(())
	}
}

/// Determine which template directory to use based on the resolved args.
fn template_dir_for(args: &ResolvedArgs) -> Result<std::path::PathBuf, GenerateError> {
	let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
	let base = Path::new(&manifest_dir).join("templates");

	match args.mode {
		PassMode::SinglePass => Ok(base.join("effect").join("single-pass")),
		PassMode::MultiPass => Ok(base.join("effect").join("multi-pass")),
	}
}

/// Build the unified Tera template context from resolved arguments.
#[derive(Serialize)]
pub struct TemplateContext {
	pub crate_name: String,
	pub display_name: String,
	pub match_name: String,
	pub effect_name: String,
	pub prefix: String,
	pub short_prefix: String,
	pub has_prefix: bool,
	pub is_premiere: bool,
	pub is_afterfx: bool,
	pub is_single_pass: bool,
	pub is_multi_pass: bool,
	pub kernel_name: String,
	pub kernel_params_name: String,
	pub pass1_kernel_name: String,
	pub pass1_kernel_params_name: String,
	pub pass2_kernel_name: String,
	pub pass2_kernel_params_name: String,
}

fn to_pascal_case(snake: &str) -> String {
	snake
		.split('_')
		.map(|word| {
			let mut chars = word.chars();
			match chars.next() {
				None => String::new(),
				Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
			}
		})
		.collect()
}

impl TemplateContext {
	pub fn from_args(args: &ResolvedArgs) -> Self {
		let display_name = derive_display_name(&args.name);
		let (match_name, effect_name) = derive_pipl_names(&args.prefix, &display_name);

		let prefix = args.prefix.clone().unwrap_or_default();
		let short_prefix = if prefix.is_empty() {
			String::new()
		} else {
			prefix[..2].to_string()
		};

		let is_premiere = args.app.contains(&AppTarget::Premiere);
		let is_afterfx = args.app.contains(&AppTarget::Afterfx);

		let kernel_name = args.name.clone();
		let kernel_params_name = format!("{}Params", to_pascal_case(&kernel_name));

		// Multi-pass kernel names are derived from the crate name so that
		// `declare_kernel!` can find the matching `.vekl` file.
		let pass1_kernel_name = format!("{kernel_name}_edge");
		let pass1_kernel_params_name = format!("{}EdgeParams", to_pascal_case(&kernel_name));
		let pass2_kernel_name = format!("{kernel_name}_tint");
		let pass2_kernel_params_name = format!("{}TintParams", to_pascal_case(&kernel_name));

		Self {
			crate_name: args.name.clone(),
			display_name,
			match_name,
			effect_name,
			prefix,
			short_prefix,
			has_prefix: args.prefix.is_some(),
			is_premiere,
			is_afterfx,
			is_single_pass: args.mode == PassMode::SinglePass,
			is_multi_pass: args.mode == PassMode::MultiPass,
			kernel_name,
			kernel_params_name,
			pass1_kernel_name,
			pass1_kernel_params_name,
			pass2_kernel_name,
			pass2_kernel_params_name,
		}
	}

	pub fn to_tera_context(&self) -> tera::Context {
		let mut ctx = tera::Context::new();
		ctx.insert("crate_name", &self.crate_name);
		ctx.insert("display_name", &self.display_name);
		ctx.insert("match_name", &self.match_name);
		ctx.insert("effect_name", &self.effect_name);
		ctx.insert("prefix", &self.prefix);
		ctx.insert("short_prefix", &self.short_prefix);
		ctx.insert("has_prefix", &self.has_prefix);
		ctx.insert("is_premiere", &self.is_premiere);
		ctx.insert("is_afterfx", &self.is_afterfx);
		ctx.insert("is_single_pass", &self.is_single_pass);
		ctx.insert("is_multi_pass", &self.is_multi_pass);
		ctx.insert("kernel_name", &self.kernel_name);
		ctx.insert("kernel_params_name", &self.kernel_params_name);
		ctx.insert("pass1_kernel_name", &self.pass1_kernel_name);
		ctx.insert("pass1_kernel_params_name", &self.pass1_kernel_params_name);
		ctx.insert("pass2_kernel_name", &self.pass2_kernel_name);
		ctx.insert("pass2_kernel_params_name", &self.pass2_kernel_params_name);
		ctx
	}
}
