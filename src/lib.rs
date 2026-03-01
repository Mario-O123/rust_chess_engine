#![recursion_limit = "256"] //necessary for neural
pub mod board;
pub mod bot;
pub mod engine;
pub mod evaluation;
pub mod movegen;
pub mod nn_model;
pub mod position;
pub mod search;
pub mod trainer_rust;
pub mod utils;
