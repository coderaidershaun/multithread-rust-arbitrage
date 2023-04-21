use std::collections::HashMap;
use tokio::sync::Mutex;
use std::sync::Arc;


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

  // Acquire the Mutex lock
  let mut locked_prices = shared_prices.lock().await;

  // Store price in data
  locked_prices[db_position][0][db_index] = ask_price;
  locked_prices[db_position][1][db_index] = bid_price;

  // // Show updated DB
  // println!("{:?}", locked_prices);
}
