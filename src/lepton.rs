use std::fmt;

use embedded_hal::{i2c::I2c, spi};
use crate::lepton_cci::LEPTONCCI;
use crate::lepton_error::LepStatus;

const PACKETSIZE:usize = 164;
const FRAMEPACKETS:usize = 60;
extern crate esp_idf_sys as esp;

pub struct Lepton <I2C, SPI> {
    cci: LEPTONCCI<I2C>,
    spi: SPI,
    frame: Box<[u8; FRAMEPACKETS * PACKETSIZE]>
}

impl<I2C, SPI, E1> Lepton<I2C, SPI> 
    where
    I2C: I2c<Error = E1>,
    SPI: spi::SpiDevice,
    E1: core::fmt::Debug,
    {
        pub fn new(i2c: I2C, spi: SPI) -> Result<Self, E1> {
            let cci = LEPTONCCI::new(i2c)?;
            Ok( Lepton { cci, spi, frame: Box::new([0; FRAMEPACKETS * PACKETSIZE]) } )
        }

        pub fn set_phase_delay(&mut self, phase_delay: i16) -> Result<LepStatus,  LeptonError<E1, SPI::Error>> {
            self.cci.set_phase_delay(phase_delay).map_err(LeptonError::I2c)?;
            self.cci.get_status_code().map_err(LeptonError::I2c)
        }

        pub fn get_phase_delay(&mut self) -> Result<(i16, LepStatus),  LeptonError<E1, SPI::Error>> {
            self.cci.get_phase_delay().map_err(LeptonError::I2c)
        }

        pub fn set_gpio_mode(&mut self, gpio_mode: u16) -> Result<LepStatus,  LeptonError<E1, SPI::Error>> {
            self.cci.set_gpio_mode(gpio_mode).map_err(LeptonError::I2c)
        }

        pub fn get_gpio_mode(&mut self) -> Result<(u16, LepStatus),  LeptonError<E1, SPI::Error>> {
            self.cci.get_gpio_mode().map_err(LeptonError::I2c)
        }

        pub fn set_video_output_source(&mut self, source: u16) -> Result<LepStatus,  LeptonError<E1, SPI::Error>> {
            self.cci.set_oem_video_output_source(source).map_err(LeptonError::I2c)
        }

        pub fn get_video_output_source(&mut self) -> Result<(u16, LepStatus),  LeptonError<E1, SPI::Error>> {
            self.cci.get_oem_video_output_source().map_err(LeptonError::I2c)
        }

        pub fn set_video_output_constant(&mut self, constant: u16) -> Result<LepStatus,  LeptonError<E1, SPI::Error>> {
            self.cci.set_oem_video_output_constant(constant).map_err(LeptonError::I2c)
        }

        pub fn get_video_output_constant(&mut self) -> Result<(u16, LepStatus),  LeptonError<E1, SPI::Error>> {
            self.cci.get_oem_video_output_constant().map_err(LeptonError::I2c)
        }

        pub fn get_boot_status(&mut self) -> Result<bool,  LeptonError<E1, SPI::Error>> {
            self.cci.get_boot_status().map_err(LeptonError::I2c)
        }

        pub fn get_interface_status(&mut self) -> Result<bool,  LeptonError<E1, SPI::Error>> {
            self.cci.get_interface_status().map_err(LeptonError::I2c)
        }

        pub fn set_telemetry_mode(&mut self, mode: u16) -> Result<LepStatus,  LeptonError<E1, SPI::Error>> {
            self.cci.set_telemetry_mode(mode).map_err(LeptonError::I2c)
        }

        pub fn get_telemetry_mode(&mut self) -> Result<(u16, LepStatus),  LeptonError<E1, SPI::Error>> {
            self.cci.get_telemetry_mode().map_err(LeptonError::I2c)
        }

        pub fn get_agc_enable(&mut self) -> Result<(u16, LepStatus),  LeptonError<E1, SPI::Error>> {
            self.cci.get_agc_enable().map_err(LeptonError::I2c)
        }

        pub fn set_agc_enable(&mut self, mode: u16) -> Result<LepStatus,  LeptonError<E1, SPI::Error>> {
            self.cci.set_agc_enable(mode).map_err(LeptonError::I2c)
        }

        pub fn read_frame(&mut self) -> Result<Vec<u8>,  LeptonError<E1, SPI::Error>> {

            let first_packet: [u8; PACKETSIZE];

            loop {
                match self.check_packet() {
                    Ok(packet) => {
                        if u16::from_be_bytes([packet[0], packet[1]]) == 0 {first_packet = packet; break}
                    }
                    Err(_) => {}
                }
            }

            let mut frame = vec![0u8; FRAMEPACKETS * PACKETSIZE];

            frame[..PACKETSIZE].copy_from_slice(&first_packet);

            self.spi.read(&mut frame[PACKETSIZE..]).map_err(LeptonError::Spi)?;

            Ok(frame)
        }

        fn check_packet(&mut self) -> Result<[u8; PACKETSIZE], LeptonError<I2C, SPI::Error>> {
            let mut packet = [0 as u8; PACKETSIZE];
            self.spi.read(&mut packet).map_err(LeptonError::Spi)?;

            return Ok(packet)
        }

        pub fn get_frame(&mut self) -> &Box<[u8; FRAMEPACKETS * PACKETSIZE]> {
            &self.frame
        }

        pub fn set_frame(&mut self, data: &[u8]) -> Result<(), &'static str> {
            if data.len() != FRAMEPACKETS * PACKETSIZE {
                return Err("Data length does not match frame buffer size");
            }
            self.frame.copy_from_slice(data);
            Ok(())
        }
    


    }


#[derive(Debug)]
pub enum LeptonError<I2C, SPI> {
    Spi(SPI),
    I2c(I2C)
}

// Implement Display for LeptonError
impl<I2C: fmt::Debug, SPI: fmt::Debug> fmt::Display for LeptonError<I2C, SPI> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LeptonError::Spi(e) => write!(f, "SPI Error: {:?}", e),
            LeptonError::I2c(e) => write!(f, "I2C Error: {:?}", e),
        }
    }
}

impl<I2C: fmt::Debug + fmt::Display, SPI: fmt::Debug + fmt::Display> std::error::Error for LeptonError<I2C, SPI> {}
