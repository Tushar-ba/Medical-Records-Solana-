const { PublicKey } = require('@solana/web3.js');
const programId = new PublicKey('JCASyF1M3MVV6ADByM1F6jEFWZGg29YLCiCjjJtBBcEP');
const [adminPda] = PublicKey.findProgramAddressSync(
  [Buffer.from('admin')],
  programId
);
console.log('Admin PDA:', adminPda.toBase58());