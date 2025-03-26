# API Documentation

## Base URL
`http://127.0.0.1:8080/api`

## Authentication
All endpoints except `/auth` require a valid JWT token in the `Authorization` header as `Bearer <token>`. Obtain the token via the `/auth` endpoint.

---

## Endpoints

### 1. POST /auth
Authenticate a user and generate a JWT token.

- **Method**: `POST`
- **Path**: `/api/auth`
- **Description**: Verifies a user's public key and signature, returning a JWT token for authenticated requests.
- **Headers**: None
- **Request Body**:
  ```json
  {
    "public_key": "string",
    "signature": "string",
    "timestamp": integer
  }
  ```
  - `public_key`: Solana public key (base58-encoded).
  - `signature`: Signature of the message `Timestamp: <timestamp>` (base58-encoded).
  - `timestamp`: Unix timestamp in seconds.

- **Example Request**:
  ```bash
  curl -X POST http://127.0.0.1:8080/api/auth \
  -H "Content-Type: application/json" \
  -d '{"public_key": "HtGXcunbPUU54wMa9ZiXdMXvv1b5ppT7DeFLJWdtH7Lr", "signature": "2k3j4k5l6m7n8p9q0r1s2t3u4v5w6x7y8z9a0b1c2d3e4f5g6h7i8j9k0l1m2n3o4p5q6r7s8t9u0v1w2x3y4z5", "timestamp": 1698765432}'
  ```

- **Example Response** (Success - 200 OK):
  ```json
  {
    "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJIdEdYY3VuYlBVV TU0d01hOVppWGNNWHZ2MWI1cHBUN0RlRk5KV2R0SDdMciIsImV4cCI6MTY5ODc2OTAzMn0.4z5t9u0v1w2x3y4z5a0b1c2d3e4f5g6h7i8j9k0l1m2n",
    "expires_in": 3600,
    "public_key": "HtGXcunbPUU54wMa9ZiXdMXvv1b5ppT7DeFLJWdtH7Lr"
  }
  ```

- **Error Response** (401 Unauthorized):
  ```json
  "Unauthorized: Signature verification failed"
  ```

---

### 2. POST /transactions/prepare/add-read-authority
Prepare a transaction to add a read authority.

- **Method**: `POST`
- **Path**: `/api/transactions/prepare/add-read-authority`
- **Description**: Prepares a Solana transaction to add a read authority to the admin account.
- **Headers**:
  - `Authorization: Bearer <jwt_token>`
- **Request Body**:
  ```json
  {
    "new_authority": "string"
  }
  ```
  - `new_authority`: Public key of the new read authority (base58-encoded).

- **Example Request**:
  ```bash
  curl -X POST http://127.0.0.1:8080/api/transactions/prepare/add-read-authority \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{"new_authority": "GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN"}'
  ```

- **Example Response** (200 OK):
  ```json
  {
    "serialized_transaction": "base64_encoded_transaction",
    "transaction_type": "add_read_authority",
    "metadata": "{\"user_pubkey\":\"HtGXcunbPUU54wMa9ZiXdMXvv1b5ppT7DeFLJWdtH7Lr\",\"new_authority\":\"GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN\"}"
  }
  ```

- **Error Response** (401 Unauthorized):
  ```json
  "Unauthorized: Invalid token"
  ```

---

### 3. POST /transactions/prepare/remove-read-authority
Prepare a transaction to remove a read authority.

- **Method**: `POST`
- **Path**: `/api/transactions/prepare/remove-read-authority`
- **Description**: Prepares a Solana transaction to remove a read authority from the admin account.
- **Headers**:
  - `Authorization: Bearer <jwt_token>`
- **Request Body**:
  ```json
  {
    "authority_to_remove": "string"
  }
  ```
  - `authority_to_remove`: Public key of the read authority to remove (base58-encoded).

- **Example Request**:
  ```bash
  curl -X POST http://127.0.0.1:8080/api/transactions/prepare/remove-read-authority \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{"authority_to_remove": "GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN"}'
  ```

- **Example Response** (200 OK):
  ```json
  {
    "serialized_transaction": "base64_encoded_transaction",
    "transaction_type": "remove_read_authority",
    "metadata": "{\"user_pubkey\":\"HtGXcunbPUU54wMa9ZiXdMXvv1b5ppT7DeFLJWdtH7Lr\",\"authority_to_remove\":\"GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN\"}"
  }
  ```

- **Error Response** (401 Unauthorized):
  ```json
  "Unauthorized: Invalid token"
  ```

---

### 4. POST /transactions/prepare/add-write-authority
Prepare a transaction to add a write authority.

- **Method**: `POST`
- **Path**: `/api/transactions/prepare/add-write-authority`
- **Description**: Prepares a Solana transaction to add a write authority to the admin account.
- **Headers**:
  - `Authorization: Bearer <jwt_token>`
