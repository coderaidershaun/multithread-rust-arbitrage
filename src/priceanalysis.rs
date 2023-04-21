use crate::utils;
use tokio::sync::Mutex;
use std::sync::Arc;


// Calculate price differences
fn divide_inner_arrays(data: &utils::DBArray) -> Vec<[f64; utils::DB_SIZE]> {
  let mut result = Vec::new();

  for collection in data.iter() {
    let mut divided_values = [0.0; utils::DB_SIZE];
    let first_inner_array = &collection[0];
    let second_inner_array = &collection[1];

    for i in 0..utils::DB_SIZE {
      if first_inner_array[i] != 0.0 && second_inner_array[i] != 0.0 {
        let val = first_inner_array[i] / second_inner_array[i];
        let rounded_result = format!("{:.5}", val).parse::<f64>().unwrap_or_default();
        divided_values[i] = rounded_result;
      } else {
        divided_values[i] = 0.0;
      }
    }

    result.push(divided_values);
  }

  println!("{:?}", &result);
  return result;
}


// Import analysis
pub async fn analyze_prices(shared_prices: Arc<Mutex<[[[f64; 10]; 2]; 6]>>) {

  // Period to wait for checking prices
  println!("Starting analysis in 5 seconds...");
  utils::sleep(5000).await;

  loop {
      // Period to wait for checking prices
      utils::sleep(1000).await;

      // Get a new instance of the shared prices
      // Adding in brackets to ensure lock removed whilst operation continues
      let cloned_prices: [[[f64; 10]; 2]; 6];
      {
        // Access the shared prices and analyze them
        let locked_prices = shared_prices.lock().await;
        cloned_prices = locked_prices.clone();
      }
      
      // Get price differences
      divide_inner_arrays(&cloned_prices);


      // // Find arbitrage
      // println!("{:?}", cloned_prices);
  }
}
