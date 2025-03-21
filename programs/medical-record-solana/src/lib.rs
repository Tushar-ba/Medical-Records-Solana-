use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;

declare_id!("Bgsncv4N8H6oWjYHa9KaxCKaWcwKxCMa9FHDHsGkAUzW");

#[program]
pub mod medical_record_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let admin_account = &mut ctx.accounts.admin_account;
        if admin_account.authority == Pubkey::default() {
            admin_account.authority = ctx.accounts.authority.key();
            admin_account.read_authorities = vec![ctx.accounts.authority.key()];
            admin_account.write_authorities = vec![ctx.accounts.authority.key()];
        } else if admin_account.authority != ctx.accounts.authority.key() {
            return Err(ErrorCode::Unauthorized.into());
        }
        Ok(())
    }

    pub fn add_read_authority(ctx: Context<AddAuthority>, new_authority: Pubkey) -> Result<()> {
        if ctx.accounts.admin_account.authority != ctx.accounts.authority.key() {
            return Err(ErrorCode::Unauthorized.into());
        }
        let admin = &mut ctx.accounts.admin_account;
        if !admin.read_authorities.contains(&new_authority) {
            admin.read_authorities.push(new_authority);
            ctx.accounts.history.add_entry(
                ctx.accounts.authority.key(),
                new_authority,
                true,
                true,
                Clock::get()?.unix_timestamp,
            )?;
        }
        Ok(())
    }

    pub fn add_write_authority(ctx: Context<AddAuthority>, new_authority: Pubkey) -> Result<()> {
        if ctx.accounts.admin_account.authority != ctx.accounts.authority.key() {
            return Err(ErrorCode::Unauthorized.into());
        }
        let admin = &mut ctx.accounts.admin_account;
        if !admin.write_authorities.contains(&new_authority) {
            admin.write_authorities.push(new_authority);
            ctx.accounts.history.add_entry(
                ctx.accounts.authority.key(),
                new_authority,
                true,
                false,
                Clock::get()?.unix_timestamp,
            )?;
        }
        Ok(())
    }

    pub fn remove_read_authority(ctx: Context<RemoveAuthority>, authority_to_remove: Pubkey) -> Result<()> {
        if ctx.accounts.admin_account.authority != ctx.accounts.authority.key() {
            return Err(ErrorCode::Unauthorized.into());
        }
        let admin = &mut ctx.accounts.admin_account;
        if let Some(idx) = admin.read_authorities.iter().position(|&x| x == authority_to_remove) {
            admin.read_authorities.swap_remove(idx);
            ctx.accounts.history.add_entry(
                ctx.accounts.authority.key(),
                authority_to_remove,
                false,
                true,
                Clock::get()?.unix_timestamp,
            )?;
        }
        Ok(())
    }

    pub fn remove_write_authority(ctx: Context<RemoveAuthority>, authority_to_remove: Pubkey) -> Result<()> {
        if ctx.accounts.admin_account.authority != ctx.accounts.authority.key() {
            return Err(ErrorCode::Unauthorized.into());
        }
        let admin = &mut ctx.accounts.admin_account;
        if let Some(idx) = admin.write_authorities.iter().position(|&x| x == authority_to_remove) {
            admin.write_authorities.swap_remove(idx);
            ctx.accounts.history.add_entry(
                ctx.accounts.authority.key(),
                authority_to_remove,
                false,
                false,
                Clock::get()?.unix_timestamp,
            )?;
        }
        Ok(())
    }

    pub fn create_patient(ctx: Context<CreatePatient>, encrypted_data: String) -> Result<()> {
        if !ctx.accounts.admin_account.write_authorities.contains(&ctx.accounts.authority.key()) {
            return Err(ErrorCode::Unauthorized.into());
        }
        let patient = &mut ctx.accounts.patient;
        patient.patient_address = patient.key();
        patient.is_initialized = true;
        patient.encrypted_data = encrypted_data.clone();
        patient.data_hash = hash(encrypted_data.as_bytes()).to_bytes();
        Ok(())
    }

    pub fn update_patient(ctx: Context<UpdatePatient>, encrypted_data: String) -> Result<()> {
        if !ctx.accounts.admin_account.write_authorities.contains(&ctx.accounts.authority.key()) {
            return Err(ErrorCode::Unauthorized.into());
        }
        let patient = &mut ctx.accounts.patient;
        patient.encrypted_data = encrypted_data.clone();
        patient.data_hash = hash(encrypted_data.as_bytes()).to_bytes();
        Ok(())
    }

    pub fn get_patient(ctx: Context<GetPatient>) -> Result<()> {
        if !ctx.accounts.admin_account.read_authorities.contains(&ctx.accounts.authority.key()) {
            return Err(ErrorCode::Unauthorized.into());
        }
        let patient = &ctx.accounts.patient;
        let computed_hash = hash(patient.encrypted_data.as_bytes()).to_bytes();
        if computed_hash != patient.data_hash {
            return Err(ErrorCode::DataIntegrityFailed.into());
        }
        msg!("Encrypted patient data: {}", patient.encrypted_data);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + Admin::INIT_SPACE,
        seeds = [b"admin"],
        bump
    )]
    pub admin_account: Account<'info, Admin>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct AddAuthority<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"admin"],
        bump,
        constraint = admin_account.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub admin_account: Account<'info, Admin>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + AuthorityHistory::INIT_SPACE,
        seeds = [b"history", authority.key().as_ref()],
        bump
    )]
    pub history: Account<'info, AuthorityHistory>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveAuthority<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"admin"],
        bump,
        constraint = admin_account.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub admin_account: Account<'info, Admin>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + AuthorityHistory::INIT_SPACE,
        seeds = [b"history", authority.key().as_ref()],
        bump
    )]
    pub history: Account<'info, AuthorityHistory>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatePatient<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Patient::INIT_SPACE,
        seeds = [b"patient", admin_account.authority.as_ref(), patient_seed.key().as_ref()],
        bump
    )]
    pub patient: Account<'info, Patient>,
    /// CHECK: Used as a seed for the patient PDA
    pub patient_seed: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(seeds = [b"admin"], bump)]
    pub admin_account: Account<'info, Admin>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePatient<'info> {
    #[account(
        mut,
        seeds = [b"patient", admin_account.authority.as_ref(), patient_seed.key().as_ref()],
        bump,
        constraint = patient.is_initialized @ ErrorCode::PatientDoesNotExist
    )]
    pub patient: Account<'info, Patient>,
    /// CHECK: Used as a seed for the patient PDA
    pub patient_seed: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(seeds = [b"admin"], bump)]
    pub admin_account: Account<'info, Admin>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetPatient<'info> {
    #[account(
        seeds = [b"patient", admin_account.authority.as_ref(), patient_seed.key().as_ref()],
        bump,
        constraint = patient.is_initialized @ ErrorCode::PatientDoesNotExist
    )]
    pub patient: Account<'info, Patient>,
    /// CHECK: Used as a seed for the patient PDA
    pub patient_seed: AccountInfo<'info>,
    pub authority: Signer<'info>,
    #[account(seeds = [b"admin"], bump)]
    pub admin_account: Account<'info, Admin>,
}

