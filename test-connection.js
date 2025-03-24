// test-connection.js
const { Connection } = require('@solana/web3.js');

console.log('Setting up connection with Solana Web3.js 2.0.0');
const connection = new Connection("https://api.devnet.solana.com", "confirmed");
console.log("Connected to Solana Devnet");

connection.getSlot().then(slot => {
  console.log('Current slot:', slot);
}).catch(err => {
  console.error('Error fetching slot:', err);
});