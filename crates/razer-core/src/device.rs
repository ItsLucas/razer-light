use crate::detect::{DeviceDescriptor, SUPPORTED_DEVICES};
use crate::packet::Packet;

use anyhow::{anyhow, Context, Result};
use std::{thread, time::Duration};

pub const RAZER_VID: u16 = 0x1532;
const MAX_RETRIES: usize = 5;

/// Info about a Razer USB HID interface found on the system.
#[derive(Debug, Clone)]
pub struct HidDeviceInfo {
    pub vid: u16,
    pub pid: u16,
    pub path: String,
    pub manufacturer: String,
    pub product: String,
    pub interface_number: i32,
    pub usage_page: u16,
    pub usage: u16,
}

/// A connected Razer HID device.
pub struct Device {
    hid: hidapi::HidDevice,
    pub descriptor: DeviceDescriptor,
}

// hidapi::HidDevice is Send on all platforms we care about
unsafe impl Send for Device {}

impl Device {
    /// Open a device by descriptor. Iterates HID interfaces to find one that accepts feature reports.
    pub fn open(descriptor: DeviceDescriptor) -> Result<Self> {
        Self::open_by_pid(descriptor.pid, Some(descriptor))
    }

    /// Open by PID alone (for CLI use when SKU detection isn't available).
    /// If `descriptor` is None, creates a synthetic one.
    pub fn open_by_pid(pid: u16, descriptor: Option<DeviceDescriptor>) -> Result<Self> {
        let api = hidapi::HidApi::new().context("Failed to init HID API")?;

        for info in api
            .device_list()
            .filter(|i| i.vendor_id() == RAZER_VID && i.product_id() == pid)
        {
            let path = info.path();
            if let Ok(dev) = api.open_path(path) {
                if dev.send_feature_report(&[0, 0]).is_ok() {
                    let desc = descriptor.unwrap_or_else(|| DeviceDescriptor {
                        sku_prefix: "",
                        name: "Unknown Razer Device",
                        pid,
                        features: &[],
                    });
                    return Ok(Self {
                        hid: dev,
                        descriptor: desc,
                    });
                }
            }
        }

        anyhow::bail!("No usable HID interface found for PID 0x{pid:04X}")
    }

    /// Auto-detect: try SKU first, fall back to first known PID.
    pub fn auto_open() -> Result<Self> {
        // Try SKU-based detection
        match crate::detect::detect_device() {
            Ok(desc) => return Self::open(desc),
            Err(e) => log::info!("SKU detection failed ({e}), trying PID scan..."),
        }

        // Fall back: scan for any known PID
        let api = hidapi::HidApi::new()?;
        let known_pids: std::collections::HashSet<u16> =
            SUPPORTED_DEVICES.iter().map(|d| d.pid).collect();

        for info in api
            .device_list()
            .filter(|i| i.vendor_id() == RAZER_VID && known_pids.contains(&i.product_id()))
        {
            let pid = info.product_id();
            if let Ok(dev) = Self::open_by_pid(pid, None) {
                // Try to match a descriptor by PID
                let desc = SUPPORTED_DEVICES
                    .iter()
                    .find(|d| d.pid == pid)
                    .cloned()
                    .unwrap_or(dev.descriptor.clone());
                return Ok(Self {
                    hid: dev.hid,
                    descriptor: desc,
                });
            }
        }

        anyhow::bail!("No supported Razer device found")
    }

    pub fn name(&self) -> &str {
        self.descriptor.name
    }

    pub fn pid(&self) -> u16 {
        self.descriptor.pid
    }

    /// Send a command packet and return the response.
    pub fn send(&self, request: Packet) -> Result<Packet> {
        let mut response_buf = vec![0u8; 1 + std::mem::size_of::<Packet>()];

        for attempt in 0..MAX_RETRIES {
            thread::sleep(Duration::from_millis(1));

            let mut send_buf = vec![0u8];
            send_buf.extend_from_slice(&Vec::<u8>::from(&request));

            self.hid
                .send_feature_report(&send_buf)
                .context("Failed to send feature report")?;

            thread::sleep(Duration::from_millis(2));

            let n = self
                .hid
                .get_feature_report(&mut response_buf)
                .context("Failed to get feature report")?;

            if n != response_buf.len() {
                return Err(anyhow!("Unexpected response size: {n}"));
            }

            let response = Packet::try_from(&response_buf[1..])?;

            match request.validate_response(&response) {
                Ok(()) => return Ok(response),
                Err(e) if attempt < MAX_RETRIES - 1 => {
                    log::debug!("Retry {}/{}: {}", attempt + 1, MAX_RETRIES, e);
                    thread::sleep(Duration::from_millis(500));
                }
                Err(e) => return Err(e),
            }
        }

        Err(anyhow!("Failed after {MAX_RETRIES} retries"))
    }

    /// List all Razer USB HID interfaces on the system.
    pub fn enumerate() -> Result<Vec<HidDeviceInfo>> {
        let api = hidapi::HidApi::new()?;
        let devices: Vec<HidDeviceInfo> = api
            .device_list()
            .filter(|i| i.vendor_id() == RAZER_VID)
            .map(|i| HidDeviceInfo {
                vid: i.vendor_id(),
                pid: i.product_id(),
                path: i.path().to_string_lossy().into_owned(),
                manufacturer: i
                    .manufacturer_string()
                    .unwrap_or_default()
                    .to_string(),
                product: i.product_string().unwrap_or_default().to_string(),
                interface_number: i.interface_number(),
                usage_page: i.usage_page(),
                usage: i.usage(),
            })
            .collect();

        if devices.is_empty() {
            anyhow::bail!("No Razer USB devices found");
        }
        Ok(devices)
    }
}
