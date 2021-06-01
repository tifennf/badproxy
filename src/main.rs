use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use regex::bytes::Regex;

const IP: &str = "0.0.0.0:3000";

struct HttpHostRegex {
    line: Option<Regex>,
    name: Option<Regex>,
    port: Option<Regex>,
}

fn main() {
    let regex = HttpHostRegex {
		line: Some(Regex::new(
			r"(Host: )((([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\-]*([A-Za-z0-9])\.)*([A-Za-z0-9]))",
		).unwrap()),
		name: Some(Regex::new(
			r"(([a-zA-Z0-9]|[a-zA-Z0-9][a-zA-Z0-9\-]*[a-zA-Z0-9])\.)*([A-Za-z0-9]|[A-Za-z0-9][A-Za-z0-9\-]*([A-Za-z0-9])\.)*([A-Za-z0-9])",
		).unwrap()),
		port: Some(Regex::new(
			r":([A-Za-z0-9]*)",
		).unwrap()),
	};

    let listener = TcpListener::bind(IP).unwrap();

    for stream in listener.incoming() {
        let mut request = [0; 500];
        let mut response = [0; 4000];

        let mut stream = stream.unwrap();
        log(&stream);

        stream.read(&mut request).unwrap();

        let format_request = String::from_utf8_lossy(&request);
        println!("Initial request: \n{}", format_request);

        let (hostline, hostname, port) = query_host(&regex, &request).unwrap();

        let hostname_list = dns_lookup::lookup_host(&String::from_utf8_lossy(hostname)).unwrap();

		println!("hostvec {:?}", hostname);

		let req = format!("GET /index.html HTTP/1.1\r\nHost: {}\r\n\r\n", String::from_utf8_lossy(&hostname));

		let request = req.as_bytes();

		println!("HTTP request: {}", req);

        let target_socket = SocketAddr::new(*hostname_list.iter().next().unwrap(), 80);

        let mut proxy_stream = TcpStream::connect(target_socket).expect("failed to connect");

        proxy_stream
            .write(&request)
            .expect("failed to write request");

        proxy_stream
            .read(&mut response)
            .expect("failed to read response");

        stream.write(&response).expect("failed to write response");

        println!("{:?}", proxy_stream);

        println!(
            "Data read from response: \n{}",
            String::from_utf8_lossy(&response)
        );
    }
}

fn log(stream: &TcpStream) {
    println!("STREAM:: {:?}", stream);
    println!("localaddr: {}", stream.local_addr().unwrap());
    println!("peeraddrr: {}\n\n", stream.peer_addr().unwrap());
}

fn query_host<'a>(
    regex: &HttpHostRegex,
    request: &'a [u8],
) -> Result<(&'a [u8], &'a [u8], Option<&'a [u8]>), &'a str> {
    //query the host line in http request

    let host_line = match regex.line.as_ref().unwrap().find(request) {
        Some(host_line) => host_line.as_bytes(),
        None => return Err("Invalid request"),
    };

    //query the host inside host line
    let host = &host_line[6..];

    let hostname = match regex.name.as_ref().unwrap().find(host) {
        Some(name) => name.as_bytes(),
        None => return Err("Hostname not found"),
    };

    //query the port if exist inside full host
    let port = match regex.port.as_ref().unwrap().find(host) {
        Some(port) => {
            let port = port.as_bytes();

            //remove the ':' char from the bytes
            Some(&port[1..])
        }
        None => None,
    };

    Ok((host_line, hostname, port))
}
