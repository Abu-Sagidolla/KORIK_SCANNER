use tokio::{
    io::{AsyncWriteExt, BufWriter, AsyncReadExt, BufReader},
    net::TcpStream,
};
use tokio_rustls::{rustls, TlsConnector};
use std::{
    io, sync::Arc
};
use std::net::ToSocketAddrs;
use tokio::io::{stdin as tokio_stdin, stdout as tokio_stdout};
use rustls::pki_types::ServerName;
use tokio_rustls::client::TlsStream;
use tokio::io::AsyncBufReadExt;
use std::str;
use anyhow::anyhow;
use url::Url;
use reqwest::header::HeaderMap;
use time::Duration;
use rustls::{ClientConfig, RootCertStore};
use webpki_roots::TLS_SERVER_ROOTS;

enum Stream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}


pub fn prepare_cookie(cookies: Option<String>) -> String {
    let mut toqen = "".to_string();
    if let Some(value) = cookies {
        if !value.contains("\r\n") && !value.is_empty() {
            toqen = format!("{}\r\n", value);
        } else if value.is_empty() {
            toqen = "".to_string();
        } else {
            toqen = value.replace("\r\n", "\r\n");
        }
    }
    toqen
}

pub async fn smuggle(
    url: String,
    endpoint: &str,
    http_method: &str,
    cookie: Option<String>,
    payload: Option<String>,
    tls_connector: Option<TlsConnector>, // Pass in the TLS connector to avoid recreating it
) -> Result<(String, String), anyhow::Error> {
    // Parse URL
    let parsed_url = Url::parse(&url)?;
    let host = parsed_url
        .host_str()
        .ok_or_else(|| anyhow!("Invalid host"))?
        .to_string();

    // Determine port based on scheme
    let port = parsed_url
        .port_or_known_default()
        .unwrap_or_else(|| if parsed_url.scheme() == "https" { 443 } else { 80 });

    let address = format!("{}:{}", host, port);

    // Prepare request payload
    let payload_value = payload.unwrap_or_default();
    let cookie_pie = prepare_cookie(cookie);
    let content = format!(
        "{} {} HTTP/1.1\r\nHost: {}\r\n{}Content-Length: {}\r\nUser-Agent: Mozilla/5.0\r\n\r\n{}",
        http_method.to_uppercase(),
        endpoint,
        host,
        cookie_pie,
        payload_value.len(),
        payload_value
    );

    println!("Request being sent:\n{}", content);

    // Handle HTTP and HTTPS connections
    let mut stream = if parsed_url.scheme() == "https" {
        // Reuse the provided TLS connector to avoid re-creating it every time
        let tls_connector = tls_connector.ok_or_else(|| anyhow!("TLS connector not provided"))?;
        let tcp_stream = TcpStream::connect(&address).await?;
        let domain = ServerName::try_from(host.clone())
            .map_err(|_| anyhow::anyhow!("Invalid DNS name"))?;
        let tls_stream = tls_connector.connect(domain, tcp_stream).await?;
        Stream::Tls(tls_stream)
    } else {
        let tcp_stream = TcpStream::connect(&address).await?;
        Stream::Plain(tcp_stream)
    };

    // Send the request using a buffered writer
    match stream {
        Stream::Plain(ref mut s) => {
            let mut writer = BufWriter::new(s);
            writer.write_all(content.as_bytes()).await?;
            writer.flush().await?;
        }
        Stream::Tls(ref mut s) => {
            let mut writer = BufWriter::new(s);
            writer.write_all(content.as_bytes()).await?;
            writer.flush().await?;
        }
    }

    // Read response with buffered reader
    let mut response = Vec::with_capacity(4096); // Pre-allocate buffer
    match stream {
        Stream::Plain(ref mut s) => {
            let mut reader = BufReader::new(s);
            reader.read_to_end(&mut response).await?;
        }
        Stream::Tls(ref mut s) => {
            let mut reader = BufReader::new(s);
            reader.read_to_end(&mut response).await?;
        }
    }

    Ok((content, String::from_utf8_lossy(&response).to_string()))
}

// Helper function to build the root certificate store
fn build_root_cert_store() -> RootCertStore {
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.extend(TLS_SERVER_ROOTS.iter().cloned());
    root_cert_store
}

// Create the TLS connector once and pass it to smuggle
pub fn create_tls_connector() -> TlsConnector {
    let config = Arc::new(
        ClientConfig::builder()
            .with_root_certificates(build_root_cert_store())
            .with_no_client_auth()
    );
    TlsConnector::from(config)
}
pub async fn jiber(url: &str) -> Result<(String, HeaderMap, String), Box<reqwest::Error>> {
    let client = reqwest::Client::builder()  .danger_accept_invalid_certs(true) .build()?;

    let response = match client.get(url).header("User-Agent","Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36").send().await {
        Ok(res) => res,
        Err(err) => {
            //e//("Error fetching {}: {}", url, err);
            return Err(Box::new(err));
        }
    };

    let headers = response.headers().clone();
    let status = response.status();
    // let setter = response.headers();
    ////("{:?}",setter["set-cookie"]);

    let body = response.text().await?;

    /*let headers_str = headers.clone()
        .into_iter()
        .map(|(name, value)| format!("{:?}: {:?}", name, value))?;
        .collect::<Vec<_>>()
        .join("\n");
    */
    Ok((body, headers, status.to_string()))
}
