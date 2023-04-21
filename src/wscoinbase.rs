use crate::utils;

use tungstenite::{connect, Message};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;
use url::Url;


static COINBASE_WS_API: &str = "wss://ws-feed.pro.coinbase.com";

static DB_POSITION: usize = 2;

pub async fn websocket_coinbase(shared_prices: Arc<Mutex<utils::DBArray>>) {

  // Monitoring for price changes. Store last price in hashmap
  let mut last_prices: HashMap<String, f64> = HashMap::new();

  // Confirm URL
  let coinbase_url = format!("{}", COINBASE_WS_API);

  // Connect to websocket
  let (mut socket, _) = connect(Url::parse(&coinbase_url).unwrap()).expect("Can't connect.");

  // Subscribe message
  let subscribe_msg = r#"{
    "type": "subscribe",
    "product_ids": [
        "BTC-USD",
        "ETH-USD",
        "LINK-USD"
    ],
    "channels": ["ticker_batch"]
  }"#;

  // Subscribe to Coinbase markets
  socket.write_message(Message::Text(subscribe_msg.to_string())).unwrap();

  // Read WS data
  let mut symbol: &str = "";
  let mut current_ask: f64 = 0.0;
  let mut current_bid: f64 = 0.0;
  loop {

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

      // Extract symbol
      symbol = parsed_data["product_id"].as_str().unwrap();

      // Get reference
      let symbol_ref_ask: String = symbol.to_owned() + "ask";
      let symbol_ref_bid: String = symbol.to_owned() + "bid";

      // Extract bid and ask
      current_ask = parsed_data["best_ask"].as_str().unwrap().parse().unwrap();
      current_bid = parsed_data["best_bid"].as_str().unwrap().parse().unwrap();

      // Check if price has changed
      let is_price_changed = utils::current_price_check(symbol, &mut last_prices, &current_ask, &current_bid);
      
      // Store price in data if datetime allows
      if is_price_changed {
        utils::update_prices_db(shared_prices.clone(), symbol, DB_POSITION, current_ask, current_bid).await;
        // println!("{:?}", "coinbase updated");
      }
      continue;
    }
  }
}
