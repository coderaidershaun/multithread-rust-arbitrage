use crate::utils;

use tungstenite::{connect, Message};
use tokio::sync::Mutex;
use std::sync::Arc;
use serde_json::Value;
use url::Url;


static BINANCE_WS_API: &str = "wss://stream.binance.com:9443";

static DB_POSITION: usize = 0;

pub async fn websocket_binance(shared_prices: Arc<Mutex<[[[f64; 10]; 2]; 6]>>) {

  // Confirm URL
  let binance_url = format!("{}/stream?streams=btcusdt@bookTicker/ethusdt@bookTicker/linkusdt@bookTicker", BINANCE_WS_API);

  // Connect to websocket
  let (mut socket, _) = connect(Url::parse(&binance_url).unwrap()).expect("Can't connect.");

  // Read WS data
  loop {

    // Get socket message
    let msg = socket.read_message().expect("Error reading message");
    let msg = match msg {
      Message::Text(s) => s,
      _ => panic!("Error getting text"),
    };

    // Parse text data
    let parsed_data: Value = serde_json::from_str(&msg).expect("Unable to parse Binance message");

    // Determine data position
    let symbol = parsed_data["data"]["s"].as_str().unwrap().to_uppercase();

    // Extract Bid and Ask price
    let best_ask = parsed_data["data"]["a"].as_str().unwrap().parse::<f64>().unwrap();
    let best_bid = parsed_data["data"]["b"].as_str().unwrap().parse::<f64>().unwrap();

    // Store price in data
    utils::update_prices_db(shared_prices.clone(), &symbol, DB_POSITION, best_ask, best_bid).await;
  }
}
