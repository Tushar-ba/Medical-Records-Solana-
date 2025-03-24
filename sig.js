const { Transaction, Keypair } = require('@solana/web3.js');

// The serialized transaction from the backend (update with the new one after re-preparing)
const serializedTransaction = "AXMi6q0sufBSe8LirvSIPRb3HFH5OMb9H0sa3k6BzXMGaTTbA6Z3RwRw6cE6hKMCChqsmUvyGGtl85Im12VCzggBAAIF+t0rZ/m2aG8+M6Llre9p4jc3v/AeM2Q+iT5ivckQtUcsCTTW8bNksHMlhA47DqLSytEcBCZeF9ZB9aS1j4wxN2qmvGv8EwEqvp+BhjMh7Fa6ZkGDa415WUBDkriJcRrxAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD/cr/PQyHvw/b//iHuvvZ2pLZEjGepqjtM3Dg2d5w9REsFJ6Fn6Tfz8Gw8Rfiq9Z9R/GzzHEASyuVCS9Ua/mCZAQQEAAIBAyh7LUNZAQIDBKmaCPeajxk93T7iBEbgMyqNZEZnr7qTxpsONAGraa9b";

// Your private key as a byte array
const privateKeyBytes = Uint8Array.from([
    136, 87, 238, 120, 158, 176, 198, 253, 22, 69, 120, 173, 78, 54, 41, 198,
    32, 246, 56, 157, 165, 115, 168, 235, 89, 159, 83, 221, 128, 226, 248, 102,
    250, 221, 43, 103, 249, 182, 104, 111, 62, 51, 162, 229, 173, 239, 105, 226,
    55, 55, 191, 240, 30, 51, 100, 62, 137, 62, 98, 189, 201, 16, 181, 71
]);

// Convert your private key to a Keypair
let keypair;
try {
    keypair = Keypair.fromSecretKey(privateKeyBytes);
    console.log('Your Public Key:', keypair.publicKey.toBase58());
} catch (error) {
    console.error('Error creating Keypair from your private key:', error);
    process.exit(1);
}

// Deserialize the transaction
console.log('Deserializing transaction...');
let transaction;
try {
    const transactionBytes = Buffer.from(serializedTransaction, 'base64');
    transaction = Transaction.from(transactionBytes);
    console.log('Transaction deserialized successfully');
} catch (error) {
    console.error('Error deserializing transaction:', error);
    process.exit(1);
}

// Log transaction details before signing
console.log('Fee Payer:', transaction.feePayer.toBase58());
console.log('Required Signers (before signing):', transaction.signatures.map(sig => ({
    publicKey: sig.publicKey.toBase58(),
    signature: sig.signature ? 'Signed' : 'Not Signed'
})));

// Sign the transaction with the user's keypair
console.log('Signing transaction...');
try {
    transaction.sign(keypair);
    console.log('Transaction signed successfully');
} catch (error) {
    console.error('Error signing transaction:', error);
    process.exit(1);
}

// Log transaction details after signing
console.log('Required Signers (after signing):', transaction.signatures.map(sig => ({
    publicKey: sig.publicKey.toBase58(),
    signature: sig.signature ? 'Signed' : 'Not Signed'
})));

// Serialize the signed transaction to base64
let signedSerializedTransaction;
try {
    signedSerializedTransaction = Buffer.from(transaction.serialize()).toString('base64');
    console.log('Signed Serialized Transaction (Base64):', signedSerializedTransaction);
} catch (error) {
    console.error('Error serializing signed transaction:', error);
    process.exit(1);
}