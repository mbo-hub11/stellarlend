#![no_std]
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    contract, contracterror, contracttype, symbol_short, Address, Env, Map, String, Symbol, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ReferralError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    AlreadyRegistered = 4,
    InvalidReferrer = 5,
    SelfReferral = 6,
    NothingToClaim = 7,
    InvalidCode = 8,
    InvalidAmount = 9,
    Overflow = 10,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct AffiliateCode {
    pub owner: Address,
    pub code: Symbol,
    pub created_at: u64,
    pub total_earned: i128,
    pub total_referrals: u32,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ReferralRecord {
    pub referrer: Address,
    pub referee: Address,
    pub registered_at: u64,
    pub total_fees_generated: i128,
    pub referrer_earned: i128,
    pub referee_discount_bps: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct AffiliateStats {
    pub total_referrals: u32,
    pub total_earned: i128,
    pub total_claimed: i128,
    pub claimable: i128,
    pub lifetime_fees_generated: i128,
    pub tier: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct ReferralConfig {
    pub admin: Address,
    pub reward_bps: u32,
    pub referee_discount_bps: u32,
    pub tier1_threshold: u32,
    pub tier1_bonus_bps: u32,
    pub tier2_threshold: u32,
    pub tier2_bonus_bps: u32,
    pub tier3_threshold: u32,
    pub tier3_bonus_bps: u32,
    pub min_deposit_to_qualify: i128,
    pub paused: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub struct GlobalReferralMetrics {
    pub total_codes_created: u32,
    pub total_referrals: u32,
    pub total_rewards_distributed: i128,
    pub total_fees_generated: i128,
    pub total_referee_discounts: i128,
}

const DEFAULT_REWARD_BPS: u32 = 2500;
const DEFAULT_REFEREE_DISCOUNT_BPS: u32 = 500;
const TIER1_THRESHOLD: u32 = 5;
const TIER1_BONUS_BPS: u32 = 500;
const TIER2_THRESHOLD: u32 = 15;
const TIER2_BONUS_BPS: u32 = 1000;
const TIER3_THRESHOLD: u32 = 50;
const TIER3_BONUS_BPS: u32 = 2000;
const MIN_DEPOSIT: i128 = 1_000_0000;
const BASIS_POINTS: i128 = 10_000;

#[contract]
pub struct ReferralContract;

#[contractimpl]
impl ReferralContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), ReferralError> {
        let config_key = Symbol::new(&env, "config");
        if env.storage().persistent().has(&config_key) {
            return Err(ReferralError::AlreadyInitialized);
        }
        admin.require_auth();
        let config = ReferralConfig {
            admin: admin.clone(),
            reward_bps: DEFAULT_REWARD_BPS,
            referee_discount_bps: DEFAULT_REFEREE_DISCOUNT_BPS,
            tier1_threshold: TIER1_THRESHOLD,
            tier1_bonus_bps: TIER1_BONUS_BPS,
            tier2_threshold: TIER2_THRESHOLD,
            tier2_bonus_bps: TIER2_BONUS_BPS,
            tier3_threshold: TIER3_THRESHOLD,
            tier3_bonus_bps: TIER3_BONUS_BPS,
            min_deposit_to_qualify: MIN_DEPOSIT,
            paused: false,
        };
        env.storage().persistent().set(&config_key, &config);
        env.storage().persistent().set(
            &Symbol::new(&env, "global_metrics"),
            &GlobalReferralMetrics {
                total_codes_created: 0,
                total_referrals: 0,
                total_rewards_distributed: 0,
                total_fees_generated: 0,
                total_referee_discounts: 0,
            },
        );
        Ok(())
    }

    pub fn register_code(env: Env, owner: Address, code: Symbol) -> Result<(), ReferralError> {
        let config = get_config(&env);
        if config.paused {
            return Err(ReferralError::Unauthorized);
        }
        owner.require_auth();
        let code_key = Symbol::new(&env, "code_");
        let code_key = symbol_from_parts(&env, &code_key, &code);
        if env.storage().persistent().has(&code_key) {
            return Err(ReferralError::AlreadyRegistered);
        }
        let affiliate_code = AffiliateCode {
            owner: owner.clone(),
            code: code.clone(),
            created_at: env.ledger().timestamp(),
            total_earned: 0,
            total_referrals: 0,
            is_active: true,
        };
        env.storage().persistent().set(&code_key, &affiliate_code);
        let owner_code_key = Symbol::new(&env, "owner_code_");
        let owner_code_key =
            symbol_from_parts(&env, &owner_code_key, &Symbol::new(&env, &to_str_val(&owner)));
        env.storage().persistent().set(&owner_code_key, &code);
        let mut metrics = get_global_metrics(&env);
        metrics.total_codes_created = metrics.total_codes_created.saturating_add(1);
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, "global_metrics"), &metrics);
        Ok(())
    }

    pub fn refer(env: Env, referee: Address, code: Symbol) -> Result<(), ReferralError> {
        let config = get_config(&env);
        if config.paused {
            return Err(ReferralError::Unauthorized);
        }
        referee.require_auth();
        let code_key = Symbol::new(&env, "code_");
        let code_key = symbol_from_parts(&env, &code_key, &code);
        let affiliate_code: AffiliateCode = env
            .storage()
            .persistent()
            .get(&code_key)
            .ok_or(ReferralError::InvalidCode)?;
        if !affiliate_code.is_active {
            return Err(ReferralError::InvalidCode);
        }
        if affiliate_code.owner == referee {
            return Err(ReferralError::SelfReferral);
        }
        let ref_key = Symbol::new(&env, "ref_");
        let ref_key = symbol_from_parts(&env, &ref_key, &Symbol::new(&env, &to_str_val(&referee)));
        if env.storage().persistent().has(&ref_key) {
            return Err(ReferralError::AlreadyRegistered);
        }
        let record = ReferralRecord {
            referrer: affiliate_code.owner.clone(),
            referee: referee.clone(),
            registered_at: env.ledger().timestamp(),
            total_fees_generated: 0,
            referrer_earned: 0,
            referee_discount_bps: config.referee_discount_bps,
        };
        env.storage().persistent().set(&ref_key, &record);
        let mut updated_code = affiliate_code.clone();
        updated_code.total_referrals = updated_code.total_referrals.saturating_add(1);
        env.storage().persistent().set(&code_key, &updated_code);
        let mut metrics = get_global_metrics(&env);
        metrics.total_referrals = metrics.total_referrals.saturating_add(1);
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, "global_metrics"), &metrics);
        Ok(())
    }

    pub fn record_fee(
        env: Env,
        referee: Address,
        fee_amount: i128,
    ) -> Result<(i128, i128), ReferralError> {
        if fee_amount <= 0 {
            return Err(ReferralError::InvalidAmount);
        }
        let config = get_config(&env);
        let ref_key = Symbol::new(&env, "ref_");
        let ref_key = symbol_from_parts(&env, &ref_key, &Symbol::new(&env, &to_str_val(&referee)));
        let mut record: ReferralRecord = env
            .storage()
            .persistent()
            .get(&ref_key)
            .ok_or(ReferralError::InvalidReferrer)?;
        let reward = (fee_amount * config.reward_bps as i128) / BASIS_POINTS;
        let tier_bonus = get_tier_bonus_bps(&config, record.referrer.clone());
        let bonus_amount = if tier_bonus > 0 {
            (reward * tier_bonus as i128) / BASIS_POINTS
        } else {
            0
        };
        let total_reward = reward.checked_add(bonus_amount).ok_or(ReferralError::Overflow)?;
        record.total_fees_generated = record
            .total_fees_generated
            .checked_add(fee_amount)
            .ok_or(ReferralError::Overflow)?;
        record.referrer_earned = record
            .referrer_earned
            .checked_add(total_reward)
            .ok_or(ReferralError::Overflow)?;
        env.storage().persistent().set(&ref_key, &record);
        let code_key = Symbol::new(&env, "code_");
        let owner_code_key = Symbol::new(&env, "owner_code_");
        let owner_code_key = symbol_from_parts(
            &env,
            &owner_code_key,
            &Symbol::new(&env, &to_str_val(&record.referrer)),
        );
        if let Some(code_symbol) = env.storage().persistent().get::<Symbol, Symbol>(&owner_code_key) {
            let code_lookup = symbol_from_parts(&env, &code_key, &code_symbol);
            if let Some(mut affiliate_code) = env.storage().persistent().get::<Symbol, AffiliateCode>(&code_lookup) {
                affiliate_code.total_earned = affiliate_code
                    .total_earned
                    .checked_add(total_reward)
                    .ok_or(ReferralError::Overflow)?;
                env.storage().persistent().set(&code_lookup, &affiliate_code);
            }
        }
        let discount = (fee_amount * config.referee_discount_bps as i128) / BASIS_POINTS;
        let mut metrics = get_global_metrics(&env);
        metrics.total_rewards_distributed = metrics
            .total_rewards_distributed
            .checked_add(total_reward)
            .ok_or(ReferralError::Overflow)?;
        metrics.total_fees_generated = metrics
            .total_fees_generated
            .checked_add(fee_amount)
            .ok_or(ReferralError::Overflow)?;
        metrics.total_referee_discounts = metrics
            .total_referee_discounts
            .checked_add(discount)
            .ok_or(ReferralError::Overflow)?;
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, "global_metrics"), &metrics);
        Ok((total_reward, discount))
    }

    pub fn claim_rewards(env: Env, claimant: Address) -> Result<i128, ReferralError> {
        claimant.require_auth();
        let config = get_config(&env);
        let owner_code_key = Symbol::new(&env, "owner_code_");
        let owner_code_key =
            symbol_from_parts(&env, &owner_code_key, &Symbol::new(&env, &to_str_val(&claimant)));
        let code_symbol: Symbol = env
            .storage()
            .persistent()
            .get(&owner_code_key)
            .ok_or(ReferralError::InvalidCode)?;
        let code_key = Symbol::new(&env, "code_");
        let code_key = symbol_from_parts(&env, &code_key, &code_symbol);
        let mut affiliate_code: AffiliateCode = env
            .storage()
            .persistent()
            .get(&code_key)
            .ok_or(ReferralError::InvalidCode)?;
        if affiliate_code.total_earned <= 0 {
            return Err(ReferralError::NothingToClaim);
        }
        let claimable = affiliate_code.total_earned;
        affiliate_code.total_earned = 0;
        env.storage().persistent().set(&code_key, &affiliate_code);
        Ok(claimable)
    }

    pub fn get_affiliate_stats(env: Env, user: Address) -> Result<AffiliateStats, ReferralError> {
        let owner_code_key = Symbol::new(&env, "owner_code_");
        let owner_code_key =
            symbol_from_parts(&env, &owner_code_key, &Symbol::new(&env, &to_str_val(&user)));
        let code_symbol: Symbol = env
            .storage()
            .persistent()
            .get(&owner_code_key)
            .ok_or(ReferralError::InvalidCode)?;
        let code_key = Symbol::new(&env, "code_");
        let code_key = symbol_from_parts(&env, &code_key, &code_symbol);
        let affiliate_code: AffiliateCode = env
            .storage()
            .persistent()
            .get(&code_key)
            .ok_or(ReferralError::InvalidCode)?;
        let config = get_config(&env);
        let tier = compute_tier(&config, affiliate_code.total_referrals);
        Ok(AffiliateStats {
            total_referrals: affiliate_code.total_referrals,
            total_earned: affiliate_code.total_earned,
            total_claimed: 0,
            claimable: affiliate_code.total_earned,
            lifetime_fees_generated: 0,
            tier,
        })
    }

    pub fn get_referral_record(env: Env, referee: Address) -> Option<ReferralRecord> {
        let ref_key = Symbol::new(&env, "ref_");
        let ref_key = symbol_from_parts(&env, &ref_key, &Symbol::new(&env, &to_str_val(&referee)));
        env.storage().persistent().get(&ref_key)
    }

    pub fn get_affiliate_code(env: Env, owner: Address) -> Option<AffiliateCode> {
        let owner_code_key = Symbol::new(&env, "owner_code_");
        let owner_code_key =
            symbol_from_parts(&env, &owner_code_key, &Symbol::new(&env, &to_str_val(&owner)));
        let code_symbol: Symbol = match env.storage().persistent().get(&owner_code_key) {
            Some(s) => s,
            None => return None,
        };
        let code_key = Symbol::new(&env, "code_");
        let code_key = symbol_from_parts(&env, &code_key, &code_symbol);
        env.storage().persistent().get(&code_key)
    }

    pub fn get_global_metrics(env: Env) -> GlobalReferralMetrics {
        get_global_metrics(&env)
    }

    pub fn get_config(env: Env) -> ReferralConfig {
        get_config(&env)
    }

    pub fn update_config(
        env: Env,
        admin: Address,
        reward_bps: Option<u32>,
        referee_discount_bps: Option<u32>,
        paused: Option<bool>,
    ) -> Result<(), ReferralError> {
        let mut config = get_config(&env);
        if admin != config.admin {
            return Err(ReferralError::Unauthorized);
        }
        admin.require_auth();
        if let Some(r) = reward_bps {
            config.reward_bps = r;
        }
        if let Some(d) = referee_discount_bps {
            config.referee_discount_bps = d;
        }
        if let Some(p) = paused {
            config.paused = p;
        }
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, "config"), &config);
        Ok(())
    }
}

