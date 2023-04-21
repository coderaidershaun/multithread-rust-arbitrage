mod utils;
mod wsbinance;
mod wsbybit;
mod wscoinbase;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::watch;

use wsbinance::websocket_binance;
use wsbybit::websocket_bybit;
use wscoinbase::websocket_coinbase;


#[tokio::main]
async fn main() {

    // Create price storage array
    // 10 represents the number of tokens
    let prices: [[[f64; 10]; 2]; 6] = [
        // Asks     // Bids
        [[0.0; 10], [0.0; 10]], // Binance
        [[0.0; 10], [0.0; 10]], // Coinbase
        [[0.0; 10], [0.0; 10]], // ByBit
        [[0.0; 10], [0.0; 10]], // Uniswap V3
        [[0.0; 10], [0.0; 10]], // Uniswap V2
        [[0.0; 10], [0.0; 10]], // Curve
    ];

    // Create variable on the heap to allow for sharing and locking variable
    let shared_prices = Arc::new(Mutex::new(prices));

    // Engage Binance websocket
    let binance_shared_prices = shared_prices.clone();
    let binance_task = tokio::spawn(async move {
        websocket_binance(binance_shared_prices).await;
    });

    // Engage ByBit websocket
    let bybit_shared_prices = shared_prices.clone();
    let bybit_task = tokio::spawn(async move {
        websocket_bybit(bybit_shared_prices).await;
    });

    // Engage Coinbase websocket
    let coinbase_shared_prices = shared_prices.clone();
    let coinbase_task = tokio::spawn(async move {
        websocket_coinbase(coinbase_shared_prices).await;
    });

    // Wait for all tasks to complete
    let result = tokio::try_join!(binance_task, bybit_task, coinbase_task);
}
