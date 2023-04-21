use crate::utils;

use tungstenite::{connect, Message};
use tokio::sync::Mutex;
use std::sync::Arc;
use serde_json::Value;
use url::Url;

static COINBASE_WS_API: &str = "wss://ws-feed.pro.coinbase.com";

static DB_POSITION: usize = 2;

pub async fn websocket_coinbase(shared_prices: Arc<Mutex<[[[f64; 10]; 2]; 6]>>) {

  // Confirm URL
  let coinbase_url = format!("{}", COINBASE_WS_API);

  // Connect to websocket
  let (mut socket, _) = connect(Url::parse(&coinbase_url).unwrap()).expect("Can't connect.");

  // Subscribe message
  let subscribe_msg = r#"{
    "type": "subscribe",
    "product_ids": [
        "BTC-USD",
        "ETH-USD"
    ],
    "channels": ["ticker_batch"]
  }"#;

  // Subscribe to Coinbase markets
  socket.write_message(Message::Text(subscribe_msg.to_string())).unwrap();

  // Read WS data
  loop {

    // Initialize best bid and best ask
    let mut symbol: &str = "";
    let mut best_bid: f64 = 0.0;
    let mut best_ask: f64 = 0.0;

    // Get socket message
    let msg = socket.read_message().expect("Error reading message");
    let msg = match msg {
        Message::Text(s) => s,
        _ => panic!("Error getting text"),
    };

    // Parse text data
    let parsed_data: Value = serde_json::from_str(&msg).expect("Unable to parse message");

    // Handle Subscribed
    if parsed_data["type"] == "subscriptions" {
      println!("Successfully subscribed to Coinbase.");
      continue;
    }

    // Handle Order book update
    if parsed_data["type"] == "ticker" {

      // Update best ask
      best_ask = parsed_data["best_ask"].as_str().unwrap().parse().unwrap();

      // Update best bid
      best_bid = parsed_data["best_bid"].as_str().unwrap().parse().unwrap();

      // Extract symbol
      symbol = parsed_data["product_id"].as_str().unwrap();

      // Update DB with latest prices
      utils::update_prices_db(shared_prices.clone(), symbol, DB_POSITION, best_ask, best_bid).await;
      continue;
    }
  }
}
