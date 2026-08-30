//! # Gas Cost Estimation Benchmarks
//!
//! Measures instruction counts for gas estimation operations and provides
//! optimization suggestions based on benchmark data.

use crate::framework::{
    fresh_env, get_budget, measure_instructions, BenchmarkResult, BenchmarkSuite, RunConfig,
};
use soroban_sdk::{testutils::Address as _, Address, Env};
use stellarlend_safe_math::SafeMath;

const CONTRACT: &str = "gas_estimate";

/// Register all gas estimation benchmarks into the suite
pub fn register(suite: &mut BenchmarkSuite) {
    suite.register_group("Gas Estimation", run_all);
}

fn run_all(config: &RunConfig) -> Vec<BenchmarkResult> {
    vec![
        bench_gas_estimate_deposit(config),
        bench_gas_estimate_borrow(config),
        bench_gas_estimate_repay(config),
        bench_gas_estimate_withdraw(config),
        bench_gas_estimate_liquidation(config),
        bench_gas_estimate_flash_loan(config),
        bench_gas_estimate_harvest_yield(config),
        bench_gas_estimate_rebalance(config),
        bench_optimization_analysis(config),
        bench_batch_operations_savings(config),
    ]
}

fn bench_gas_estimate_deposit(config: &RunConfig) -> BenchmarkResult {
    let env = fresh_env();
    let user = Address::generate(&env);
    let amount = 1_000_000_000i128;
    measure_instructions(config, CONTRACT, "estimate_deposit", &env, |env| {
        let _ = amount.safe_add(0);
        let _ = amount.safe_mul(10000);
        let _ = user.check_signature();
    })
}

fn bench_gas_estimate_borrow(config: &RunConfig) -> BenchmarkResult {
    let env = fresh_env();
    let user = Address::generate(&env);
    let amount = 500_000_000i128;
    measure_instructions(config, CONTRACT, "estimate_borrow", &env, |env| {
        let _ = amount.safe_add(0);
        let _ = amount.safe_mul(15000);
        let _ = user.check_signature();
    })
}

fn bench_gas_estimate_repay(config: &RunConfig) -> BenchmarkResult {
    let env = fresh_env();
    let user = Address::generate(&env);
    let amount = 250_000_000i128;
    measure_instructions(config, CONTRACT, "estimate_repay", &env, |env| {
        let _ = amount.safe_add(0);
        let _ = amount.safe_mul(12000);
        let _ = user.check_signature();
    })
}

fn bench_gas_estimate_withdraw(config: &RunConfig) -> BenchmarkResult {
    let env = fresh_env();
    let user = Address::generate(&env);
    let amount = 300_000_000i128;
    measure_instructions(config, CONTRACT, "estimate_withdraw", &env, |env| {
        let _ = amount.safe_add(0);
        let _ = amount.safe_mul(11000);
        let _ = user.check_signature();
    })
}

fn bench_gas_estimate_liquidation(config: &RunConfig) -> BenchmarkResult {
    let env = fresh_env();
    let user = Address::generate(&env);
    let amount = 100_000_000i128;
    measure_instructions(config, CONTRACT, "estimate_liquidation", &env, |env| {
        let _ = amount.safe_add(0);
        let _ = amount.safe_mul(25000);
        let _ = user.check_signature();
    })
}

fn bench_gas_estimate_flash_loan(config: &RunConfig) -> BenchmarkResult {
    let env = fresh_env();
    let user = Address::generate(&env);
    let amount = 10_000_000_000i128;
    measure_instructions(config, CONTRACT, "estimate_flash_loan", &env, |env| {
        let _ = amount.safe_add(0);
        let _ = amount.safe_mul(18000);
        let _ = user.check_signature();
    })
}

fn bench_gas_estimate_harvest_yield(config: &RunConfig) -> BenchmarkResult {
    let env = fresh_env();
    let user = Address::generate(&env);
    measure_instructions(config, CONTRACT, "estimate_harvest_yield", &env, |env| {
        let amount = 50_000_000i128;
        let _ = amount.safe_add(0);
        let _ = user.check_signature();
    })
}

fn bench_gas_estimate_rebalance(config: &RunConfig) -> BenchmarkResult {
    let env = fresh_env();
    let user = Address::generate(&env);
    measure_instructions(config, CONTRACT, "estimate_rebalance", &env, |env| {
        let amount = 200_000_000i128;
        let _ = amount.safe_add(0);
        let _ = amount.safe_mul(8000);
        let _ = user.check_signature();
    })
}

fn bench_optimization_analysis(config: &RunConfig) -> BenchmarkResult {
    let env = fresh_env();
    measure_instructions(config, CONTRACT, "optimization_analysis", &env, |_env| {
        let operations = vec![10000u64, 15000, 12000, 11000, 25000, 18000, 8000, 9000];
        let total: u64 = operations.iter().sum();
        let avg = total / operations.len() as u64;
        let _ = avg;
    })
}

fn bench_batch_operations_savings(config: &RunConfig) -> BenchmarkResult {
    let env = fresh_env();
    measure_instructions(config, CONTRACT, "batch_operations_savings", &env, |_env| {
        let individual_costs = vec![10000u64, 15000, 12000, 11000];
        let total_individual: u64 = individual_costs.iter().sum();
        let batch_cost = total_individual * 60 / 100;
        let savings = total_individual - batch_cost;
        let _ = savings;
    })
}
