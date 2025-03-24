const anchor = require('@coral-xyz/anchor');
const { Connection, Keypair, PublicKey, clusterApiUrl } = require('@solana/web3.js');
const fs = require('fs');
const path = require('path');
const os = require('os');

async function main() {
  try {
    // Properly expand the home directory tilde
    const homedir = os.homedir();
    const keypairPath = path.join(homedir, '.config/solana/id.json');
    
    console.log(`Looking for keypair at: ${keypairPath}`);
    const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf8'));
    const wallet = Keypair.fromSecretKey(new Uint8Array(keypairData));
    
    // Set up the connection to devnet
    const connection = new Connection(clusterApiUrl('devnet'), 'confirmed');
    
    console.log(`Using wallet address: ${wallet.publicKey.toString()}`);
    
    // Create the provider
    const provider = new anchor.AnchorProvider(
      connection, 
      new anchor.Wallet(wallet),
      { commitment: 'confirmed' }
    );
    
    // Set the provider globally
    anchor.setProvider(provider);
    
    // Load the workspace (this uses Anchor.toml config)
    const program = anchor.workspace.medicalRecordSolana;
    
    // Find the admin account PDA
    const [adminPDA] = PublicKey.findProgramAddressSync(
      [Buffer.from(anchor.utils.bytes.utf8.encode('admin'))],
      program.programId
    );
    
    console.log(`Admin PDA: ${adminPDA.toString()}`);
    console.log(`Program ID: ${program.programId.toString()}`);
    
    // Initialize the program
    console.log('Initializing the Medical Record contract...');
    
    const tx = await program.methods
      .initialize()
      .accounts({
        authority: wallet.publicKey,
        adminAccount: adminPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([wallet])
      .rpc();
    
    console.log(`Transaction signature: ${tx}`);
    console.log('Program initialized successfully');
    
    // Optionally, fetch the admin account to verify it was created
    const adminAccount = await program.account.admin.fetch(adminPDA);
    console.log('Admin Account:', {
      authority: adminAccount.authority.toString(),
      readAuthorities: adminAccount.readAuthorities.map(auth => auth.toString()),
      writeAuthorities: adminAccount.writeAuthorities.map(auth => auth.toString())
    });
    
  } catch (error) {
    console.error('Error executing the transaction:', error);
  }
}

// Run the main function
main().then(
  () => process.exit(0),
  (error) => {
    console.error(error);
    process.exit(1);
  }
);