use crate::utils;

use tungstenite::{connect, Message};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;
use url::Url;


static BYBIT_WS_API: &str = "wss://stream.bybit.com/v5/public/spot";

static DB_POSITION: usize = 1;

pub async fn websocket_bybit(shared_prices: Arc<Mutex<utils::DBArray>>) {

  // Monitoring for price changes. Store last price in hashmap
  let mut last_prices: HashMap<String, f64> = HashMap::new();

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
        "orderbook.1.ETHUSDT",
        "orderbook.1.LINKUSDT"
    ]
  }"#;

  // Subscribe to DYDX markets
  socket.write_message(Message::Text(subscribe_msg.to_string())).unwrap();

  // Read WS data
  let mut symbol: &str = "";
  let mut current_ask: f64 = 0.0;
  let mut current_bid: f64 = 0.0;
  loop {
      
    // Get current timestamp
    let ts = utils::timenow();

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
    if parsed_data["type"] == "snapshot" {

      // Extract data
      symbol = parsed_data["data"]["s"].as_str().unwrap();
      current_ask = parsed_data["data"]["a"][0][0].as_str().unwrap().parse().unwrap();
      current_bid = parsed_data["data"]["b"][0][0].as_str().unwrap().parse().unwrap();

      // Check if price has changed
      let is_price_changed = utils::current_price_check(symbol, &mut last_prices, &current_ask, &current_bid);

      // Store price in data if datetime allows
      if is_price_changed {
        utils::update_prices_db(shared_prices.clone(), symbol, DB_POSITION, current_ask, current_bid).await;
      }
      continue;
    }

    // Handle Delta
    if parsed_data["type"] == "delta" {

      // Confirm if number is present
      let is_askdelta: bool = parsed_data["data"]["a"].as_array().unwrap().len() > 0;
      let is_biddelta: bool = parsed_data["data"]["b"].as_array().unwrap().len() > 0;

      // Extract symbol
      symbol = parsed_data["data"]["s"].as_str().unwrap();

      // Get reference
      let symbol_ref_ask: String = symbol.to_owned() + "ask";
      let symbol_ref_bid: String = symbol.to_owned() + "bid";

      // Update best ask
      if is_askdelta {
        current_ask = parsed_data["data"]["a"][0][0].as_str().unwrap().parse().unwrap();
      } else {
        current_ask = last_prices[&symbol_ref_ask];
      }

      // Update best bid
      if is_biddelta {
        current_bid = parsed_data["data"]["b"][0][0].as_str().unwrap().parse().unwrap();
      } else {
        current_bid = last_prices[&symbol_ref_bid];
      }

      // Check if price has changed
      let is_price_changed = utils::current_price_check(symbol, &mut last_prices, &current_ask, &current_bid);
      
      // Store price in data if datetime allows
      if is_price_changed {
        utils::update_prices_db(shared_prices.clone(), symbol, DB_POSITION, current_ask, current_bid).await;
        // println!("{:?}", "bybit updated");
      }
      continue;
    }
  }
}
