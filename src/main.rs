use std::collections::HashMap;
use sysinfo::{get_current_pid, CpuExt, ProcessExt, System, SystemExt};
use wmi::{COMLibrary, Variant, WMIConnection};
use tiny_ansi::TinyAnsi;

#[derive(Debug)]
struct Content {
    key: String,
    padding: usize,
    value: Vec<String>,
}

impl Content {
    fn new(key: &str, value: &[&str]) -> Self {
        Content {
            key: key.to_owned(),
            padding: 0,
            value: value.iter().map(|x| x.to_string()).collect(),
        }
    }

    fn build(&self) -> Vec<String> {
        if self.value.len() == 1 {
            vec![format!(
                "{}: {}{}",
                self.key.bright_blue(),
                " ".repeat(self.padding),
                self.value[0]
            )]
        } else {
            let mut v = Vec::new();
            self.value.iter().enumerate().for_each(|(i, x)| {
                if i == 0 {
                    v.push(format!(
                        "{}: {}- {}",
                        self.key.bright_blue(),
                        " ".repeat(self.padding),
                        self.value[0]
                    ));
                } else {
                    v.push(format!(
                        "{}  - {}",
                        " ".repeat(self.key.len() + self.padding),
                        x
                    ));
                }
            });
            v
        }
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

    let user = format!(
        "{}@{}",
        whoami::username().bright_blue(),
        whoami::hostname().bright_blue()
    );
    let under_line = "-".repeat(user.len());

    let os = Content::new("OS", &vec![whoami::distro().as_str()]);

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
            &vec![format!(
                "{} weeks {} days {} hours {} minutes",
                weeks, days, hours, minutes
            )
            .as_str()],
        )
    };

    let shell = {
        let current_process_pid = get_current_pid().unwrap();
        let current_process = sys.process(current_process_pid).unwrap();
        let parent_process_pid = current_process.parent().unwrap();
        let parent_process = sys.process(parent_process_pid).unwrap();
        Content::new(
            "Shell",
            &vec![match parent_process.name() {
                "cmd.exe" => "CommandPrompt",
                "powershell.exe" => "PowerShell",
                "nu.exe" => "Nu",
                _ => "Unkwon",
            }],
        )
    };

    let cpu = {
        let cpu = sys.global_cpu_info().brand();
        let cores_and_threads = &format!(
            "{} Cores {} Threads",
            num_cpus::get_physical(),
            num_cpus::get(),
        );
        Content::new("CPU", &vec![cpu, cores_and_threads])
    };

    let gpu = {
        let com_con = COMLibrary::new().unwrap();
        let wmi_con = WMIConnection::new(com_con.into()).unwrap();
        let result: Vec<HashMap<String, Variant>> = wmi_con
            .raw_query("SELECT Caption FROM Win32_VideoController")
            .unwrap();
        let v = result
            .iter()
            .map(|x| {
                let v = &*x.get("Caption").unwrap().to_owned();
                match v {
                    Variant::String(info) => info.as_str(),
                    _ => "",
                }
            })
            .collect::<Vec<&str>>();
        Content::new("GPU", &v)
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

        // `Task Manager` apparently displays `GiB` values in `GB` units, so use `GB`.
        // NOTE: It seems that `systeminfo` also displays `MiB` values in `MB` units.
        Content::new(
            "Memory",
            &vec![format!("{:.1}GB / {:.1} GB ({:.1}%)", used, total, usage_rate).as_str()],
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
        Content::new("Monitors", &vec![resolutions.join(" ").as_str()])
    };

    let mut info = Vec::new();

    info.push(user);
    info.push(under_line);

    let mut contents = {
        let mut v = vec![os, uptime, shell, cpu, gpu, memory, monitors];
        let max_key_length = v.iter().map(|x| x.key.len()).max().unwrap();
        v.iter_mut().for_each(|x| {
            x.padding = max_key_length - x.key.len();
        });
        v.iter().map(|x| x.build()).flatten().collect()
    };
    info.append(&mut contents);

    let mut logo: Vec<String> = get_logo().lines().map(|x| x.to_owned()).collect();

    if logo.len() > info.len() {
        info.resize(logo.len(), String::default());
    } else if logo.len() < info.len() {
        logo.resize(
            info.len(),
            " ".repeat(logo.iter().map(|x| x.len()).max().unwrap()),
        );
    }

    let logo: Vec<String> = logo.iter().map(|x| x.bright_blue()).collect();

    logo.iter().zip(info.iter()).for_each(|(left, right)| {
        println!("    {}  {}", left, right);
    });
}
