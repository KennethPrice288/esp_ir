use embedded_hal::{i2c::I2c, spi};
use crate::lepton_cci::LEPTONCCI;
use crate::lepton_error::LepStatus;
use std::ops::DerefMut;
// use esp_idf_svc::hal::delay::FreeRtos;

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
    SPI: spi::SpiDevice<Error = esp_idf_hal::spi::SpiError>,
    E1: core::fmt::Debug,
    {
        pub fn new(i2c: I2C, spi: SPI) -> Result<Self, E1> {
            let cci = LEPTONCCI::new(i2c)?;
            Ok( Lepton { cci, spi, frame: Box::new([0; FRAMEPACKETS * PACKETSIZE]) } )
        }

        pub fn set_phase_delay(&mut self, phase_delay: i16) -> Result<LepStatus, E1> {
            self.cci.set_phase_delay(phase_delay)?;
            self.cci.get_status_code()
        }

        pub fn get_phase_delay(&mut self) -> Result<(i16, LepStatus), E1> {
            self.cci.get_phase_delay()
        }

        pub fn set_gpio_mode(&mut self, gpio_mode: u16) -> Result<LepStatus, E1> {
            self.cci.set_gpio_mode(gpio_mode)
        }

        pub fn get_gpio_mode(&mut self) -> Result<(u16, LepStatus), E1> {
            self.cci.get_gpio_mode()
        }

        pub fn set_video_output_source(&mut self, source: u16) -> Result<LepStatus, E1> {
            self.cci.set_oem_video_output_source(source)
        }

        pub fn get_video_output_source(&mut self) -> Result<(u16, LepStatus), E1> {
            self.cci.get_oem_video_output_source()
        }

        pub fn set_video_output_constant(&mut self, constant: u16) -> Result<LepStatus, E1> {
            self.cci.set_oem_video_output_constant(constant)
        }

        pub fn get_video_output_constant(&mut self) -> Result<(u16, LepStatus), E1> {
            self.cci.get_oem_video_output_constant()
        }

        pub fn get_boot_status(&mut self) -> Result<bool, E1> {
            self.cci.get_boot_status()
        }

        pub fn get_interface_status(&mut self) -> Result<bool, E1> {
            self.cci.get_interface_status()
        }

        pub fn set_telemetry_mode(&mut self, mode: u16) -> Result<LepStatus, E1> {
            self.cci.set_telemetry_mode(mode)
        }

        pub fn get_telemetry_mode(&mut self) -> Result<(u16, LepStatus), E1> {
            self.cci.get_telemetry_mode()
        }

        pub fn read_frame(&mut self) -> Result<(), esp_idf_hal::spi::SpiError> {

            let mut first_packet = [0 as u8; PACKETSIZE];

            loop {
                match self.check_packet() {
                    Ok(packet) => {
                        if u16::from_be_bytes([packet[0], packet[1]]) == 0 {first_packet = packet; break}
                    }
                    Err(_) => {}
                }
            }

            let frame = self.frame.deref_mut();

            frame[..PACKETSIZE].copy_from_slice(&first_packet);

            self.spi.read(&mut frame[PACKETSIZE..])?;

            let mut structured: Vec<[u8; PACKETSIZE]> = Vec::with_capacity(FRAMEPACKETS);

            for chunk in frame.chunks_exact(PACKETSIZE) {
                let mut array = [0u8; PACKETSIZE];
                array.copy_from_slice(chunk);
                structured.push(array);
            }
        
            // log::info!("{:?}", structured);
            for (i, line) in structured.into_iter().enumerate() {
                log::info!("line: {}, {:?}", i, line);
            }
            Ok(())
        }

        fn check_packet(&mut self) -> Result<[u8; PACKETSIZE], esp_idf_hal::spi::SpiError> {
            let mut packet = [0 as u8; PACKETSIZE];
            self.spi.read(&mut packet)?;

            return Ok(packet)
        }

        pub fn get_frame(&mut self) -> &Box<[u8; FRAMEPACKETS * PACKETSIZE]> {
            &self.frame
        }
    }
