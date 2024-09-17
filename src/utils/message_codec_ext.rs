use tokio::io::{AsyncRead, AsyncReadExt};

// pub trait MessageCodecReader: AsyncRead {
//     async fn read_command_type(&mut self) -> crate::utils::result::Result<CommandType>
//     where
//         Self: Unpin,
//     {
//         return match self.read_u16().await {
//             Ok(command_type) => Result::Ok(command_type),
//             Err(err) => Result::Err(Box::new(err)),
//         };
//     }
// }