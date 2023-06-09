use crate::utils;

use tungstenite::{connect, Message};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;
use url::Url;


static BINANCE_WS_API: &str = "wss://stream.binance.com:9443";

static DB_POSITION: usize = 0;

pub async fn websocket_binance(shared_prices: Arc<Mutex<utils::DBArray>>) {

  // Monitoring for price changes. Store last price in hashmap
  let mut last_prices: HashMap<String, f64> = HashMap::new();

  // Confirm URL
  let binance_url = format!("{}/stream?streams=btcusdt@bookTicker/ethusdt@bookTicker/linkusdt@bookTicker", BINANCE_WS_API);

  // Connect to websocket
  let (mut socket, _) = connect(Url::parse(&binance_url).unwrap()).expect("Can't connect.");
  println!("Successfully subscribed to Binance.");

  // Read WS data
  let mut current_ask: f64;
  let mut current_bid: f64;
  loop {

    // Get socket message
    let msg = socket.read_message().expect("Error reading message");
    let msg = match msg {
      Message::Text(s) => s,
      _ => { 
        println!("Binance not connected");
        continue;
      },
    };

    // Parse text data
    let parsed_data: Value = serde_json::from_str(&msg).expect("Unable to parse Binance message");

    // Determine data position
    let symbol = &parsed_data["data"]["s"].as_str().unwrap().to_uppercase();

    // Get reference
    let symbol_ref_ask: String = symbol.to_owned() + "ask";
    let symbol_ref_bid: String = symbol.to_owned() + "bid";

    // Extract Bid and Ask price
    let current_ask = parsed_data["data"]["a"].as_str().unwrap().parse::<f64>().unwrap();
    let current_bid = parsed_data["data"]["b"].as_str().unwrap().parse::<f64>().unwrap();

    // Check if price has changed
    let is_price_changed = utils::current_price_check(symbol, &mut last_prices, &current_ask, &current_bid);
    
    // Store price in data if datetime allows
    if is_price_changed {
      utils::update_prices_db(shared_prices.clone(), symbol, DB_POSITION, current_ask, current_bid).await;
      // println!("{:?}", "binance updated");
    }
  }
}