#[account]
#[derive(InitSpace)]
pub struct Admin {
    pub authority: Pubkey,
    #[max_len(50)]
    pub read_authorities: Vec<Pubkey>,
    #[max_len(50)]
    pub write_authorities: Vec<Pubkey>,
}

#[account]
#[derive(InitSpace)]
pub struct Patient {
    pub patient_address: Pubkey,
    pub is_initialized: bool,
    #[max_len(500)]
    pub encrypted_data: String,
    pub data_hash: [u8; 32],
}

#[account]
#[derive(InitSpace)]
pub struct AuthorityHistory {
    #[max_len(100)]
    pub entries: Vec<HistoryEntry>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct HistoryEntry {
    pub admin: Pubkey,
    pub authority: Pubkey,
    pub added: bool,
    pub is_read: bool,
    pub timestamp: i64,
}

impl AuthorityHistory {
    fn add_entry(&mut self, admin: Pubkey, authority: Pubkey, added: bool, is_read: bool, timestamp: i64) -> Result<()> {
        self.entries.push(HistoryEntry {
            admin,
            authority,
            added,
            is_read,
            timestamp,
        });
        Ok(())
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Patient record already exists")]
    PatientAlreadyExists,
    #[msg("Patient record does not exist")]
    PatientDoesNotExist,
    #[msg("Data integrity check failed")]
    DataIntegrityFailed,
}