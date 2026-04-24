use anyhow::{ensure, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::fmt;

/// 90-byte USB HID Feature Report matching the openrazer `razer_report` struct.
///
/// Layout:
/// ```text
/// [0]    status           0x00=New, 0x02=Success, etc.
/// [1]    transaction_id   Random, matches request/response
/// [2..4] remaining_pkts   Big-endian u16
/// [4]    protocol_type    Always 0x00
/// [5]    data_size        Number of arg bytes
/// [6]    command_class    High byte of command
/// [7]    command_id       Low byte of command
/// [8..88] arguments       80 bytes, zero-padded
/// [88]   crc              XOR of bytes [2..88]
/// [89]   reserved         Always 0x00
/// ```
#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    pub status: u8,
    pub id: u8,
    pub remaining_packets: u16,
    pub protocol_type: u8,
    pub data_size: u8,
    pub command_class: u8,
    pub command_id: u8,
    #[serde(with = "BigArray")]
    pub args: [u8; 80],
    pub crc: u8,
    pub reserved: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CommandStatus {
    New = 0x00,
    Busy = 0x01,
    Successful = 0x02,
    Failure = 0x03,
    Timeout = 0x04,
    NotSupported = 0x05,
}

impl Packet {
    /// Create a new command packet.
    /// `command` is `(class << 8) | id`, e.g. `0x0D02` for set perf mode.
    pub fn new(command: u16, args: &[u8]) -> Self {
        let mut args_buffer = [0u8; 80];
        let len = args.len().min(80);
        args_buffer[..len].copy_from_slice(&args[..len]);

        Self {
            status: CommandStatus::New as u8,
            id: rand::thread_rng().r#gen(),
            remaining_packets: 0x0000,
            protocol_type: 0x00,
            data_size: len as u8,
            command_class: (command >> 8) as u8,
            command_id: (command & 0xFF) as u8,
            args: args_buffer,
            crc: 0x00,
            reserved: 0x00,
        }
    }

    pub fn command(&self) -> u16 {
        ((self.command_class as u16) << 8) | (self.command_id as u16)
    }

    pub fn get_args(&self) -> &[u8] {
        &self.args[..self.data_size as usize]
    }

    /// Validate that a response matches this request.
    pub fn validate_response(&self, response: &Packet) -> Result<()> {
        ensure!(
            response.command_class == self.command_class && response.command_id == self.command_id,
            "Response command 0x{:04X} doesn't match request 0x{:04X}",
            response.command(),
            self.command()
        );
        ensure!(
            response.id == self.id,
            "Response transaction ID 0x{:02X} doesn't match request 0x{:02X}",
            response.id,
            self.id
        );
        ensure!(
            response.status != CommandStatus::NotSupported as u8,
            "Command 0x{:04X} not supported",
            self.command()
        );
        ensure!(
            response.status == CommandStatus::Successful as u8,
            "Command 0x{:04X} failed with status 0x{:02X}",
            self.command(),
            response.status
        );
        Ok(())
    }

    /// Human-readable status name.
    pub fn status_name(&self) -> &'static str {
        match self.status {
            0x00 => "New",
            0x01 => "Busy",
            0x02 => "OK",
            0x03 => "Fail",
            0x04 => "Timeout",
            0x05 => "NotSupported",
            _ => "Unknown",
        }
    }
}

impl fmt::Display for Packet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] cmd=0x{:04X} size={} args=[{}]",
            self.status_name(),
            self.command(),
            self.data_size,
            self.args[..self.data_size as usize]
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(" "),
        )
    }
}

/// Serialize a Packet to 90 bytes.
impl From<&Packet> for Vec<u8> {
    fn from(packet: &Packet) -> Vec<u8> {
        bincode::serialize(packet).expect("Packet serialization should not fail")
    }
}

/// Deserialize a Packet from 90 bytes.
impl TryFrom<&[u8]> for Packet {
    type Error = anyhow::Error;

    fn try_from(data: &[u8]) -> Result<Self> {
        ensure!(
            data.len() == std::mem::size_of::<Packet>(),
            "Invalid packet size: expected {}, got {}",
            std::mem::size_of::<Packet>(),
            data.len()
        );
        Ok(bincode::deserialize(data)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_size_is_90() {
        assert_eq!(std::mem::size_of::<Packet>(), 90);
    }

    #[test]
    fn packet_roundtrip() {
        let pkt = Packet::new(0x0D02, &[0x01, 0x01, 0x00, 0x00]);
        let bytes: Vec<u8> = (&pkt).into();
        assert_eq!(bytes.len(), 90);
        let decoded = Packet::try_from(bytes.as_slice()).unwrap();
        assert_eq!(decoded.command_class, 0x0D);
        assert_eq!(decoded.command_id, 0x02);
        assert_eq!(decoded.data_size, 4);
        assert_eq!(&decoded.args[..4], &[0x01, 0x01, 0x00, 0x00]);
    }
}
