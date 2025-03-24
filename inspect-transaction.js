const { Transaction, Keypair } = require('@solana/web3.js');

// The serialized transaction from the backend
const serializedTransaction = "AwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMAAgX63Stn+bZobz4zouWt72niNze/8B4zZD6JPmK9yRC1RyM6ZLZMobb2UNOxXnYW/eUhqh7u3ry56D8FmCFG+Tc7ns20BgCXWmjV2Z+zIJUHGYHspsLQJwrHOxt8r4ibVNIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAM0WyppuSC1ujFwkY+AU8mhHWQG/M8/zVaB3RYDldom2hLnh1/UGH+okyvsj5GeinW3VaHaYCWig3FkktzHIbkcBBAQAAQIDKHstQ1kBAgMEqZoI95qPGT3dPuIERuAzKo1kRmevupPGmw40Aatpr1s=";

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

// Log transaction details
console.log('Fee Payer:', transaction.feePayer.toBase58());
console.log('Required Signers:', transaction.signatures.map(sig => sig.publicKey.toBase58()));
console.log('Instructions:');
transaction.instructions.forEach((ix, i) => {
    console.log(`  Instruction ${i}:`);
    console.log(`    Program ID: ${ix.programId.toBase58()}`);
    console.log('    Accounts:');
    ix.keys.forEach((key, j) => {
        console.log(`      Account ${j}:`);
        console.log(`        Public Key: ${key.pubkey.toBase58()}`);
        console.log(`        Is Signer: ${key.isSigner}`);
        console.log(`        Is Writable: ${key.isWritable}`);
    });
    console.log(`    Data: ${Buffer.from(ix.data).toString('hex')}`);
});