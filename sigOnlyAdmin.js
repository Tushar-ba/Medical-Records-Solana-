const { Connection, Keypair, Transaction } = require('@solana/web3.js');

// The serialized transaction from the backend (update this with the new output after fixing the backend)
const serializedTransaction = "AZzbPVUK7qaGoIpb51RhhFf32vAREidTqQS7D7Iky/hD969MpcolhI58+PCu3aeQNSTRhJxVx+yFQj27cFC2/QsBAAIF+t0rZ/m2aG8+M6Llre9p4jc3v/AeM2Q+iT5ivckQtUcsCTTW8bNksHMlhA47DqLSytEcBCZeF9ZB9aS1j4wxN2qmvGv8EwEqvp+BhjMh7Fa6ZkGDa415WUBDkriJcRrxAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAD/cr/PQyHvw/b//iHuvvZ2pLZEjGepqjtM3Dg2d5w9RG2mrULOv3WQ8idag5jmi9C/BYy1LohexDcYfs6xk3cUAQQEAAIBAyh57nosbIeMSun1oUGuT49GDvL+saq2A+4gQv2Pqrckd9fbE6JaVgHU";

// User keypair (GkHELS6i7BZefYckfmWdxPH6id2rywimHxG1Vf8XBUMq)
const privateKeyBytes = Uint8Array.from([
    136, 87, 238, 120, 158, 176, 198, 253, 22, 69, 120, 173, 78, 54, 41, 198,
    32, 246, 56, 157, 165, 115, 168, 235, 89, 159, 83, 221, 128, 226, 248, 102,
    250, 221, 43, 103, 249, 182, 104, 111, 62, 51, 162, 229, 173, 239, 105, 226,
    55, 55, 191, 240, 30, 51, 100, 62, 137, 62, 98, 189, 201, 16, 181, 71
]);

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