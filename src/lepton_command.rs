#[derive(Clone, Copy, PartialEq)]
pub enum Module {
    AGC = 0x100,
    SYS = 0x200,
    VID = 0x300,
    OEM = 0x800,
    RAD = 0xE00
}

#[derive(Clone, Copy)]
pub enum CommandType {
    Get = 0x0,
    Set = 0x1,
    Run = 0x2,
    Invalid = 0x3
}

impl std::ops::Add<u16> for CommandType {
    type Output = u16;

    fn add(self, rhs: u16) -> u16 {
        self as u16 + rhs
    }
}

impl std::ops::Add<u16> for Module {
    type Output = u16;

    fn add(self, rhs: u16) -> u16 {
        self as u16 + rhs
    }
}

pub struct LepCommand {
    command_id: u16,
    data_length: u16

}

macro_rules! lep_command_fn {
    ($fn_name:ident, $module:expr, $command_type:expr, $base_id:expr, $data_length:expr) => {
        pub fn $fn_name() -> LepCommand {
            LepCommand::new($module, $command_type, $base_id, $data_length)
        }
    };
}

#[allow(unused)]
impl LepCommand {
    fn new(module: Module, command_type: CommandType, base_id: u16, data_length: u16) -> Self {

        let mut command_id = module as u16 + command_type as u16 + base_id;

        match module {
            Module::OEM => {command_id += 0b0100_0000_0000_0000}
            Module::RAD => {command_id += 0b0100_0000_0000_0000}
            _ => {}
        };

        LepCommand { command_id, data_length }
    }

    pub fn get_command_id(&self) -> [u8; 2] {
        self.command_id.to_be_bytes()
    }

    pub fn get_data_length(&self) -> [u8; 2] {
        self.data_length.to_be_bytes()
    }

    lep_command_fn!(set_agc_enable, Module::AGC, CommandType::Set, 0x00, 2);
    lep_command_fn!(get_agc_enable, Module::AGC, CommandType::Get, 0x00, 2);
    lep_command_fn!(set_agc_policy, Module::AGC, CommandType::Set, 0x04, 2);
    lep_command_fn!(get_agc_policy, Module::AGC, CommandType::Set, 0x04, 2);
    lep_command_fn!(set_agc_roi, Module::AGC, CommandType::Set, 0x08, 4);
    lep_command_fn!(get_agc_roi, Module::AGC, CommandType::Get, 0x08, 4);
    lep_command_fn!(get_agc_histogram_statistics, Module::AGC, CommandType::Get, 0x0C, 4);
    lep_command_fn!(set_oem_phase_delay, Module::OEM, CommandType::Set, 0x58, 1);
    lep_command_fn!(get_oem_phase_delay, Module::OEM, CommandType::Get, 0x58, 1);
    lep_command_fn!(set_oem_gpio_mode, Module::OEM, CommandType::Set, 0x54, 1);
    lep_command_fn!(get_oem_gpio_mode, Module::OEM, CommandType::Get, 0x54, 1);
    lep_command_fn!(set_oem_video_output_source, Module::OEM, CommandType::Set, 0x2C, 1);
    lep_command_fn!(get_oem_video_output_source, Module::OEM, CommandType::Get, 0x2C, 1);
    lep_command_fn!(set_oem_video_output_source_constant, Module::OEM, CommandType::Set, 0x3C, 1);
    lep_command_fn!(get_oem_video_output_source_constant, Module::OEM, CommandType::Get, 0x3C, 1);
    lep_command_fn!(set_sys_telemetry_mode, Module::SYS, CommandType::Set, 0x18, 1);
    lep_command_fn!(get_sys_telemetry_mode, Module::SYS, CommandType::Get, 0x18, 1);

}
