use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::Manager;
use futures::FutureExt;
use m365::clone_connection::ScooterConnection;
use std::error::Error;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let target_mac = std::env::args().nth(1).expect("Please provide MAC address");
    println!("Looking for clone scooter with MAC: {}", target_mac);

    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters
        .into_iter()
        .nth(0)
        .expect("No Bluetooth adapters found");

    central.start_scan(ScanFilter::default()).await?;
    time::sleep(Duration::from_secs(5)).await;

    let peripherals = central.peripherals().await?;
    let device = peripherals
        .into_iter()
        .find(|p| {
            if let Some(Ok(Some(props))) = p.properties().now_or_never() {
                props.address.to_string().contains(&target_mac)
            } else {
                false
            }
        })
        .expect("Scooter not found");

    println!("Found device, connecting...");

    let connection = ScooterConnection::connect(&device).await?;
    println!("Connected and subscribed!");

    // Optional: Try Unlock (Xbot/Lenzod)
    println!("Sending Unlock commands...");
    let _ = connection.send_command(&[0x5A]).await; // Xbot Z
    time::sleep(Duration::from_millis(500)).await;
    let _ = connection
        .send_command(&[0xA6, 0x12, 0x02, 0x10, 0x14])
        .await; // Lenzod
    time::sleep(Duration::from_millis(500)).await;

    println!("Reading Firmware Version...");
    match connection.get_version().await {
        Ok(data) => println!("Version Data: {:02X?}", data),
        Err(e) => println!("Failed to read version: {}", e),
    }

    println!("Reading Battery Level...");
    match connection.get_battery_level().await {
        Ok(level) => println!("Battery Level: {}%", level),
        Err(e) => println!("Failed to read battery: {}", e),
    }

    println!("Disconnecting...");
    device.disconnect().await?;
    Ok(())
}