- **Request Body**:
  ```json
  {
    "new_authority": "string"
  }
  ```
  - `new_authority`: Public key of the new write authority (base58-encoded).

- **Example Request**:
  ```bash
  curl -X POST http://127.0.0.1:8080/api/transactions/prepare/add-write-authority \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{"new_authority": "GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN"}'
  ```

- **Example Response** (200 OK):
  ```json
  {
    "serialized_transaction": "base64_encoded_transaction",
    "transaction_type": "add_write_authority",
    "metadata": "{\"user_pubkey\":\"HtGXcunbPUU54wMa9ZiXdMXvv1b5ppT7DeFLJWdtH7Lr\",\"new_authority\":\"GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN\"}"
  }
  ```

- **Error Response** (401 Unauthorized):
  ```json
  "Unauthorized: Invalid token"
  ```

---

### 5. POST /transactions/prepare/remove-write-authority
Prepare a transaction to remove a write authority.

- **Method**: `POST`
- **Path**: `/api/transactions/prepare/remove-write-authority`
- **Description**: Prepares a Solana transaction to remove a write authority from the admin account.
- **Headers**:
  - `Authorization: Bearer <jwt_token>`
- **Request Body**:
  ```json
  {
    "authority_to_remove": "string"
  }
  ```
  - `authority_to_remove`: Public key of the write authority to remove (base58-encoded).

- **Example Request**:
  ```bash
  curl -X POST http://127.0.0.1:8080/api/transactions/prepare/remove-write-authority \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{"authority_to_remove": "GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN"}'
  ```

- **Example Response** (200 OK):
  ```json
  {
    "serialized_transaction": "base64_encoded_transaction",
    "transaction_type": "remove_write_authority",
    "metadata": "{\"user_pubkey\":\"HtGXcunbPUU54wMa9ZiXdMXvv1b5ppT7DeFLJWdtH7Lr\",\"authority_to_remove\":\"GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN\"}"
  }
  ```

- **Error Response** (401 Unauthorized):
  ```json
  "Unauthorized: Invalid token"
  ```

---

### 6. POST /transactions/prepare/create-patient
Prepare a transaction to create a patient record.

- **Method**: `POST`
- **Path**: `/api/transactions/prepare/create-patient`
- **Description**: Prepares a Solana transaction to create a new patient record with encrypted data.
- **Headers**:
  - `Authorization: Bearer <jwt_token>`
- **Request Body**:
  ```json
  {
    "patient_data": "string"
  }
  ```
  - `patient_data`: Plaintext patient data to be encrypted (e.g., JSON string).

- **Example Request**:
  ```bash
  curl -X POST http://127.0.0.1:8080/api/transactions/prepare/create-patient \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{"patient_data": "name: \"John Doe\", bloodType: \"O+\""}'
  ```

- **Example Response** (200 OK):
  ```json
  {
    "serialized_transaction": "base64_encoded_transaction",
    "transaction_type": "create_patient",
    "encrypted_data_with_seed": "encrypted_data_base64|nonce_base64|patient_seed_pubkey"
  }
  ```

- **Error Response** (401 Unauthorized):
  ```json
  "Unauthorized: Invalid token"
  ```

---

### 7. POST /transactions/prepare/update-patient
Prepare a transaction to update a patient record.

- **Method**: `POST`
- **Path**: `/api/transactions/prepare/update-patient`
- **Description**: Prepares a Solana transaction to update an existing patient record with new encrypted data.
- **Headers**:
  - `Authorization: Bearer <jwt_token>`
- **Request Body**:
  ```json
  {
    "patient_seed": "string",
    "patient_data": "string"
  }
  ```
  - `patient_seed`: Public key of the patient seed (base58-encoded).
  - `patient_data`: Updated plaintext patient data to be encrypted.

- **Example Request**:
  ```bash
  curl -X POST http://127.0.0.1:8080/api/transactions/prepare/update-patient \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{"patient_seed": "GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN", "patient_data": "name: \"John Doe\", bloodType: \"O+\", previousReport: \"Updated Report\""}'
  ```

- **Example Response** (200 OK):
  ```json
  {
    "serialized_transaction": "base64_encoded_transaction",
    "transaction_type": "update_patient",
    "encrypted_data_with_seed": "encrypted_data_base64|nonce_base64|GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN"
  }
  ```

- **Error Response** (401 Unauthorized):
  ```json
  "Unauthorized: Invalid token"
  ```

---

### 8. POST /transactions/submit
Submit a signed transaction to the Solana network.

- **Method**: `POST`
- **Path**: `/api/transactions/submit`
- **Description**: Submits a signed transaction to the Solana network and returns the transaction signature.
- **Headers**:
  - `Authorization: Bearer <jwt_token>`
- **Request Body**:
  ```json
  {
    "serialized_transaction": "string"
  }
  ```
  - `serialized_transaction`: Base64-encoded, signed transaction from a prepare endpoint.

