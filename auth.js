const { Keypair, PublicKey } = require('@solana/web3.js');
const bs58 = require('bs58');

const privateKeyBytes = Uint8Array.from([
    136, 87, 238, 120, 158, 176, 198, 253, 22, 69, 120, 173, 78, 54, 41, 198,
    32, 246, 56, 157, 165, 115, 168, 235, 89, 159, 83, 221, 128, 226, 248, 102,
    250, 221, 43, 103, 249, 182, 104, 111, 62, 51, 162, 229, 173, 239, 105, 226,
    55, 55, 191, 240, 30, 51, 100, 62, 137, 62, 98, 189, 201, 16, 181, 71
]);
const keypair = Keypair.fromSecretKey(privateKeyBytes);
const timestamp = Math.floor(Date.now() / 1000);
const message = `Timestamp: ${timestamp}`;
const messageBytes = Buffer.from(message);

// Using the Ed25519 signature property of the keypair
const signature = keypair.secretKey.slice(0, 32);
const signedMessage = ed25519.sign(messageBytes, signature);

console.log('Timestamp:', timestamp);
console.log('Signature:', bs58.encode(signedMessage));
console.log('Public Key:', keypair.publicKey.toBase58());