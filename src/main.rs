use std::num::NonZeroU32;

use anyhow::Result;
use embedded_hal::delay::DelayNs;
use esp_idf_svc::hal::{
    spi,
    delay::FreeRtos, gpio::{self, PinDriver, InterruptType, Pull}, i2c::{I2cConfig, I2cDriver}, peripherals::Peripherals, prelude::*, spi::{SpiDriver, SPI2},
    task::notification::Notification
};
use esp_idf_hal;
use log;

use esp_ir::lepton_error::LepStatus;

use esp_ir::lepton::Lepton;


fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    //setup i2c interface
    let sda = peripherals.pins.gpio7;
    let scl = peripherals.pins.gpio8;

    //setup reset pin
    let mut reset_l = PinDriver::output(peripherals.pins.gpio5)?;


    //startup sequence
    log::info!("starting camera boot sequence");
    reset_l.set_low()?;
    FreeRtos.delay_ms(500u32);
    reset_l.set_high()?;
    FreeRtos.delay_ms(10000u32);
    log::info!("completed camera boot sequence");


    let i2c_config = I2cConfig::new().baudrate(400.kHz().into());
    let mut i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &i2c_config)?;

    let miso = peripherals.pins.gpio4;
    let mosi = peripherals.pins.gpio3;
    let spi_clk = peripherals.pins.gpio2;

    let spi_driver_config = spi::config::DriverConfig::new();
    let mut spi_driver = SpiDriver::new(
        peripherals.spi2, spi_clk, mosi, Some(miso), &spi_driver_config)?;

    let spi_config = spi::config::Config::new().baudrate(10.MHz().into());
    let spi = spi::SpiBusDriver::new(spi_driver, &spi_config)?;

    let mut lepton = Lepton::new(i2c, spi)?;

    let mut vsync = PinDriver::input(peripherals.pins.gpio21)?;
    vsync.set_pull(gpio::Pull::Down)?;
    vsync.set_interrupt_type(InterruptType::PosEdge)?;

    let notification = Notification::new();
    let notifier = notification.notifier();

    unsafe {
        vsync.subscribe(move || {
            notifier.notify_and_yield(NonZeroU32::new(1).unwrap());
        })?;
    }

    loop {
        loop {
            let boot_status = match lepton.get_boot_status() {
                Ok(false) => {log::info!("retrying camera in 5 seconds"); false},
                Ok(true) => {log::info!("Camera booted"); true},
                Err(e) => {log::error!("ERROR: {:?}, retrying in 5 seconds", e); false}
            };
            if boot_status {break}
            FreeRtos.delay_ms(5000u32);
        }

        log::info!("setting phase delay: {:?}", lepton.set_phase_delay(3)?);

        // log::info!("setting gpio mode: {:?}", lepton.set_gpio_mode(5)?);

        // log::info!("gpio mode: {}", lepton.get_gpio_mode()?);
        let (phase_delay, status) = lepton.get_phase_delay()?;
        log::info!("phase delay: {} status: {}", phase_delay, status);

        log::info!("Camera booted successfuly, waiting for frame");
        vsync.enable_interrupt()?;
        notification.wait(esp_idf_svc::hal::delay::BLOCK);
        log::info!("reading frame");
        lepton.read_frame()?;
        for chunk in lepton.get_frame().chunks(256) {
            log::info!("{:?}", chunk);
        }
    }
    Ok(())
}
