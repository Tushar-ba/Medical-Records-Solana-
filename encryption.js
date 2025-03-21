const crypto = require('crypto');

// Generate a secure 256-bit (32-byte) encryption key
const ENCRYPTION_KEY = crypto.randomBytes(32);
// Generate a 16-byte initialization vector
const IV = crypto.randomBytes(16);

/**
 * Encrypts data using AES-256-CBC
 * @param {string} data - The data to encrypt
 * @returns {string} - Base64 encoded encrypted string
 */
function encrypt(data) {
    try {
        // Ensure data is a string
        const text = typeof data === 'string' ? data : JSON.stringify(data);
        
        const cipher = crypto.createCipheriv('aes-256-cbc', ENCRYPTION_KEY, IV);
        let encrypted = cipher.update(text, 'utf8', 'base64');
        encrypted += cipher.final('base64');
        return encrypted;
    } catch (error) {
        console.error('Encryption error:', error);
        throw new Error('Failed to encrypt data');
    }
}

/**
 * Decrypts data using AES-256-CBC
 * @param {string} encrypted - Base64 encoded encrypted string
 * @returns {string} - Decrypted string
 */
function decrypt(encrypted) {
    try {
        const decipher = crypto.createDecipheriv('aes-256-cbc', ENCRYPTION_KEY, IV);
        let decrypted = decipher.update(encrypted, 'base64', 'utf8');
        decrypted += decipher.final('utf8');
        return decrypted;
    } catch (error) {
        console.error('Decryption error:', error);
        throw new Error('Failed to decrypt data');
    }
}

/**
 * Helper function to encrypt patient data object
 * @param {Object} patientData - Patient data object to encrypt
 * @returns {string} - Encrypted string
 */
function encryptPatientData(patientData) {
    return encrypt(JSON.stringify(patientData));
}

/**
 * Helper function to decrypt and parse patient data
 * @param {string} encryptedData - Encrypted patient data
 * @returns {Object} - Decrypted patient data object
 */
function decryptPatientData(encryptedData) {
    const decrypted = decrypt(encryptedData);
    return JSON.parse(decrypted);
}

// Example usage
function demo() {
    // Sample patient data
    const patientData = {
        name: "John Doe",
        bloodType: "O+",
        previousReport: "Healthy",
        phNo: "1234567890",
        file: "report.pdf"
    };

    try {
        // Encrypt the data
        const encrypted = encryptPatientData(patientData);
        console.log('Encrypted data:', encrypted);

        // Decrypt the data
        const decrypted = decryptPatientData(encrypted);
        console.log('Decrypted data:', decrypted);

        // Verify
        console.log('Original matches decrypted:', 
            JSON.stringify(patientData) === JSON.stringify(decrypted));
    } catch (error) {
        console.error('Demo error:', error);
    }
}

// Export functions for use in other modules
module.exports = {
    encrypt,
    decrypt,
    encryptPatientData,
    decryptPatientData,
    ENCRYPTION_KEY,
    IV
};

// Run demo if script is run directly
if (require.main === module) {
    demo();
}