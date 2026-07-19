from pathlib import Path

ROOT = Path("perax-contracts/programs/perax-core/src")


def find_struct(text: str, name: str) -> tuple[int, int]:
    pos = text.index(f"pub struct {name}")
    start = text.rfind("#[derive(Accounts)]", 0, pos)
    end = text.index("\n}", pos) + 2
    return start, end


contexts_path = ROOT / "settlement_v2.rs"
contexts = contexts_path.read_text()
start, end = find_struct(contexts, "PlanSettlementV2")
replacement = '''#[derive(Accounts)]
#[instruction(params: PlanSettlementParams)]
pub struct PlanSettlementV2<'info> {
    #[account(seeds = [b"perax-state"], bump = state.bump)]
    pub state: Box<Account<'info, PeraxState>>,
    #[account(seeds = [b"settlement-policy", state.key().as_ref()], bump = settlement_policy.bump)]
    pub settlement_policy: Box<Account<'info, SettlementPolicy>>,
    #[account(seeds = [b"product-settlement", params.product_id.as_ref()], bump = product_policy.bump)]
    pub product_policy: Box<Account<'info, ProductSettlementPolicy>>,
    #[account(seeds = [b"apc-config", state.key().as_ref()], bump = apc_config.bump)]
    pub apc_config: Box<Account<'info, ApcConfig>>,
    #[account(seeds = [b"apc-state", apc_config.key().as_ref()], bump = apc_state.bump)]
    pub apc_state: Box<Account<'info, ApcState>>,
    #[account(seeds = [b"apc-observation", params.observation_id.as_ref()], bump = observation.bump)]
    pub observation: Box<Account<'info, ApcObservation>>,
    #[account(
        init,
        payer = initiator,
        space = 8 + SettlementRecord::INIT_SPACE,
        seeds = [b"settlement", params.settlement_id.as_ref()],
        bump
    )]
    pub settlement_record: Box<Account<'info, SettlementRecord>>,
    #[account(
        init,
        payer = initiator,
        space = 8 + SettlementCustody::INIT_SPACE,
        seeds = [b"settlement-custody", params.settlement_id.as_ref()],
        bump
    )]
    pub settlement_custody: Box<Account<'info, SettlementCustody>>,
    #[account(mut)]
    pub settlement_pex_vault: Box<Account<'info, TokenAccount>>,
    pub pex_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub initiator: Signer<'info>,
    pub system_program: Program<'info, System>,
}'''
contexts_path.write_text(contexts[:start] + replacement + contexts[end:])

handler_path = ROOT / "instructions/settlement_v2.rs"
handler = handler_path.read_text()
header = "pub fn plan_settlement(ctx: Context<PlanSettlementV2>, params: PlanSettlementParams) -> Result<()> {\n"
insert = '''    let record_key = ctx.accounts.settlement_record.key();
    let (authority_key, authority_bump) = Pubkey::find_program_address(
        &[b"settlement-custody-authority", record_key.as_ref()],
        ctx.program_id,
    );
    require!(
        ctx.accounts.settlement_pex_vault.owner == authority_key
            && ctx.accounts.settlement_pex_vault.mint == ctx.accounts.pex_mint.key(),
        SettlementError::InvalidSettlementDestination
    );
'''
handler = handler.replace("    let record_key = ctx.accounts.settlement_record.key();\n", "", 1)
handler = handler.replace("    let authority_key = ctx.accounts.settlement_authority.key();\n", "", 1)
if insert.strip() not in handler:
    handler = handler.replace(header, header + insert, 1)
handler = handler.replace(
    "    custody.authority_bump = ctx.bumps.settlement_authority;",
    "    custody.authority_bump = authority_bump;",
)
handler_path.write_text(handler)

client_path = Path("perax-market-engine/src/anchor-client.ts")
client = client_path.read_text()
client = client.replace("        settlementAuthority: addresses.settlementAuthority,\n", "", 1)
old = '''        settlementPexVault: addresses.settlementPexVault,
        pexMint: this.pexMint,
        initiator: this.config.provider.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      }),
    );'''
new = '''        settlementPexVault: addresses.settlementPexVault,
        pexMint: this.pexMint,
        initiator: this.config.provider.publicKey,
        systemProgram: web3.SystemProgram.programId,
      }).preInstructions([
        createAssociatedTokenAccountIdempotentInstruction(
          this.config.provider.publicKey,
          addresses.settlementPexVault,
          addresses.settlementAuthority,
          this.pexMint,
          TOKEN_PROGRAM_ID,
          ASSOCIATED_TOKEN_PROGRAM_ID,
        ),
      ]),
    );'''
if old not in client and new not in client:
    raise RuntimeError("Settlement planner client block was not found")
client_path.write_text(client.replace(old, new, 1))

idl_guard_path = Path("perax-contracts/scripts/validate-settlement-idl.js")
idl_guard = idl_guard_path.read_text()
idl_guard = idl_guard.replace(
    '  "settlementAuthority",\n  "settlementPexVault",',
    '  "settlementPexVault",',
    1,
)
idl_guard_path.write_text(idl_guard)

source_guard_path = Path("perax-contracts/scripts/validate-settlement-source.js")
source_guard = source_guard_path.read_text()
old_guard = '''assertContains(
  contexts,
  'seeds = [b"settlement-custody-authority", settlement_record.key().as_ref()]',
  "settlement_v2.rs",
);'''
new_guard = '''assertContains(
  handlers,
  'b"settlement-custody-authority"',
  "settlement authority derivation",
);'''
if old_guard in source_guard:
    source_guard = source_guard.replace(old_guard, new_guard, 1)
source_guard = source_guard.replace(
    'settlement_pex_vault.owner == ctx.accounts.settlement_authority.key()',
    'settlement_pex_vault.owner == authority_key',
)
source_guard_path.write_text(source_guard)
