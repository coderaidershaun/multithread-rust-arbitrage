use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;


// Confirm Database Size
pub const DB_SIZE: usize = 10;


// Confirm Database Type
pub type DBArray = [[[f64; DB_SIZE]; 2]; 6];


// Sleep
pub async fn sleep(milliseconds: u64) {
  tokio::time::sleep(tokio::time::Duration::from_millis(milliseconds)).await;
}


// Get time in milliseconds
pub fn timenow() -> u128 {
  let now = SystemTime::now();
  let duration_since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
  duration_since_epoch.as_millis()
}


// Create hashmap index of where prices will be stored
// Storing the index on the prices array for any differing symbols
// This is so the websockets know where to update the bid and ask prices
pub fn coin_indexes() -> HashMap<String, usize> {
    let mut crypto_indices: HashMap<String, usize> = HashMap::new();
    crypto_indices.insert("BTCUSDT".to_string(), 0);
    crypto_indices.insert("BTC-USD".to_string(), 0);

    crypto_indices.insert("ETHUSDT".to_string(), 1);
    crypto_indices.insert("ETH-USD".to_string(), 1);

    crypto_indices.insert("LINKUSDT".to_string(), 2);
    crypto_indices.insert("LINK-USD".to_string(), 2);

    return crypto_indices;
}


// Update prices array db
pub async fn update_prices_db(
  shared_prices: Arc<Mutex<[[[f64; 10]; 2]; 6]>>, 
  symbol: &str,
  db_position: usize,
  ask_price: f64,
  bid_price: f64,
) {

  // Get coin index hashmap
  let index_lookup = coin_indexes();

  // Determine trading pair index
  let db_index = index_lookup[symbol];
  { 
    // Acquire the Mutex lock
    let mut locked_prices = shared_prices.lock().await;
  
    // Store price in data
    locked_prices[db_position][0][db_index] = ask_price;
    locked_prices[db_position][1][db_index] = bid_price;
  }
}


// Check for price change
pub fn current_price_check(
  symbol: &str,
  last_prices: &mut HashMap<String, f64>,
  current_ask: &f64,
  current_bid: &f64
) -> bool {
  let mut last_ask: f64 = 0.0;
  let mut last_bid: f64 = 0.0;

  // Get or set last ask and bid price for symbol
  let mut is_initial: bool = false;
  let symbol_ref_ask: String = symbol.to_owned() + "ask";
  let symbol_ref_bid: String = symbol.to_owned() + "bid";
  match last_prices.get(symbol_ref_ask.as_str()) {
    Some(val) => {
      last_ask = *val;
      last_bid = last_prices[symbol_ref_bid.as_str()];
    },
    None => {
      is_initial = true;
      last_ask = *current_ask;
      last_bid = *current_bid;
    }
  }

  // Update last prices with current ask and bid
  last_prices.insert(symbol_ref_ask, last_ask);
  last_prices.insert(symbol_ref_bid, last_bid);

  // Return boolean
  return is_initial || *current_ask != last_ask || *current_bid != last_bid;
}