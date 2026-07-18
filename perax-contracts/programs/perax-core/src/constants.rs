pub const PEX_DECIMALS: u64 = 1_000_000;
pub const PEX_MINT_DECIMALS: u8 = 6;
pub const PEX_TOTAL_SUPPLY: u64 = 1_000_000_000 * PEX_DECIMALS;
pub const PEX_LAUNCH_PRICE_SCALED: u64 = 1_200;
pub const APC_FIRST_ACTIVATION_MULTIPLIER: u64 = 3;
pub const APC_RISK_TIER_COUNT: usize = 4;
pub const APC_THRESHOLD_COUNT: usize = 3;
pub const APC_BPS_DENOMINATOR: u128 = 10_000;
pub const APC_QUOTE_DECIMALS: u8 = 6;
pub const APC_MAX_METRIC_BPS: u32 = 10_000_000;
pub const APC_POLICY_VERSION: u16 = 1;
pub const APC_POLICY_HASH_SHA256: [u8; 32] = [
    0x17, 0xf9, 0x3b, 0xac, 0xb0, 0xcf, 0xa5, 0x34, 0x6a, 0x46, 0x62, 0x58, 0x11, 0x79, 0x08, 0x06,
    0x8f, 0x1f, 0x0c, 0xd6, 0x70, 0x54, 0xf8, 0xb6, 0x1c, 0x7d, 0x40, 0x81, 0x8d, 0xfe, 0x84, 0xbb,
];
pub const APC_PRICE_SCALE: u64 = 100_000_000;
pub const APC_FIRST_ACTIVATION_PRICE_SCALED: u64 = 3_600;
pub const APC_MINIMUM_BAND_INTERVAL_BPS: u16 = 750;
pub const APC_MAXIMUM_BAND_INTERVAL_BPS: u16 = 2_000;
pub const APC_RISK_VELOCITY_THRESHOLDS_BPS: [u32; 3] = [500, 1_500, 3_000];
pub const APC_RISK_VOLATILITY_THRESHOLDS_BPS: [u32; 3] = [400, 1_200, 2_500];
pub const APC_RISK_PRICE_IMPACT_THRESHOLDS_BPS: [u32; 3] = [250, 750, 1_500];
pub const APC_INTERVAL_BPS_BY_RISK: [u16; 4] = [2_000, 1_500, 1_000, 750];
pub const APC_RELEASE_BPS_BY_RISK: [u16; 4] = [10_000, 7_500, 5_000, 2_500];
pub const APC_CASCADE_REDUCTION_BPS: [u16; 4] = [10_000, 7_500, 5_000, 2_500];
pub const APC_MAXIMUM_OBSERVATION_AGE_SECONDS: i64 = 90;
pub const APC_MAXIMUM_FUTURE_CLOCK_SKEW_SECONDS: i64 = 5;
pub const APC_MINIMUM_TWAP_MINUTES: u64 = 60;
pub const APC_MINIMUM_LIQUIDITY_USD: u64 = 27_360;
pub const APC_MINIMUM_QUOTE_LIQUIDITY_USD: u64 = 13_680;
pub const APC_MINIMUM_VOLUME_USD: u64 = 6_840;
pub const APC_MINIMUM_BUY_PRESSURE_BPS: u16 = 5_500;
pub const APC_BASE_BAND_RELEASE_CAP: u64 = 2_000_000 * PEX_DECIMALS;
pub const APC_HOURLY_RELEASE_CAP: u64 = 2_500_000 * PEX_DECIMALS;
pub const APC_PUMP_WINDOW_RELEASE_CAP: u64 = 6_000_000 * PEX_DECIMALS;
pub const APC_PUMP_WINDOW_SECONDS: i64 = 21_600;
pub const APC_MINIMUM_COUNTERWEIGHT_COVERAGE_BPS: u16 = 5_000;
pub const APC_COUNTERWEIGHT_PROCEEDS_ALLOCATION_BPS: u16 = 7_000;
pub const APC_LIQUIDITY_REINFORCEMENT_ALLOCATION_BPS: u16 = 2_000;
pub const APC_BURN_RESERVE_ALLOCATION_BPS: u16 = 500;
pub const APC_OPERATIONS_ALLOCATION_BPS: u16 = 500;
pub const APC_DEFERRED_BURN_RESUMPTION_RATE_BPS: u16 = 1_000;
pub const APC_DEFERRED_BURN_WINDOW_CAP: u64 = 400_000 * PEX_DECIMALS;
pub const APC_DEFERRED_BURN_WINDOW_SECONDS: i64 = 3_600;
pub const APC_DEFERRED_BURN_COOLDOWN_SECONDS: i64 = 900;
pub const APC_USDC_BASE_UNITS: u64 = 1_000_000;
pub const APC_RECOVERY_TOTAL_SPENDING_CAP: u64 = 3_000 * APC_USDC_BASE_UNITS;
pub const APC_MAXIMUM_RECOVERY_PURCHASE_BPS: u16 = 1_500;
pub const APC_MINIMUM_COUNTERWEIGHT_RESERVE_BPS: u16 = 3_000;
pub const APC_RECOVERY_WINDOW_CAP: u64 = 500 * APC_USDC_BASE_UNITS;
pub const APC_RECOVERY_WINDOW_SECONDS: i64 = 21_600;
pub const APC_RECOVERY_COOLDOWN_SECONDS: i64 = 1_800;
pub const APC_RECOVERY_SUPPORT_DRAWDOWN_BPS: [u16; 4] = [1_000, 2_500, 5_000, 7_500];
pub const APC_RECOVERY_PURCHASE_BPS_BY_SUPPORT: [u16; 4] = [500, 750, 1_000, 1_500];

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
