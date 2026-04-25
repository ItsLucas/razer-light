use anyhow::Result;

/// Static descriptor for a known Razer Blade model.
#[derive(Debug, Clone)]
pub struct DeviceDescriptor {
    /// First 10 chars of SystemSKU (e.g. "RZ09-0483T")
    pub sku_prefix: &'static str,
    /// Human-readable name
    pub name: &'static str,
    /// USB Product ID
    pub pid: u16,
    /// Supported feature set
    pub features: &'static [Feature],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    Fan,
    Perf,
    BatteryCare,
    KbdBacklight,
    LidLogo,
    LightsAlwaysOn,
}

/// All known/supported devices.
pub const SUPPORTED_DEVICES: &[DeviceDescriptor] = &[
    DeviceDescriptor {
        sku_prefix: "RZ09-0421N",
        name: "Razer Blade 15 (2022)",
        pid: 0x028A,
        features: &[
            Feature::Fan,
            Feature::Perf,
            Feature::BatteryCare,
            Feature::KbdBacklight,
        ],
    },
    DeviceDescriptor {
        sku_prefix: "RZ09-0482X",
        name: "Razer Blade 14 (2023)",
        pid: 0x029D,
        features: &[
            Feature::Fan,
            Feature::Perf,
            Feature::BatteryCare,
            Feature::KbdBacklight,
            Feature::LightsAlwaysOn,
        ],
    },
    DeviceDescriptor {
        sku_prefix: "RZ09-0483T",
        name: "Razer Blade 16 (2023)",
        pid: 0x029F,
        features: &[
            Feature::Fan,
            Feature::Perf,
            Feature::BatteryCare,
            Feature::KbdBacklight,
            Feature::LidLogo,
            Feature::LightsAlwaysOn,
        ],
    },
    DeviceDescriptor {
        sku_prefix: "RZ09-0510S",
        name: "Razer Blade 16 (2024)",
        pid: 0x02B7,
        features: &[
            Feature::Fan,
            Feature::Perf,
            Feature::BatteryCare,
            Feature::KbdBacklight,
            Feature::LidLogo,
            Feature::LightsAlwaysOn,
        ],
    },
    DeviceDescriptor {
        sku_prefix: "RZ09-05289",
        name: "Razer Blade 16 (2025) RTX 5090",
        pid: 0x02C6,
        features: &[
            Feature::Fan,
            Feature::Perf,
            Feature::BatteryCare,
            Feature::KbdBacklight,
            Feature::LidLogo,
            Feature::LightsAlwaysOn,
        ],
    },
    DeviceDescriptor {
        sku_prefix: "RZ09-05288",
        name: "Razer Blade 16 (2025) RTX 5080",
        pid: 0x02C6,
        features: &[
            Feature::Fan,
            Feature::Perf,
            Feature::BatteryCare,
            Feature::KbdBacklight,
            Feature::LidLogo,
            Feature::LightsAlwaysOn,
        ],
    },
    DeviceDescriptor {
        sku_prefix: "RZ09-05818",
        name: "Razer Blade 16 (2026) RTX 5080",
        pid: 0x02E0,
        features: &[
            Feature::Fan,
            Feature::Perf,
            Feature::BatteryCare,
            Feature::KbdBacklight,
            Feature::LidLogo,
            Feature::LightsAlwaysOn,
        ],
    },
];

/// Read the system SKU and find a matching device descriptor.
pub fn detect_device() -> Result<DeviceDescriptor> {
    let sku = read_system_sku()?;
    log::info!("System SKU: {sku}");

    let prefix: String = sku.chars().take(10).collect();

    SUPPORTED_DEVICES
        .iter()
        .find(|d| prefix == d.sku_prefix)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Unsupported device: SKU={sku}"))
}

#[cfg(target_os = "windows")]
fn read_system_sku() -> Result<String> {
    let hklm = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
    let bios = hklm.open_subkey("HARDWARE\\DESCRIPTION\\System\\BIOS")?;
    let sku: String = bios.get_value("SystemSKU")?;
    Ok(sku)
}

#[cfg(target_os = "linux")]
fn read_system_sku() -> Result<String> {
    let sku = std::fs::read_to_string("/sys/devices/virtual/dmi/id/product_sku")
        .map(|s| s.trim().to_string())?;
    Ok(sku)
}
