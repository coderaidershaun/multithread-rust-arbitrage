use crate::utils;
use tokio::sync::Mutex;
use std::sync::Arc;


// Calculate price differences
fn calculate_price_differences(data: &utils::DBArray) 
-> [[[f64; utils::EXCH_COUNT]; utils::COIN_COUNT]; utils::EXCH_COUNT] {

  /*
    When bid / ask > 1: There is an arbitrage opportunity
    The bid / ask > 1 on the most inner array represents the exch you would buy at
    The respective index of the next level up represents the asset you would trade
    The respective index of the final level up represents the exch you would sell at
    This function provides the output structure to make such calculations

    Output Structure:
      [
          exch0     exch2    exch3    exch4    exch5   exch6    <- buy exchanges
        [sell exchange 0
          [1.01234, 0.99999, 1.00008, 1.21233, 0.9998, 0.9997], <- asset 0
          [0.99823, 0.99465, 0.99765, 0.98762, 1.0523, 0.9945], <- asset 1
          [1.34565, 1.01287, 1.00203, 0.99972, 0.9922, 1.0978], <- asset 2
          ...
          [1.02562, 1.11289, 1.02302, 0.99972, 0.8427, 0.7865], <- asset 10
        ],
        ...
        [sell exchange 6],
      ]
  */

  let mut ratios = [[[0.0; utils::EXCH_COUNT]; utils::COIN_COUNT]; utils::EXCH_COUNT];

  // For each exchange
  for (index, asksbids) in data.iter().enumerate() {
    let bids_array = &asksbids[1];
    let mut exch_array = [[0.0; utils::EXCH_COUNT]; utils::COIN_COUNT];

    // For each bid price
    for i in 0..utils::COIN_COUNT {
      let bid_price = bids_array[i];
      let mut divided_values = [0.0; utils::EXCH_COUNT];

      // For each exchange same coin as bid price coin
      for j in 0..utils::EXCH_COUNT {
        let ask_price = data[j][0][i];

        // Ensure ask and bid price
        let mut rounded_val: f64 = 0.0;
        if ask_price != 0.0 && bid_price != 0.0 {

          // Get bid price ratio to ask price
          let division = bid_price / ask_price;
          rounded_val = format!("{:.5}", division).parse::<f64>().unwrap_or_default();
        }

        // Place value in array
        divided_values[j] = rounded_val;
      }
      
      // Add to exch array
      exch_array[i] = divided_values;
    }
    
    // Add to ratios
    ratios[index] = exch_array;
  }

  // Return result
  return ratios;
}


// Import analysis
pub async fn analyze_prices(shared_prices: Arc<Mutex<utils::DBArray>>) {

  // Period to wait for checking prices
  println!("Starting analysis in 5 seconds...");
  utils::sleep(5000).await;

  loop {
      // Period to wait for checking prices
      utils::sleep(500).await;

      // Get a new instance of the shared prices
      // Adding in brackets to ensure lock removed whilst operation continues
      let cloned_prices: utils::DBArray;
      {
        // Access the shared prices and analyze them
        let locked_prices = shared_prices.lock().await;
        cloned_prices = locked_prices.clone();
      }
      
      // Get price differences
      let ratios = calculate_price_differences(&cloned_prices);

      // // Find arbitrage
      println!("{:?}", ratios);
  }
}
