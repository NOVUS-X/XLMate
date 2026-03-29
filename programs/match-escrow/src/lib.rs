use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

/// 24 hours expressed in seconds
const TWENTY_FOUR_HOURS: i64 = 24 * 60 * 60;

#[program]
pub mod match_escrow {
    use super::*;

    /// Player deposits lamports into the escrow for a given match.
    /// Records the on-chain (Ledger) timestamp at deposit time.
    pub fn deposit(ctx: Context<Deposit>, match_id: u64, amount: u64) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        let clock = Clock::get()?;

        escrow.match_id = match_id;
        escrow.player = ctx.accounts.player.key();
        escrow.amount = amount;
        escrow.deposit_time = clock.unix_timestamp;
        escrow.bump = ctx.bumps.escrow;

        // Transfer lamports from player to escrow PDA
        let ix = anchor_lang::solana_program::system_instruction::transfer(
            &ctx.accounts.player.key(),
            &ctx.accounts.escrow.key(),
            amount,
        );
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.player.to_account_info(),
                ctx.accounts.escrow.to_account_info(),
            ],
        )?;

        emit!(DepositEvent {
            match_id,
            player: ctx.accounts.player.key(),
            amount,
            deposit_time: clock.unix_timestamp,
        });

        Ok(())
    }

    /// If the match has not been resolved by the backend within 24 hours,
    /// the player can reclaim their deposit.
    pub fn claim_timeout_refund(ctx: Context<ClaimTimeoutRefund>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;
        let clock = Clock::get()?;

        let elapsed = clock
            .unix_timestamp
            .checked_sub(escrow.deposit_time)
            .ok_or(EscrowError::ArithmeticOverflow)?;

        require!(elapsed >= TWENTY_FOUR_HOURS, EscrowError::TimeoutNotReached);

        let refund_amount = escrow.amount;
        let player = ctx.accounts.player.key();

        // Drain lamports from escrow PDA back to player
        **ctx
            .accounts
            .escrow
            .to_account_info()
            .try_borrow_mut_lamports()? -= refund_amount;
        **ctx
            .accounts
            .player
            .to_account_info()
            .try_borrow_mut_lamports()? += refund_amount;

        emit!(RefundEvent {
            match_id: escrow.match_id,
            player,
            amount: refund_amount,
            refund_time: clock.unix_timestamp,
        });

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(match_id: u64)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        init,
        payer = player,
        space = EscrowAccount::LEN,
        seeds = [b"escrow", player.key().as_ref(), &match_id.to_le_bytes()],
        bump
    )]
    pub escrow: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimTimeoutRefund<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    #[account(
        mut,
        close = player,
        seeds = [b"escrow", player.key().as_ref(), &escrow.match_id.to_le_bytes()],
        bump = escrow.bump,
        has_one = player @ EscrowError::UnauthorizedPlayer,
    )]
    pub escrow: Account<'info, EscrowAccount>,

    pub system_program: Program<'info, System>,
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

#[account]
pub struct EscrowAccount {
    /// Unique identifier for the match
    pub match_id: u64,
    /// The player who made the deposit
    pub player: Pubkey,
    /// Lamports held in escrow
    pub amount: u64,
    /// Unix timestamp recorded at deposit (from Clock sysvar)
    pub deposit_time: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl EscrowAccount {
    // discriminator(8) + match_id(8) + player(32) + amount(8) + deposit_time(8) + bump(1)
    pub const LEN: usize = 8 + 8 + 32 + 8 + 8 + 1;
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[event]
pub struct DepositEvent {
    pub match_id: u64,
    pub player: Pubkey,
    pub amount: u64,
    pub deposit_time: i64,
}

#[event]
pub struct RefundEvent {
    pub match_id: u64,
    pub player: Pubkey,
    pub amount: u64,
    pub refund_time: i64,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[error_code]
pub enum EscrowError {
    #[msg("24-hour timeout has not been reached yet")]
    TimeoutNotReached,
    #[msg("Only the depositing player can claim a refund")]
    UnauthorizedPlayer,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
}
