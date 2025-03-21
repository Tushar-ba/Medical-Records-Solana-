use anchor_lang::prelude::*;

declare_id!("Bgsncv4N8H6oWjYHa9KaxCKaWcwKxCMa9FHDHsGkAUzW");

#[program]
pub mod medical_record_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Initializing");
    
        let admin_account = &mut ctx.accounts.admin_account;
        if admin_account.authority == Pubkey::default() {
            // Only set authority if it hasn't been initialized
            admin_account.authority = ctx.accounts.authority.key();
        } else if admin_account.authority != ctx.accounts.authority.key() {
            return Err(ErrorCode::Unauthorized.into());
        }
        Ok(())
    }

    pub fn create_patient(
        ctx: Context<CreatePatient>, 
        name: String, 
        blood_type: String, 
        previous_report: String, 
        ph_no: u64, 
        file: String
    ) -> Result<()> {
        // Check if the caller is the authorized admin
        if ctx.accounts.admin_account.authority != ctx.accounts.authority.key() {
            return Err(ErrorCode::Unauthorized.into());
        }

        // Get the patient account's key first
        let patient_pubkey = ctx.accounts.patient.key();
        
        // Then update fields
        let patient = &mut ctx.accounts.patient;
        patient.patient_address = patient_pubkey;
        patient.is_initialized = true;
        patient.name = name;
        patient.blood_type = blood_type;
        patient.previous_report = previous_report;
        patient.ph_no = ph_no;
        patient.file = file;
        
        Ok(())
    }

    pub fn update_patient(
        ctx: Context<UpdatePatient>, 
        name: String, 
        blood_type: String, 
        previous_report: String, 
        ph_no: u64, 
        file: String
    ) -> Result<()> {
        // Check if the caller is the authorized admin
        if ctx.accounts.admin_account.authority != ctx.accounts.authority.key() {
            return Err(ErrorCode::Unauthorized.into());
        }

        let patient = &mut ctx.accounts.patient;
        patient.name = name;
        patient.blood_type = blood_type;
        patient.previous_report = previous_report;
        patient.ph_no = ph_no;
        patient.file = file;
        
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
    
    /// CHECK: This is used as a seed for the patient PDA and does not require additional validation
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
    
    /// CHECK: This is used as a seed for the patient PDA and does not require additional validation
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

#[account]
pub struct Admin {
    pub authority: Pubkey,
}

#[account]
#[derive(InitSpace)]
pub struct Patient {
    pub patient_address: Pubkey,
    pub is_initialized: bool,
    #[max_len(50)]
    pub name: String,
    #[max_len(50)]
    pub blood_type: String,
    #[max_len(300)]
    pub previous_report: String,
    pub ph_no: u64,
    #[max_len(50)]
    pub file: String, 
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Patient record already exists")]
    PatientAlreadyExists,
    #[msg("Patient record does not exist")]
    PatientDoesNotExist,
}