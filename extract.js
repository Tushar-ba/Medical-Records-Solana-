const { Keypair } = require('@solana/web3.js');
const bs58 = require('bs58');
const fs = require('fs');

// The key array you provided
const keyArray = [218,50,191,34,207,156,140,43,112,205,135,154,203,36,214,120,197,194,82,198,33,209,199,76,21,13,253,254,163,7,16,9,108,136,111,129,207,168,42,96,40,53,164,42,246,245,69,63,102,239,64,184,79,44,157,247,188,223,231,149,53,33,166,146];

// Convert the array to a Uint8Array
const privateKeyBuffer = new Uint8Array(keyArray);

// Try to get the base58 encoding function in different ways
const encode = bs58.encode || (bs58.default && bs58.default.encode);

if (!encode) {
  console.error('Could not find bs58 encode function. bs58 structure:', Object.keys(bs58));
  process.exit(1);
}

try {
  // Create a Keypair from the private key
  const keypair = Keypair.fromSecretKey(privateKeyBuffer);

  // Get the public key
  const publicKey = keypair.publicKey;

  // Encode private key to base58
  const privateKeyBase58 = encode(privateKeyBuffer);

  console.log('Private Key (base58):', privateKeyBase58);
  console.log('Public Key (base58):', publicKey.toBase58());

  // Save to a JSON file
  fs.writeFileSync('keypair.json', JSON.stringify({
    privateKey: privateKeyBase58,
    publicKey: publicKey.toBase58()
  }, null, 2));
} catch (error) {
  console.error('Error processing keypair:', error);
}