use crate::device::Device;
use crate::packet::Packet;
use crate::types::*;

use anyhow::{Result, ensure};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn send_cmd(device: &Device, command: u16, args: &[u8]) -> Result<Packet> {
    let response = device.send(Packet::new(command, args))?;
    ensure!(
        response.get_args().starts_with(args),
        "Response args don't match request for cmd 0x{command:04X}"
    );
    Ok(response)
}

// ── Performance Mode ─────────────────────────────────────────────────────────

/// Set performance mode on both fan zones.
pub fn set_perf_mode(device: &Device, mode: PerfMode, fan_mode: FanMode) -> Result<()> {
    for zone in [1u8, 2] {
        send_cmd(device, 0x0D02, &[0x01, zone, mode as u8, fan_mode as u8])?;
    }
    Ok(())
}

/// Get current performance mode and fan mode.
pub fn get_perf_mode(device: &Device) -> Result<(PerfMode, FanMode)> {
    let resp = device.send(Packet::new(0x0D82, &[0x00, 0x01, 0x00, 0x00]))?;
    let args = resp.get_args();
    Ok((PerfMode::try_from(args[2])?, FanMode::try_from(args[3])?))
}

// ── CPU/GPU Boost ────────────────────────────────────────────────────────────

pub fn set_cpu_boost(device: &Device, boost: CpuBoost) -> Result<()> {
    ensure!(
        get_perf_mode(device)?.0 == PerfMode::Custom,
        "CPU boost requires Custom performance mode"
    );
    send_cmd(device, 0x0D07, &[0x01, Cluster::Cpu as u8, boost as u8])?;
    Ok(())
}

pub fn set_gpu_boost(device: &Device, boost: GpuBoost) -> Result<()> {
    ensure!(
        get_perf_mode(device)?.0 == PerfMode::Custom,
        "GPU boost requires Custom performance mode"
    );
    send_cmd(device, 0x0D07, &[0x01, Cluster::Gpu as u8, boost as u8])?;
    Ok(())
}

pub fn get_cpu_boost(device: &Device) -> Result<CpuBoost> {
    let resp = device.send(Packet::new(0x0D87, &[0x00, Cluster::Cpu as u8, 0x00]))?;
    CpuBoost::try_from(resp.get_args()[2])
}

pub fn get_gpu_boost(device: &Device) -> Result<GpuBoost> {
    let resp = device.send(Packet::new(0x0D87, &[0x00, Cluster::Gpu as u8, 0x00]))?;
    GpuBoost::try_from(resp.get_args()[2])
}

// ── Fan Control ──────────────────────────────────────────────────────────────

/// Set fan RPM for both zones (0–5500, rounded to nearest 100).
pub fn set_fan_rpm(device: &Device, rpm: u16) -> Result<()> {
    ensure!((0..=5500).contains(&rpm), "Fan RPM must be 0–5500");
    let value = (rpm / 100) as u8;
    for zone in [FanZone::Zone1, FanZone::Zone2] {
        send_cmd(device, 0x0D01, &[0x00, zone as u8, value])?;
    }
    Ok(())
}

/// Get target fan RPM for a zone.
pub fn get_fan_rpm(device: &Device, zone: FanZone) -> Result<u16> {
    let resp = device.send(Packet::new(0x0D81, &[0x00, zone as u8, 0x00]))?;
    Ok(resp.get_args()[2] as u16 * 100)
}

/// Get actual (measured) fan RPM for a zone.
pub fn get_fan_actual_rpm(device: &Device, zone: FanZone) -> Result<u16> {
    let resp = device.send(Packet::new(0x0D88, &[0x00, zone as u8, 0x00]))?;
    Ok(resp.get_args()[2] as u16 * 100)
}

pub fn set_max_fan_speed(device: &Device, enabled: bool) -> Result<()> {
    let mode = if enabled {
        MaxFanSpeed::Enabled
    } else {
        MaxFanSpeed::Disabled
    };
    send_cmd(device, 0x070F, &[mode as u8])?;
    Ok(())
}

pub fn get_max_fan_speed(device: &Device) -> Result<bool> {
    let resp = device.send(Packet::new(0x078F, &[0x00]))?;
    Ok(MaxFanSpeed::try_from(resp.get_args()[0])? == MaxFanSpeed::Enabled)
}

