use sysinfo::{get_current_pid, ProcessExt, System, SystemExt};

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let user = format!("{}@{}", whoami::username(), whoami::hostname());
    let under_line = "-".repeat(user.len());

    let os = format!("OS: {}", whoami::distro());

    let uptime = {
        const SECONDS_PER_MINUTE: u64 = 60;
        const SECONDS_PER_HOUR: u64 = 3600;
        const SECONDS_PER_DAY: u64 = 86400;
        const SECONDS_PER_WEEK: u64 = 604800;

        let uptime = sys.uptime();
        let weeks = uptime / SECONDS_PER_WEEK;
        let remaining = uptime - weeks * SECONDS_PER_WEEK;
        let days = remaining / SECONDS_PER_DAY;
        let remaining = remaining - days * SECONDS_PER_DAY;
        let hours = remaining / SECONDS_PER_HOUR;
        let remaining = remaining - hours * SECONDS_PER_HOUR;
        let minutes = remaining / SECONDS_PER_MINUTE;

        format!(
            "Uptime: {} weeks {} days {} hours {} minutes",
            weeks, days, hours, minutes
        )
    };

    let shell = {
        // TODO: use anyhow and thiserror
        let current_process_pid = get_current_pid().unwrap();
        let current_process = sys.process(current_process_pid).unwrap();
        let parent_process_pid = current_process.parent().unwrap();
        let parent_process = sys.process(parent_process_pid).unwrap();
        let shell = match parent_process.name() {
            // "cmd.exe" => "CommandPrompt", // I won't use this.
            "powershell.exe" => "PowerShell",
            "nu.exe" => "Nu",
            _ => "Unkwon",
        };
        format!("Shell: {}", shell)
    };

    let info = format!(
        r#"
██████████████  ██████████████  {}
██████████████  ██████████████  {}
██████████████  ██████████████  {}
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
"#,
        &user, &under_line, &os, &uptime, &shell
    );
    println!("{info}");
}
