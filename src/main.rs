use sysinfo::{get_current_pid, ProcessExt, ProcessorExt, System, SystemExt};

trait Color {
    fn blue(&self) -> String;
}

impl Color for str {
    fn blue(&self) -> String {
        format!("\u{001b}[94m{}\u{001b}[0m", self)
    }
}

#[derive(Debug)]
struct Content {
    key: String,
    padding: usize,
    value: String,
}

impl Content {
    fn new(key: &str, value: &str) -> Self {
        Content {
            key: key.to_owned(),
            padding: 0,
            value: value.to_owned(),
        }
    }

    fn to_string(&self) -> String {
        format!(
            "{}: {}{}",
            self.key.blue(),
            " ".repeat(self.padding),
            self.value
        )
    }
}

fn get_logo() -> String {
    r#"
###############  ###############
###############  ###############
###############  ###############
###############  ###############
###############  ###############
###############  ###############
###############  ###############
                                
###############  ###############
###############  ###############
###############  ###############
###############  ###############
###############  ###############
###############  ###############
###############  ###############
    "#
    .trim()
    .to_owned()
}

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let user = format!("{}@{}", whoami::username(), whoami::hostname());
    let under_line = "-".repeat(user.len());

    let os = Content::new("OS", &whoami::distro());

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

        Content::new(
            "Uptime",
            &format!(
                "{} weeks {} days {} hours {} minutes",
                weeks, days, hours, minutes
            ),
        )
    };

    let shell = {
        // TODO: use anyhow and thiserror
        let current_process_pid = get_current_pid().unwrap();
        let current_process = sys.process(current_process_pid).unwrap();
        let parent_process_pid = current_process.parent().unwrap();
        let parent_process = sys.process(parent_process_pid).unwrap();
        Content::new(
            "Shell",
            match parent_process.name() {
                // "cmd.exe" => "CommandPrompt", // I won't use this.
                "powershell.exe" => "PowerShell",
                "nu.exe" => "Nu",
                _ => "Unkwon",
            },
        )
    };

    let cpu = Content::new("CPU", sys.global_processor_info().brand());

    let memory = {
        let total = sys.total_memory() as f64
            * 1000. / 1024. // In Windows, KB is actually often KiB
            / 1024. // MB
            / 1024. // GB
            ;

        let used = sys.used_memory() as f64
            * 1000. / 1024. // In Windows, KB is actually often KiB
            / 1024. // MB
            / 1024. // GB
            ;

        let usage_rate = (used / total * 100.).ceil() as u64;
        let total = total.ceil() as u64;
        let used = used.ceil() as u64;
        Content::new(
            "Memory",
            &format!("{}GB / {} GB ({}%)", used, total, usage_rate),
        )
    };

    let monitors = {
        let event_loop = winit::event_loop::EventLoop::new();
        let resolutions: Vec<String> = event_loop
            .available_monitors()
            .map(|x| {
                let physical_size = x.size();
                format!("{}x{}", physical_size.width, physical_size.height)
            })
            .collect();
        Content::new("Monitors", &resolutions.join(" "))
    };

    let mut info = Vec::new();

    info.push(user.blue());
    info.push(under_line);

    let mut contents = {
        let mut v = vec![os, uptime, shell, cpu, memory, monitors];
        let max_key_length = v.iter().map(|x| x.key.len()).max().unwrap();
        v.iter_mut().for_each(|x| {
            x.padding = max_key_length - x.key.len();
        });
        v.iter().map(|x| x.to_string()).collect()
    };
    info.append(&mut contents);

    let logo: Vec<String> = get_logo().lines().map(|x| x.blue()).collect();

    // assert!(logo.len() >= user.to_string().lines().count() + info.len());

    info.resize(logo.len(), String::default());

    logo.iter().zip(info.iter()).for_each(|(left, right)| {
        println!("    {}  {}", left, right);
    });
}