// ── Battery Care ─────────────────────────────────────────────────────────────

pub fn set_battery_care(device: &Device, level: BatteryCare) -> Result<()> {
    send_cmd(device, 0x0712, &[level as u8])?;
    Ok(())
}

pub fn get_battery_care(device: &Device) -> Result<BatteryCare> {
    let resp = device.send(Packet::new(0x0792, &[0x00]))?;
    BatteryCare::try_from(resp.get_args()[0])
}

// ── Keyboard Brightness ─────────────────────────────────────────────────────

pub fn set_keyboard_brightness(device: &Device, brightness: u8) -> Result<()> {
    send_cmd(device, 0x0303, &[0x01, 0x05, brightness])?;
    Ok(())
}

pub fn get_keyboard_brightness(device: &Device) -> Result<u8> {
    let resp = device.send(Packet::new(0x0383, &[0x01, 0x05, 0x00]))?;
    Ok(resp.get_args()[2])
}

// ── Lid Logo ─────────────────────────────────────────────────────────────────

pub fn set_logo_mode(device: &Device, mode: LogoMode) -> Result<()> {
    match mode {
        LogoMode::Off => {
            send_cmd(device, 0x0300, &[0x01, 0x04, 0x00])?;
        }
        LogoMode::Static => {
            send_cmd(device, 0x0302, &[0x01, 0x04, 0x00])?;
            send_cmd(device, 0x0300, &[0x01, 0x04, 0x01])?;
        }
        LogoMode::Breathing => {
            send_cmd(device, 0x0302, &[0x01, 0x04, 0x02])?;
            send_cmd(device, 0x0300, &[0x01, 0x04, 0x01])?;
        }
    }
    Ok(())
}

pub fn get_logo_mode(device: &Device) -> Result<LogoMode> {
    let power_resp = device.send(Packet::new(0x0380, &[0x01, 0x04, 0x00]))?;
    if power_resp.get_args()[2] == 0 {
        return Ok(LogoMode::Off);
    }
    let mode_resp = device.send(Packet::new(0x0382, &[0x01, 0x04, 0x00]))?;
    match mode_resp.get_args()[2] {
        0 => Ok(LogoMode::Static),
        2 => Ok(LogoMode::Breathing),
        v => anyhow::bail!("Unknown logo mode: {v}"),
    }
}

// ── Raw / Debug ──────────────────────────────────────────────────────────────

/// Send a raw command (for debugging). Returns the full response packet.
pub fn raw_command(device: &Device, command: u16, args: &[u8]) -> Result<Packet> {
    let request = Packet::new(command, args);
    log::debug!("TX: {request}");
    let response = device.send(request)?;
    log::debug!("RX: {response}");
    Ok(response)
}

/// Read all current device state in one shot.
pub fn read_all_state(device: &Device) -> Result<DeviceState> {
    let (perf_mode, fan_mode) = get_perf_mode(device)?;
    let fan1 = get_fan_actual_rpm(device, FanZone::Zone1).unwrap_or(0);
    let fan2 = get_fan_actual_rpm(device, FanZone::Zone2).unwrap_or(0);
    let battery = get_battery_care(device).ok();
    let brightness = get_keyboard_brightness(device).unwrap_or(0);
    let cpu_boost = if perf_mode == PerfMode::Custom {
        get_cpu_boost(device).ok()
    } else {
        None
    };
    let gpu_boost = if perf_mode == PerfMode::Custom {
        get_gpu_boost(device).ok()
    } else {
        None
    };
    let logo = get_logo_mode(device).ok();

    Ok(DeviceState {
        perf_mode,
        fan_mode,
        fan1_rpm: fan1,
        fan2_rpm: fan2,
        battery_care: battery,
        kbd_brightness: brightness,
        cpu_boost,
        gpu_boost,
        logo_mode: logo,
    })
}

/// Snapshot of all readable device state.
#[derive(Debug, Clone)]
pub struct DeviceState {
    pub perf_mode: PerfMode,
    pub fan_mode: FanMode,
    pub fan1_rpm: u16,
    pub fan2_rpm: u16,
    pub battery_care: Option<BatteryCare>,
    pub kbd_brightness: u8,
    pub cpu_boost: Option<CpuBoost>,
    pub gpu_boost: Option<GpuBoost>,
    pub logo_mode: Option<LogoMode>,
}
