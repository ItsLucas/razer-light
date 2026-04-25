use clap::{Parser, Subcommand, ValueEnum};
use razer_core::{commands, device::Device, packet::Packet, types::*};

/// RHelper CLI — debug and control tool for Razer Blade laptops
#[derive(Parser)]
#[command(name = "rhelper-cli", version, about)]
struct Cli {
    /// USB Product ID override (hex, e.g. 0x029f). Auto-detects if omitted.
    #[arg(short, long, value_parser = parse_hex_u16)]
    pid: Option<u16>,

    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

fn parse_hex_u16(s: &str) -> Result<u16, String> {
    let s = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    u16::from_str_radix(s, 16).map_err(|e| format!("Invalid hex PID: {e}"))
}

#[derive(Subcommand)]
enum Commands {
    /// List all Razer USB HID devices
    Enumerate,

    /// Show full device state
    Info,

    /// Performance mode
    Perf {
        #[command(subcommand)]
        action: PerfAction,
    },

    /// Fan control
    Fan {
        #[command(subcommand)]
        action: FanAction,
    },

    /// Battery care (charge limit)
    Battery {
        #[command(subcommand)]
        action: BatteryAction,
    },

    /// Keyboard backlight brightness
    Kbd {
        #[command(subcommand)]
        action: KbdAction,
    },

    /// Lid logo control
    Logo {
        #[command(subcommand)]
        action: LogoAction,
    },

