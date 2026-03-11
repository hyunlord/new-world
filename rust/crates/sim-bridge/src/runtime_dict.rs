use godot::builtin::GString;
use godot::prelude::VarDictionary;

/// Reads a string value from a Godot dictionary key.
pub(crate) fn dict_get_string(dict: &VarDictionary, key: &str) -> Option<String> {
    let value = dict.get(key)?;
    Some(value.to::<GString>().to_string())
}
