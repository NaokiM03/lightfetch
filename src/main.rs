use std::collections::HashMap;
use sysinfo::{get_current_pid, ProcessExt, ProcessorExt, System, SystemExt};
use wmi::{COMLibrary, Variant, WMIConnection};

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
                self.key.blue(),
                " ".repeat(self.padding),
                self.value[0]
            )]
        } else {
            let mut v = Vec::new();
            self.value.iter().enumerate().for_each(|(i, x)| {
                if i == 0 {
                    v.push(format!(
                        "{}: {}- {}",
                        self.key.blue(),
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
        whoami::username().blue(),
        whoami::hostname().blue()
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

    let cpu = Content::new("CPU", &vec![sys.global_processor_info().brand()]);

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
            &vec![format!("{}GB / {} GB ({}%)", used, total, usage_rate).as_str()],
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

    let logo: Vec<String> = logo.iter().map(|x| x.blue()).collect();

    logo.iter().zip(info.iter()).for_each(|(left, right)| {
        println!("    {}  {}", left, right);
    });
}
