//! USB device information gathering

use crate::{InventoryError, Result};
use serde::{Deserialize, Serialize};
use usb_ids::Device;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDevice {
    pub id: Uuid,
    pub bus: u8,
    pub address: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub vendor_name: String,
    pub product_name: String,
    pub class: UsbClass,
    pub subclass: u8,
    pub protocol: u8,
    pub max_power: u16,
    pub speed: UsbSpeed,
    pub ports: Vec<u8>,
    pub serial_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsbClass {
    Audio,
    Communication,
    Hid,
    Physical,
    Image,
    Printer,
    MassStorage,
    Hub,
    CdcData,
    SmartCard,
    ContentSecurity,
    Video,
    PersonalHealthcare,
    AudioVideo,
    Billboard,
    UsbTypeCBridge,
    DiagnosticDevice,
    WirelessController,
    Miscellaneous,
    ApplicationSpecific,
    VendorSpecific,
    Unknown(u8),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsbSpeed {
    Low,
    Full,
    High,
    SuperSpeed,
    SuperSpeedPlus,
    Unknown,
}

pub fn gather_usb_devices() -> Result<Vec<UsbDevice>> {
    let mut devices = Vec::new();

    // On Windows, use Windows API or WMI
    // For now, return empty vec

    Ok(devices)
}

fn lookup_usb_name(vid: u16, pid: u16) -> (String, String) {
    if let Some(device) = Device::from_vid_pid(vid, pid) {
        (device.vendor().name().to_string(), device.name().to_string())
    } else {
        (
            format!("Unknown ({:04X})", vid),
            format!("Unknown ({:04X})", pid),
        )
    }
}
