use std::io;

#[derive(Debug)]
pub enum LepStatus {
      OK,
      CommOK,
      Error,
      NotReady,
      RangeError,
      ChecksumError,
      BadArgPointerError,
      DataSizeError,
      UndefinedFunctionError,
      FunctionNotSupported,
      DataOutOfRangeError,
      CommandNotAllowed,

      //OTP access errors
      OTPWriteError,
      OTPReadError,
      OTPNotProgrammedError,

      //I2C Errors
      I2CBusNotReady,
      I2CBufferOverflow,
      I2CArbitrationLost,
      I2CBusError,
      I2CNackReceived,
      I2CFail,

      //Processing errors
      DivZeroError,

      //Comm errors
      CommPortNotOpen,
      CommInvalidPortError,
      CommRangeError,
      ErrorCreatingComm,
      ErrorStartingComm,
      ErrorClosingComm,
      CommChecksumError,
      CommNoDev,
      TimeoutError,
      CommErrorWritingComm,
      CommErrorReadingComm,
      CommCountError,

      //Other
      OperationCanceled,
      UndefinedErrorCode
}

impl From<i8> for LepStatus {
    fn from(value: i8) -> Self {
        match value {
            0 => LepStatus::OK,
            -1 => LepStatus::Error,
            -2 => LepStatus::NotReady,
            -3 => LepStatus::RangeError,
            -4 => LepStatus::ChecksumError,
            -5 => LepStatus::BadArgPointerError,
            -6 => LepStatus::DataSizeError,
            -7 => LepStatus::UndefinedFunctionError,
            -8 => LepStatus::FunctionNotSupported,
            -9 => LepStatus::DataOutOfRangeError,
            -11 => LepStatus::CommandNotAllowed,
            -15 => LepStatus::OTPWriteError,
            -16 => LepStatus::OTPReadError,
            -18 => LepStatus::OTPNotProgrammedError,
            -20 => LepStatus::I2CBusNotReady,
            -22 => LepStatus::I2CBufferOverflow,
            -23 => LepStatus::I2CArbitrationLost,
            -24 => LepStatus::I2CBusError,
            -25 => LepStatus::I2CNackReceived,
            -26 => LepStatus::I2CFail,
            -80 => LepStatus::DivZeroError,
            -101 => LepStatus::CommPortNotOpen,
            -102 => LepStatus::CommInvalidPortError,
            -103 => LepStatus::CommRangeError,
            -104 => LepStatus::ErrorCreatingComm,
            -105 => LepStatus::ErrorStartingComm,
            -106 => LepStatus::ErrorClosingComm,
            -107 => LepStatus::CommChecksumError,
            -108 => LepStatus::CommNoDev,
            -109 => LepStatus::TimeoutError,
            -110 => LepStatus::CommErrorWritingComm,
            -111 => LepStatus::CommErrorReadingComm,
            -112 => LepStatus::CommCountError,
            -126 => LepStatus::OperationCanceled,
            -127 => LepStatus::UndefinedErrorCode,
            _ => LepStatus::UndefinedErrorCode,
        }
    }
}

impl Into<i8> for LepStatus {
    fn into(self) -> i8 {
        match self {
            LepStatus::OK => 0,
            LepStatus::CommOK => 0,
            LepStatus::Error => -1,
            LepStatus::NotReady => -2,
            LepStatus::RangeError => -3,
            LepStatus::ChecksumError => -4,
            LepStatus::BadArgPointerError => -5,
            LepStatus::DataSizeError => -6,
            LepStatus::UndefinedFunctionError => -7,
            LepStatus::FunctionNotSupported => -8,
            LepStatus::DataOutOfRangeError => -9,
            LepStatus::CommandNotAllowed => -11,
            LepStatus::OTPWriteError => -15,
            LepStatus::OTPReadError => -16,
            LepStatus::OTPNotProgrammedError => -18,
            LepStatus::I2CBusNotReady => -20,
            LepStatus::I2CBufferOverflow => -22,
            LepStatus::I2CArbitrationLost => -23,
            LepStatus::I2CBusError => -24,
            LepStatus::I2CNackReceived => -25,
            LepStatus::I2CFail => -26,
            LepStatus::DivZeroError => -80,
            LepStatus::CommPortNotOpen => -101,
            LepStatus::CommInvalidPortError => -102,
            LepStatus::CommRangeError => -103,
            LepStatus::ErrorCreatingComm => -104,
            LepStatus::ErrorStartingComm => -105,
            LepStatus::ErrorClosingComm => -106,
            LepStatus::CommChecksumError => -107,
            LepStatus::CommNoDev => -108,
            LepStatus::TimeoutError => -109,
            LepStatus::CommErrorWritingComm => -110,
            LepStatus::CommErrorReadingComm => -111,
            LepStatus::CommCountError => -112,
            LepStatus::OperationCanceled => -126,
            LepStatus::UndefinedErrorCode => -127,
        }
    }
}

