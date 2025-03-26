const { Connection, Keypair, Transaction } = require('@solana/web3.js');

// The serialized transaction from the backend
const serializedTransaction = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAQG+t0rZ/m2aG8+M6Llre9p4jc3v/AeM2Q+iT5ivckQtUfGMdxhfsieIe4kke3u/QGXgiiBQBnkC+W115sJKlhKEQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAaqa8a/wTASq+n4GGMyHsVrpmQYNrjXlZQEOSuIlxGvGBVb2oZTpWaxStUe+f2ZGz+XRyIjGUqeklrIPl+NC1Ov9yv89DIe/D9v/+Ie6+9naktkSMZ6mqO0zcODZ3nD1EMf8VpB4+54ps+8P4pZ/heLTDPHPmunWTWOI7Mxj5a3EBBQUBBAADAvkBsFXSnLNKPMvtAAAAcGRPRllmbXpVajdZNGdWVEpLUkNnaXVHV1pBZ2NHckxkNXl4RThkTzE4Z0tLK1lnTkg0QVlHZWdCS28ybXFhcEVPWGdCTXZ0SEtTV3VMTFlwYWtSYm9LU285ZGdPWGlnVDhRcHJYMFEvdUlpYm1zYUg0V2Z5V3Fsc3N4S0JlWnNFWUxucmc2QjR2VFJWSGdQK0puT3ZPZVlRQkZaYW5Ta0J1YXoxRDRkYU9nOUxpT3Yrb05uQjVMdGpTOEJPYk9zRnFNNHNZdlAxTldWRjEzcDdtbUplOWZyR1MwPXxORWVFZ2FoZ2J1VmhRWm9j";

// Your private key as a byte array (also the admin keypair)
const privateKeyBytes = Uint8Array.from([
    136, 87, 238, 120, 158, 176, 198, 253, 22, 69, 120, 173, 78, 54, 41, 198,
    32, 246, 56, 157, 165, 115, 168, 235, 89, 159, 83, 221, 128, 226, 248, 102,
    250, 221, 43, 103, 249, 182, 104, 111, 62, 51, 162, 229, 173, 239, 105, 226,
    55, 55, 191, 240, 30, 51, 100, 62, 137, 62, 98, 189, 201, 16, 181, 71
]);

// Set up the connection to Solana Devnet
const connection = new Connection("https://api.devnet.solana.com", { commitment: "confirmed" });
console.log("Connected to Solana Devnet");

// Convert your private key to a Keypair
let keypair;
try {
    keypair = Keypair.fromSecretKey(privateKeyBytes);
    console.log('Your Public Key (User and Admin):', keypair.publicKey.toBase58());
} catch (error) {
    console.error('Error creating Keypair from private key:', error);
    process.exit(1);
}

// Check wallet balance
async function checkBalance() {
    const balance = await connection.getBalance(keypair.publicKey);
    console.log('Wallet Balance:', balance / 1_000_000_000, 'SOL');
    if (balance < 1_000_000) { // 0.001 SOL minimum for fees
        console.error('Insufficient funds in wallet. Please fund the wallet with at least 0.001 SOL.');
        process.exit(1);
    }
}

// Deserialize and sign the transaction
async function signTransaction() {
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

    // Fetch a fresh blockhash
    console.log('Fetching latest blockhash...');
    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash({ commitment: 'confirmed' });
    transaction.recentBlockhash = blockhash;
    console.log('Updated Blockhash:', blockhash);

    // Sign the transaction
    console.log('Signing transaction...');
    try {
        transaction.sign(keypair); // Sign with the keypair (user and admin)
        console.log('Transaction signed successfully');
    } catch (error) {
        console.error('Error signing transaction:', error);
        process.exit(1);
    }

    // Log signatures
    console.log('Required Signers (after signing):');
    transaction.signatures.forEach((sig, i) => {
        const pubKey = sig.publicKey.toBase58();
        const isSigned = sig.signature !== null;
        console.log(`  ${i}: ${pubKey} - ${isSigned ? 'Signed' : 'Not Signed'}`);
    });

    // Serialize the signed transaction
    let signedSerializedTransaction;
    try {
        signedSerializedTransaction = Buffer.from(transaction.serialize()).toString('base64');
        console.log('Signed Serialized Transaction (Base64):');
        console.log(signedSerializedTransaction);
    } catch (error) {
        console.error('Error serializing signed transaction:', error);
        process.exit(1);
    }

    return signedSerializedTransaction;
}

// Main function to execute the steps
async function main() {
    await checkBalance();
    const signedTx = await signTransaction();
    return { signedTx };
}

// Run the script
main()
    .then(result => {
        console.log('Signed transaction ready to submit:', result.signedTx);
        console.log('Send this to http://127.0.0.1:8080/api/transactions/submit in the following JSON format:');
        console.log(JSON.stringify({ serialized_transaction: result.signedTx }, null, 2));
    })
    .catch(error => {
        console.error('Error in main execution:', error);
        process.exit(1);
    });