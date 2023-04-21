use crate::utils;

use tungstenite::{connect, Message};
use tokio::sync::Mutex;
use std::sync::Arc;
use serde_json::Value;
use url::Url;


static BYBIT_WS_API: &str = "wss://stream.bybit.com/v5/public/spot";

static DB_POSITION: usize = 1;

pub async fn websocket_bybit(shared_prices: Arc<Mutex<[[[f64; 10]; 2]; 6]>>) {

  // Confirm URL
  let bybit_url = format!("{}", BYBIT_WS_API);

  // Connect to websocket
  let (mut socket, _) = connect(Url::parse(&bybit_url).unwrap()).expect("Can't connect.");

  // Subscribe message
  let subscribe_msg = r#"{
    "req_id": "bybit",
    "op": "subscribe",
    "args": [
        "orderbook.1.BTCUSDT",
        "orderbook.1.ETHUSDT"
    ]
  }"#;

  // Subscribe to DYDX markets
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
    if parsed_data["ret_msg"] == "subscribe" {
      println!("Successfully subscribed to Bybit.");
      continue;
    }

    // Handle Snapshot
    // Overwrite best bid and best ask
    if parsed_data["type"] == "snapshot" {
      symbol = parsed_data["data"]["s"].as_str().unwrap();
      best_ask = parsed_data["data"]["a"][0][0].as_str().unwrap().parse().unwrap();
      best_bid = parsed_data["data"]["b"][0][0].as_str().unwrap().parse().unwrap();
      utils::update_prices_db(shared_prices.clone(), symbol, DB_POSITION, best_ask, best_bid).await;
      continue;
    }

    // Handle Delta
    if parsed_data["type"] == "delta" {

      // Confirm if number is present
      let is_askdelta: bool = parsed_data["data"]["a"].as_array().unwrap().len() > 0;
      let is_biddelta: bool = parsed_data["data"]["b"].as_array().unwrap().len() > 0;

      // Update best ask
      if is_askdelta {
        best_ask = parsed_data["data"]["a"][0][0].as_str().unwrap().parse().unwrap();
      }

      // Update best bid
      if is_biddelta {
        best_bid = parsed_data["data"]["b"][0][0].as_str().unwrap().parse().unwrap();
      }

      // Extract symbol
      symbol = parsed_data["data"]["s"].as_str().unwrap();

      // Update DB with latest prices
      utils::update_prices_db(shared_prices.clone(), symbol, DB_POSITION, best_ask, best_bid).await;
      continue;
    }
  }
}
