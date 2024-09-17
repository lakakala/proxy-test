use crate::utils::command::{AuthCommand, AuthResponse, Conn, RpcResult};
use crate::utils::result::Result;
use tokio::net::TcpStream;

struct Server {}

impl Server {
    async fn start_client(&mut self, stream: TcpStream) -> Result<()> {
        let mut conn = Conn::new(stream);

        let (auth_req, auth_sender) = conn.recv::<AuthCommand>().await?;

        if auth_req.get_token().eq("1") {
            auth_sender
                .send(RpcResult::Ok(AuthResponse::new(1)))
                .await?;
        } else if auth_req.get_token().eq("2") {
            auth_sender
            .send(RpcResult::Ok(AuthResponse::new(1)))
            .await?;
        } else {

        }

        Ok(())
    }
}


struct Client {
    conn: Conn
}