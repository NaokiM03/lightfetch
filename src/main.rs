fn main() {
    let user = format!("{}@{}", whoami::username(), whoami::hostname());

    let under_line = "-".repeat(user.len());

    let info = format!(
        r#"
██████████████  ██████████████  {}
██████████████  ██████████████  {}
██████████████  ██████████████
██████████████  ██████████████
██████████████  ██████████████
██████████████  ██████████████
██████████████  ██████████████

██████████████  ██████████████
██████████████  ██████████████
██████████████  ██████████████
██████████████  ██████████████
██████████████  ██████████████
██████████████  ██████████████
██████████████  ██████████████
"#,
        &user, &under_line,
    );
    println!("{info}");
}
