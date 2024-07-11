use crate::lepton_command::LepCommand;
use embedded_hal::i2c::I2c;
use esp_idf_svc::hal::delay;
use crate::lepton_error::LepStatus;

macro_rules! generate_get_set_functions {
    (
        $set_fn_name:ident, $get_fn_name:ident, $param_ty:ty, $set_command:expr, $get_command:expr
    ) => {
        pub fn $set_fn_name(&mut self, value: $param_ty) -> Result<LepStatus, E> {
            self.write_register(Register::CCIDataReg0, &value.to_be_bytes())?;
            let command = $set_command;
            self.write_command(command)?;
            self.poll_status()?;
            self.get_status_code()
        }

        pub fn $get_fn_name(&mut self) -> Result<($param_ty, LepStatus), E> {
            let command = $get_command;
            self.write_command(command)?;
            self.poll_status()?;
            let data = self.read_register(Register::CCIDataReg0)?;
            let status_code = self.get_status_code()?;
            Ok((data.try_into().unwrap(), status_code))
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LEPTONCCI <I2C> {
    i2c: I2C,
    address: u8,
}

impl<I2C, E> LEPTONCCI<I2C>
where 
    I2C: I2c<Error = E>, E: core::fmt::Debug
    {


    pub fn new(i2c: I2C) -> Result<Self, E> {

        Ok(LEPTONCCI {i2c, address:0x2a})
    }


    pub fn get_boot_status(&mut self) -> Result<bool, E> {
        let response = self.read_register(Register::CCIStatus)?;
        //camera has booted if bit 2 is 1
        Ok(response & (0b0000_00010) != 0)
    }


    pub fn get_interface_status(&mut self) -> Result<bool, E> {
        let response = self.read_register(Register::CCIStatus)?;
        Ok(response & (0b0000_0001) == 0)
    }

    pub fn get_status_code(&mut self) -> Result<LepStatus, E> {
        let response = self.read_register(Register::CCIStatus)?;
        let status = (response >> 8) as u8;
        Ok(LepStatus::from(status as i8))
    }

    generate_get_set_functions!(
        set_oem_video_output_source, get_oem_video_output_source, u16, 
        LepCommand::set_oem_video_output_source(), 
        LepCommand::get_oem_video_output_source()
    );

    generate_get_set_functions!(
        set_oem_video_output_constant, get_oem_video_output_constant, u16,
        LepCommand::set_oem_video_output_source_constant(),
        LepCommand::get_oem_video_output_source_constant()
    );
    
    generate_get_set_functions!(
       set_gpio_mode, get_gpio_mode, u16, 
        LepCommand::set_oem_gpio_mode(), 
        LepCommand::get_oem_gpio_mode()
    );

    generate_get_set_functions!(
        set_phase_delay, get_phase_delay, i16, 
         LepCommand::set_oem_phase_delay(), 
         LepCommand::get_oem_phase_delay()
     );

    generate_get_set_functions!(
        set_telemetry_mode, get_telemetry_mode, u16,
        LepCommand::set_sys_telemetry_mode(),
        LepCommand::get_sys_telemetry_mode()
    );

    generate_get_set_functions!(
        set_agc_enable, get_agc_enable, u16,
        LepCommand::set_agc_enable(),
        LepCommand::get_agc_enable()
    );

    /// Writes into a register
    #[allow(unused)]
    fn write_register(&mut self, register: Register, payload: &[u8]) -> Result<(), E> {
        // Value that will be written as u8
        let mut write_vec = std::vec::Vec::with_capacity(2 + payload.len());
        let address = register.address().to_be_bytes();
        write_vec.extend_from_slice(&address);
        write_vec.extend_from_slice(payload);
        // i2c write
        self.i2c
        .write(self.address as u8, &write_vec)
    }

    //Write a command
    fn write_command(&mut self, command: LepCommand) -> Result<(), E> {
        let command_id = command.get_command_id();
        let data_length = command.get_data_length();
        self.write_register(Register::CCIDataLength, &data_length)?;
        self.write_register(Register::CCICommandID, &command_id)
    }

    /// Reads a register using a `write_read` method.
    fn read_register(&mut self, register: Register) -> Result<u16, E> {
        // Buffer for values
        let mut data: [u8; 2] = [0; 2];
        // i2c write_read
        self.i2c
        .write_read(self.address as u8, &register.address().to_be_bytes(), &mut data)?;
        Ok(u16::from_be_bytes(data))
    }

    fn poll_status(&mut self) -> Result<(), E> {
        loop {
            let command_finished = self.get_interface_status()?;
            if command_finished {break} else {let delay = delay::Delay::new_default(); delay.delay_ms(1);}
        }
        Ok(())
    }
}


#[derive(Clone, Copy)]
pub enum Register {
    CCIPower = 0x0000,
    CCIStatus = 0x0002,
    CCICommandID = 0x0004,
    CCIDataLength = 0x0006,
    CCIDataReg0 = 0x0008
}

impl Register {
    fn address(&self) -> u16 {
        *self as u16
    }
}
