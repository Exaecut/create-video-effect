use crate::error::GenerateError;

/// Validate a Rust crate name per crates.io rules.
///
/// Rules:
/// - Must be non-empty
/// - Must start with an ASCII lowercase letter
/// - Only ASCII lowercase letters, digits, and underscores
/// - No consecutive underscores
/// - No leading or trailing underscores
pub fn validate_crate_name(name: &str) -> Result<(), GenerateError> {
	if name.is_empty() {
		return Err(GenerateError::InvalidCrateName {
			name: name.to_string(),
			reason: "crate name must not be empty".to_string(),
		});
	}

	let mut chars = name.chars().peekable();
	let first = chars.next().unwrap();

	if !first.is_ascii_lowercase() {
		return Err(GenerateError::InvalidCrateName {
			name: name.to_string(),
			reason: format!("crate name must start with an ASCII lowercase letter, found '{first}'"),
		});
	}

	if name.starts_with('_') {
		return Err(GenerateError::InvalidCrateName {
			name: name.to_string(),
			reason: "crate name must not start with an underscore".to_string(),
		});
	}

	if name.ends_with('_') {
		return Err(GenerateError::InvalidCrateName {
			name: name.to_string(),
			reason: "crate name must not end with an underscore".to_string(),
		});
	}

	let mut prev_underscore = false;
	for ch in name.chars() {
		if ch == '_' {
			if prev_underscore {
				return Err(GenerateError::InvalidCrateName {
					name: name.to_string(),
					reason: "crate name must not contain consecutive underscores".to_string(),
				});
			}
			prev_underscore = true;
		} else {
			prev_underscore = false;
		}

		if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && ch != '_' {
			return Err(GenerateError::InvalidCrateName {
				name: name.to_string(),
				reason: format!(
					"crate name must only contain ASCII lowercase letters, digits, and underscores, found '{ch}'"
				),
			});
		}
	}

	Ok(())
}

/// Convert a crate name to a display name.
///
/// `super_bloom` → `"Super Bloom"`
pub fn derive_display_name(crate_name: &str) -> String {
	crate_name
		.split('_')
		.map(|word| {
			let mut chars = word.chars();
			match chars.next() {
				None => String::new(),
				Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
			}
		})
		.collect::<Vec<_>>()
		.join(" ")
}

/// Derive PIPL match name and effect name from prefix + display name.
///
/// No prefix: match_name = `"Super Bloom"`, effect_name = `"Super Bloom"`
/// Prefix `"ADBE"`: match_name = `"ADBE Super Bloom"`, effect_name = `"AD Super Bloom"`
pub fn derive_pipl_names(prefix: &Option<String>, display_name: &str) -> (String, String) {
	match prefix {
		None => (display_name.to_string(), display_name.to_string()),
		Some(p) => (
			format!("{p} {display_name}"),
			format!("{} {display_name}", &p[..2]),
		),
	}
}

/// Validate the prefix flag — must be 2-6 uppercase ASCII letters.
pub fn validate_prefix(prefix: &str) -> Result<(), GenerateError> {
	if prefix.len() < 2 || prefix.len() > 6 {
		return Err(GenerateError::InvalidPrefix {
			prefix: prefix.to_string(),
		});
	}
	if !prefix.chars().all(|c| c.is_ascii_uppercase()) {
		return Err(GenerateError::InvalidPrefix {
			prefix: prefix.to_string(),
		});
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn valid_crate_names() {
		assert!(validate_crate_name("vignette").is_ok());
		assert!(validate_crate_name("super_bloom").is_ok());
		assert!(validate_crate_name("a").is_ok());
		assert!(validate_crate_name("my_effect_2").is_ok());
	}

	#[test]
	fn invalid_crate_names() {
		assert!(validate_crate_name("").is_err());
		assert!(validate_crate_name("_leading").is_err());
		assert!(validate_crate_name("trailing_").is_err());
		assert!(validate_crate_name("double__underscore").is_err());
		assert!(validate_crate_name("1starts_with_digit").is_err());
		assert!(validate_crate_name("has-UPPER").is_err());
		assert!(validate_crate_name("has-dash").is_err());
	}

	#[test]
	fn display_name_derivation() {
		assert_eq!(derive_display_name("vignette"), "Vignette");
		assert_eq!(derive_display_name("super_bloom"), "Super Bloom");
		assert_eq!(derive_display_name("radial_blur"), "Radial Blur");
	}

	#[test]
	fn pipl_names_no_prefix() {
		let (match_name, effect_name) = derive_pipl_names(&None, "Super Bloom");
		assert_eq!(match_name, "Super Bloom");
		assert_eq!(effect_name, "Super Bloom");
	}

	#[test]
	fn pipl_names_with_prefix() {
		let (match_name, effect_name) = derive_pipl_names(&Some("ADBE".to_string()), "Super Bloom");
		assert_eq!(match_name, "ADBE Super Bloom");
		assert_eq!(effect_name, "AD Super Bloom");
	}

	#[test]
	fn prefix_validation() {
		assert!(validate_prefix("AB").is_ok());
		assert!(validate_prefix("ADBE").is_ok());
		assert!(validate_prefix("ABCDEF").is_ok());
		assert!(validate_prefix("A").is_err());
		assert!(validate_prefix("ABCDEFG").is_err());
		assert!(validate_prefix("ab").is_err());
		assert!(validate_prefix("AB1").is_err());
	}
}
