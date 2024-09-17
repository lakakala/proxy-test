use super::result::Result;
use core::sync;
use pin_project_lite::pin_project;
use serde::{Deserialize, Serialize};
use serde_json::de::Read;
use std::{collections, mem::MaybeUninit};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    sync::oneshot,
};
use tokio::io::ReadBuf;

type CommandType = u16;

const AUTH_REQ_COMMAND_TYPE: CommandType = 1;
const AUTH_RESP_COMMAND_TYPE: CommandType = 2;

trait Message {
    fn cmd() -> CommandType;
}

trait Command {
    type Request: Message;
    type Response: Message;

    async fn encode_request<T: AsyncWrite + Unpin>(
        writer: &mut T,
        req: Self::Request,
    ) -> Result<()>;

    async fn decode_request<T: AsyncRead + Unpin>(stream: &mut T) -> Result<Self::Request>;

    async fn encode_response<T: AsyncWrite + Unpin>(
        writer: &mut T,
        req: Self::Response,
    ) -> Result<()>;

    async fn decode_response<T: AsyncRead + Unpin>(reader: &mut T) -> Result<Self::Response>;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthRequest {
    token: String,
}

impl AuthRequest {
    pub fn get_token(&self) -> &String {
        return self.get_token();
    }
}

impl Message for AuthRequest {
    fn cmd() -> CommandType {
        return AUTH_REQ_COMMAND_TYPE;
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AuthResponse {
    client_id: u64,
}

impl AuthResponse {
    pub fn new(client_id: u64) -> AuthResponse {
        return AuthResponse { client_id };
    }
}

impl Message for AuthResponse {
    fn cmd() -> CommandType {
        return AUTH_RESP_COMMAND_TYPE;
    }
}

pub struct AuthCommand {}

impl Command for AuthCommand {
    type Request = AuthRequest;

    type Response = AuthResponse;

    async fn encode_request<T: AsyncWrite + Unpin>(
        writer: &mut T,
        req: Self::Request,
    ) -> Result<()> {
        let data = serde_json::to_vec(&req)?;

        writer.write_u32(data.len() as u32).await?;
        writer.write_all(&data).await;

        Ok(())
    }

    async fn decode_request<T: AsyncRead + Unpin>(reader: &mut T) -> Result<Self::Request> {
        let len = reader.read_u32().await?;

        let mut data = Vec::with_capacity(len as usize);
        reader.read_exact(&mut data).await?;

        let req = serde_json::from_slice(&data)?;

        return Ok(req);
    }

    async fn encode_response<T: AsyncWrite + Unpin>(
        writer: &mut T,
        req: Self::Response,
    ) -> Result<()> {
        let data = serde_json::to_vec(&req)?;

        writer.write_u32(data.len() as u32).await?;
        writer.write_all(&data).await;

        Ok(())
    }

    async fn decode_response<T: AsyncRead + Unpin>(reader: &mut T) -> Result<Self::Response> {
        let len = reader.read_u32().await?;

        let mut data = Vec::with_capacity(len as usize);
        reader.read_exact(&mut data).await?;

        let req = serde_json::from_slice(&data)?;

        return Ok(req);
    }
}

pin_project! {
    struct MessageBodyReader<'a, T: AsyncRead> {
        reader: &'a T,
        len: u32,
        // buf: ReadBuf<'a>,
    }
}

impl<'a, T: AsyncRead> MessageBodyReader<'a, T> {
    fn new(reader: &'a mut T, len: u32) -> MessageBodyReader<'a, T> {
        MessageBodyReader {
             reader, 
             len,
            // buf: ReadBuf::uninit([MaybeUninit]{len})}
    }
}

impl<'a, T: AsyncRead> AsyncRead for MessageBodyReader<'a, T> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let project = self.project();
        if *project.len <= 0 {
            return std::task::Poll::Ready(std::io::Result::Ok(()));
        }

        ReadBuf::uninit(buf.as_mut());

   match AsyncRead::poll_read(project, cx, buf){}
        todo!()
    }
}

struct SendMessage {
    tx_id: u64,
    sender: oneshot::Sender<MessageHeader>,
}

struct MessageHeader {
    cmd: CommandType,
    tx_id: u64,
}

pub struct Conn {
    tx_id: sync::atomic::AtomicU64,
    stream: TcpStream,
    send_msgs: collections::HashMap<u64, SendMessage>,
}

impl Conn {
    pub fn new(stream: TcpStream) -> Conn {
        return Conn {
            tx_id: sync::atomic::AtomicU64::new(0),
            stream,
            send_msgs: collections::HashMap::new(),
        };
    }

    pub async fn send<T: Command>(&mut self, req: T::Request) -> Result<RpcResult<T::Response>> {
        let tx_id = self.tx_id.fetch_add(1, sync::atomic::Ordering::Acquire);

        self.stream.write_u16(T::Request::cmd()).await?;
        self.stream.write_u64(tx_id).await?;

        T::encode_request(&mut self.stream, req).await?;

        let (sender, receiver) = oneshot::channel::<MessageHeader>();

        self.send_msgs.insert(tx_id, SendMessage { tx_id, sender });

        let msg_header = receiver.await?;

        todo!()
    }

    async fn recv1<T: Command, F: Fn(T::Request) -> T::Response>(&mut self, f: F) {}

    pub async fn recv<T: Command>(&mut self) -> Result<(T::Request, Sender<T::Response>)> {
        todo!()
    }
}

struct Sender<T> {
    sender: oneshot::Sender<T>,
}

impl<T> Sender<T> {
    pub async fn send(mut self, resp: RpcResult<T>) -> Result<()> {
        todo!()
    }
}

pub enum RpcResult<T> {
    Ok(T),
    Err(i32),
}

struct Frame {
    command_id: u32,
    tx_id: u64,
}
