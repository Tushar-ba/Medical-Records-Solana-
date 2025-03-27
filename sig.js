const { Connection, Keypair, Transaction, sendAndConfirmRawTransaction } = require('@solana/web3.js');

// The serialized transaction from the backend
const serializedTransaction = "Ad5f+lfXce2+w7T9AfDuJGpfH1OseQwaevUoiMXOssijgZpOXFiiWFJsWdWHPBOAMfYMPGzJzS4QBcQXoIf+kgUBAAIF+t0rZ/m2aG8+M6Llre9p4jc3v/AeM2Q+iT5ivckQtUcsCTTW8bNksHMlhA47DqLSytEcBCZeF9ZB9aS1j4wxN2qmvGv8EwEqvp+BhjMh7Fa6ZkGDa415WUBDkriJcRrxAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD/cr/PQyHvw/b//iHuvvZ2pLZEjGepqjtM3Dg2d5w9RPc96d9E0bzBHjH4P6+ReNmsBBl7hbY1KhiWvaa7PlqEAQQEAAIBAyh57nosbIeMSun1oUGuT49GDvL+saq2A+4gQv2Pqrckd9fbE6JaVgHU";

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
    if (balance < 1_000_000_000) {
        console.error('Insufficient funds in wallet. Please fund the wallet with at least 1 SOL.');
        process.exit(1);
    }
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
console.log('Required Signers (before signing):');
transaction.signatures.forEach((sig, i) => {
    const pubKey = sig.publicKey.toBase58();
    const isSigned = sig.signature !== null;
    console.log(`  ${i}: ${pubKey} - ${isSigned ? 'Signed' : 'Not Signed'}`);
});

// Fetch a fresh blockhash
async function updateBlockhash() {
    console.log('Fetching latest blockhash...');
    const { blockhash } = await connection.getLatestBlockhash({ commitment: 'confirmed' });
    transaction.recentBlockhash = blockhash;
    console.log('Updated Blockhash:', blockhash);
}

// Sign the transaction with the keypair (user and admin are the same)
async function signTransaction() {
    await updateBlockhash();

    console.log('Signing transaction...');
    try {
        // Clear existing signatures for the keypair
        transaction.signatures = transaction.signatures.map(sigPair => {
            if (sigPair.publicKey.equals(keypair.publicKey)) {
                return { publicKey: sigPair.publicKey, signature: null };
            }
            return sigPair;
        });

        // Sign with the keypair
        transaction.sign(keypair);
        console.log('Transaction signed successfully by user and admin');
    } catch (error) {
        console.error('Error signing transaction:', error);
        process.exit(1);
    }

    // Log transaction details after signing
    console.log('Required Signers (after signing):');
    transaction.signatures.forEach((sig, i) => {
        const pubKey = sig.publicKey.toBase58();
        const isSigned = sig.signature !== null;
        console.log(`  ${i}: ${pubKey} - ${isSigned ? 'Signed' : 'Not Signed'}`);
    });
}

// Serialize the signed transaction to base64
async function serializeTransaction() {
    let signedSerializedTransaction;
    try {
        signedSerializedTransaction = Buffer.from(transaction.serialize()).toString('base64');
        console.log('Signed Serialized Transaction (Base64):', signedSerializedTransaction);
    } catch (error) {
        console.error('Error serializing signed transaction:', error);
        process.exit(1);
    }
    return signedSerializedTransaction;
}

// Submit the transaction to Devnet
async function submitTransaction() {
    console.log('Submitting transaction to Devnet...');
    try {
        const serializedTx = transaction.serialize();
        const signature = await sendAndConfirmRawTransaction(connection, serializedTx, {
            skipPreflight: false,
            commitment: 'confirmed',
            maxRetries: 5,
        });
        console.log('Transaction Signature:', signature);
        console.log('Transaction confirmed successfully');
        console.log('Verify the transaction on Solana Explorer:');
        console.log(`https://explorer.solana.com/tx/${signature}?cluster=devnet`);
        return signature;
    } catch (error) {
        console.error('Error submitting transaction:', error);
        if (error.logs) {
            console.error('Transaction Logs:', error.logs);
        }
        throw error;
    }
}

// Main function to execute the steps
async function main() {
    await checkBalance();
    await signTransaction();
    const signedTx = await serializeTransaction();
    const signature = await submitTransaction();
    return { signedTx, signature };
}

// Run the script
main().then(result => {
    console.log('Signed transaction:', result.signedTx);
    console.log('Transaction signature:', result.signature);
}).catch(error => {
    console.error('Error in main execution:', error);
    process.exit(1);
});