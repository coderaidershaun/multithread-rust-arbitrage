mod priceanalysis;
mod utils;
mod wsbinance;
mod wsbybit;
mod wscoinbase;

use std::sync::Arc;
use tokio::sync::Mutex;

use priceanalysis::analyze_prices;
use wsbinance::websocket_binance;
use wsbybit::websocket_bybit;
use wscoinbase::websocket_coinbase;
use utils::{ DBArray, COIN_COUNT };


// Executes tasks concurrently


#[tokio::main]
async fn main() {

    // Create price storage array
    // 10 represents the number of token symbols (i.e BTCUSDT, ETHUSDT, etc...)
    let prices: DBArray = [
        // Asks     // Bids
        [[0.0; COIN_COUNT], [0.0; COIN_COUNT]], // Binance
        [[0.0; COIN_COUNT], [0.0; COIN_COUNT]], // Coinbase
        [[0.0; COIN_COUNT], [0.0; COIN_COUNT]], // ByBit
        [[0.0; COIN_COUNT], [0.0; COIN_COUNT]], // Uniswap V3
        [[0.0; COIN_COUNT], [0.0; COIN_COUNT]], // Uniswap V2
        [[0.0; COIN_COUNT], [0.0; COIN_COUNT]], // Curve
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

    // Spawn the analyze_prices task
    let analyze_shared_prices = shared_prices.clone();
    let analyze_task = tokio::spawn(async move {
        analyze_prices(analyze_shared_prices).await;
    });

    // Wait for all tasks to complete
    let result = tokio::try_join!(binance_task, bybit_task, coinbase_task, analyze_task);
}
