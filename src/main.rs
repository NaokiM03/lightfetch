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
    format!("{s: <12}").bright_blue()
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

    let lightfetch_process_pid = get_current_pid().unwrap();
    let lightfetch_process = sys.process(lightfetch_process_pid).unwrap();
    let shell_process_pid = lightfetch_process.parent().unwrap();
    let shell_process = sys.process(shell_process_pid).unwrap();
    let terminal_process_pid = shell_process.parent().unwrap();
    let terminal_process = sys.process(terminal_process_pid).unwrap();

    let terminal = {
        let label = create_label("Terminal");
        let terminal = match terminal_process.name() {
            "explorer.exe" => "Win32 console",
            "WindowsTerminal.exe" => "Windows Terminal",
            "Code.exe" => "Visual Studio Code",
            _ => "Unkwon",
        };
        format!("{label}{terminal}")
    };

    let shell = {
        let label = create_label("Shell");
        let shell = match shell_process.name() {
            "cmd.exe" => "CommandPrompt",
            "powershell.exe" => "PowerShell",
            "nu.exe" => "Nu",
            _ => "Unkwon",
        };
        format!("{label}{shell}")
    };

    let cpu = {
        let label = create_label("CPU");
        let cpu = sys.global_cpu_info().brand();
        let cores_and_threads = format!("({}C/{}T)", num_cpus::get_physical(), num_cpus::get(),);
        format!("{label}{cpu} {cores_and_threads}")
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
                let label = create_label(&format!("GPU #{}", i));
                format!("{label}{gpu}")
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

    let resolution = {
        let event_loop = winit::event_loop::EventLoop::new();
        let resolution = event_loop
            .available_monitors()
            .map(|x| {
                let physical_size = x.size();
                format!("{}x{}", physical_size.width, physical_size.height)
            })
            .collect::<Vec<String>>()
            .join(" ");

        let label = create_label("Resolution");
        format!("{label}{resolution}")
    };

    let mut right_content = Vec::new();
    right_content.push(user);
    right_content.push(under_line);
    right_content.push(os);
    right_content.push(uptime);
    right_content.push(terminal);
    right_content.push(shell);
    right_content.push(cpu);
    right_content.append(&mut gpu);
    right_content.push(memory);
    right_content.push(resolution);

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
