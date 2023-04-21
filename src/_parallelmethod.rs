mod priceanalysis;
mod utils;
mod wsbinance;
mod wsbybit;
mod wscoinbase;

use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::watch;

use priceanalysis::analyze_prices;
use wsbinance::websocket_binance;
use wsbybit::websocket_bybit;
use wscoinbase::websocket_coinbase;


// Executes in parallel


#[tokio::main]
async fn main() {
    // Create price storage array
    // 10 represents the number of token symbols (i.e BTCUSDT, ETHUSDT, etc...)
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
    let binance_task = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(websocket_binance(binance_shared_prices));
    });

    // Engage ByBit websocket
    let bybit_shared_prices = shared_prices.clone();
    let bybit_task = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(websocket_bybit(bybit_shared_prices));
    });

    // Engage Coinbase websocket
    let coinbase_shared_prices = shared_prices.clone();
    let coinbase_task = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(websocket_coinbase(coinbase_shared_prices));
    });

    // Spawn the analyze_prices task
    let analyze_shared_prices = shared_prices.clone();
    let analyze_task = tokio::task::spawn_blocking(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(analyze_prices(analyze_shared_prices));
    });

    // Wait for all tasks to complete
    let result = tokio::try_join!(binance_task, bybit_task, coinbase_task, analyze_task);
}
