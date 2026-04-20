use std::path::PathBuf;

use clap::Parser;

/// CLI scaffold tool for PrGPU/VEKL-based video effects and transitions
#[derive(Parser, Debug)]
#[command(
    name = "cargo-create-video-effect",
    bin_name = "cargo create-video-effect",
    version,
    about
)]
pub struct Cli {
    /// Project type: "effect" or "transition" (default: effect).
    /// If the value doesn't match a known type it is interpreted as the project name instead.
    pub r#type: Option<String>,

    /// Effect or transition name — must be a valid Rust crate name
    pub name: Option<String>,

    /// Target Adobe applications — comma-separated (default: premiere,afterfx)
    #[arg(short, long, value_delimiter = ',')]
    pub app: Option<Vec<String>>,

    /// Rendering mode: single-pass or multi-pass
    #[arg(short, long, default_value = "single-pass")]
    pub mode: PassMode,

    /// PIPL name prefix — 2 to 6 uppercase letters (e.g. "ADBE")
    #[arg(short, long)]
    pub prefix: Option<String>,

    /// Output directory override (default: ./<name>)
    #[arg(short, long)]
    pub dir: Option<PathBuf>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProjectType {
    Effect,
    Transition,
}

#[derive(Clone, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum PassMode {
    SinglePass,
    MultiPass,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppTarget {
    Premiere,
    Afterfx,
}

/// Fully resolved arguments after CLI parsing + TUI fallback
#[derive(Clone, Debug)]
pub struct ResolvedArgs {
    #[allow(dead_code)]
    pub project_type: ProjectType,
    pub name: String,
    pub app: Vec<AppTarget>,
    pub mode: PassMode,
    pub prefix: Option<String>,
    pub dir: Option<PathBuf>,
}

impl Cli {
    /// Parse the two positional arguments into (project_type, name).
    ///
    /// Resolution logic:
    /// - Both provided and first matches a known type → (type, name)
    /// - First doesn't match a known type → treat first as name, type defaults to Effect
    /// - Only one provided and matches a type → (type, None)
    /// - Only one provided and doesn't match a type → (Effect, Some(value))
    /// - None provided → (Effect, None)
    pub fn resolve_type_and_name(&self) -> (ProjectType, Option<String>) {
        match (&self.r#type, &self.name) {
            (Some(first), Some(second)) => match first.as_str() {
                "effect" => (ProjectType::Effect, Some(second.clone())),
                "transition" => (ProjectType::Transition, Some(second.clone())),
                _ => (ProjectType::Effect, Some(first.clone())),
            },
            (Some(first), None) => match first.as_str() {
                "effect" => (ProjectType::Effect, None),
                "transition" => (ProjectType::Transition, None),
                _ => (ProjectType::Effect, Some(first.clone())),
            },
            (None, _) => (ProjectType::Effect, self.name.clone()),
        }
    }

    /// Resolve the --app flag into a Vec<AppTarget>.
    /// Returns None if the flag was not provided (use default).
    pub fn resolve_app(&self) -> Option<Result<Vec<AppTarget>, String>> {
        self.app.as_ref().map(|vals| {
            if vals.is_empty() {
                return Err("At least one app must be specified with --app".to_string());
            }
            let mut targets = Vec::with_capacity(vals.len());
            for v in vals {
                match v.to_lowercase().as_str() {
                    "premiere" => targets.push(AppTarget::Premiere),
                    "afterfx" | "after-effects" | "ae" => targets.push(AppTarget::Afterfx),
                    other => return Err(format!("Unknown app target: '{other}'. Valid: premiere, afterfx")),
                }
            }
            Ok(targets)
        })
    }
}
