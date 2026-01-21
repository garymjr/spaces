use std::io::{self, Write};

pub fn log_info(msg: &str) {
    eprintln!("[OK] {msg}");
}

pub fn log_warn(msg: &str) {
    eprintln!("[!] {msg}");
}

pub fn log_error(msg: &str) {
    eprintln!("[x] {msg}");
}

pub fn log_step(msg: &str) {
    eprintln!("==> {msg}");
}

pub fn prompt_input(prompt: &str) -> io::Result<String> {
    eprint!("[?] {prompt} ");
    io::stderr().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

pub fn prompt_yes_no(prompt: &str, default_yes: bool) -> io::Result<bool> {
    let suffix = if default_yes { "[Y/n]" } else { "[y/N]" };
    eprint!("[?] {prompt} {suffix} ");
    io::stderr().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let reply = input.trim();
    if reply.is_empty() {
        return Ok(default_yes);
    }
    let yes = matches!(reply, "y" | "Y" | "yes" | "YES" | "Yes");
    Ok(yes)
}
