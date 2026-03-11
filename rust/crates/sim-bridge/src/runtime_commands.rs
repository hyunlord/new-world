use godot::builtin::GString;
use godot::prelude::{Array, VarDictionary};

use crate::runtime_bindings::{is_supported_compute_mode, normalize_compute_mode_for_domain};
use crate::runtime_dict::{dict_get_bool, dict_get_i32, dict_get_string};
use crate::runtime_registry::{
    clamp_speed_index, runtime_system_key_from_name, upsert_runtime_system_entry, RuntimeState,
    RUNTIME_COMPUTE_DOMAINS,
};

pub(crate) fn registry_snapshot(state: &RuntimeState) -> Array<VarDictionary> {
    let mut out: Array<VarDictionary> = Array::new();
    for entry in &state.registered_systems {
        let mut dict = VarDictionary::new();
        dict.set("name", entry.name.clone());
        dict.set("system_key", entry.system_key.clone());
        dict.set("priority", entry.priority);
        dict.set("tick_interval", entry.tick_interval);
        dict.set("active", entry.active);
        dict.set("registration_index", entry.registration_index);
        dict.set("rust_implemented", entry.rust_implemented);
        dict.set("rust_registered", entry.rust_registered);
        dict.set("exec_backend", entry.exec_backend.clone());
        out.push(&dict);
    }
    out
}

pub(crate) fn clear_registry(state: &mut RuntimeState) {
    state.registered_systems.clear();
    state.rust_registered_systems.clear();
    state.engine.clear_systems();
}

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
        if command_id == "clear_registry" {
            clear_registry(state);
            continue;
        }
        if command_id == "register_system" {
            let Some(payload_var) = command.get("payload") else {
                continue;
            };
            let payload = payload_var.to::<VarDictionary>();
            let Some(name) = dict_get_string(&payload, "name") else {
                continue;
            };
            let priority = dict_get_i32(&payload, "priority").unwrap_or(100);
            let tick_interval = dict_get_i32(&payload, "tick_interval").unwrap_or(1);
            let active = dict_get_bool(&payload, "active").unwrap_or(true);
            let registration_index =
                dict_get_i32(&payload, "registration_index").unwrap_or(i32::MAX);
            let system_key = runtime_system_key_from_name(&name);
            upsert_runtime_system_entry(
                state,
                &name,
                system_key.as_str(),
                priority,
                tick_interval,
                active,
                registration_index,
            );
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
