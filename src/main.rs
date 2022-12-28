use std::collections::HashMap;
use sysinfo::{get_current_pid, CpuExt, ProcessExt, System, SystemExt};
use tiny_ansi::TinyAnsi;
use wmi::{COMLibrary, Variant, WMIConnection};

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
const LOGO_WIDTH: usize = 32;

fn create_label(s: &str) -> String {
    let s = format!("{s}:");
    format!("{s: <10}").bright_blue()
}

fn create_fake_label() -> String {
    let empty_str = "";
    format!("{empty_str: <10}")
}

fn main() {
    let mut sys = System::new_all();
    sys.refresh_all();

    let user = {
        let user_name = whoami::username().bright_blue();
        let host_name = whoami::hostname().bright_blue();
        format!("{user_name}@{host_name}")
    };
    let under_line = "-".repeat(user.len());

    let os = {
        let label = create_label("OS");
        let os = whoami::distro();
        format!("{label}{os}")
    };

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

        let label = create_label("Uptime");
        let uptime = format!("{weeks} weeks {days} days {hours} hours {minutes} minutes");
        format!("{label}{uptime}")
    };

    let shell = {
        let current_process_pid = get_current_pid().unwrap();
        let current_process = sys.process(current_process_pid).unwrap();
        let parent_process_pid = current_process.parent().unwrap();
        let parent_process = sys.process(parent_process_pid).unwrap();

        let label = create_label("Shell");
        let shell = match parent_process.name() {
            "cmd.exe" => "CommandPrompt",
            "powershell.exe" => "PowerShell",
            "nu.exe" => "Nu",
            _ => "Unkwon",
        };
        format!("{label}{shell}")
    };

    let mut cpu = {
        let label = create_label("CPU");
        let fake_label = create_fake_label();
        let cpu = sys.global_cpu_info().brand();
        let cores_and_threads = format!(
            "{} Cores {} Threads",
            num_cpus::get_physical(),
            num_cpus::get(),
        );
        vec![
            format!("{label}- {cpu}"),
            format!("{fake_label}- {cores_and_threads}"),
        ]
    };

    let mut gpu = {
        let com_con = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::new(com_con.into()).unwrap();
        let result: Vec<HashMap<String, Variant>> = wmi_con
            .raw_query("SELECT Caption FROM Win32_VideoController")
            .unwrap();
        let gpus = result.iter().map(|x| {
            let v = &*x.get("Caption").unwrap().to_owned();
            match v {
                Variant::String(info) => info.to_owned(),
                _ => "".to_owned(),
            }
        });
        gpus.enumerate()
            .map(|(i, gpu)| {
                if i == 0 {
                    let label = create_label("GPU");
                    format!("{label}- {gpu}")
                } else {
                    let fake_label = create_fake_label();
                    format!("{fake_label}- {gpu}")
                }
            })
            .collect::<Vec<String>>()
    };

    let memory = {
        let used = sys.used_memory() as f64
            / 1024f64 // KiB
            / 1024f64 // MiB
            / 1024f64 // GiB
            ;
        let total = sys.total_memory() as f64
            / 1024f64 // KiB
            / 1024f64 // MiB
            / 1024f64 // GiB
            ;
        let usage_rate = used / total * 100f64;

        let label = create_label("Memory");
        let memory = format!("{:.1}GB / {:.1} GB ({:.1}%)", used, total, usage_rate);

        // `Task Manager` apparently displays `GiB` values in `GB` units, so use `GB`.
        // NOTE: It seems that `systeminfo` also displays `MiB` values in `MB` units.
        format!("{label}{memory}")
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

        let label = create_label("Monitors");
        let monitors = resolutions.join(" ");
        format!("{label}{monitors}")
    };

    let mut right_content = Vec::new();
    right_content.push(user);
    right_content.push(under_line);
    right_content.push(os);
    right_content.push(uptime);
    right_content.push(shell);
    right_content.append(&mut cpu);
    right_content.append(&mut gpu);
    right_content.push(memory);
    right_content.push(monitors);

    let logo = get_logo();
    let mut left_content: Vec<String> = logo.lines().map(|x| x.to_owned().bright_blue()).collect();

    if left_content.len() >= right_content.len() {
        right_content.resize(left_content.len(), String::default());
    } else {
        left_content.resize(right_content.len(), String::default().repeat(LOGO_WIDTH));
    }
    left_content
        .iter()
        .zip(right_content.iter())
        .for_each(|(left, right)| {
            println!("    {left}  {right}");
        });
}