fn get_config(env: &Env) -> ReferralConfig {
    env.storage()
        .persistent()
        .get(&Symbol::new(env, "config"))
        .unwrap_or(ReferralConfig {
            admin: Address::from_string(&String::from_str(env, "______________________________________________________________________")),
            reward_bps: DEFAULT_REWARD_BPS,
            referee_discount_bps: DEFAULT_REFEREE_DISCOUNT_BPS,
            tier1_threshold: TIER1_THRESHOLD,
            tier1_bonus_bps: TIER1_BONUS_BPS,
            tier2_threshold: TIER2_THRESHOLD,
            tier2_bonus_bps: TIER2_BONUS_BPS,
            tier3_threshold: TIER3_THRESHOLD,
            tier3_bonus_bps: TIER3_BONUS_BPS,
            min_deposit_to_qualify: MIN_DEPOSIT,
            paused: false,
        })
}

fn get_global_metrics(env: &Env) -> GlobalReferralMetrics {
    env.storage()
        .persistent()
        .get(&Symbol::new(env, "global_metrics"))
        .unwrap_or(GlobalReferralMetrics {
            total_codes_created: 0,
            total_referrals: 0,
            total_rewards_distributed: 0,
            total_fees_generated: 0,
            total_referee_discounts: 0,
        })
}

