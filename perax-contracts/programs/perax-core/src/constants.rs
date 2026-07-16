pub const PEX_DECIMALS: u64 = 1_000_000;
pub const PEX_MINT_DECIMALS: u8 = 6;
pub const PEX_TOTAL_SUPPLY: u64 = 1_000_000_000 * PEX_DECIMALS;
pub const PEX_LAUNCH_PRICE_SCALED: u64 = 1_200;
pub const GROWTH_PRICE_MULTIPLIER: u64 = 3;
pub const MIN_GROWTH_TWAP_MINUTES: u64 = 60;
pub const INITIAL_LIQUIDITY_USD: u64 = 4_560;
pub const MIN_GROWTH_LIQUIDITY_USD: u64 = INITIAL_LIQUIDITY_USD * 3;
pub const MIN_NET_BUY_VOLUME_BPS: u16 = 5_000;
pub const DAILY_RELEASE_CAP: u64 = 10_000_000 * PEX_DECIMALS;
pub const MONTHLY_RELEASE_CAP: u64 = 150_000_000 * PEX_DECIMALS;
pub const RELEASE_COOLDOWN_SECONDS: i64 = 86_400;
pub const EMERGENCY_DOWNSIDE_TRIGGER_BPS: u16 = 3_000;
pub const EMERGENCY_LIQUIDITY_DRAIN_TRIGGER_BPS: u16 = 6_000;
pub const EMERGENCY_HOURLY_RESERVE_RELEASE_BPS: u16 = 50;

pub const MIN_BURN_RATE_BPS: u16 = 200;
pub const DEFAULT_BURN_RATE_BPS: u16 = 1_000;
pub const MAX_BURN_RATE_BPS: u16 = 3_000;
pub const EARLY_DAILY_BURN_CAP_BPS: u16 = 500;
pub const CONSERVATION_SUPPLY_THRESHOLD_BPS: u16 = 8_500;
pub const CONSERVATION_BURN_RATE_BPS: u16 = 50;
pub const CONSERVATION_DAILY_BURN_CAP_BPS: u16 = 50;

pub const ALLOCATION_LIQUIDITY_POOL: [u8; 32] = padded_allocation_id(b"liquidity_pool");
pub const ALLOCATION_COMMUNITY_REWARDS: [u8; 32] =
    padded_allocation_id(b"community_utility_rewards");
pub const ALLOCATION_TREASURY: [u8; 32] = padded_allocation_id(b"treasury");
pub const ALLOCATION_ECOSYSTEM_MARKETING: [u8; 32] = padded_allocation_id(b"ecosystem_marketing");
pub const ALLOCATION_TRADING_OPERATIONS: [u8; 32] =
    padded_allocation_id(b"trading_company_operations");
pub const ALLOCATION_DEVELOPMENT_TEAM: [u8; 32] = padded_allocation_id(b"development_team");
pub const ALLOCATION_FOUNDER: [u8; 32] = padded_allocation_id(b"founder");
pub const ALLOCATION_FUTURE_TEAM_INCENTIVES: [u8; 32] =
    padded_allocation_id(b"future_team_incentives");
pub const ALLOCATION_TEAM_EMERGENCY_RESERVE: [u8; 32] =
    padded_allocation_id(b"team_emergency_reserve");
pub const ALLOCATION_PRIVATE_STRATEGIC: [u8; 32] =
    padded_allocation_id(b"private_strategic_investors");
pub const ALLOCATION_ADVISOR_1: [u8; 32] = padded_allocation_id(b"advisor_wallet_1");
pub const ALLOCATION_ADVISOR_2: [u8; 32] = padded_allocation_id(b"advisor_wallet_2");
pub const ALLOCATION_ADVISOR_3: [u8; 32] = padded_allocation_id(b"advisor_wallet_3");

pub const fn padded_allocation_id(label: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    let mut index = 0;
    while index < label.len() && index < 32 {
        output[index] = label[index];
        index += 1;
    }
    output
}
