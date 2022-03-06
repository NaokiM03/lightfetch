use sysinfo::{System, SystemExt};

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

    let info = format!(
        r#"
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
██████████████  ██████████████
"#,
        &user, &under_line, &os, &uptime
    );
    println!("{info}");
}
