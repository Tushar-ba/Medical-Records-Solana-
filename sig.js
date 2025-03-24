const { Transaction, Keypair } = require('@solana/web3.js');

// The serialized transaction from the backend
const serializedTransaction = "AwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMAAgX63Stn+bZobz4zouWt72niNze/8B4zZD6JPmK9yRC1RyM6ZLZMobb2UNOxXnYW/eUhqh7u3ry56D8FmCFG+Tc7ns20BgCXWmjV2Z+zIJUHGYHspsLQJwrHOxt8r4ibVNIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM0WyppuSC1ujFwkY+AU8mhHWQG/M8/zVaB3RYDldom2hLnh1/UGH+okyvsj5GeinW3VaHaYCWig3FkktzHIbkcBBAQAAQIDKHstQ1kBAgMEqZoI95qPGT3dPuIERuAzKo1kRmevupPGmw40Aatpr1s=";

// Your private key as a byte array (e.g., from solana-keygen)
const privateKeyBytes = Uint8Array.from([136,87,238,120,158,176,198,253,22,69,120,173,78,54,41,198,32,246,56,157,165,115,168,235,89,159,83,221,128,226,248,102,250,221,43,103,249,182,104,111,62,51,162,229,173,239,105,226,55,55,191,240,30,51,100,62,137,62,98,189,201,16,181,71]);

// Convert the private key to a Keypair
let keypair;
try {
    keypair = Keypair.fromSecretKey(privateKeyBytes);
    console.log('Public Key:', keypair.publicKey.toBase58());
} catch (error) {
    console.error('Error creating Keypair from private key:', error);
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

// Sign the transaction with the private key
console.log('Signing transaction...');
try {
    transaction.sign(keypair);
    console.log('Transaction signed successfully');
} catch (error) {
    console.error('Error signing transaction:', error);
    process.exit(1);
}

// Serialize the signed transaction to base64
const signedSerializedTransaction = Buffer.from(transaction.serialize()).toString('base64');
console.log('Signed Serialized Transaction (Base64):', signedSerializedTransaction);