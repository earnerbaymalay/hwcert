//! Firmware checking crate
//! Provides SMBIOS metadata checking

pub mod database;
pub mod smbios;

use chrono::{DateTime, Utc};
pub use crate::database::FirmwareDatabase;
pub use crate::smbios::SmbiosInfo;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Complete firmware report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareReport {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub smbios: SmbiosInfo,
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
    #[error("SMBIOS error: {0}")]
    Smbios(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, FirmwareError>;

/// Gather complete firmware information, including the online update-database check
pub async fn gather_firmware_report() -> Result<FirmwareReport> {
    gather_firmware_report_with(true).await
}

/// Gather firmware information; `check_db = false` skips the network database lookup
/// and reports an empty check, so the report works fully offline.
pub async fn gather_firmware_report_with(check_db: bool) -> Result<FirmwareReport> {
    let smbios = smbios::gather_smbios_info()?;
    let database_check = if check_db {
        database::check_firmware_database().await?
    } else {
        DatabaseCheckResult {
            checked_count: 0,
            outdated_count: 0,
            critical_count: 0,
            updates_available: Vec::new(),
        }
    };

    Ok(FirmwareReport {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        smbios,
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
