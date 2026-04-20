use crate::cli::{AppTarget, PassMode, ProjectType, ResolvedArgs};
use crate::error::GenerateError;
use crate::naming::validate_crate_name;

/// Resolve missing mandatory arguments via interactive TUI prompts.
///
/// Only prompts for arguments that are missing and don't have a default.
/// `--prefix` and `--dir` are never prompted (CLI-only).
pub fn resolve_missing_args(
    project_type: ProjectType,
    name: Option<String>,
    app: Option<Vec<AppTarget>>,
    mode: Option<PassMode>,
    prefix: Option<String>,
    dir: Option<std::path::PathBuf>,
) -> Result<ResolvedArgs, GenerateError> {
    let resolved_type = resolve_project_type(project_type)?;
    let resolved_name = resolve_name(name)?;
    let resolved_app = resolve_app(app)?;
    let resolved_mode = resolve_mode(mode);

    Ok(ResolvedArgs {
        project_type: resolved_type,
        name: resolved_name,
        app: resolved_app,
        mode: resolved_mode,
        prefix,
        dir,
    })
}

fn resolve_project_type(project_type: ProjectType) -> Result<ProjectType, GenerateError> {
    // Type has a default (effect), so only prompt if user explicitly wants to change
    // In practice the type is already resolved from CLI before TUI, so this is a no-op
    Ok(project_type)
}

fn resolve_name(name: Option<String>) -> Result<String, GenerateError> {
    match name {
        Some(n) => {
            validate_crate_name(&n)?;
            Ok(n)
        }
        None => {
            let result = inquire::Text::new("Effect name (must be a valid Rust crate name):")
                .with_validator(|input: &str| {
                    match validate_crate_name(input) {
                        Ok(()) => Ok(inquire::validator::Validation::Valid),
                        Err(e) => Ok(inquire::validator::Validation::Invalid(
                            inquire::validator::ErrorMessage::Custom(e.to_string()),
                        )),
                    }
                })
                .prompt();
            match result {
                Ok(n) => Ok(n),
                Err(e) => Err(GenerateError::InvalidCrateName {
                    name: String::new(),
                    reason: format!("Failed to read name from TUI: {e}"),
                }),
            }
        }
    }
}

fn resolve_app(app: Option<Vec<AppTarget>>) -> Result<Vec<AppTarget>, GenerateError> {
    match app {
        Some(targets) if !targets.is_empty() => Ok(targets),
        _ => {
            let options = vec!["premiere", "afterfx"];
            let defaults = options.iter().enumerate().filter(|_| true).map(|(i, _)| i).collect::<Vec<_>>();
            let result = inquire::MultiSelect::new("Target applications:", options)
                .with_default(&defaults)
                .prompt();
            match result {
                Ok(selections) => {
                    let targets: Vec<AppTarget> = selections
                        .into_iter()
                        .map(|s| match s {
                            "premiere" => AppTarget::Premiere,
                            "afterfx" => AppTarget::Afterfx,
                            _ => AppTarget::Premiere,
                        })
                        .collect();
                    if targets.is_empty() {
                        return Err(GenerateError::NoAppSpecified);
                    }
                    Ok(targets)
                }
                Err(_) => Ok(vec![AppTarget::Premiere, AppTarget::Afterfx]),
            }
        }
    }
}

fn resolve_mode(mode: Option<PassMode>) -> PassMode {
    match mode {
        Some(m) => m,
        None => {
            let options = vec!["single-pass", "multi-pass"];
            let result = inquire::Select::new("Rendering mode:", options)
                .with_starting_cursor(0)
                .prompt();
            match result {
                Ok("multi-pass") => PassMode::MultiPass,
                _ => PassMode::SinglePass,
            }
        }
    }
}
