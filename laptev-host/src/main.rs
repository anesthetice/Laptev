use std::{
    sync::Arc
};
use tokio::{
    sync::RwLock
};
use axum::{extract::FromRef, Router};

mod config;
mod data;
mod error;
mod utils;
mod web;

#[tokio::main]
async fn main() {

}