    /// Send a raw HID command (for debugging)
    Raw {
        /// Command as hex (e.g. 0d82)
        #[arg(value_parser = parse_hex_u16)]
        command: u16,
        /// Argument bytes as hex (e.g. "00 01 00 00")
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
enum PerfAction {
    /// Get current performance mode
    Get,
    /// Set performance mode
    Set {
        #[arg(value_enum)]
        mode: PerfModeArg,
    },
    /// Get/set CPU boost (custom mode only)
    CpuBoost {
        #[arg(value_enum)]
        level: Option<CpuBoostArg>,
    },
    /// Get/set GPU boost (custom mode only)
    GpuBoost {
        #[arg(value_enum)]
        level: Option<GpuBoostArg>,
    },
}

#[derive(Subcommand)]
enum FanAction {
    /// Get current fan RPMs
    Get,
    /// Set fan mode (auto/manual)
    Mode {
        #[arg(value_enum)]
        mode: FanModeArg,
    },
    /// Set fan RPM (both zones, requires manual mode)
    Set {
        /// RPM (0–5500, snapped to nearest 100)
        rpm: u16,
    },
    /// Enable/disable max fan speed
    MaxSpeed {
        #[arg(value_enum)]
        toggle: Toggle,
    },
}

#[derive(Subcommand)]
enum BatteryAction {
    /// Get current charge limit
    Get,
    /// Set charge limit
    Set {
        #[arg(value_enum)]
        level: BatteryCareArg,
    },
}

#[derive(Subcommand)]
enum KbdAction {
    /// Get keyboard brightness
    Get,
    /// Set keyboard brightness (0–255)
    Set { brightness: u8 },
}

#[derive(Subcommand)]
enum LogoAction {
    /// Get logo mode
    Get,
    /// Set logo mode
    Set {
        #[arg(value_enum)]
        mode: LogoModeArg,
    },
}

// ── ValueEnum wrappers (clap needs these) ────────────────────────────────────

#[derive(Clone, Copy, ValueEnum)]
enum PerfModeArg {
    Balanced,
    Performance,
    Custom,
    Silent,
    Battery,
    Hyperboost,
}

impl From<PerfModeArg> for PerfMode {
    fn from(a: PerfModeArg) -> Self {
        match a {
            PerfModeArg::Balanced => PerfMode::Balanced,
            PerfModeArg::Performance => PerfMode::Performance,
            PerfModeArg::Custom => PerfMode::Custom,
            PerfModeArg::Silent => PerfMode::Silent,
            PerfModeArg::Battery => PerfMode::Battery,
            PerfModeArg::Hyperboost => PerfMode::Hyperboost,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum CpuBoostArg {
    Low,
    Medium,
    High,
    Boost,
    Overclock,
}

impl From<CpuBoostArg> for CpuBoost {
    fn from(a: CpuBoostArg) -> Self {
        match a {
            CpuBoostArg::Low => CpuBoost::Low,
            CpuBoostArg::Medium => CpuBoost::Medium,
            CpuBoostArg::High => CpuBoost::High,
            CpuBoostArg::Boost => CpuBoost::Boost,
            CpuBoostArg::Overclock => CpuBoost::Overclock,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum GpuBoostArg {
    Low,
    Medium,
    High,
}

impl From<GpuBoostArg> for GpuBoost {
    fn from(a: GpuBoostArg) -> Self {
        match a {
            GpuBoostArg::Low => GpuBoost::Low,
            GpuBoostArg::Medium => GpuBoost::Medium,
            GpuBoostArg::High => GpuBoost::High,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum FanModeArg {
    Auto,
    Manual,
}

#[derive(Clone, Copy, ValueEnum)]
enum Toggle {
    Enable,
    Disable,
}

#[derive(Clone, Copy, ValueEnum)]
enum BatteryCareArg {
    #[value(name = "50")]
    Percent50,
    #[value(name = "55")]
    Percent55,
    #[value(name = "60")]
    Percent60,
    #[value(name = "65")]
    Percent65,
    #[value(name = "70")]
    Percent70,
    #[value(name = "75")]
    Percent75,
    #[value(name = "80")]
    Percent80,
    #[value(name = "off")]
    Disabled,
}

impl From<BatteryCareArg> for BatteryCare {
    fn from(a: BatteryCareArg) -> Self {
        match a {
            BatteryCareArg::Percent50 => BatteryCare::Percent50,
            BatteryCareArg::Percent55 => BatteryCare::Percent55,
            BatteryCareArg::Percent60 => BatteryCare::Percent60,
            BatteryCareArg::Percent65 => BatteryCare::Percent65,
            BatteryCareArg::Percent70 => BatteryCare::Percent70,
            BatteryCareArg::Percent75 => BatteryCare::Percent75,
            BatteryCareArg::Percent80 => BatteryCare::Percent80,
            BatteryCareArg::Disabled => BatteryCare::Disabled,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum LogoModeArg {
    Off,
    Static,
    Breathing,
}

impl From<LogoModeArg> for LogoMode {
    fn from(a: LogoModeArg) -> Self {
        match a {
            LogoModeArg::Off => LogoMode::Off,
            LogoModeArg::Static => LogoMode::Static,
            LogoModeArg::Breathing => LogoMode::Breathing,
        }
    }
}

// ── Main ─────────────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    match cli.command {
        Commands::Enumerate => cmd_enumerate(),
        _ => {
            let device = open_device(cli.pid)?;
            match cli.command {
                Commands::Info => cmd_info(&device),
                Commands::Perf { action } => cmd_perf(&device, action),
                Commands::Fan { action } => cmd_fan(&device, action),
                Commands::Battery { action } => cmd_battery(&device, action),
                Commands::Kbd { action } => cmd_kbd(&device, action),
                Commands::Logo { action } => cmd_logo(&device, action),
                Commands::Raw { command, args } => cmd_raw(&device, command, args),
                Commands::Enumerate => unreachable!(),
            }
        }
    }
}

fn open_device(pid_override: Option<u16>) -> anyhow::Result<Device> {
    match pid_override {
        Some(pid) => {
            println!("Opening device with PID 0x{pid:04X}...");
            Device::open_by_pid(pid, None)
        }
        None => {
            println!("Auto-detecting Razer device...");
            Device::auto_open()
        }
    }
}

// ── Command Handlers ─────────────────────────────────────────────────────────

fn cmd_enumerate() -> anyhow::Result<()> {
    let devices = Device::enumerate()?;
    println!("Found {} Razer HID interface(s):\n", devices.len());
    let mut seen_pids = std::collections::HashSet::new();
    for d in &devices {
        let new_pid = seen_pids.insert(d.pid);
        println!(
            "  PID=0x{:04X} iface={:<2} usage_page=0x{:04X} usage=0x{:04X} {}{}",
            d.pid,
            d.interface_number,
            d.usage_page,
            d.usage,
            if d.product.is_empty() {
                "?"
            } else {
                &d.product
            },
            if new_pid { "" } else { " (dup)" },
        );
    }

    // Show unique PIDs
    let unique: Vec<_> = seen_pids.into_iter().collect();
    println!(
        "\nUnique PIDs: {}",
        unique
            .iter()
            .map(|p| format!("0x{p:04X}"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    Ok(())
}

fn cmd_info(device: &Device) -> anyhow::Result<()> {
    println!("Device: {} (PID 0x{:04X})", device.name(), device.pid());
    println!("Features: {:?}\n", device.descriptor.features);

    let state = commands::read_all_state(device)?;

    println!("  Performance Mode : {:?}", state.perf_mode);
    println!("  Fan Mode         : {:?}", state.fan_mode);
    println!("  CPU Fan (actual) : {} RPM", state.fan1_rpm);
    println!("  GPU Fan (actual) : {} RPM", state.fan2_rpm);
    if let Some(cpu) = state.cpu_boost {
        println!("  CPU Boost        : {:?}", cpu);
    }
    if let Some(gpu) = state.gpu_boost {
        println!("  GPU Boost        : {:?}", gpu);
    }
    if let Some(bat) = state.battery_care {
        println!("  Battery Care     : {} (0x{:02X})", bat.label(), bat as u8);
    }
    println!(
        "  Kbd Brightness   : {} ({}%)",
        state.kbd_brightness,
        state.kbd_brightness as u32 * 100 / 255
    );
    if let Some(logo) = state.logo_mode {
        println!("  Logo Mode        : {:?}", logo);
    }
    Ok(())
}

fn cmd_perf(device: &Device, action: PerfAction) -> anyhow::Result<()> {
    match action {
        PerfAction::Get => {
            let (mode, fan) = commands::get_perf_mode(device)?;
            println!("Performance: {:?}, Fan: {:?}", mode, fan);
        }
        PerfAction::Set { mode } => {
            let mode: PerfMode = mode.into();
            commands::set_perf_mode(device, mode, FanMode::Auto)?;
            println!("Set performance mode to {:?}", mode);
        }
        PerfAction::CpuBoost { level } => match level {
            Some(l) => {
                let boost: CpuBoost = l.into();
                commands::set_cpu_boost(device, boost)?;
                println!("Set CPU boost to {:?}", boost);
            }
            None => {
                let boost = commands::get_cpu_boost(device)?;
                println!("CPU Boost: {:?}", boost);
            }
        },
        PerfAction::GpuBoost { level } => match level {
            Some(l) => {
                let boost: GpuBoost = l.into();
                commands::set_gpu_boost(device, boost)?;
                println!("Set GPU boost to {:?}", boost);
            }
            None => {
                let boost = commands::get_gpu_boost(device)?;
                println!("GPU Boost: {:?}", boost);
            }
        },
    }
    Ok(())
}

fn cmd_fan(device: &Device, action: FanAction) -> anyhow::Result<()> {
    match action {
        FanAction::Get => {
            let fan1 = commands::get_fan_actual_rpm(device, FanZone::Zone1)?;
            let fan2 = commands::get_fan_actual_rpm(device, FanZone::Zone2)?;
            let (_, fan_mode) = commands::get_perf_mode(device)?;
            println!("Fan Mode: {:?}", fan_mode);
            println!("CPU Fan: {} RPM", fan1);
            println!("GPU Fan: {} RPM", fan2);
            if fan_mode == FanMode::Manual {
                let t1 = commands::get_fan_rpm(device, FanZone::Zone1)?;
                let t2 = commands::get_fan_rpm(device, FanZone::Zone2)?;
                println!("Target CPU: {} RPM, GPU: {} RPM", t1, t2);
            }
        }
        FanAction::Mode { mode } => {
            let (perf, _) = commands::get_perf_mode(device)?;
            let fm = match mode {
                FanModeArg::Auto => FanMode::Auto,
                FanModeArg::Manual => FanMode::Manual,
            };
            commands::set_perf_mode(device, perf, fm)?;
            println!("Set fan mode to {:?}", fm);
        }
        FanAction::Set { rpm } => {
            let rpm = (rpm / 100) * 100; // snap
            commands::set_fan_rpm(device, rpm)?;
            println!("Set fan RPM to {}", rpm);
        }
        FanAction::MaxSpeed { toggle } => {
            let enabled = matches!(toggle, Toggle::Enable);
            commands::set_max_fan_speed(device, enabled)?;
            println!(
                "Max fan speed: {}",
                if enabled { "enabled" } else { "disabled" }
            );
        }
    }
    Ok(())
}

fn cmd_battery(device: &Device, action: BatteryAction) -> anyhow::Result<()> {
    match action {
        BatteryAction::Get => {
            let care = commands::get_battery_care(device)?;
            println!("Battery care: {} (0x{:02X})", care.label(), care as u8);
        }
        BatteryAction::Set { level } => {
            let level: BatteryCare = level.into();
            commands::set_battery_care(device, level)?;
            println!("Set battery care to {}", level.label());
        }
    }
    Ok(())
}

fn cmd_kbd(device: &Device, action: KbdAction) -> anyhow::Result<()> {
    match action {
        KbdAction::Get => {
            let b = commands::get_keyboard_brightness(device)?;
            println!("Keyboard brightness: {} ({}%)", b, b as u32 * 100 / 255);
        }
        KbdAction::Set { brightness } => {
            commands::set_keyboard_brightness(device, brightness)?;
            println!("Set keyboard brightness to {}", brightness);
        }
    }
    Ok(())
}

fn cmd_logo(device: &Device, action: LogoAction) -> anyhow::Result<()> {
    match action {
        LogoAction::Get => {
            let mode = commands::get_logo_mode(device)?;
            println!("Logo mode: {:?}", mode);
        }
        LogoAction::Set { mode } => {
            let mode: LogoMode = mode.into();
            commands::set_logo_mode(device, mode)?;
            println!("Set logo mode to {:?}", mode);
        }
    }
    Ok(())
}

fn cmd_raw(device: &Device, command: u16, args: Vec<String>) -> anyhow::Result<()> {
    let arg_bytes: Vec<u8> = args
        .iter()
        .flat_map(|s| s.split_whitespace())
        .map(|s| {
            let s = s
                .strip_prefix("0x")
                .or_else(|| s.strip_prefix("0X"))
                .unwrap_or(s);
            u8::from_str_radix(s, 16)
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!("Invalid hex arg byte: {e}"))?;

    println!(
        "Sending cmd=0x{command:04X} args=[{}]",
        arg_bytes
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<Vec<_>>()
            .join(" ")
    );

    let request = Packet::new(command, &arg_bytes);
    println!("TX: {request}");

    let response = device.send(request)?;
    println!("RX: {response}");

    // Also dump raw response args for debugging
    let resp_args = response.get_args();
    if !resp_args.is_empty() {
        println!("    args[0..{}] = {:02x?}", resp_args.len(), resp_args);
    }

    Ok(())
}
