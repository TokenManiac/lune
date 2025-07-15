use std::{io::{Error, Result}, sync::Arc};

use async_net::TcpStream;
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use futures_rustls::{TlsConnector, TlsStream};
use rustls_pki_types::ServerName;
use url::Url;

use crate::client::{rustls::CLIENT_CONFIG, stream::MaybeTlsStream};

pub async fn connect_socks5(proxy: &Url, host: &str, port: u16, tls: bool) -> Result<MaybeTlsStream> {
    let proxy_host = proxy.host_str().ok_or_else(|| Error::other("missing proxy host"))?.to_string();
    let proxy_port = proxy.port_or_known_default().ok_or_else(|| Error::other("missing proxy port"))?;
    let mut stream = TcpStream::connect((proxy_host.as_str(), proxy_port)).await?;

    let username = proxy.username();
    let password = proxy.password().unwrap_or("");
    let use_auth = !username.is_empty() || !password.is_empty();

    if use_auth {
        stream.write_all(&[0x05, 0x01, 0x02]).await?;
    } else {
        stream.write_all(&[0x05, 0x01, 0x00]).await?;
    }
    let mut resp = [0u8; 2];
    stream.read_exact(&mut resp).await?;
    if resp[0] != 0x05 || resp[1] == 0xFF {
        return Err(Error::other("proxy refused authentication"));
    }
    if resp[1] == 0x02 {
        let uname = username.as_bytes();
        let pass = password.as_bytes();
        if uname.len() > 255 || pass.len() > 255 {
            return Err(Error::other("username or password too long"));
        }
        let mut auth = Vec::with_capacity(3 + uname.len() + pass.len());
        auth.push(0x01);
        auth.push(uname.len() as u8);
        auth.extend_from_slice(uname);
        auth.push(pass.len() as u8);
        auth.extend_from_slice(pass);
        stream.write_all(&auth).await?;
        stream.read_exact(&mut resp).await?;
        if resp[1] != 0x00 {
            return Err(Error::other("authentication failed"));
        }
    }

    let mut req = Vec::with_capacity(6 + host.len());
    req.push(0x05);
    req.push(0x01);
    req.push(0x00);
    req.push(0x03);
    req.push(host.len() as u8);
    req.extend_from_slice(host.as_bytes());
    req.extend_from_slice(&port.to_be_bytes());
    stream.write_all(&req).await?;

    let mut header = [0u8; 4];
    stream.read_exact(&mut header).await?;
    if header[1] != 0x00 {
        return Err(Error::other("proxy connect failed"));
    }
    let atyp = header[3];
    let addr_len = match atyp {
        0x01 => 4,
        0x03 => {
            let mut l = [0u8;1];
            stream.read_exact(&mut l).await?;
            l[0] as usize
        }
        0x04 => 16,
        _ => return Err(Error::other("invalid address type")),
    };
    let mut skip = vec![0u8; addr_len + 2];
    stream.read_exact(&mut skip).await?;

    let stream = if tls {
        let servname = ServerName::try_from(host).map_err(Error::other)?.to_owned();
        let connector = TlsConnector::from(Arc::clone(&CLIENT_CONFIG));
        let tls = connector.connect(servname, stream).await?;
        MaybeTlsStream::Tls(Box::new(TlsStream::Client(tls)))
    } else {
        MaybeTlsStream::Plain(Box::new(stream))
    };
    Ok(stream)
}
