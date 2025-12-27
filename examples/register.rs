use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;

use anyhow::Result;
use btleplug::api::BDAddr;
use btleplug::platform::Peripheral;
use pretty_hex::*;
use std::env;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tracing_subscriber;

use m365::{
    AuthToken, ConnectionHelper, RegistrationError, RegistrationRequest, ScannerEvent,
    ScooterScanner,
};

async fn save_token(token: &AuthToken) -> Result<()> {
    let path = Path::new(".mi-token");
    tracing::info!(
        "Saving token at {:?} with content {:?}",
        path,
        token.hex_dump()
    );
    let f = File::create(path).await?;
    {
        let mut writer = BufWriter::new(f);
        writer.write(token).await?;
        writer.flush().await?;
    }
    Ok(())
}

async fn register(device: &Peripheral) -> Result<()> {
    let connection = ConnectionHelper::new(&device);

    loop {
        tracing::info!(">>> Press power button up to 5 seconds after beep!");
        connection.reconnect().await?;

        // Add a small delay after reconnection to ensure stability
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let mut request = match RegistrationRequest::new(&device).await {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Failed to create registration request: {}", e);
                continue;
            }
        };

        match request.start().await {
            Ok(token) => {
                save_token(&token).await?;
                break;
            }
            Err(RegistrationError::RestartNeeded) => {
                tracing::debug!("Restarting...");
                continue;
            }
            Err(e) => {
                tracing::error!("Unhandled error: {}", e);
                // Optional: break or continue depending on if you want to retry on other errors
                // break;
            }
        }
    }

    Ok(())
}

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1].is_empty() {
        panic!("First argument is scooter mac address");
    }

    let mac = BDAddr::from_str_delim(&args[1]).expect("Invalid mac address");
    tracing::info!("Searching scooter with address: {}", mac);

    let mut scanner = ScooterScanner::new().await?;
    let mut rx = scanner.start().await?;

    while let Some(event) = rx.recv().await {
        match event {
            ScannerEvent::DiscoveredScooter(scooter) => {
                if scooter.addr == mac {
                    tracing::info!("Found your scooter, starting registration");
                    let device = scanner.peripheral(&scooter).await?;
                    register(&device).await?;
                    break;
                } else {
                    tracing::info!(
                        "Found scooter nearby: {} with mac: {}",
                        scooter.name.unwrap(),
                        scooter.addr
                    );
                }
            }
        }
    }

    Ok(())
}
