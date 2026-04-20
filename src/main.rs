mod cli;
mod commands;
mod error;
mod generator;
mod naming;
mod tui;
mod workspace;

use clap::Parser;
use cli::{AppTarget, ProjectType, ResolvedArgs};
use error::GenerateError;
use generator::Generator;

fn main() {
	env_logger::init();

	if let Err(e) = run() {
		eprintln!("\x1b[31mError\x1b[0m: {e}");
		std::process::exit(1);
	}
}

fn run() -> Result<(), GenerateError> {
	let cli_args = cli::Cli::parse();

	let (project_type, name) = cli_args.resolve_type_and_name();

	if project_type == ProjectType::Transition {
		return Err(GenerateError::TransitionNotImplemented);
	}

	let app = match cli_args.resolve_app() {
		Some(Ok(targets)) => targets,
		Some(Err(e)) => return Err(GenerateError::Workspace(e)),
		None => {
			// Flag not provided — use default
			vec![AppTarget::Premiere, AppTarget::Afterfx]
		}
	};

	let needs_tui = name.is_none();

	let resolved = if needs_tui {
		tui::resolve_missing_args(project_type, name, Some(app), Some(cli_args.mode.clone()), cli_args.prefix.clone(), cli_args.dir.clone())?
	} else {
		ResolvedArgs {
			project_type,
			name: name.unwrap(),
			app,
			mode: cli_args.mode.clone(),
			prefix: cli_args.prefix.clone(),
			dir: cli_args.dir.clone(),
		}
	};

	commands::effect::EffectGenerator::validate(&resolved)?;
	let ctx = generator::GenerateContext::new(&resolved)?;
	commands::effect::EffectGenerator::generate(&resolved, &ctx)?;

	match commands::effect::EffectGenerator::post_generate(&resolved, &ctx) {
		Ok(()) => {
			println!("\n\x1b[32m✓\x1b[0m Effect \x1b[1m{}\x1b[0m created at {}", resolved.name, ctx.output_dir.display());
		}
		Err(GenerateError::CargoCheckFailed) => {
			println!(
				"\n\x1b[33m⚠\x1b[0m Effect \x1b[1m{}\x1b[0m created at {} but cargo check failed.",
				resolved.name,
				ctx.output_dir.display()
			);
		}
		Err(e) => return Err(e),
	}

	Ok(())
}
