use std::fs::canonicalize;

pub fn get_config_path() -> Option<String> {
    let exp_path = shellexpand::full("$HOME/.cache/clippy/db").ok()?;
    let can_path = canonicalize(exp_path.as_ref()).ok()?;
    can_path.into_os_string().into_string().ok()
}
