use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;

declare_id!("Bgsncv4N8H6oWjYHa9KaxCKaWcwKxCMa9FHDHsGkAUzW");

#[program]
pub mod medical_record_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Initializing");
        let admin_account = &mut ctx.accounts.admin_account;
        if admin_account.authority == Pubkey::default() {
            admin_account.authority = ctx.accounts.authority.key();
        } else if admin_account.authority != ctx.accounts.authority.key() {
            return Err(ErrorCode::Unauthorized.into());
        }
        Ok(())
    }

    pub fn create_patient(
        ctx: Context<CreatePatient>,
        encrypted_data: String,
    ) -> Result<()> {
        if ctx.accounts.admin_account.authority != ctx.accounts.authority.key() {
            return Err(ErrorCode::Unauthorized.into());
        }

        let patient = &mut ctx.accounts.patient;
        patient.patient_address = patient.key();
        patient.is_initialized = true;
        patient.encrypted_data = encrypted_data.clone();
        patient.data_hash = hash(encrypted_data.as_bytes()).to_bytes();
        Ok(())
    }

    pub fn update_patient(
        ctx: Context<UpdatePatient>,
        encrypted_data: String,
    ) -> Result<()> {
        if ctx.accounts.admin_account.authority != ctx.accounts.authority.key() {
            return Err(ErrorCode::Unauthorized.into());
        }

        let patient = &mut ctx.accounts.patient;
        patient.encrypted_data = encrypted_data.clone();
        patient.data_hash = hash(encrypted_data.as_bytes()).to_bytes();
        Ok(())
    }

    pub fn get_patient(ctx: Context<GetPatient>) -> Result<()> {
        if ctx.accounts.admin_account.authority != ctx.accounts.authority.key() {
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
        space = 8 + 32,
        seeds = [b"admin"],
        bump
    )]
    pub admin_account: Account<'info, Admin>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatePatient<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Patient::INIT_SPACE,
        seeds = [b"patient", authority.key().as_ref(), patient_seed.key().as_ref()],
        bump
    )]
    pub patient: Account<'info, Patient>,
    /// CHECK: Used as a seed for the patient PDA
    pub patient_seed: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"admin"],
        bump,
        constraint = admin_account.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub admin_account: Account<'info, Admin>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdatePatient<'info> {
    #[account(
        mut,
        seeds = [b"patient", authority.key().as_ref(), patient_seed.key().as_ref()],
        bump,
        constraint = patient.is_initialized @ ErrorCode::PatientDoesNotExist
    )]
    pub patient: Account<'info, Patient>,
    /// CHECK: Used as a seed for the patient PDA
    pub patient_seed: AccountInfo<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"admin"],
        bump,
        constraint = admin_account.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub admin_account: Account<'info, Admin>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct GetPatient<'info> {
    #[account(
        seeds = [b"patient", admin_account.authority.as_ref(), patient_seed.key().as_ref()], // Use admin's key
        bump,
        constraint = patient.is_initialized @ ErrorCode::PatientDoesNotExist
    )]
    pub patient: Account<'info, Patient>,
    /// CHECK: Used as a seed for the patient PDA
    pub patient_seed: AccountInfo<'info>,
    pub authority: Signer<'info>,
    #[account(
        seeds = [b"admin"],
        bump,
        constraint = admin_account.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub admin_account: Account<'info, Admin>,
}

#[account]
pub struct Admin {
    pub authority: Pubkey,
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