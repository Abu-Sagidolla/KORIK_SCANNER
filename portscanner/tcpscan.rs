use futures::{stream, StreamExt};
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use tokio::net::TcpStream;

use indicatif::{ProgressBar, ProgressStyle};
use std::sync::Arc;

pub const MOST_COMMON_PORTS_1002: &[u16] = &[
    443, 6001, 5601, 9300, 80, 23, 443, 21, 22, 25, 3389, 110, 445, 139, 143, 53, 135, 3306, 8080,
    1723, 111, 995, 993, 5900, 1025, 587, 8888, 199, 1720, 465, 548, 113, 81, 6001, 10000, 514,
    5060, 179, 1026, 2000, 8443, 8000, 32768, 554, 26, 1433, 49152, 2001, 515, 8008, 49154, 1027,
    5666, 646, 5000, 5631, 631, 49153, 8081, 2049, 88, 79, 5800, 106, 2121, 1110, 49155, 6000, 513,
    990, 5357, 427, 49156, 543, 544, 5101, 144, 7, 389, 8009, 3128, 444, 9999, 5009, 7070, 5190,
    3000, 5432, 1900, 3986, 13, 1029, 9, 5051, 6646, 49157, 1028, 873, 1755, 2717, 4899, 9100, 119,
    37, 1000, 3001, 5001, 82, 10010, 1030, 9090, 2107, 1024, 2103, 6004, 1801, 5050, 19, 8031,
    1041, 255, 8291, 54663, 54664, 8012, 61616, 26379, 3389, 29017, 6001,
];

pub fn get_ports(full: bool) -> Box<dyn Iterator<Item = u16>> {
    if full {
        Box::new((1..=u16::MAX).into_iter())
    } else {
        Box::new(MOST_COMMON_PORTS_1002.to_owned().into_iter())
    }
}

pub async fn scan_port(target: IpAddr, port: u16, timeout: u64) -> Option<u16> {
    let timeout = Duration::from_secs(timeout);
    let socket_address = SocketAddr::new(target.clone(), port);

    match tokio::time::timeout(timeout, TcpStream::connect(&socket_address)).await {
        Ok(Ok(_)) => Some(port),
        _ => Some(0),
    }
}

/* pub   async fn scan(target: IpAddr, full: bool, concurrency: usize, timeout: u64) {
     let ports = stream::iter(get_ports(full));

     ports
         .for_each_concurrent(concurrency, |port| scan_port(target, port, timeout))
         .await;
  }
*/

pub async fn scan(target: IpAddr, full: bool, concurrency: usize, timeout: u64) -> Vec<u16> {
    let mut open_ports = Vec::new();
    let ports = get_ports(full).collect::<Vec<_>>();
    let total_ports = ports.len() as u64;

    let pb = Arc::new(ProgressBar::new(total_ports));
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] [{wide_bar}] {pos}/{len} ({percent}%)")
            .expect("REASON")
            .progress_chars("=> "),
    );

    let pb_clone = pb.clone();

    let port_stream = stream::iter(ports)
        .map(|port| {
            let pb = pb_clone.clone();
            async move {
                let result = scan_port(target, port, timeout).await;
                pb.inc(1);
                result
            }
        })
        .buffer_unordered(concurrency);

    let results = port_stream.collect::<Vec<_>>().await;

    for result in results {
        if let Some(open_port) = result {
            if open_port != 0 {
                open_ports.push(open_port);
            }
        }
    }

    pb.finish_with_message("Scan completed");

    open_ports
}
