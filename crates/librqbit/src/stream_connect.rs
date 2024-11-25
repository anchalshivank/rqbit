use std::net::SocketAddr;

use anyhow::Context;
use crate::session::Protocol;

#[derive(Debug, Clone)]
pub(crate) struct SocksProxyConfig {
    pub host: String,
    pub port: u16,
    pub username_password: Option<(String, String)>,
}

impl SocksProxyConfig {
    pub fn parse(url: &str) -> anyhow::Result<Self> {
        let url = ::url::Url::parse(url).context("invalid proxy URL")?;
        if url.scheme() != "socks5" {
            anyhow::bail!("proxy URL should have socks5 scheme");
        }
        let host = url.host_str().context("missing host")?;
        let port = url.port().context("missing port")?;
        let up = url
            .password()
            .map(|p| (url.username().to_owned(), p.to_owned()));
        Ok(Self {
            host: host.to_owned(),
            port,
            username_password: up,
        })
    }

    async fn connect_tcp(
        &self,
        addr: SocketAddr,
    ) -> anyhow::Result<impl tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin> {
        let proxy_addr = (self.host.as_str(), self.port);

        if let Some((username, password)) = self.username_password.as_ref() {
            tokio_socks::tcp::Socks5Stream::connect_with_password(
                proxy_addr,
                addr,
                username.as_str(),
                password.as_str(),
            )
                .await
                .context("error connecting to proxy")
        } else {
            tokio_socks::tcp::Socks5Stream::connect(proxy_addr, addr)
                .await
                .context("error connecting to proxy")
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct StreamConnector {
    proxy_config: Option<SocksProxyConfig>,
    protocol: Protocol,
}

impl StreamConnector {
    pub fn new(proxy_config: Option<SocksProxyConfig>, protocol: Protocol) -> Self {
        Self {
            proxy_config,
            protocol
        }
    }
}

pub(crate) trait AsyncReadWrite:
    tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Unpin
{
}

impl<T> AsyncReadWrite for T where T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Send + Unpin {}

impl StreamConnector {
    pub async fn connect(&self, addr: SocketAddr) -> anyhow::Result<Box<dyn AsyncReadWrite>> {

        match self.protocol {
            Protocol::Tcp => {

                if let Some(proxy) = self.proxy_config.as_ref() {
                    return Ok(Box::new(proxy.connect_tcp(addr).await?));
                }
                Ok(Box::new(
                    tokio::net::TcpStream::connect(addr)
                        .await
                        .context("error connecting")?,
                ))

            }
            Protocol::Udp => {
                //Todo: Implement Udp protocol
                panic!("Udp protocol is not implemented");
            }
            Protocol::Utp => {
                // Todo: Implement Utp protocol
                panic!("UTP protocol is not implemented");
            }
        }
    }
}
