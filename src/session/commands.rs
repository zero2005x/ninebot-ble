use core::fmt::Debug;
use pretty_hex::*;

#[derive(Clone)]
pub enum Direction {
    MasterToMotor,
    MasterToBattery,
    MotorToMaster,
    BatteryToMaster,
}

impl Direction {
    fn value(&self) -> u8 {
        match self {
            Direction::MasterToMotor => 0x20,
            Direction::MasterToBattery => 0x22,
            Direction::MotorToMaster => 0x23,
            Direction::BatteryToMaster => 0x25,
        }
    }
}

#[derive(Clone)]
pub enum ReadWrite {
    Read,
    Write,
}

impl ReadWrite {
    fn value(&self) -> u8 {
        match self {
            ReadWrite::Read => 0x01,
            ReadWrite::Write => 0x03,
        }
    }
}

#[derive(Clone)]
pub enum Attribute {
    GeneralInfo,
    MotorInfo,
    DistanceLeft,
    Speed,
    TripDistance,
    BatteryVoltage,
    BatteryCurrent,
    BatteryPercent,
    BatteryCellVoltages,
    Supplementary,
    Cruise,
    TailLight,
    BatteryInfo,
    Kers,
}

impl Attribute {
    fn value(&self) -> u8 {
        match self {
            Attribute::GeneralInfo => 0x10,
            Attribute::DistanceLeft => 0x25,
            Attribute::Speed => 0xB5,
            Attribute::TripDistance => 0xB9,
            Attribute::BatteryVoltage => 0x34,
            Attribute::BatteryCurrent => 0x33,
            Attribute::BatteryPercent => 0x32,
            Attribute::MotorInfo => 0xB0,
            Attribute::BatteryCellVoltages => 0x40,
            Attribute::Supplementary => 0x7B,
            Attribute::Cruise => 0x7C,
            Attribute::TailLight => 0x7D,
            Attribute::BatteryInfo => 0x31,
            Attribute::Kers => 0x7E,
        }
    }
}

#[derive(Clone)]
pub struct ScooterCommand {
    pub direction: Direction,
    pub read_write: ReadWrite,
    pub attribute: Attribute,
    pub payload: Vec<u8>,
}

impl Debug for ScooterCommand {
    fn fmt(&self, form: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let message = format!("{:?}", self.as_bytes().hex_dump());
        form.write_str(&message).unwrap();
        Ok(())
    }
}

impl ScooterCommand {
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![
            self.payload.len() as u8 + 2u8,
            self.direction.value(),
            self.read_write.value(),
            self.attribute.value(),
        ];
        bytes.extend_from_slice(&self.payload);
        bytes
    }
}
