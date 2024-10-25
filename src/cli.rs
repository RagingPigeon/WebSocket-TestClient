use crate::edge_view;
use clap::Parser;
use jsonwebtoken::{
    Algorithm,
    encode,
    EncodingKey,
    Header,
};
use tracing::{event, Level};

#[derive(serde::Serialize)]
#[derive(Clone, Parser, Debug)]
pub struct Args {
    #[arg(long = "spin_client", value_parser, num_args = 1.., value_delimiter = ',')]
    pub spin_client: Option<Vec<String>>,
}

impl Args {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

async fn spin_client(endpoint: String) {

    _ = edge_view::client::ws_connect(
        edge_view::client::SERVER_PORT,
        Algorithm::HS256,
        endpoint.as_str());

    event!(Level::DEBUG, "Moving into the spin loop");
    loop {
        // We will stay here forever to keep the server connection
        // live.
    }
}

pub fn process_arguments() {
    let args = Args::parse();

    match args.spin_client {
        Some(clients) => {
            for endpoint in clients {
                event!(Level::DEBUG, "Spawning spin client for endpoint: {}", endpoint);

                tokio::spawn(spin_client(endpoint.clone()));
            }
        }
        _ => {}
    }
}