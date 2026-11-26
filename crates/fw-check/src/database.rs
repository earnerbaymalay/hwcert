//! Firmware database for update checking

use crate::{DatabaseCheckResult, FirmwareUpdate, UpdateSeverity, FirmwareError, Result, NvmeFirmware};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareDatabase {
    pub version: u32,
    pub last_updated: DateTime<Utc>,
    pub entries: HashMap<String, FirmwareDatabaseEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirmwareDatabaseEntry {
    pub device_id: String,
    pub vendor: String,
    pub model: String,
    pub latest_version: String,
    pub release_date: DateTime<Utc>,
    pub severity: UpdateSeverity,
    pub description: String,
    pub download_url: Option<String>,
    pub checksum: Option<String>,
    pub size_bytes: u64,
}

pub struct FirmwareDatabaseClient {
    client: Client,
    base_url: String,
    cache: Option<FirmwareDatabase>,
}

impl FirmwareDatabaseClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            cache: None,
        }
    }

    pub async fn update_database(&mut self) -> Result<()> {
        let url = format!("{}/firmware_database.json", self.base_url);
        let response = reqwest::get(&url).await
            .map_err(|e| FirmwareError::Http(e))?;
        
        let database: FirmwareDatabase = response.json().await
            .map_err(|e| FirmwareError::Database(e.to_string()))?;
        
        self.cache = Some(database);
        Ok(())
    }

    pub async fn check_firmware(&self, devices: &[NvmeFirmware]) -> Result<DatabaseCheckResult> {
        let mut checked = 0;
        let mut outdated = 0;
        let mut critical = 0;
        let mut updates = Vec::new();

        if let Some(db) = &self.cache {
            for device in devices {
                checked += 1;
                let key = format!("{}:{}", device.vendor_id, device.device_id);
                
                if let Some(entry) = db.entries.get(&key) {
                    if entry.latest_version != device.firmware_revision {
                        outdated += 1;
                        if matches!(entry.severity, UpdateSeverity::Critical | UpdateSeverity::High) {
                            critical += 1;
                        }
                        updates.push(FirmwareUpdate {
                            device_id: format!("{}:{}", device.vendor_id, device.device_id),
                            current_version: device.firmware_revision.clone(),
                            latest_version: entry.latest_version.clone(),
                            severity: entry.severity.clone(),
                            release_date: entry.release_date,
                            description: entry.description.clone(),
                            download_url: entry.download_url.clone(),
                        });
                    }
                }
            }
        }

        Ok(DatabaseCheckResult {
            checked_count: checked,
            outdated_count: outdated,
            critical_count: critical,
            updates_available: updates,
        })
    }
}

pub async fn check_firmware_database(devices: &[NvmeFirmware]) -> Result<DatabaseCheckResult> {
    let mut client = FirmwareDatabaseClient::new(
        "https://earnerbaymalay.github.io/hwcert-firmware-db".to_string()
    );
    
    client.update_database().await?;
    client.check_firmware(devices).await
}