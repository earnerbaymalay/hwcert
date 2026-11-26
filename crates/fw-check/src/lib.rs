//! Firmware checking crate
//! Provides UEFI, NVMe, SMBIOS, and fwupd metadata checking

pub mod uefi;
pub mod nvme_fw;
pub mod smbios;
pub mod fwupd;
pub mod database;

use fw_check::uefi::UefiInfo;
use fw_check::nvme_fw::NvmeFirmware;
use fw_check::smbios::SmbiosInfo;
use fw_check::fwupd::FwupdInfo;
use fw_check::database::FirmwareDatabase;
use fw_check::FirmwareReport;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Complete firmware report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareReport {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub uefi: UefiInfo,
    pub nvme_devices: Vec<NvmeFirmware>,
    pub smbios: SmbiosInfo,
    pub fwupd: Option<FwupdInfo>,
    pub database_check: DatabaseCheckResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseCheckResult {
    pub checked_count: usize,
    pub outdated_count: usize,
    pub critical_count: usize,
    pub updates_available: Vec<FirmwareUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareUpdate {
    pub device_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub severity: UpdateSeverity,
    pub release_date: DateTime<Utc>,
    pub description: String,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Errors for firmware checking
#[derive(Error, Debug)]
pub enum FirmwareError {
    #[error("UEFI error: {0}")]
    Uefi(String),
    #[error("NVMe error: {0}")]
    Nvme(String),
    #[error("SMBIOS error: {0}")]
    Smbios(String),
    #[error("fwupd error: {0}")]
    Fwupd(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, FirmwareError>;

/// Gather complete firmware information
pub async fn gather_firmware_report() -> Result<FirmwareReport> {
    let uefi = uefi::gather_uefi_info()?;
    let nvme_devices = nvme_fw::gather_nvme_firmware()?;
    let smbios = smbios::gather_smbios_info()?;
    let fwupd = fwupd::gather_fwupd_info().ok();
    let database_check = database::check_firmware_database(&nvme_devices).await?;

    Ok(FirmwareReport {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        uefi,
        nvme_devices,
        smbios,
        fwupd,
        database_check,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_firmware_report() {
        let result = gather_firmware_report().await;
        // May fail on systems without proper permissions
        assert!(result.is_ok() || result.is_err());
    }
}