fn compute_tier(config: &ReferralConfig, total_referrals: u32) -> u32 {
    if total_referrals >= config.tier3_threshold {
        3
    } else if total_referrals >= config.tier2_threshold {
        2
    } else if total_referrals >= config.tier1_threshold {
        1
    } else {
        0
    }
}

fn get_tier_bonus_bps(config: &ReferralConfig, referrer: Address) -> u32 {
    let owner_code_key = Symbol::new(&env_no_op(), "owner_code_");
    let owner_code_key = symbol_from_parts(
        &env_no_op(),
        &owner_code_key,
        &Symbol::new(&env_no_op(), &to_str_val(&referrer)),
    );
    let code_symbol: Option<Symbol> = None;
    if let Some(sym) = code_symbol {
        let code_key = Symbol::new(&env_no_op(), "code_");
        let code_key = symbol_from_parts(&env_no_op(), &code_key, &sym);
        if let Some(affiliate_code) = None {
            let tier = compute_tier(config, affiliate_code.total_referrals);
            return match tier {
                3 => config.tier3_bonus_bps,
                2 => config.tier2_bonus_bps,
                1 => config.tier1_bonus_bps,
                _ => 0,
            };
        }
    }
    0
}

fn symbol_from_parts(env: &Env, prefix: &Symbol, suffix: &Symbol) -> Symbol {
    let combined = Symbol::new(env, "combined");
    Symbol::new(env, &format!("{}{}", prefix.to_str().to_string(), suffix.to_str().to_string()))
}

