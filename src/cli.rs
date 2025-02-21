use crate::edge_view;
use clap::Parser;
use jsonwebtoken::{
    Algorithm,
    encode,
    EncodingKey,
    Header,
};
use std::{thread, time};
use thread_id;
use tokio::{
    task::JoinHandle,
    task::JoinSet,
};
use tracing::{event, Level};

#[derive(serde::Serialize)]
#[derive(Clone, Parser, Debug)]
pub struct Args {

    #[arg(long = "server_ip", default_value_t = String::from("0.0.0.0"))]
    pub server_ip: String,

    #[arg(long = "server_port", default_value_t = 7878)]
    pub server_port: u16,

    #[arg(long = "spin_client", value_parser, num_args = 1.., value_delimiter = ',')]
    pub spin_client: Option<Vec<String>>,

    #[arg(long = "test_get_users", default_value_t = false)]
    pub test_get_users: bool,

    #[arg(long = "test_get_users_and_listen", default_value_t = false)]
    pub test_get_users_and_listen: bool,
}

impl Args {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

pub fn process_arguments() -> JoinSet<()> {

    let mut return_value: JoinSet<()> = JoinSet::new();
    let args = Args::parse();
    

    // if args.test_get_users {
    //     event!(Level::DEBUG, "Spawning test_get_users thread.");
    //     return_value.spawn(edge_view::client::test_get_users());
    // }

    // if args.test_get_users_and_listen {
    //     event!(Level::DEBUG, "Spawning test_get_users_and_listen thread.");
    //     return_value.spawn(edge_view::client::test_get_users_and_listen());
    // }

    // thread::sleep(time::Duration::from_secs(5));

    // match args.spin_client {
    //     Some(clients) => {
    //         for endpoint in clients {
    //             event!(Level::DEBUG, "Spawning spin client for endpoint: {}", endpoint);

    //             return_value.spawn(edge_view::client::spin_client(endpoint.clone()));
    //         }
    //     }
    //     _ => {}
    // }

    return_value
} // end process arguments