mod datagram;
mod stream;

use crate::{
    Dispatcher,
    common::auth::ThreadSafeAuthenticator,
    proxy::{inbound::InboundHandlerTrait, utils::apply_tcp_options},
    session::{Network, Session, Type},
};

use async_trait::async_trait;
use std::{net::SocketAddr, sync::Arc};
pub use stream::handle_tcp;
use tokio::net::TcpListener;
use tracing::warn;

use crate::common::errors::new_io_error;
pub use datagram::Socks5UDPCodec;

pub struct SocksInbound {
    addr: SocketAddr,
    allow_lan: bool,
    dispatcher: Arc<Dispatcher>,
    authenticator: ThreadSafeAuthenticator,
    fw_mark: Option<u32>,
}

impl Drop for SocksInbound {
    fn drop(&mut self) {
        warn!("SOCKS5 inbound listener on {} stopped", self.addr);
    }
}

impl SocksInbound {
    pub fn new(
        addr: SocketAddr,
        allow_lan: bool,
        dispatcher: Arc<Dispatcher>,
        authenticator: ThreadSafeAuthenticator,
        fw_mark: Option<u32>,
    ) -> Self {
        Self {
            addr,
            allow_lan,
            dispatcher,
            authenticator,
            fw_mark,
        }
    }
}

#[async_trait]
impl InboundHandlerTrait for SocksInbound {
    fn handle_tcp(&self) -> bool {
        true
    }

    fn handle_udp(&self) -> bool {
        false
    }

    async fn listen_tcp(&self) -> std::io::Result<()> {
        let listener = TcpListener::bind(self.addr).await?;

        loop {
            let (socket, _) = listener.accept().await?;
            let src_addr = socket.peer_addr()?;
            if !self.allow_lan && src_addr.ip() != socket.local_addr()?.ip() {
                warn!("Connection from {} is not allowed", src_addr);
                continue;
            }
            apply_tcp_options(&socket)?;

            let mut sess = Session {
                network: Network::Tcp,
                typ: Type::Socks5,
                source: socket.peer_addr()?,
                so_mark: self.fw_mark,

                ..Default::default()
            };

            let dispatcher = self.dispatcher.clone();
            let authenticator = self.authenticator.clone();

            tokio::spawn(async move {
                handle_tcp(&mut sess, socket, dispatcher, authenticator).await
            });
        }
    }

    async fn listen_udp(&self) -> std::io::Result<()> {
        Err(new_io_error("UDP is not supported"))
    }
}
