# ninebot-ble

![MIT license](https://img.shields.io/github/license/zero2005x/ninebot-ble)
![Crates.io version](https://img.shields.io/crates/v/ninebot-ble)

A lightweight Rust library for BLE communication with Ninebot/Xiaomi electric scooters (M365, Mi Pro, etc.).

> 📖 **[中文文档 / Chinese Documentation](./doc/README_zh.md)**

## Features

- 🔍 **Scanner** - Find nearby M365 scooters
- 🔐 **Registration** - Pair with scooter using ECDH key exchange
- 🔑 **Login** - Authenticate with saved token
- 📊 **Read Data** - Battery, speed, distance, temperature, etc.
- ⚙️ **Settings** - Control cruise mode, tail light, KERS level
- 🎮 **Interactive Controller** - Real-time monitoring and control

## Supported Platforms

This library uses [btleplug](https://crates.io/crates/btleplug) for cross-platform BLE support:

- Windows 10/11
- macOS
- Linux
- iOS

## Supported Scooters

| Model             | Status       |
| ----------------- | ------------ |
| Xiaomi M365       | ✅ Supported |
| Xiaomi Mi 1S      | ✅ Supported |
| Xiaomi Mi Pro     | ✅ Supported |
| Xiaomi Mi Pro 2   | ✅ Supported |
| Xiaomi Mi Pro 3   | ✅ Supported |
| Clone controllers | ⚠️ Partial   |

## Quick Start

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ninebot-ble = "0.1"
```

### Examples

#### 1. Find MAC Address

```bash
cargo run --example scanner
```

Output:

```
INFO scanner: Found scooter nearby: MIScooter7353 with mac: D5:01:45:37:ED:FD
```

#### 2. Register (First Time Only)

⚠️ **Warning:** Registering unpairs the device from all other apps!

```bash
cargo run --example register D5:01:45:37:ED:FD
```

This saves the auth token to `.mi-token` file.

#### 3. Login

```bash
cargo run --example login D5:01:45:37:ED:FD
```

#### 4. Read Information

```bash
cargo run --example about D5:01:45:37:ED:FD
```

Output:

```
Battery info: BatteryInfo { capacity: 7392, percent: 63, voltage: 36.74 }
Serial number: 26354/00467353
Motor info: MotorInfo { speed_kmh: 0, total_distance_m: 1306083 }
```

#### 5. Interactive Controller

```bash
cargo run --example controller D5:01:45:37:ED:FD
```

## BLE Protocol

### Services & Characteristics

| UUID           | Name         | Description               |
| -------------- | ------------ | ------------------------- |
| `FE95`         | AUTH Service | Xiaomi Authentication     |
| `0x0010`       | UPNP         | Command Control           |
| `0x0019`       | AVDTP        | Data Exchange             |
| `6e400002-...` | TX           | Write (Client → Scooter)  |
| `6e400003-...` | RX           | Notify (Scooter → Client) |

### UART Frame Format

```
+-----+-----+-----+-----+-----+-----+-------+------+------+
| 0x55| 0xAA|  L  |  D  |  T  |  C  |  ...  | CK0  | CK1  |
+-----+-----+-----+-----+-----+-----+-------+------+------+
  Header     Len   Dev   Cmd   Attr  Payload  Checksum
```

| Field       | Description                                        |
| ----------- | -------------------------------------------------- |
| `0x55 0xAA` | Frame header                                       |
| `L`         | Length = payload + 2                               |
| `D`         | Device: `0x20`=Master→Motor, `0x22`=Master→Battery |
| `T`         | Type: `0x01`=Read, `0x03`=Write                    |
| `CK0, CK1`  | Checksum = (sum of bytes from L) XOR 0xFFFF        |

## Cryptographic Flow

### Registration (Once)

```
Client                              Scooter
  │                                   │
  │──── CMD_GET_INFO ────────────────►│
  │◄──── Remote Info ─────────────────│
  │                                   │
  │  Generate ECDH KeyPair (P-256)    │
  │──── My Public Key ───────────────►│
  │◄──── Scooter Public Key ──────────│
  │                                   │
  │  Calculate:                       │
  │  - SharedSecret (ECDH)            │
  │  - Token, BindKey (HKDF-SHA256)   │
  │  - DID_CT (AES-CCM encrypted)     │
  │                                   │
  │──── DID_CT ──────────────────────►│
  │◄──── AUTH_OK ─────────────────────│
  │                                   │
  │  Save Token (12 bytes)            │
```

### Login (Every Connection)

```
Client                              Scooter
  │                                   │
  │──── CMD_LOGIN ───────────────────►│
  │──── Random Key (16 bytes) ───────►│
  │◄──── Remote Random Key ───────────│
  │◄──── Remote Info (32 bytes) ──────│
  │                                   │
  │  Derive Keys (HKDF-SHA256):       │
  │  - DevKey, AppKey (AES-128)       │
  │  - DevIV, AppIV (4 bytes each)    │
  │                                   │
  │  Verify: HMAC(DevKey, salt)       │
  │                                   │
  │──── DID Info ────────────────────►│
  │◄──── LOGIN_OK ────────────────────│
```

### UART Encryption (AES-128-CCM)

```
Encrypt (Client → Scooter):
  nonce = AppIV + "0000" + counter
  ciphertext = AES-CCM(AppKey, message, nonce)

Decrypt (Scooter → Client):
  nonce = DevIV + "0000" + counter
  plaintext = AES-CCM(DevKey, ciphertext, nonce)
```

## Available Data

| Category | Data                                                |
| -------- | --------------------------------------------------- |
| Motor    | Speed, Average Speed, Distance, Uptime, Temperature |
| Battery  | Voltage, Current, Capacity, %, Cell Voltages, Temp  |
| Settings | Cruise Mode, Tail Light, KERS Level                 |
| Info     | Serial Number, PIN, Firmware Version                |

## API Reference

### Scanner

```rust
use ninebot_ble::scanner::ScooterScanner;

let scanner = ScooterScanner::new().await?;
let scooters = scanner.scooters().await;
```

### Registration

```rust
use ninebot_ble::register::MiRegister;

let device = scanner.connect_to("D5:01:45:37:ED:FD").await?;
let mut register = MiRegister::new(&device).await?;
let token = register.register().await?;
```

### Login & Session

```rust
use ninebot_ble::login::MiLogin;

let mut login = MiLogin::new(&device, &token).await?;
let session = login.start().await?;

// Read data
let battery = session.battery_info().await?;
let motor = session.motor_info().await?;
```

## Project Structure

```
m365/
├── src/
│   ├── lib.rs           # Library entry point
│   ├── scanner.rs       # BLE device scanner
│   ├── connection.rs    # BLE connection management
│   ├── protocol.rs      # MiAuth protocol implementation
│   ├── register.rs      # Device registration
│   ├── login.rs         # Authentication
│   ├── mi_crypto.rs     # Cryptographic operations
│   ├── consts.rs        # Constants
│   └── session/         # Session commands
│       ├── mi_session.rs
│       ├── battery.rs
│       ├── info.rs
│       ├── settings.rs
│       └── commands.rs
├── examples/
│   ├── scanner.rs       # Find scooters
│   ├── register.rs      # Register with scooter
│   ├── login.rs         # Login example
│   ├── about.rs         # Read all info
│   ├── settings.rs      # Change settings
│   └── controller.rs    # Interactive controller
└── tests/
    ├── crypto_test.rs
    ├── motor_info_test.rs
    └── uart_test.rs
```

## License

This project is licensed under the MIT License - see the [LICENSE.md](LICENSE.md) file for details.

## Acknowledgments

- Based on research from [CamiAlfa's M365-BLE-PROTOCOL](https://github.com/CamiAlfa/M365-BLE-PROTOCOL)
- Uses [btleplug](https://crates.io/crates/btleplug) for cross-platform BLE support
