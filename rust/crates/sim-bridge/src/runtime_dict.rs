use godot::builtin::GString;
use godot::prelude::VarDictionary;

pub(crate) fn dict_get_string(dict: &VarDictionary, key: &str) -> Option<String> {
    let value = dict.get(key)?;
    Some(value.to::<GString>().to_string())
}

pub(crate) fn dict_get_i32(dict: &VarDictionary, key: &str) -> Option<i32> {
    let value = dict.get(key)?;
    Some(value.to::<i64>() as i32)
}

pub(crate) fn dict_get_bool(dict: &VarDictionary, key: &str) -> Option<bool> {
    let value = dict.get(key)?;
    Some(value.to::<bool>())
}