- **Example Request**:
  ```bash
  curl -X POST http://127.0.0.1:8080/api/transactions/submit \
  -H "Authorization: Bearer <jwt_token>" \
  -H "Content-Type: application/json" \
  -d '{"serialized_transaction": "base64_encoded_signed_transaction"}'
  ```

- **Example Response** (200 OK):
  ```json
  {
    "signature": "5Ej9Xj7z9k3j4k5l6m7n8p9q0r1s2t3u4v5w6x7y8z9a0b1c2d3e4f5g6h7i8j9k0l1m2n3o4p5q6r7s8t9u0v1"
  }
  ```

- **Error Response** (400 Bad Request):
  ```json
  "Bad Request: User signature not found"
  ```

---

### 9. GET /transactions/authorities
Get the list of authorities.

- **Method**: `GET`
- **Path**: `/api/transactions/authorities`
- **Description**: Retrieves the admin authority and lists of read and write authorities from the Solana program.
- **Headers**:
  - `Authorization: Bearer <jwt_token>`
- **Request Body**: None

- **Example Request**:
  ```bash
  curl -X GET http://127.0.0.1:8080/api/transactions/authorities \
  -H "Authorization: Bearer <jwt_token>"
  ```

- **Example Response** (200 OK):
  ```json
  {
    "authority": "HtGXcunbPUU54wMa9ZiXdMXvv1b5ppT7DeFLJWdtH7Lr",
    "read_authorities": ["GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN"],
    "write_authorities": ["FaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnM"]
  }
  ```

- **Error Response** (400 Bad Request):
  ```json
  "Bad Request: Admin account not owned by program"
  ```

---

### 10. GET /patient/{patient_seed}
Get a time-limited URL to view patient data.

- **Method**: `GET`
- **Path**: `/api/patient/{patient_seed}`
- **Description**: Returns a unique, time-limited URL (valid for 1 hour) to access decrypted patient data.
- **Headers**:
  - `Authorization: Bearer <jwt_token>`
- **Path Parameters**:
  - `patient_seed`: Public key of the patient seed (base58-encoded).
- **Request Body**: None

- **Example Request**:
  ```bash
  curl -X GET http://127.0.0.1:8080/api/patient/GaoZEZtR62xeRez7WTPHWd4pdDSAHGscgj5pwoHkLgnN \
  -H "Authorization: Bearer <jwt_token>"
  ```

- **Example Response** (200 OK):
  ```json
  {
    "view_url": "http://localhost:8080/api/view_patient/550e8400-e29b-41d4-a716-446655440000"
  }
  ```

- **Error Response** (400 Bad Request):
  ```json
  "Bad Request: Patient record not initialized"
  ```

---

### 11. GET /view_patient/{token}
View decrypted patient data.

- **Method**: `GET`
- **Path**: `/api/view_patient/{token}`
- **Description**: Returns the decrypted patient data using a time-limited token from `/patient/{patient_seed}`.
- **Headers**:
  - `Authorization: Bearer <jwt_token>`
- **Path Parameters**:
  - `token`: UUID token from the `view_url` (e.g., `550e8400-e29b-41d4-a716-446655440000`).
- **Request Body**: None

- **Example Request**:
  ```bash
  curl -X GET http://127.0.0.1:8080/api/view_patient/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer <jwt_token>"
  ```

- **Example Response** (200 OK):
  ```
  name: "John Doe", bloodType: "O+", previousReport: "Updated Report"
  ```

- **Error Response** (401 Unauthorized):
  ```json
  "Unauthorized: Token expired"
  ```

- **Error Response** (500 Internal Server Error):
  ```json
  "Internal Server Error: Decryption failed: ..."
  ```

---

## Error Codes
- **200 OK**: Request succeeded.
- **400 Bad Request**: Invalid input or data not found.
- **401 Unauthorized**: Invalid or expired token/signature.
- **500 Internal Server Error**: Server-side error (e.g., encryption/decryption failure).

## Notes
- Replace `<jwt_token>` with the token obtained from `/auth`.
- The `serialized_transaction` in responses is a base64-encoded string that needs to be signed by the client and submitted via `/transactions/submit`.
- The `/view_patient/{token}` endpoint is valid for 1 hour from the time the URL is generated.

---

This documentation covers all endpoints with detailed examples. Let me know if you need additional clarifications or adjustments!
  
  
  medical-record-solana
    ✔ Initializes the admin account (1656ms)
    ✔ Adds a read authority and logs history (1089ms)
    ✔ Adds a write authority and logs history (1083ms)
    ✔ Fails to add authority as non-admin (1029ms)
    ✔ Creates a patient record with encrypted data (970ms)
    ✔ Updates a patient record with encrypted data (998ms)
    ✔ Gets patient data (authorized) (998ms)
    ✔ Fails to get patient data (unauthorized) (1141ms)
    ✔ Removes a write authority and logs history (2124ms)
  9 passing (11s)

