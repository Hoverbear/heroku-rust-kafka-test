#![feature(lookup_host)] 

extern crate iron;
extern crate kafka;
extern crate openssl;
#[macro_use] extern crate lazy_static;

use std::{env, fs, path};
use std::io::Write;
use std::net::lookup_host;
use kafka::client::{SecurityConfig, KafkaClient};
use openssl::ssl::{Ssl, SslContext, SslMethod, SSL_VERIFY_NONE};
use openssl::x509::X509FileType;
use iron::prelude::*;
use iron::status;

lazy_static! {
    // HTTP server related.
    static ref PORT: u64 = env::var("PORT")
        .unwrap()
        .parse::<_>()
        .unwrap();
    static ref URL: String = format!("0.0.0.0:{}", *PORT);
    // Kafka relatex statics.
    static ref KAFKA_URLS: Vec<String> = env::var("KAFKA_URL")
        .and_then(|line| {
            Ok(line.split(',').map(|raw_hostname| {
                // Get rid of the `kafka+ssl://` bit
                let hostname_no_prefix = raw_hostname.split("//").skip(1).next().unwrap();
                // Need to strip, then re-add the port.
                let mut splitter = hostname_no_prefix.split(":");
                let hostname = splitter.next().unwrap();
                let port = splitter.next().unwrap();
                // Get the host.
                let mut hosts = lookup_host(hostname).unwrap();
                let mut host = hosts.next().unwrap().unwrap();
                // Re-strip port
                host.set_port(u16::from_str_radix(port, 10).unwrap());
                println!("{}", host);
                format!("{}", host)
            }).collect())
        })
        .unwrap();
    static ref KAFKA_CLIENT_CERT: String = env::var("KAFKA_CLIENT_CERT")
        .unwrap();
    static ref KAFKA_CLIENT_CERT_KEY: String = env::var("KAFKA_CLIENT_CERT_KEY")
        .unwrap();
    // Let's not get too crazy here.
    static ref KAFKA_CLIENT_CERT_PATH: path::PathBuf = "cert.crt".into();
    static ref KAFKA_CLIENT_KEY_PATH: path::PathBuf = "cert.key".into();
}

fn hello_world(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((status::Ok, "Hello World!")))
}

fn main() {

    // Create the files.
    let mut cert_fd = fs::File::create(&*KAFKA_CLIENT_CERT_PATH)
        .unwrap();
    let mut key_fd = fs::File::create(&*KAFKA_CLIENT_KEY_PATH)
        .unwrap();
    write!(cert_fd, "{}", *KAFKA_CLIENT_CERT)
        .unwrap();
    write!(key_fd, "{}", *KAFKA_CLIENT_CERT_KEY)
        .unwrap();
    
    // OpenSSL offers a variety of complex configurations. Here is an example:
    let mut context = SslContext::new(SslMethod::Sslv23).unwrap();
    context.set_cipher_list("DEFAULT").unwrap();
    context.set_certificate_file(&*KAFKA_CLIENT_CERT_PATH, X509FileType::PEM).unwrap();
    context.set_private_key_file(&*KAFKA_CLIENT_KEY_PATH, X509FileType::PEM).unwrap();
    context.set_verify(SSL_VERIFY_NONE, None);
    let ssl = Ssl::new(&context).unwrap();
    
    let mut client = KafkaClient::new_secure((*KAFKA_URLS).clone(), SecurityConfig::new(ssl));
    client.load_metadata_all().unwrap();

    client.load_metadata_all().unwrap();
    let topics = client.topics();
    println!("Topics: {:?}", topics);

    println!("Binding on {:?}", *URL);
    Iron::new(hello_world).http(&(*URL)[..]).unwrap();
}
