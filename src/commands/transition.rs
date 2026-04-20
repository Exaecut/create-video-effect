use crate::error::GenerateError;
use crate::generator::{GenerateContext, Generator};
use crate::cli::ResolvedArgs;

/// Transition generator stub - not implemented yet.
#[allow(dead_code)]
pub struct TransitionGenerator;

impl Generator for TransitionGenerator {
	fn validate(_args: &ResolvedArgs) -> Result<(), GenerateError> {
		Err(GenerateError::TransitionNotImplemented)
	}

	fn generate(_args: &ResolvedArgs, _ctx: &GenerateContext) -> Result<(), GenerateError> {
		Err(GenerateError::TransitionNotImplemented)
	}

	fn post_generate(_args: &ResolvedArgs, _ctx: &GenerateContext) -> Result<(), GenerateError> {
		Err(GenerateError::TransitionNotImplemented)
	}
}
