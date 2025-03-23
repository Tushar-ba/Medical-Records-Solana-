const { Connection, Transaction, Keypair } = require('@solana/web3.js');
const bs58 = require('bs58');
const fs = require('fs');

// Load the admin keypair from a file (e.g., admin-keypair.json)
const adminKeypairBytes = JSON.parse(fs.readFileSync('path/to/admin-keypair.json', 'utf8'));
const adminKeypair = Keypair.fromSecretKey(new Uint8Array(adminKeypairBytes));

// The serialized transaction from the backend
const serializedTransaction = "base64EncodedTransactionHere"; // Replace with the actual serialized transaction

// Decode the serialized transaction
const transactionBytes = Buffer.from(serializedTransaction, 'base64');
const transaction = Transaction.from(transactionBytes);

// Sign the transaction
transaction.sign(adminKeypair);

// Serialize the signed transaction
const signedSerializedTransaction = Buffer.from(transaction.serialize()).toString('base64');

console.log("Signed serialized transaction:", signedSerializedTransaction);