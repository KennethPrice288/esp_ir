use std::cell::RefCell;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use esp_idf_hal::io::Write;
use anyhow::Result;
use embedded_hal::delay::DelayNs;
use esp_idf_hal::i2c::I2cError;
use esp_idf_svc::{eventloop::EspSystemEventLoop, hal::{
    delay::FreeRtos, gpio::{self, InterruptType, PinDriver}, i2c::{I2cConfig, I2cDriver}, peripherals::Peripherals, prelude::*, spi::{self, SpiDriver}, task::notification::Notification
}, http::{self, server::EspHttpServer}};
use esp_ir::lepton::LeptonError;

const PACKETSIZE:usize = 164;

static CRC:crc::Crc::<u16> = crc::Crc::<u16>::new(&crc::CRC_16_XMODEM);
static LEPTON: Mutex<RefCell<Option<Lepton<I2cDriver, spi::SpiDeviceDriver<SpiDriver>>>>> = Mutex::new(RefCell::new(None));

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

#[allow(unreachable_code)]
fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    let sysloop = EspSystemEventLoop::take()?;

    unsafe {
        // esp_idf_sys::esp_task_wdt_deinit();
    }

    let peripherals = Peripherals::take().unwrap();

    //setup camera

    //setup i2c interface
    let sda = peripherals.pins.gpio7;
    let scl = peripherals.pins.gpio8;

    //setup reset pin
    let mut reset_l = PinDriver::output(peripherals.pins.gpio5)?;

    //setup cs pin
    // let mut cs_l = PinDriver::output(peripherals.pins.gpio2)?;
    let cs_l = peripherals.pins.gpio6;

    //startup sequence
    log::info!("starting camera boot sequence");
    reset_l.set_low()?;
    FreeRtos.delay_ms(500u32);
    reset_l.set_high()?;
    FreeRtos.delay_ms(10000u32);
    log::info!("completed camera boot sequence");


    let i2c_config = I2cConfig::new().baudrate(400.kHz().into());
    let i2c = I2cDriver::new(peripherals.i2c0, sda, scl, &i2c_config)?;

    let miso = peripherals.pins.gpio1;
    let mosi = peripherals.pins.gpio9;
    let spi_clk = peripherals.pins.gpio3;

    let spi_driver_config = spi::config::DriverConfig::new();

    let spi_driver = SpiDriver::new(
        peripherals.spi2, spi_clk, mosi, Some(miso), &spi_driver_config)?;

    let spi_config = spi::config::Config::new().baudrate(20.MHz().into()).data_mode(spi::config::MODE_3);
    // let spi = spi::SpiBusDriver::new(spi_driver, &spi_config)?;
    let spi = spi::SpiDeviceDriver::new(spi_driver, Some(cs_l), &spi_config)?;

    let mut lepton = Lepton::new(i2c, spi)?;

    let mut vsync = PinDriver::input(peripherals.pins.gpio4)?;
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


    let mut server = EspHttpServer::new(&http::server::Configuration::default())?;
    let frame_data_clone = frame_data.clone();

    server.fn_handler(
        "/",
        http::Method::Get,
        move |request| -> core::result::Result<(), esp_idf_svc::io::EspIOError> {

            log::info!("responding to request");

            let mut response = request.into_ok_response()?;
            {
            let frame_data = frame_data_clone.lock().unwrap();

            response.write_all(&**frame_data)?;
            }
            log::info!("finished responding to request");

            Ok(())
        },
    )?;

    setup_camera(&mut lepton)?;
    // Get the handle of the currently running task
    let current_task = unsafe { esp_idf_sys::xTaskGetCurrentTaskHandle() };

    // Get the current priority of the task
    let current_priority = unsafe { esp_idf_sys::uxTaskPriorityGet(current_task) };

    // Set the task priority to a high value (e.g., configMAX_PRIORITIES - 1)
    let high_priority = esp_idf_sys::configMAX_PRIORITIES - 1;
    loop {

        log::info!("waiting for frame");
        vsync.enable_interrupt()?;
        notification.wait(esp_idf_svc::hal::delay::BLOCK);
        log::info!("reading frame");

        
        
        unsafe {
            esp_idf_sys::vTaskPrioritySet(current_task, high_priority);
        }
        let mut frame_data = frame_data.lock().unwrap();
        // unsafe {esp_idf_sys::vTaskSuspendAll();}
        let read_frame_result = lepton.read_frame();
        // unsafe {esp_idf_sys::xTaskResumeAll();}
        unsafe {
            esp_idf_sys::vTaskPrioritySet(current_task, high_priority);
        }
        log::info!("frame read");
            match read_frame_result {
                Ok(frame) => {
                    log::info!("received frame");
                    let mut discard = false;
                    for packet in frame.chunks(PACKETSIZE) {
                        if packet[0] & 0x0F == 0x0F {
                            discard = true;
                            log::warn!("DISCARD");
                            break;
                        }

                        if !check_crc(packet) {
                            log::warn!("DESYNC");
                            discard = true;
                            break;
                        }
                    }
                    if !discard {
                    log::info!("setting frame");
                    lepton.set_frame(&frame).unwrap();
                    *frame_data = lepton.get_frame().clone();
                    } else {
                        log::warn!("found discard, trying to resync");
                        FreeRtos::delay_ms(500);

                    }
                },
                Err(_) => {log::warn!("SPI Error!")}
        }
    }
    Ok(())
}

