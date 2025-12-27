use anyhow::Result;
use btleplug::api::Peripheral as _;
use btleplug::platform::Peripheral;
use std::time::Duration;
use tokio::time;

pub struct ConnectionHelper {
    device: Peripheral,
}

impl ConnectionHelper {
    pub fn new(device: &Peripheral) -> Self {
        Self {
            device: device.clone(),
        }
    }

    pub async fn connect(&self) -> Result<bool, btleplug::Error> {
        tracing::debug!("Connecting to device.");
        let mut retries = 5;
        while retries >= 0 {
            if self.device.is_connected().await? {
                tracing::debug!("Connected to device");
                break;
            }
            match self.device.connect().await {
                Ok(_) => {
                    // Wait longer for connection to stabilize
                    time::sleep(Duration::from_millis(2000)).await;
                    if self.device.is_connected().await? {
                        tracing::debug!("Connected to device");
                        break;
                    } else {
                        tracing::debug!("Connect call succeeded but device is not connected");
                    }
                }
                Err(err) if retries > 0 => {
                    retries -= 1;
                    tracing::debug!(
                        "Retrying connection: {} retries left, reason: {}",
                        retries,
                        err
                    );
                    time::sleep(Duration::from_secs(2)).await;
                }

                Err(err) => return Err(err),
            }
        }

        Ok(true)
    }

    pub async fn disconnect(&self) -> Result<bool> {
        if !self.device.is_connected().await? {
            tracing::debug!("Already disconnected.");
            return Ok(true);
        }

        if let Err(error) = self.device.disconnect().await {
            tracing::error!("Could not disconnect: {}", error);
            return Ok(false);
        }

        tracing::debug!("Disconnected from device");
        Ok(true)
    }

    pub async fn reconnect(&self) -> Result<bool> {
        tracing::debug!("Reconnecting...");
        self.disconnect().await?;
        time::sleep(Duration::from_secs(5)).await;
        self.connect().await?;
        Ok(true)
    }
}
