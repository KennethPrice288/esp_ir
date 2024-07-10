use std::num::NonZeroU32;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use esp_idf_hal::io::Write;
use anyhow::Result;
use embedded_hal::delay::DelayNs;
use esp_idf_hal::i2c::I2cError;
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::{
    delay::FreeRtos, gpio::{self, InterruptType, PinDriver}, i2c::{I2cConfig, I2cDriver}, peripherals::Peripherals, prelude::*, spi::{self, SpiDriver}, task::notification::Notification
}, http::{self, server::EspHttpServer}};

use log;

use esp_ir::lepton::Lepton;

use esp_ir::wifi::wifi;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let sysloop = EspSystemEventLoop::take()?;

    let peripherals = Peripherals::take().unwrap();

    //setup camera

    //setup i2c interface
    let sda = peripherals.pins.gpio7;
    let scl = peripherals.pins.gpio8;

    //setup reset pin
    let mut reset_l = PinDriver::output(peripherals.pins.gpio5)?;

    //setup cs pin
    let mut cs_l = PinDriver::output(peripherals.pins.gpio2)?;


    //startup sequence
    log::info!("starting camera boot sequence");
    reset_l.set_low()?;
    FreeRtos.delay_ms(500u32);
    reset_l.set_high()?;
    FreeRtos.delay_ms(10000u32);
    log::info!("completed camera boot sequence");


    let i2c_config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &i2c_config)?;

    let miso = peripherals.pins.gpio0;
    let mosi = peripherals.pins.gpio9;
    let spi_clk = peripherals.pins.gpio3;

    let spi_driver_config = spi::config::DriverConfig::new();

    let spi_driver = SpiDriver::new(
        peripherals.spi2, spi_clk, mosi, Some(miso), &spi_driver_config)?;

    let spi_config = spi::config::Config::new().baudrate(10.MHz().into()).data_mode(spi::config::MODE_3);
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

    let frame_data = Arc::new(Mutex::new(lepton.get_frame().clone()));

    //start http server
    let app_config = CONFIG;

    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    )?;

    log::warn!("about to start server");

    let mut server = EspHttpServer::new(&http::server::Configuration::default())?;
    log::warn!("server started");
    let frame_data_clone = frame_data.clone();

    server.fn_handler(
        "/",
        http::Method::Get,
        move |request| -> core::result::Result<(), esp_idf_svc::io::EspIOError> {
            let mut frame_data = frame_data_clone.lock().unwrap();
            let mut response = request.into_ok_response()?;
            log::info!("there are: {} bytes in the frame", frame_data.len());
            // response.write_all(frame_data.deref_mut().deref_mut())?;
            let mut total_bytes_transferred = 0;
            for chunk in frame_data.chunks(512) {
                log::info!("transferred {} bytes", chunk.len());
                response.write_all(chunk)?;
                total_bytes_transferred += chunk.len();
            }
            log::info!("transferred {} bytes total", total_bytes_transferred);
            Ok(())
        },
    )?;

    setup_camera(&mut lepton)?;
    loop {

        // log::info!("Camera booted successfuly, waiting for frame");
        vsync.enable_interrupt()?;
        notification.wait(esp_idf_svc::hal::delay::BLOCK);
        // log::info!("reading frame");
        cs_l.set_low()?;
        lepton.read_frame()?;
        let mut frame_data = frame_data.lock().unwrap();
        *frame_data = lepton.get_frame().clone();
        cs_l.set_high()?;
    }
    Ok(())
}

fn setup_camera<'a>(lepton: &mut Lepton<I2cDriver, spi::SpiBusDriver<'a, spi::SpiDriver<'a>>>) -> Result<(), I2cError> {
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

    log::info!("setting gpio mode: {:?}", lepton.set_gpio_mode(5)?);

    // log::info!("setting video constant: {:?}", lepton.set_video_output_constant(1)?);

    // log::info!("changing video output to constant: {:?}", lepton.set_video_output_source(3)?);

    let (gpio_mode, gpio_command_status) = lepton.get_gpio_mode()?;
    log::info!("gpio mode: {} status: {}", gpio_mode, gpio_command_status);

    let (phase_delay, phase_delay_command_status) = lepton.get_phase_delay()?;
    log::info!("phase delay: {} status: {}", phase_delay, phase_delay_command_status);
    Ok(())
}
