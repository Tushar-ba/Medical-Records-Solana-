// getAuthorities.js
const { Connection, PublicKey } = require('@solana/web3.js');

// Connect to Solana Devnet
const connection = new Connection('https://api.devnet.solana.com', 'confirmed');
const adminPda = new PublicKey('8BKgLHWMxX53aUxcTRKRjpU4bD2fZb1VuUy8QyRKFGEL');

async function getAuthorityList() {
    console.log('Fetching admin account data...');
    const accountInfo = await connection.getAccountInfo(adminPda);
    if (!accountInfo) {
        console.log('Admin account not found');
        return;
    }

    const data = accountInfo.data;
    let offset = 0;

    // Discriminator (8 bytes) - Anchor prepends this to account data
    const discriminator = data.slice(offset, offset + 8);
    offset += 8;
    console.log('Discriminator:', Array.from(discriminator));

    // Authority (32 bytes)
    const authority = new PublicKey(data.slice(offset, offset + 32)).toBase58();
    offset += 32;
    console.log('Authority:', authority);

    // Read authorities (4 bytes length + N * 32 bytes)
    const readLen = data.readUInt32LE(offset);
    offset += 4;
    const readAuthorities = [];
    for (let i = 0; i < readLen; i++) {
        readAuthorities.push(new PublicKey(data.slice(offset, offset + 32)).toBase58());
        offset += 32;
    }
    console.log('Read Authorities:', readAuthorities);

    // Write authorities (4 bytes length + M * 32 bytes)
    const writeLen = data.readUInt32LE(offset);
    offset += 4;
    const writeAuthorities = [];
    for (let i = 0; i < writeLen; i++) {
        writeAuthorities.push(new PublicKey(data.slice(offset, offset + 32)).toBase58());
        offset += 32;
    }
    console.log('Write Authorities:', writeAuthorities);

    console.log('Total bytes parsed:', offset);
    console.log('Account data size:', data.length);
}

getAuthorityList().catch(err => {
    console.error('Error:', err);
});