//! PCI device information gathering

use crate::{PciDevice, InventoryError, Result};
use pci_ids::{Device as PciDeviceId, Class as PciClass};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PciDevice {
    pub id: Uuid,
    pub domain: u16,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub vendor_name: String,
    pub device_name: String,
    pub class: PciDeviceClass,
    pub subclass: u8,
    prog_if: u8,
    pub revision: u8,
    pub driver: Option<String>,
    pub numa_node: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PciDeviceClass {
    Unclassified,
    MassStorage,
    Network,
    Display,
    Multimedia,
    Memory,
    Bridge,
    SimpleCommunication,
    BaseSystemPeripheral,
    InputDevice,
    DockingStation,
    Processor,
    SerialBus,
    Wireless,
    IntelligentIO,
    SatelliteCommunication,
    Encryption,
    SignalProcessing,
    Coprocessor,
    Unknown(u8),
}

pub fn gather_pci_devices() -> Result<Vec<PciDevice>> {
    let mut devices = Vec::new();

    // On Windows, use Windows API or WMI
    // For now, return empty vec

    Ok(devices)
}

fn lookup_pci_class(class_code: u8, subclass: u8, prog_if: u8) -> PciDeviceClass {
    match class_code {
        0x00 => PciDeviceClass::Unclassified,
        0x01 => PciDeviceClass::MassStorage,
        0x02 => PciDeviceClass::Network,
        0x03 => PciDeviceClass::Display,
        0x04 => PciDeviceClass::Multimedia,
        0x05 => PciDeviceClass::Memory,
        0x06 => PciDeviceClass::Bridge,
        0x07 => PciDeviceClass::SimpleCommunication,
        0x08 => PciDeviceClass::BaseSystemPeripheral,
        0x09 => PciDeviceClass::InputDevice,
        0x0A => PciDeviceClass::DockingStation,
        0x0B => PciDeviceClass::Processor,
        0x0C => PciDeviceClass::SerialBus,
        0x0D => PciDeviceClass::Wireless,
        0x0E => PciDeviceClass::IntelligentIO,
        0x0F => PciDeviceClass::SatelliteCommunication,
        0x10 => PciDeviceClass::Encryption,
        0x11 => PciDeviceClass::SignalProcessing,
        0x12 => PciDeviceClass::Coprocessor,
        _ => PciDeviceClass::Unknown(class_code),
    }
}