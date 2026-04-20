use std::path::{Path, PathBuf};

use crate::error::GenerateError;

/// Walk parent directories (up to 10 levels) looking for a Cargo.toml with `[workspace]`.
pub fn detect_workspace(output_dir: &Path) -> Option<PathBuf> {
    let mut dir = output_dir;
    for _ in 0..10 {
        if let Some(parent) = dir.parent() {
            let cargo_toml = parent.join("Cargo.toml");
            if cargo_toml.exists() && is_workspace(&cargo_toml) {
                return Some(cargo_toml);
            }
            dir = parent;
        } else {
            break;
        }
    }
    None
}

/// Check if a Cargo.toml contains a `[workspace]` section.
fn is_workspace(path: &Path) -> bool {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return false,
    };
    content.contains("[workspace]")
}

/// Add a crate name to the workspace members array.
///
/// If the crate name is already a member, this is a no-op.
pub fn add_workspace_member(workspace_toml: &Path, crate_name: &str) -> Result<(), GenerateError> {
    let content = std::fs::read_to_string(workspace_toml).map_err(|e| {
        GenerateError::Workspace(format!("Failed to read workspace Cargo.toml: {e}"))
    })?;

    let mut doc = content.parse::<toml_edit::DocumentMut>().map_err(|e| {
        GenerateError::Workspace(format!("Failed to parse workspace Cargo.toml: {e}"))
    })?;

    let workspace = doc.entry("workspace").or_insert_with(|| {
        let mut table = toml_edit::Table::new();
        table.set_implicit(true);
        toml_edit::Item::Table(table)
    });

    let members = workspace
        .as_table_mut()
        .ok_or_else(|| GenerateError::Workspace("workspace is not a table".to_string()))?
        .entry("members")
        .or_insert_with(|| {
            toml_edit::Item::Value(toml_edit::Value::Array(toml_edit::Array::new()))
        });

    let arr = members
        .as_value_mut()
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| GenerateError::Workspace("workspace.members is not an array".to_string()))?;

    // Check if already a member
    for item in arr.iter() {
        if let Some(s) = item.as_str()
            && s == crate_name
        {
            return Ok(()); // Already a member
        }
    }

    // Add the new member
    arr.push(crate_name);

    std::fs::write(workspace_toml, doc.to_string()).map_err(|e| {
        GenerateError::Workspace(format!("Failed to write workspace Cargo.toml: {e}"))
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn detect_workspace_finds_parent() {
        let base = std::env::temp_dir().join("cve_test_detect");
        let _ = fs::remove_dir_all(&base);
        let sub_dir = base.join("my_effect");
        fs::create_dir_all(&sub_dir).unwrap();

        let cargo_toml = base.join("Cargo.toml");
        fs::write(&cargo_toml, "[workspace]\nmembers = []\n").unwrap();

        assert_eq!(detect_workspace(&sub_dir), Some(cargo_toml));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn add_member_to_workspace() {
        let base = std::env::temp_dir().join("cve_test_add_member");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();

        let cargo_toml = base.join("Cargo.toml");
        fs::write(&cargo_toml, "[workspace]\nmembers = [\"existing\"]\n").unwrap();

        add_workspace_member(&cargo_toml, "new_effect").unwrap();

        let content = fs::read_to_string(&cargo_toml).unwrap();
        assert!(content.contains("new_effect"));
        assert!(content.contains("existing"));
        let _ = fs::remove_dir_all(&base);
    }
}
