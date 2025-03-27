const { Keypair } = require('@solana/web3.js');
const bs58 = require('bs58');
const fs = require('fs');

// The key array you provided
const keyArray = [82, 28, 221, 12, 243, 121, 128, 239, 117, 11, 62, 191, 5, 76, 17, 47, 243, 244, 75, 102, 96, 0, 124, 231, 148, 176, 190, 82, 57, 180, 200, 108, 233, 245, 161, 65, 174, 79, 143, 70, 14, 242, 254, 177, 170, 182, 3, 238, 32, 66, 253, 143, 170, 183, 36, 119, 215, 219, 19, 162, 90, 86, 1, 212];

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