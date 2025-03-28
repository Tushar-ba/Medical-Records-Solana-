const { Connection, Keypair, Transaction } = require('@solana/web3.js');

// The serialized transaction from the backend (update this with the new output after fixing the backend)
const serializedTransaction = "AQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABAAQG6fWhQa5Pj0YO8v6xqrYD7iBC/Y+qtyR319sTolpWAdQBfp1TZdLWrJ8MUtn9h6W5bJzhw4WRwCBCYSiqtMyGJQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAACi7hG6qnaKpO+hhAfht+8bO5av9I8EEFfb465GAqheRqprxr/BMBKr6fgYYzIexWumZBg2uNeVlAQ5K4iXEa8f9yv89DIe/D9v/+Ie6+9naktkSMZ6mqO0zcODZ3nD1EgDtTSzKDMY+dx0fdNxkRkCPO4ug9PWbcZqprUGLfkMEBBQUBAwAEAu0BsFXSnLNKPMvhAAAASm94eEQ4UTFvNGFvOXJlbHRMNzhPWDBBd2VsWmNZNCtFQWNIN2N5NWRwbUNkTzEwb2V4OFNVZTU1cHBwcmI5aTcyakRlelNQcUordmZGdmwwRTlDa2xSd1FSdUtMN3dibjNYMTZCU05CWE5nSWZkbUhZZUNZVk1vdXpMZUxGVWJ2amN5NktSTGt6R3BnanlQeTNNbnJFaDBvR0hWdkk4L3ZIdENsbzdJYUVrUXVQL1RVL2x4a05XN0M5Z2NMdFlnd09vUlZJTjA4Q2F2bS9nPXxmT0ZkWDkrbk5vUkNUMVFY";

// User keypair (GkHELS6i7BZefYckfmWdxPH6id2rywimHxG1Vf8XBUMq)
const privateKeyBytes = Uint8Array.from([82, 28, 221, 12, 243, 121, 128, 239, 117, 11, 62, 191, 5, 76, 17, 47, 243, 244, 75, 102, 96, 0, 124, 231, 148, 176, 190, 82, 57, 180, 200, 108, 233, 245, 161, 65, 174, 79, 143, 70, 14, 242, 254, 177, 170, 182, 3, 238, 32, 66, 253, 143, 170, 183, 36, 119, 215, 219, 19, 162, 90, 86, 1, 212]);

// Set up the connection to Solana Devnet
const connection = new Connection("https://api.devnet.solana.com", { commitment: "confirmed" });
console.log("Connected to Solana Devnet");

// Convert private key to Keypair
let keypair;
try {
    keypair = Keypair.fromSecretKey(privateKeyBytes);
    console.log('User Public Key:', keypair.publicKey.toBase58());
} catch (error) {
    console.error('Error creating Keypair from private key:', error);
    process.exit(1);
}

// Check wallet balance
async function checkBalance() {
    try {
        const balance = await connection.getBalance(keypair.publicKey);
        console.log('Wallet Balance:', balance / 1_000_000_000, 'SOL');
        if (balance < 1_000_000) {
            console.error('Insufficient funds in wallet. Please fund the wallet with at least 0.001 SOL.');
            process.exit(1);
        }
    } catch (error) {
        console.error('Error checking balance:', error);
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
    let blockhash;
    try {
        const latestBlockhash = await connection.getLatestBlockhash({ commitment: 'confirmed' });
        blockhash = latestBlockhash.blockhash;
        console.log('Updated Blockhash:', blockhash);
    } catch (error) {
        console.error('Error fetching latest blockhash:', error);
        process.exit(1);
    }

    // Reconstruct the transaction
    const newTransaction = new Transaction({
        recentBlockhash: blockhash,
        feePayer: keypair.publicKey,
    });
    newTransaction.add(...transaction.instructions);

    // Inspect required signers
    console.log('Required Signers (before signing):');
    const message = newTransaction.compileMessage();
    const signers = message.accountKeys
        .filter((_, i) => message.isAccountSigner(i))
        .map(key => key.toBase58());
    console.log(signers);

    // Sign the transaction
    console.log('Signing transaction...');
    try {
        newTransaction.sign(keypair);
        console.log('Transaction signed successfully');
    } catch (error) {
        console.error('Error signing transaction:', error);
        process.exit(1);
    }

    // Log signatures
    console.log('Required Signers (after signing):');
    newTransaction.signatures.forEach((sig, i) => {
        const pubKey = sig.publicKey.toBase58();
        const isSigned = sig.signature !== null;
        console.log(`  ${i}: ${pubKey} - ${isSigned ? 'Signed' : 'Not Signed'}`);
    });

    // Serialize the signed transaction
    let signedSerializedTransaction;
    try {
        signedSerializedTransaction = Buffer.from(newTransaction.serialize()).toString('base64');
        console.log('Signed Serialized Transaction (Base64):', signedSerializedTransaction);
    } catch (error) {
        console.error('Error serializing signed transaction:', error);
        process.exit(1);
    }

    return signedSerializedTransaction;
}

// Main function
async function main() {
    await checkBalance();
    const signedTx = await signTransaction();
    console.log('Signed Transaction (Base64):', signedTx);
}

// Run the script
main()
    .then(() => console.log('Script completed successfully.'))
    .catch(error => {
        console.error('Error in main execution:', error);
        process.exit(1);
    });