use std::collections::HashMap;

const GPU_ENABLED_COMPUTE_DOMAINS: [&str; 1] = ["pathfinding"];

fn domain_supports_gpu(domain: &str) -> bool {
    GPU_ENABLED_COMPUTE_DOMAINS.contains(&domain)
}

pub(crate) fn runtime_default_compute_domain_modes(domains: &[&str]) -> HashMap<String, String> {
    let mut modes = HashMap::<String, String>::new();
    for domain in domains {
        let default_mode = if domain_supports_gpu(domain) {
            "gpu_auto"
        } else {
            "cpu"
        };
        modes.insert((*domain).to_string(), default_mode.to_string());
    }
    modes
}

pub(crate) fn is_supported_compute_mode(mode: &str) -> bool {
    matches!(mode, "cpu" | "gpu_auto" | "gpu_force")
}

pub(crate) fn normalize_compute_mode_for_domain(domain: &str, mode: &str) -> Option<String> {
    if !is_supported_compute_mode(mode) {
        return None;
    }
    if domain_supports_gpu(domain) {
        return Some(mode.to_string());
    }
    Some("cpu".to_string())
}