impl core::fmt::Display for LepStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            LepStatus::OK => write!(f, "Ok"),
            LepStatus::CommOK => write!(f, "CommOk"),
            LepStatus::Error => write!(f, "Error"),
            LepStatus::NotReady => write!(f, "Not Ready"),
            LepStatus::RangeError => write!(f, "Range Error"),
            LepStatus::ChecksumError => write!(f, "Checksum Error"),
            LepStatus::BadArgPointerError => write!(f, "Bad Argument Pointer Error"),
            LepStatus::DataSizeError => write!(f, "Data Size Error"),
            LepStatus::UndefinedFunctionError => write!(f, "Undefined Function Error"),
            LepStatus::FunctionNotSupported => write!(f, "Function Not Supported"),
            LepStatus::DataOutOfRangeError => write!(f, "Data Out of Range"),
            LepStatus::CommandNotAllowed => write!(f, "Command Not Allowed"),
      
            //OTP access errors
            LepStatus::OTPWriteError => write!(f, "OTP Write Error"),
            LepStatus::OTPReadError => write!(f, "OTP Read Error"),
            LepStatus::OTPNotProgrammedError => write!(f, "OTP Not Programmed Error"),
      
            //I2C Errors
            LepStatus::I2CBusNotReady => write!(f, "I2C Bus Not Ready"),
            LepStatus::I2CBufferOverflow => write!(f, "I2C Buffer Overflow"),
            LepStatus::I2CArbitrationLost => write!(f, "I2C Arbitration Lost"),
            LepStatus::I2CBusError => write!(f, "I2C Bus Error"),
            LepStatus::I2CNackReceived => write!(f, "I2C Nack Received"),
            LepStatus::I2CFail => write!(f, "I2C Fail"),
      
            //Processing errors
            LepStatus::DivZeroError => write!(f, "Div Zero Error"),
      
            //Comm errors
            LepStatus::CommPortNotOpen => write!(f, "Comm Port Not Open"),
            LepStatus::CommInvalidPortError => write!(f, "Comm Invalid Port Error"),
            LepStatus::CommRangeError => write!(f, "Comm Range Error"),
            LepStatus::ErrorCreatingComm => write!(f, "Error Creating Comm"),
            LepStatus::ErrorStartingComm => write!(f, "Error Starting Comm"),
            LepStatus::ErrorClosingComm => write!(f, "Error Closing Commm"),
            LepStatus::CommChecksumError => write!(f, "Comm Checksum Error"),
            LepStatus::CommNoDev => write!(f, "Comm No Dev"),
            LepStatus::TimeoutError => write!(f, "Timeout Error"),
            LepStatus::CommErrorWritingComm => write!(f, "Comm Error Writing Comm"),
            LepStatus::CommErrorReadingComm => write!(f, "Comm Error Reading Comm"),
            LepStatus::CommCountError => write!(f, "Comm Count Error Reading Comm"),
      
            //Other
            LepStatus::OperationCanceled  => write!(f, "Operation Canceled"),
            LepStatus::UndefinedErrorCode  => write!(f, "Undefined Error Code"),
        }
    }
}

impl std::error::Error for LepStatus {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
