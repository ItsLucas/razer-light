use serde::{Deserialize, Serialize};
use strum::EnumIter;

/// Fan zone identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FanZone {
    Zone1 = 0x01, // CPU fan
    Zone2 = 0x02, // GPU fan
}

/// CPU/GPU cluster identifiers for boost commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Cluster {
    Cpu = 0x01,
    Gpu = 0x02,
}

/// Performance mode presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
#[repr(u8)]
pub enum PerfMode {
    Balanced = 0,
    Performance = 2,
    Custom = 4,
    Silent = 5,
    Battery = 6,
    Hyperboost = 7,
}

impl PerfMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Balanced => "Balanced",
            Self::Performance => "Performance",
            Self::Custom => "Custom",
            Self::Silent => "Silent",
            Self::Battery => "Battery Saver",
            Self::Hyperboost => "Hyperboost",
        }
    }
}

impl TryFrom<u8> for PerfMode {
    type Error = anyhow::Error;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Balanced),
            2 => Ok(Self::Performance),
            4 => Ok(Self::Custom),
            5 => Ok(Self::Silent),
            6 => Ok(Self::Battery),
            7 => Ok(Self::Hyperboost),
            _ => anyhow::bail!("Unknown PerfMode: {v}"),
        }
    }
}

/// Fan mode (auto or manual).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FanMode {
    Auto = 0,
    Manual = 1,
}

impl TryFrom<u8> for FanMode {
    type Error = anyhow::Error;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Auto),
            1 => Ok(Self::Manual),
            _ => anyhow::bail!("Unknown FanMode: {v}"),
        }
    }
}

/// CPU boost levels (available in Custom performance mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
#[repr(u8)]
pub enum CpuBoost {
    Low = 0,
    Medium = 1,
    High = 2,
    Boost = 3,
    Overclock = 4,
}

impl CpuBoost {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Boost => "Boost",
            Self::Overclock => "Overclock",
        }
    }
}

impl TryFrom<u8> for CpuBoost {
    type Error = anyhow::Error;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Low),
            1 => Ok(Self::Medium),
            2 => Ok(Self::High),
            3 => Ok(Self::Boost),
            4 => Ok(Self::Overclock),
            _ => anyhow::bail!("Unknown CpuBoost: {v}"),
        }
    }
}

/// GPU boost levels (available in Custom performance mode).
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
#[repr(u8)]
pub enum GpuBoost {
    Low = 0,
    Medium = 1,
    High = 2,
}

impl GpuBoost {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
        }
    }
}

impl TryFrom<u8> for GpuBoost {
    type Error = anyhow::Error;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Low),
            1 => Ok(Self::Medium),
            2 => Ok(Self::High),
            _ => anyhow::bail!("Unknown GpuBoost: {v}"),
        }
    }
}

/// Battery care charge limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Serialize, Deserialize)]
#[repr(u8)]
pub enum BatteryCare {
    Percent50 = 0xB2,
    Percent55 = 0xB7,
    Percent60 = 0xBC,
    Percent65 = 0xC1,
    Percent70 = 0xC6,
    Percent75 = 0xCB,
    Percent80 = 0xD0,
    Disabled = 0x50,
}

impl BatteryCare {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Percent50 => "50%",
            Self::Percent55 => "55%",
            Self::Percent60 => "60%",
            Self::Percent65 => "65%",
            Self::Percent70 => "70%",
            Self::Percent75 => "75%",
            Self::Percent80 => "80%",
            Self::Disabled => "Disabled (100%)",
        }
    }

    pub fn percent(&self) -> u8 {
        match self {
            Self::Percent50 => 50,
            Self::Percent55 => 55,
            Self::Percent60 => 60,
            Self::Percent65 => 65,
            Self::Percent70 => 70,
            Self::Percent75 => 75,
            Self::Percent80 => 80,
            Self::Disabled => 100,
        }
    }
}

impl TryFrom<u8> for BatteryCare {
    type Error = anyhow::Error;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0xB2 => Ok(Self::Percent50),
            0xB7 => Ok(Self::Percent55),
            0xBC => Ok(Self::Percent60),
            0xC1 => Ok(Self::Percent65),
            0xC6 => Ok(Self::Percent70),
            0xCB => Ok(Self::Percent75),
            0xD0 => Ok(Self::Percent80),
            0x50 => Ok(Self::Disabled),
            _ => anyhow::bail!("Unknown BatteryCare threshold: 0x{v:02X}"),
        }
    }
}

/// Lid logo mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogoMode {
    Off,
    Static,
    Breathing,
}

/// Max fan speed toggle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum MaxFanSpeed {
    Disabled = 0,
    Enabled = 2,
}

impl TryFrom<u8> for MaxFanSpeed {
    type Error = anyhow::Error;
    fn try_from(v: u8) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::Disabled),
            2 => Ok(Self::Enabled),
            _ => anyhow::bail!("Unknown MaxFanSpeed: {v}"),
        }
    }
}
