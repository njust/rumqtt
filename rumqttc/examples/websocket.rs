#[cfg(feature = "websocket")]
use rumqttc::{self, AsyncClient, MqttOptions, QoS, Transport};
#[cfg(feature = "websocket")]
use std::{error::Error, time::Duration};
use std::sync::Arc;
use std::time::SystemTime;
use rustls::client::{ServerCertVerified, ServerCertVerifier};
use rustls::{Certificate, ClientConfig, ServerName};
#[cfg(feature = "websocket")]
use tokio::{task, time};
use rumqttc::TlsConfiguration;

struct CertVerifier {

}

impl ServerCertVerifier for CertVerifier {
    fn verify_server_cert(&self, end_entity: &Certificate, intermediates: &[Certificate], server_name: &ServerName, scts: &mut dyn Iterator<Item=&[u8]>, ocsp_response: &[u8], now: SystemTime) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
}

#[cfg(feature = "websocket")]
#[tokio::main(worker_threads = 1)]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();

    // port parameter is ignored when scheme is websocket
    let mut mqttoptions = MqttOptions::new(
        "clientId-aSziq39Bpaa3",
        "wss://server",
        443,
    );
    mqttoptions.set_credentials("admin", "<pwd>");
    let tls_config = ClientConfig::builder()
        .with_safe_default_cipher_suites()
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_custom_certificate_verifier(Arc::new(CertVerifier{}))
        .with_no_client_auth();

    mqttoptions.set_transport(Transport::Wss(TlsConfiguration::Rustls(Arc::new(tls_config))));
    mqttoptions.set_keep_alive(Duration::from_secs(60));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    task::spawn(async move {
        requests(client).await;
        time::sleep(Duration::from_secs(3)).await;
    });

    loop {
        let event = eventloop.poll().await;
        match event {
            Ok(notif) => {
                println!("Event = {notif:?}");
            }
            Err(err) => {
                println!("Error = {err:?}");
                return Ok(());
            }
        }
    }
}

#[cfg(feature = "websocket")]
async fn requests(client: AsyncClient) {
    client
        .subscribe("hello/world", QoS::AtMostOnce)
        .await
        .unwrap();

    for i in 1..=10 {
        client
            .publish("hello/world", QoS::ExactlyOnce, false, vec![1; i])
            .await
            .unwrap();

        time::sleep(Duration::from_secs(1)).await;
    }

    time::sleep(Duration::from_secs(120)).await;
}

#[cfg(not(feature = "websocket"))]
fn main() {
    panic!("Enable websocket feature with `--features=websocket`");
}
