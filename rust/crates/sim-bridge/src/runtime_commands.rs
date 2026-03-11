use godot::builtin::GString;
use godot::prelude::{Array, VarDictionary};

use crate::runtime_bindings::{is_supported_compute_mode, normalize_compute_mode_for_domain};
use crate::runtime_dict::dict_get_string;
use crate::runtime_registry::{clamp_speed_index, RuntimeState, RUNTIME_COMPUTE_DOMAINS};
use crate::runtime_system::RuntimeSystemId;

/// Returns the display label exported for one typed runtime system row.
fn registry_row_name(system_id: RuntimeSystemId) -> &'static str {
    system_id.display_label()
}

/// Exports the current typed runtime registry for debug and validation callers.
pub(crate) fn registry_snapshot(state: &RuntimeState) -> Array<VarDictionary> {
    let mut out: Array<VarDictionary> = Array::new();
    for entry in &state.registered_systems {
        let mut dict = VarDictionary::new();
        dict.set("name", registry_row_name(entry.system_id));
        dict.set("system_id", entry.system_id as i64);
        dict.set("priority", entry.priority);
        dict.set("tick_interval", entry.tick_interval);
        dict.set("active", entry.active);
        dict.set("registration_index", entry.registration_index);
        dict.set("rust_implemented", true);
        dict.set(
            "rust_registered",
            state.rust_registered_systems.contains(&entry.system_id),
        );
        dict.set("exec_backend", "rust");
        out.push(&dict);
    }
    out
}

/// Applies bridge-owned runtime control commands without mutating scheduler
/// identity or system registration.
pub(crate) fn apply_commands_v2(state: &mut RuntimeState, commands: Array<VarDictionary>) {
    for command in commands.iter_shared() {
        let Some(command_id_var) = command.get("command_id") else {
            continue;
        };
        let command_id = command_id_var.to::<GString>().to_string();
        if command_id == "set_speed_index" {
            let Some(payload_var) = command.get("payload") else {
                continue;
            };
            let payload = payload_var.to::<VarDictionary>();
            let Some(speed_var) = payload.get("speed_index") else {
                continue;
            };
            let speed = speed_var.to::<i64>() as i32;
            state.speed_index = clamp_speed_index(speed);
            continue;
        }
        if command_id == "reset_accumulator" {
            state.accumulator = 0.0;
            continue;
        }
        if command_id == "set_compute_domain_mode" {
            let Some(payload_var) = command.get("payload") else {
                continue;
            };
            let payload = payload_var.to::<VarDictionary>();
            let Some(domain) = dict_get_string(&payload, "domain") else {
                continue;
            };
            let Some(mode) = dict_get_string(&payload, "mode") else {
                continue;
            };
            if !is_supported_compute_mode(mode.as_str()) {
                continue;
            }
            if !RUNTIME_COMPUTE_DOMAINS.contains(&domain.as_str()) {
                continue;
            }
            let Some(normalized_mode) =
                normalize_compute_mode_for_domain(domain.as_str(), mode.as_str())
            else {
                continue;
            };
            state.compute_domain_modes.insert(domain, normalized_mode);
            continue;
        }
        if command_id == "set_compute_mode_all" {
            let Some(payload_var) = command.get("payload") else {
                continue;
            };
            let payload = payload_var.to::<VarDictionary>();
            let Some(mode) = dict_get_string(&payload, "mode") else {
                continue;
            };
            if !is_supported_compute_mode(mode.as_str()) {
                continue;
            }
            for domain in RUNTIME_COMPUTE_DOMAINS {
                let Some(normalized_mode) =
                    normalize_compute_mode_for_domain(domain, mode.as_str())
                else {
                    continue;
                };
                state
                    .compute_domain_modes
                    .insert(domain.to_string(), normalized_mode);
            }
            continue;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_row_name_uses_display_labels() {
        assert_eq!(registry_row_name(RuntimeSystemId::StatSync), "StatSync");
        assert_eq!(registry_row_name(RuntimeSystemId::Needs), "Needs");
    }
}