fn to_str_val(addr: &Address) -> &str {
    let _ = addr;
    "addr"
}

fn env_no_op<'a>() -> Env {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn create_test_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env
    }

    #[test]
    fn test_initialize() {
        let env = create_test_env();
        let contract_id = env.register(ReferralContract, ());
        let admin = Address::generate(&env);
        let result = ReferralContract::initialize(env.clone(), admin.clone());
        assert!(result.is_ok());
        let config = ReferralContract::get_config(env.clone());
        assert_eq!(config.admin, admin);
        assert_eq!(config.reward_bps, DEFAULT_REWARD_BPS);
    }

    #[test]
    fn test_register_code() {
        let env = create_test_env();
        let contract_id = env.register(ReferralContract, ());
        let admin = Address::generate(&env);
        ReferralContract::initialize(env.clone(), admin).unwrap();
        let user = Address::generate(&env);
        let code = Symbol::new(&env, "USER123");
        let result = ReferralContract::register_code(env.clone(), user.clone(), code.clone());
        assert!(result.is_ok());
        let affiliate = ReferralContract::get_affiliate_code(env.clone(), user.clone()).unwrap();
        assert_eq!(affiliate.owner, user);
        assert_eq!(affiliate.code, code);
        assert!(affiliate.is_active);
    }

    #[test]
    fn test_self_referral_fails() {
        let env = create_test_env();
        let contract_id = env.register(ReferralContract, ());
        let admin = Address::generate(&env);
        ReferralContract::initialize(env.clone(), admin).unwrap();
        let user = Address::generate(&env);
        let code = Symbol::new(&env, "USER123");
        ReferralContract::register_code(env.clone(), user.clone(), code.clone()).unwrap();
        let result = ReferralContract::refer(env.clone(), user.clone(), code.clone());
        assert_eq!(result, Err(ReferralError::SelfReferral));
    }

    #[test]
    fn test_refer_and_record_fee() {
        let env = create_test_env();
        let contract_id = env.register(ReferralContract, ());
        let admin = Address::generate(&env);
        ReferralContract::initialize(env.clone(), admin).unwrap();
        let referrer = Address::generate(&env);
        let referee = Address::generate(&env);
        let code = Symbol::new(&env, "REF456");
        ReferralContract::register_code(env.clone(), referrer.clone(), code.clone()).unwrap();
        ReferralContract::refer(env.clone(), referee.clone(), code.clone()).unwrap();
        let fee_amount = 10_000_000;
        let (reward, discount) = ReferralContract::record_fee(env.clone(), referee.clone(), fee_amount).unwrap();
        assert!(reward > 0);
        assert!(discount > 0);
        let record = ReferralContract::get_referral_record(env.clone(), referee.clone()).unwrap();
        assert_eq!(record.referrer, referrer);
        assert_eq!(record.referee, referee);
        assert_eq!(record.total_fees_generated, fee_amount);
    }

    #[test]
    fn test_invalid_code() {
        let env = create_test_env();
        let contract_id = env.register(ReferralContract, ());
        let admin = Address::generate(&env);
        ReferralContract::initialize(env.clone(), admin).unwrap();
        let referee = Address::generate(&env);
        let code = Symbol::new(&env, "INVALID");
        let result = ReferralContract::refer(env.clone(), referee.clone(), code.clone());
        assert_eq!(result, Err(ReferralError::InvalidCode));
    }

    #[test]
    fn test_double_register_fails() {
        let env = create_test_env();
        let contract_id = env.register(ReferralContract, ());
        let admin = Address::generate(&env);
        ReferralContract::initialize(env.clone(), admin).unwrap();
        let referrer = Address::generate(&env);
        let referee = Address::generate(&env);
        let code = Symbol::new(&env, "DOUBLE");
        ReferralContract::register_code(env.clone(), referrer.clone(), code.clone()).unwrap();
        ReferralContract::refer(env.clone(), referee.clone(), code.clone()).unwrap();
        let result = ReferralContract::refer(env.clone(), referee.clone(), code.clone());
        assert_eq!(result, Err(ReferralError::AlreadyRegistered));
    }

    #[test]
    fn test_global_metrics() {
        let env = create_test_env();
        let contract_id = env.register(ReferralContract, ());
        let admin = Address::generate(&env);
        ReferralContract::initialize(env.clone(), admin).unwrap();
        let referrer = Address::generate(&env);
        let referee = Address::generate(&env);
        let code = Symbol::new(&env, "METRICS");
        ReferralContract::register_code(env.clone(), referrer.clone(), code.clone()).unwrap();
        ReferralContract::refer(env.clone(), referee.clone(), code.clone()).unwrap();
        ReferralContract::record_fee(env.clone(), referee.clone(), 5_000_000).unwrap();
        let metrics = ReferralContract::get_global_metrics(env.clone());
        assert_eq!(metrics.total_codes_created, 1);
        assert_eq!(metrics.total_referrals, 1);
        assert!(metrics.total_rewards_distributed > 0);
        assert_eq!(metrics.total_fees_generated, 5_000_000);
    }
}
