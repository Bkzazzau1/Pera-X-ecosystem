use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::AccountMeta;

pub const METEORA_SWAP_EXACT_OUT2_DISCRIMINATOR: [u8; 8] =
    [43, 215, 247, 132, 137, 60, 243, 81];
pub const METEORA_SWAP_EXACT_OUT2_FIXED_ACCOUNT_COUNT: usize = 16;
pub const METEORA_SWAP_EXACT_OUT2_MIN_ACCOUNT_COUNT: usize = 17;
pub const METEORA_SWAP_EXACT_OUT2_DATA_LENGTH_WITHOUT_HOOKS: usize = 28;
pub const MEMO_PROGRAM_ID: Pubkey =
    anchor_lang::solana_program::pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MarketAccountView {
    pub key: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<&AccountInfo<'_>> for MarketAccountView {
    fn from(account: &AccountInfo<'_>) -> Self {
        Self {
            key: *account.key,
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ExactOutMarketValidation {
    pub market_program: Pubkey,
    pub approved_pool: Pubkey,
    pub quote_source: Pubkey,
    pub pex_destination: Pubkey,
    pub authority: Pubkey,
    pub quote_mint: Pubkey,
    pub pex_mint: Pubkey,
    pub token_program: Pubkey,
    pub maximum_quote_amount: u64,
    pub exact_pex_out: u64,
    /// True when `authority` is a PDA promoted to signer by invoke_signed.
    pub authority_is_pda: bool,
}

pub(crate) fn validated_exact_out_market_metas(
    accounts: &[AccountInfo<'_>],
    data: &[u8],
    expected: ExactOutMarketValidation,
) -> Option<Vec<AccountMeta>> {
    let views: Vec<MarketAccountView> = accounts.iter().map(MarketAccountView::from).collect();
    validate_exact_out_market_instruction(&views, data, expected)?;

    Some(
        accounts
            .iter()
            .enumerate()
            .map(|(index, account)| {
                let is_signer = index == 10;
                if account.is_writable {
                    AccountMeta::new(*account.key, is_signer)
                } else {
                    AccountMeta::new_readonly(*account.key, is_signer)
                }
            })
            .collect(),
    )
}

pub(crate) fn validate_exact_out_market_instruction(
    accounts: &[MarketAccountView],
    data: &[u8],
    expected: ExactOutMarketValidation,
) -> Option<()> {
    validate_exact_out_data(
        data,
        expected.maximum_quote_amount,
        expected.exact_pex_out,
    )?;
    if accounts.len() < METEORA_SWAP_EXACT_OUT2_MIN_ACCOUNT_COUNT || accounts.len() > 64 {
        return None;
    }
    if expected.market_program == Pubkey::default()
        || expected.approved_pool == Pubkey::default()
        || expected.quote_source == Pubkey::default()
        || expected.pex_destination == Pubkey::default()
        || expected.authority == Pubkey::default()
        || expected.quote_mint == Pubkey::default()
        || expected.pex_mint == Pubkey::default()
        || expected.quote_mint == expected.pex_mint
        || expected.quote_source == expected.pex_destination
    {
        return None;
    }

    let event_authority = Pubkey::find_program_address(
        &[b"__event_authority"],
        &expected.market_program,
    )
    .0;

    // Fixed Meteora swap_exact_out2 account order.
    require_account(accounts, 0, expected.approved_pool, false, true)?;

    // Optional bin-array bitmap extension. Anchor encodes None as the program ID.
    let bitmap = accounts.get(1)?;
    if bitmap.key == expected.market_program {
        if bitmap.is_signer || bitmap.is_writable {
            return None;
        }
    } else if bitmap.is_signer || !bitmap.is_writable {
        return None;
    }

    require_flags(accounts, 2, false, true)?; // reserve X
    require_flags(accounts, 3, false, true)?; // reserve Y
    require_account(accounts, 4, expected.quote_source, false, true)?;
    require_account(accounts, 5, expected.pex_destination, false, true)?;

    let token_x_mint = accounts.get(6)?;
    let token_y_mint = accounts.get(7)?;
    if token_x_mint.is_signer
        || token_x_mint.is_writable
        || token_y_mint.is_signer
        || token_y_mint.is_writable
        || !((token_x_mint.key == expected.quote_mint
            && token_y_mint.key == expected.pex_mint)
            || (token_x_mint.key == expected.pex_mint
                && token_y_mint.key == expected.quote_mint))
    {
        return None;
    }

    require_flags(accounts, 8, false, true)?; // Meteora oracle

    // Host fees are deliberately forbidden. None is encoded as the program ID.
    require_account(accounts, 9, expected.market_program, false, false)?;

    let authority = accounts.get(10)?;
    if authority.key != expected.authority || authority.is_writable {
        return None;
    }
    if !expected.authority_is_pda && !authority.is_signer {
        return None;
    }

    // PEX and USDC are currently classic SPL Token mints. Transfer-hook slices
    // are forbidden by the exact 28-byte instruction-data validation below.
    require_account(accounts, 11, expected.token_program, false, false)?;
    require_account(accounts, 12, expected.token_program, false, false)?;
    require_account(accounts, 13, MEMO_PROGRAM_ID, false, false)?;
    require_account(accounts, 14, event_authority, false, false)?;
    require_account(accounts, 15, expected.market_program, false, false)?;

    for (index, account) in accounts.iter().enumerate() {
        if index != 10 && account.is_signer {
            return None;
        }
    }
    for account in accounts.iter().skip(METEORA_SWAP_EXACT_OUT2_FIXED_ACCOUNT_COUNT) {
        if account.is_signer || !account.is_writable || account.key == Pubkey::default() {
            return None;
        }
    }

    Some(())
}

pub(crate) fn validate_exact_out_data(
    data: &[u8],
    maximum_quote_amount: u64,
    exact_pex_out: u64,
) -> Option<()> {
    if maximum_quote_amount == 0
        || exact_pex_out == 0
        || data.len() != METEORA_SWAP_EXACT_OUT2_DATA_LENGTH_WITHOUT_HOOKS
        || data.get(..8)? != METEORA_SWAP_EXACT_OUT2_DISCRIMINATOR
    {
        return None;
    }

    let encoded_maximum = u64::from_le_bytes(data.get(8..16)?.try_into().ok()?);
    let encoded_output = u64::from_le_bytes(data.get(16..24)?.try_into().ok()?);
    let transfer_hook_slice_count = u32::from_le_bytes(data.get(24..28)?.try_into().ok()?);
    if encoded_maximum != maximum_quote_amount
        || encoded_output != exact_pex_out
        || transfer_hook_slice_count != 0
    {
        return None;
    }
    Some(())
}

fn require_account(
    accounts: &[MarketAccountView],
    index: usize,
    key: Pubkey,
    signer: bool,
    writable: bool,
) -> Option<()> {
    let account = accounts.get(index)?;
    if account.key != key || account.is_signer != signer || account.is_writable != writable {
        return None;
    }
    Some(())
}

fn require_flags(
    accounts: &[MarketAccountView],
    index: usize,
    signer: bool,
    writable: bool,
) -> Option<()> {
    let account = accounts.get(index)?;
    if account.key == Pubkey::default()
        || account.is_signer != signer
        || account.is_writable != writable
    {
        return None;
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(seed: u8) -> Pubkey {
        Pubkey::new_from_array([seed; 32])
    }

    fn instruction_data(maximum: u64, output: u64) -> Vec<u8> {
        let mut data = METEORA_SWAP_EXACT_OUT2_DISCRIMINATOR.to_vec();
        data.extend_from_slice(&maximum.to_le_bytes());
        data.extend_from_slice(&output.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data
    }

    fn fixture(authority_is_pda: bool) -> (Vec<MarketAccountView>, ExactOutMarketValidation) {
        let market_program = key(1);
        let pool = key(2);
        let quote_source = key(3);
        let pex_destination = key(4);
        let authority = key(5);
        let quote_mint = key(6);
        let pex_mint = key(7);
        let token_program = key(8);
        let event_authority =
            Pubkey::find_program_address(&[b"__event_authority"], &market_program).0;
        let accounts = vec![
            MarketAccountView { key: pool, is_signer: false, is_writable: true },
            MarketAccountView { key: market_program, is_signer: false, is_writable: false },
            MarketAccountView { key: key(9), is_signer: false, is_writable: true },
            MarketAccountView { key: key(10), is_signer: false, is_writable: true },
            MarketAccountView { key: quote_source, is_signer: false, is_writable: true },
            MarketAccountView { key: pex_destination, is_signer: false, is_writable: true },
            MarketAccountView { key: quote_mint, is_signer: false, is_writable: false },
            MarketAccountView { key: pex_mint, is_signer: false, is_writable: false },
            MarketAccountView { key: key(11), is_signer: false, is_writable: true },
            MarketAccountView { key: market_program, is_signer: false, is_writable: false },
            MarketAccountView { key: authority, is_signer: !authority_is_pda, is_writable: false },
            MarketAccountView { key: token_program, is_signer: false, is_writable: false },
            MarketAccountView { key: token_program, is_signer: false, is_writable: false },
            MarketAccountView { key: MEMO_PROGRAM_ID, is_signer: false, is_writable: false },
            MarketAccountView { key: event_authority, is_signer: false, is_writable: false },
            MarketAccountView { key: market_program, is_signer: false, is_writable: false },
            MarketAccountView { key: key(12), is_signer: false, is_writable: true },
        ];
        let expected = ExactOutMarketValidation {
            market_program,
            approved_pool: pool,
            quote_source,
            pex_destination,
            authority,
            quote_mint,
            pex_mint,
            token_program,
            maximum_quote_amount: 500,
            exact_pex_out: 1_000,
            authority_is_pda,
        };
        (accounts, expected)
    }

    #[test]
    fn accepts_exact_out_instruction_with_ordered_accounts() {
        let (accounts, expected) = fixture(false);
        assert!(validate_exact_out_market_instruction(
            &accounts,
            &instruction_data(500, 1_000),
            expected,
        )
        .is_some());
    }

    #[test]
    fn accepts_pda_authority_for_invoke_signed() {
        let (accounts, expected) = fixture(true);
        assert!(validate_exact_out_market_instruction(
            &accounts,
            &instruction_data(500, 1_000),
            expected,
        )
        .is_some());
    }

    #[test]
    fn rejects_wrong_discriminator_amount_or_transfer_hooks() {
        let (accounts, expected) = fixture(false);
        let mut wrong_discriminator = instruction_data(500, 1_000);
        wrong_discriminator[0] ^= 1;
        assert!(validate_exact_out_market_instruction(
            &accounts,
            &wrong_discriminator,
            expected,
        )
        .is_none());
        assert!(validate_exact_out_market_instruction(
            &accounts,
            &instruction_data(501, 1_000),
            expected,
        )
        .is_none());
        let mut hooks = instruction_data(500, 1_000);
        hooks[24] = 1;
        assert!(validate_exact_out_market_instruction(&accounts, &hooks, expected).is_none());
    }

    #[test]
    fn rejects_host_fee_wrong_destination_and_extra_signer() {
        let (mut accounts, expected) = fixture(false);
        accounts[9].key = key(44);
        assert!(validate_exact_out_market_instruction(
            &accounts,
            &instruction_data(500, 1_000),
            expected,
        )
        .is_none());

        let (mut accounts, expected) = fixture(false);
        accounts[5].key = key(45);
        assert!(validate_exact_out_market_instruction(
            &accounts,
            &instruction_data(500, 1_000),
            expected,
        )
        .is_none());

        let (mut accounts, expected) = fixture(false);
        accounts[16].is_signer = true;
        assert!(validate_exact_out_market_instruction(
            &accounts,
            &instruction_data(500, 1_000),
            expected,
        )
        .is_none());
    }
}
