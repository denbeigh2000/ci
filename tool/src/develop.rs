use std::process::Command;

const FALSE_VALS: [&str; 5] = ["", "false", "0", "no", "off"];
lazy_static::lazy_static! {
    pub static ref IS_DEVELOP_MODE: bool = std::env::var("DEVELOP_MODE")
    .as_ref()
    .is_ok_and(|val| !FALSE_VALS.contains(&val.as_str()));
}

pub fn print_cmd(name: &'static str, cmd: &Command) {
    let args = cmd
        .get_args()
        .map(|v| v.to_string_lossy().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let envs = cmd
        .get_envs()
        .filter_map(|(k, v)| v.map(|val| (k.to_string_lossy(), val.to_string_lossy())))
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join(" ");
    let pwd = cmd
        .get_current_dir()
        .take()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());
    eprintln!("\n---");
    eprintln!("Would run: {name} {args}");
    eprintln!("envs:\n{envs}");
    eprintln!("pwd: {}", pwd.display());
}
