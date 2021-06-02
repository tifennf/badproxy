use std::io::Read;
use std::io::Write;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use regex::bytes::Regex;
use reqwest::Url;



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

		//init
        let mut request = [0; 4000];
		let mut response = [0; 4000];
        let mut stream = stream.unwrap();
        log(&stream);
		
        stream.read(&mut request).unwrap();


		//some logs
        let format_request = String::from_utf8_lossy(&request);
        println!("Initial request: \n{}", format_request);


		//extract host and other things from initial request
        let (hostline, hostname, port) = query_host(&regex, &request).unwrap();
        // let hostname = dns_lookup::lookup_host(&String::from_utf8_lossy(hostname)).unwrap();
		// let hostname = hostname.iter().next().unwrap().to_string();
		let hostname = String::from_utf8_lossy(hostname);

		println!("{}", hostname);

		let url = format!("https://{}", hostname);
        
		let mut res = reqwest::blocking::get("https://youtube.com").unwrap();
		let mut body = String::new();


		
		// let headers = res.headers();
		
		// for (name, value) in headers.iter() {
			
		// 	println!("headers: {:?} : {:?}", name, value);
		// }

		res.read_to_string(&mut body).expect("failed to read to string");
		
		// let test = res.
		// let test = String::from_utf8_lossy(&test);
		// println!("TEST: {:?}", &res);
		
		

		let client_response = format!(
			"HTTPS/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
			body.len(),
			body
		);

		
        stream.write(&client_response.as_bytes()).expect("failed to write response");
		
        // println!(
			//     "Data read from response: \n{:?}",
			//     body
			// );
		
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

