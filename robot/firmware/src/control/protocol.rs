use embassy_stm32::{mode::Async, usart::{Error, Uart}};
use navign_shared::robot::motion::{MotionCommand, Odometry, SetVelocity, SetWeels};

pub enum ProtocolError {
    UartError(Error),
    PostcardError(postcard::Error)
}

pub struct ProtocolHandler<'a> {
    uart: Uart<'a, Async>,
}

impl<'a> ProtocolHandler<'a> {
    pub fn new(uart: Uart<'a, Async>) -> Self {
        Self { uart }
    }

    pub async fn send_odometry(&mut self, odom: &Odometry) -> Result<(), ProtocolError> {
        // Serialize and send odometry data over UART
        let serialized = postcard::to_vec::<Odometry, 256>(odom).map_err(ProtocolError::PostcardError)?;
        self.uart.write(serialized.as_slice()).await.map_err(ProtocolError::UartError)?;
        Ok(())
    }

    pub async fn receive_command(&mut self) -> Result<MotionCommand, ProtocolError> {
        // Read data from UART and deserialize into MotionCommand
        let mut buffer = [0u8; 256];
        self.uart.read(&mut buffer).await.map_err(ProtocolError::UartError)?;
        let command: MotionCommand = postcard::from_bytes(&buffer).map_err(ProtocolError::PostcardError)?;
        Ok(command)
    }
}
