pub mod utils;
mod server;

fn main() {
    println!("Hello, world!");
}

use log::{info, warn};
use std::collections::HashMap;
use std::sync::atomic;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};

type TxIDType = u64;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
type SendResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send>>;

type CommandType = u16;

struct Message {
    header: Header,
    body: Box<dyn MessageBody>,
}

struct Header {
    tx_id: TxIDType,
    command: CommandType,
    len: u32,
}

trait MessageBody {}

async fn decode_message<T: AsyncRead + Unpin, R: MessageCodec>(
    codec: &R,
    reader: &mut T,
) -> Result<Message> {
    let header = read_message_head(reader).await?;

    let body = codec.decode(reader).await?;

    return Ok(Message { header, body });
}

async fn read_message_head<T: AsyncRead + Unpin>(reader: &mut T) -> Result<Header> {
    let tx_id = read_tx_id(reader).await?;
    let command = read_command(reader).await?;
    let len = reader.read_u32().await?;

    return Ok(Header {
        tx_id,
        command,
        len,
    });
}

async fn read_command<T: AsyncRead + Unpin>(reader: &mut T) -> Result<CommandType> {
    return match reader.read_u16().await {
        Ok(command_type) => Result::Ok(command_type),
        Err(err) => Result::Err(Box::new(err)),
    };
}

async fn read_tx_id<T: AsyncRead + Unpin>(reader: &mut T) -> Result<TxIDType> {
    return match reader.read_u64().await {
        Ok(tx_id) => Result::Ok(tx_id),
        Err(err) => Result::Err(Box::new(err)),
    };
}

trait MessageCodec: Clone + Send{
    async fn decode<T: AsyncRead + Unpin>(&self, reader: &mut T) -> Result<Box<dyn MessageBody>>;
    async fn encode<T: AsyncWrite>(&self, writer: &T, msg: Message) -> Result<()>;
}

pub trait MessageCodecReader: AsyncRead {
    async fn read_command_type(&mut self) -> Result<CommandType>
    where
        Self: Unpin,
    {
        return match self.read_u16().await {
            Ok(command_type) => Result::Ok(command_type),
            Err(err) => Result::Err(Box::new(err)),
        };
    }
}

struct JsonMessageCodec {}

impl MessageCodec for JsonMessageCodec {
    async fn decode<T: AsyncRead + Unpin + Sized>(
        &self,
        reader: &mut T,
    ) -> Result<Box<dyn MessageBody>> {
        todo!()
    }

    async fn encode<T: AsyncWrite>(&self, writer: &T, msg: Message) -> Result<()> {
        todo!()
    }
}

struct SendMessage {
    tx_id: TxIDType,
    sender: oneshot::Sender<Message>,
}

struct Conn<T: MessageCodec> {
    codec: T,
    tx_id_gen: TxIDGen,
    stream: TcpStream,
    msgs: HashMap<TxIDType, SendMessage>,
    msg_receive: mpsc::Receiver<Message>,
    msg_sender: mpsc::Sender<Message>,
}

impl<T: MessageCodec> Conn<T> {
    async fn send(&mut self, msg: Message) -> Result<Message> {
        let tx_id = self.tx_id_gen.tx_id();

        info!("send message tx_id {tx_id}");

        let (sender, receiver) = oneshot::channel::<Message>();

        self.msgs.insert(tx_id, SendMessage { tx_id, sender });

        self.codec.encode(&mut self.stream, msg).await?;

        let resp_buf = receiver.await?;
        return Ok(resp_buf);
    }

    async fn do_start(&mut self, msg_sender: mpsc::Sender<Message>) -> Result<()> {
        let (sender, receiver) = mpsc::channel::<Message>(10);

        let (reader, writer) = self.stream.into_split();

        tokio::spawn(pull_msg(reader, self.codec.clone(), sender));
        Ok(())
    }

    async fn recv(&mut self) -> Result<Message> {
        let buf = match self.msg_receive.recv().await {
            Some(buf) => buf,
            None => todo!(),
        };

        Result::Ok(buf)
    }
}

async fn pull_msg<F: AsyncRead + Unpin, T: MessageCodec>(
    mut reader: F,
    codec: T,
    sender: mpsc::Sender<Message>,
) -> SendResult<()> {
    loop {
        let message = decode_message(&codec, &mut reader).await?;

        let _ = sender.send(message).await?;
    }
}

struct TxIDGen {
    tx_id: atomic::AtomicU64,
}

impl TxIDGen {
    fn new() -> TxIDGen {
        return TxIDGen {
            tx_id: atomic::AtomicU64::new(1),
        };
    }

    fn tx_id(&mut self) -> TxIDType {
        return self.tx_id.fetch_add(1, atomic::Ordering::Acquire);
    }
}
