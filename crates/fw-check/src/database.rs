//! Firmware database for update checking

use crate::{DatabaseCheckResult, FirmwareError, FirmwareUpdate, Result, UpdateSeverity};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

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
    pub current_version: String,
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
        let response = reqwest::get(&url)
            .await
            .map_err(|e| FirmwareError::Http(e))?;

        let database: FirmwareDatabase = response
            .json()
            .await
            .map_err(|e| FirmwareError::Database(e.to_string()))?;

        self.cache = Some(database);
        Ok(())
    }

    pub async fn check_firmware(&self) -> Result<DatabaseCheckResult> {
        let mut checked = 0;
        let mut outdated = 0;
        let mut critical = 0;
        let mut updates = Vec::new();

        if let Some(db) = &self.cache {
            for entry in db.entries.values() {
                checked += 1;
                if entry.latest_version != entry.current_version {
                    outdated += 1;
                    if matches!(
                        entry.severity,
                        UpdateSeverity::Critical | UpdateSeverity::High
                    ) {
                        critical += 1;
                    }
                    updates.push(FirmwareUpdate {
                        device_id: entry.device_id.clone(),
                        current_version: entry.current_version.clone(),
                        latest_version: entry.latest_version.clone(),
                        severity: entry.severity.clone(),
                        release_date: entry.release_date,
                        description: entry.description.clone(),
                        download_url: entry.download_url.clone(),
                    });
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

pub async fn check_firmware_database() -> Result<DatabaseCheckResult> {
    let mut client = FirmwareDatabaseClient::new(
        "https://earnerbaymalay.github.io/hwcert-firmware-db".to_string(),
    );

    client.update_database().await?;
    client.check_firmware().await
}
