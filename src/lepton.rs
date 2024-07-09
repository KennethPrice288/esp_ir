use embedded_hal::spi::SpiBus;
use embedded_hal::{i2c::I2c, spi};
use crate::lepton_cci::LEPTONCCI;
use crate::lepton_error::LepStatus;
use std::ops::{Deref, DerefMut};

const FRAMESIZE:usize = 60*80*2;

pub struct Lepton <I2C, SPI> {
    cci: LEPTONCCI<I2C>,
    spi: SPI,
    frame: Box<[u8; FRAMESIZE]>
}

impl<I2C, SPI, E1, E2> Lepton<I2C, SPI> 
    where
    I2C: I2c<Error = E1>,
    SPI: spi::SpiBus<Error = E2>,
    E1: core::fmt::Debug
    {
        pub fn new(i2c: I2C, spi: SPI) -> Result<Self, E1> {
            let cci = LEPTONCCI::new(i2c)?;
            Ok( Lepton { cci, spi, frame: Box::new([0; FRAMESIZE]) } )
        }

        pub fn set_phase_delay(&mut self, phase_delay: i16) -> Result<LepStatus, E1> {
            self.cci.set_phase_delay(phase_delay);
            self.cci.get_status_code()
        }

        pub fn get_phase_delay(&mut self) -> Result<(u16, LepStatus), E1> {
            self.cci.get_phase_delay()
        }

        pub fn set_gpio_mode(&mut self, gpio_mode: u16) -> Result<LepStatus, E1> {
            self.cci.set_gpio_mode(gpio_mode)
        }

        pub fn get_gpio_mode(&mut self) -> Result<(u16, LepStatus), E1> {
            self.cci.get_gpio_mode()
        }

        pub fn get_boot_status(&mut self) -> Result<bool, E1> {
            self.cci.get_boot_status()
        }

        pub fn get_interface_status(&mut self) -> Result<bool, E1> {
            self.cci.get_interface_status()
        }

        pub fn read_frame(&mut self) -> Result<(), E2> {
            let frame = self.frame.deref_mut();
            for chunk in frame.chunks_mut(256) {
                self.spi.read(chunk)?;
            }
            Ok(())
        }

        pub fn get_frame(&mut self) -> &Box<[u8; FRAMESIZE]> {
            &self.frame
        }
    }