fn check_crc(packet: &[u8]) -> bool {

    let mut data = Vec::new();
    data.extend_from_slice(packet);
    data[0] &= 0x0F; //clear 4 msb of ID
    data[2] &= 0x00;
    data[3] &= 0x00; //clear entire CRC field

    CRC.checksum(&data) == u16::from_be_bytes([packet[2], packet[3]])
}

fn setup_camera<'a>(lepton: &mut Lepton<I2cDriver, spi::SpiDeviceDriver<'a, spi::SpiDriver<'a>>>) -> Result<(), LeptonError<I2cError, spi::SpiError>> {
    loop {
        let boot_status = match lepton.get_boot_status() {
            Ok(false) => {log::info!("retrying camera in 5 seconds"); false},
            Ok(true) => {log::info!("Camera booted"); true},
            Err(e) => {log::error!("ERROR: {:?}, retrying in 5 seconds", e); false}
        };
        if boot_status {break}
        FreeRtos.delay_ms(5000u32);
    }

    log::info!("setting phase delay: {:?}", lepton.set_phase_delay(-3)?);

    log::info!("setting gpio mode: {:?}", lepton.set_gpio_mode(5)?);

    log::info!("setting AGC enable: {:?}", lepton.set_agc_enable(1)?);

    // log::info!("setting video constant: {:?}", lepton.set_video_output_constant(0xFFFF)?);

    // log::info!("changing video output to constant: {:?}", lepton.set_video_output_source(3)?);

    let (telemetry_mode, telemetry_mode_command_status) = lepton.get_telemetry_mode()?;
    log::info!("telemetry mode: {} status: {}", telemetry_mode, telemetry_mode_command_status);

    let (gpio_mode, gpio_command_status) = lepton.get_gpio_mode()?;
    log::info!("gpio mode: {} status: {}", gpio_mode, gpio_command_status);

    let (phase_delay, phase_delay_command_status) = lepton.get_phase_delay()?;
    log::info!("phase delay: {} status: {}", phase_delay, phase_delay_command_status);

    let (video_constant, video_constant_get_command_status) = lepton.get_video_output_constant()?;
    log::info!("video constant: {} status: {}", video_constant, video_constant_get_command_status);

    let (video_source, video_source_get_command_status) = lepton.get_video_output_source()?;
    log::info!("video source: {} status: {}", video_source, video_source_get_command_status);

    let (agc_enable, agc_enable_status) = lepton.get_agc_enable()?;
    log::info!("agc enable: {} status: {}", agc_enable, agc_enable_status);

    Ok(())
}
