use thiserror::Error;

#[derive(Debug, Error)]
pub enum GenerateError {
	#[error("Invalid crate name '{name}': {reason}")]
	InvalidCrateName { name: String, reason: String },

	#[error("Invalid prefix '{prefix}': must be 2-6 uppercase ASCII letters")]
	InvalidPrefix { prefix: String },

	#[error("At least one app must be specified with --app")]
	NoAppSpecified,

	#[error("Transitions are not implemented yet. Track progress at https://github.com/exaecut/create-video-effect")]
	TransitionNotImplemented,

	#[error("Template rendering failed: {0}")]
	Template(#[from] tera::Error),

	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("Workspace update failed: {0}")]
	Workspace(String),

	#[error("cargo check failed — project was created but may need fixes")]
	CargoCheckFailed,
}
