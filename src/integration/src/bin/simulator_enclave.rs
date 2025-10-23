//! Enclave to simulate communication patterns with a secure app

use std::sync::Arc;

use borsh::BorshDeserialize;
use host_primitives::enclave_client_timeout;
use qos_core::{
    client::{ClientError, SocketClient},
    io::{SocketAddress, StreamPool},
    protocol::msg::ProtocolMsg,
    server::{RequestProcessor, SharedProcessor, SocketServer},
};
use qos_nsm::types::NsmResponse;
use tokio::sync::RwLock;

#[derive(Clone)]
struct Processor {
    app_client: SocketClient,
}

impl Processor {
    pub fn new(app_client: SocketClient) -> SharedProcessor<Self> {
        Arc::new(RwLock::new(Self { app_client }))
    }

    /// Expands the app pool to given pool size
    pub async fn expand_to(&mut self, pool_size: u8) -> Result<(), ClientError> {
        self.app_client.expand_to(pool_size).await
    }
}

impl RequestProcessor for Processor {
    async fn process(&self, request: &[u8]) -> Vec<u8> {
        let msg_req = ProtocolMsg::try_from_slice(request)
            .expect("enclave_stub: Failed to deserialize request");

        match msg_req {
            ProtocolMsg::ProxyRequest { data } => {
                let resp_data = match self.app_client.call(&data).await {
                    Ok(d) => d,
                    Err(err) => panic!("Error from app: {err:?}"),
                };

                borsh::to_vec(&ProtocolMsg::ProxyResponse { data: resp_data })
                    .expect("enclave_stub: Failed to serialize response")
            }
            ProtocolMsg::LiveAttestationDocRequest => {
                let data_string = borsh::to_vec(&"MOCK_DOCUMENT".to_string())
                    .expect("unable to serialize mock document");
                let nsm_response = NsmResponse::Attestation {
                    document: data_string,
                };

                borsh::to_vec(&ProtocolMsg::LiveAttestationDocResponse {
                    nsm_response,
                    manifest_envelope: None,
                })
                .expect("enclave stub: Failed to serialize response")
            }
            other => panic!("enclave_stub: Unexpected request {other:?}"),
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().collect();

    let default_pool_size = "1".to_owned();
    let pool_size_str: &str = args.get(3).unwrap_or(&default_pool_size);
    let pool_size: u8 = pool_size_str.parse().expect("invalid pool size specified");

    let enclave_sock_path = &args[1];
    let enclave_sock_pool = StreamPool::new(SocketAddress::new_unix(enclave_sock_path), pool_size)
        .expect("unable to create enclave pool");

    let app_sock_path = &args[2];
    let app_sock_pool = StreamPool::new(SocketAddress::new_unix(app_sock_path), pool_size)
        .expect("unable to create app pool");
    let processor = Processor::new(SocketClient::new(
        app_sock_pool.shared(),
        enclave_client_timeout(),
    ));

    let mut server = SocketServer::listen_all(enclave_sock_pool, &processor)
        .expect("unable to start enclave socket server");

    server
        .listen_to(pool_size, &processor)
        .expect("unable to listen_to on the running server");
    // expand app connections to pool_size
    processor
        .write()
        .await
        .expand_to(pool_size)
        .await
        .expect("unable to expand_to on the processor app pool");

    match tokio::signal::ctrl_c().await {
        Ok(_) => eprintln!("handling ctrl+c the tokio way"),

        Err(err) => panic!("{err}"),
    }
}
