use std::{fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let action_schema_path = PathBuf::from("schemas/actions.schema.json");
    let keymap_schema_path = PathBuf::from("schemas/keymap.schema.json");

    println!("Updating keymap schema...");

    let actions_source = fs::read_to_string(&action_schema_path)?;
    let actions_schema: serde_json::Value = serde_json::from_str(&actions_source)?;

    let keymap_source = fs::read_to_string(&keymap_schema_path)?;
    let mut keymap_schema: serde_json::Value = serde_json::from_str(&keymap_source)?;

    let count = update_actions(&mut keymap_schema, &actions_schema)?;

    let formatted = serde_json::to_string_pretty(&keymap_schema)?;
    fs::write(&keymap_schema_path, formatted + "\n")?;

    println!(
        "Updated {count} actions in {}",
        keymap_schema_path.display()
    );

    Ok(())
}

fn update_actions(
    keymap_schema: &mut serde_json::Value,
    actions_schema: &serde_json::Value,
) -> Result<usize, Box<dyn std::error::Error>> {
    let actions = actions_schema
        .pointer("/$defs/ActionName/anyOf")
        .and_then(serde_json::Value::as_array)
        .ok_or("actions schema is missing $defs.ActionName.anyOf")?;
    let action_key_enum = keymap_schema
        .pointer_mut("/$defs/actionKeyEnum")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("keymap schema is missing $defs.actionKeyEnum")?;
    let actions = actions
        .iter()
        .filter(|action| action.get("const").is_some())
        .cloned()
        .collect::<Vec<_>>();
    let count = actions.len();
    action_key_enum.remove("enum");
    action_key_enum.insert("oneOf".into(), serde_json::Value::Array(actions));
    Ok(count)
}
fn format_action_description(action_name: &str, description: &str) -> String {
    let combined_length = action_name.chars().count() + 3 + description.chars().count();

    if combined_length > MAX_DESCRIPTION_LINE_LENGTH {
        format!("{action_name} +\n{description}")
    } else {
        format!("{action_name} + {description}")
    }
}

const MAX_DESCRIPTION_LINE_LENGTH: usize = 100;
