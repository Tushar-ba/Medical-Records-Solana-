import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MedicalRecordSolana } from "../target/types/medical_record_solana";
import { PublicKey, Keypair, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { expect } from "chai";
import BN from "bn.js";
import * as crypto from "crypto";

const ENCRYPTION_KEY = crypto.randomBytes(32); // 256-bit key
const IV = crypto.randomBytes(16); // Initialization vector

function encrypt(data: string): string {
  const cipher = crypto.createCipheriv("aes-256-cbc", ENCRYPTION_KEY, IV);
  let encrypted = cipher.update(data, "utf8", "base64");
  encrypted += cipher.final("base64");
  return encrypted;
}

function decrypt(encrypted: string): string {
  const decipher = crypto.createDecipheriv("aes-256-cbc", ENCRYPTION_KEY, IV);
  let decrypted = decipher.update(encrypted, "base64", "utf8");
  decrypted += decipher.final("utf8");
  return decrypted;
}

describe("medical-record-solana", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .medicalRecordSolana as Program<MedicalRecordSolana>;

  const wallet = provider.wallet as anchor.Wallet;

  const [adminPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("admin")],
    program.programId
  );

  const patientSeed = Keypair.generate();
  const [patientPDA] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("patient"), wallet.publicKey.toBuffer(), patientSeed.publicKey.toBuffer()],
    program.programId
  );

  it("Initializes the admin account", async () => {
    const tx = await program.methods
      .initialize()
      .accounts({
        authority: wallet.publicKey,
        adminAccount: adminPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("Init tx:", tx);
    const adminAccount = await program.account.admin.fetch(adminPDA);
    expect(adminAccount.authority.toString()).to.equal(wallet.publicKey.toString());
  });

  it("Creates a patient record with encrypted data", async () => {
    const patientData = {
      name: "John Doe",
      bloodType: "O+",
      previousReport: "Healthy",
      phNo: "1234567890",
      file: "report.pdf",
    };
    const encryptedData = encrypt(JSON.stringify(patientData));

    const tx = await program.methods
      .createPatient(encryptedData)
      .accounts({
        patient: patientPDA,
        patientSeed: patientSeed.publicKey,
        authority: wallet.publicKey,
        adminAccount: adminPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("Create tx:", tx);

    const patientAccount = await program.account.patient.fetch(patientPDA);
    expect(patientAccount.encryptedData).to.equal(encryptedData);
  });

  it("Updates a patient record with encrypted data", async () => {
    const updatedPatientData = {
      name: "Jane Doe",
      bloodType: "A-",
      previousReport: "Minor cold",
      phNo: "9876543210",
      file: "updated_report.pdf",
    };
    const encryptedData = encrypt(JSON.stringify(updatedPatientData));

    const tx = await program.methods
      .updatePatient(encryptedData)
      .accounts({
        patient: patientPDA,
        patientSeed: patientSeed.publicKey,
        authority: wallet.publicKey,
        adminAccount: adminPDA,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .rpc();
    console.log("Update tx:", tx);

    const patientAccount = await program.account.patient.fetch(patientPDA);
    expect(patientAccount.encryptedData).to.equal(encryptedData);
  });

  it("Gets patient data (authorized)", async () => {
    const tx = await program.methods
      .getPatient()
      .accounts({
        patient: patientPDA,
        patientSeed: patientSeed.publicKey,
        authority: wallet.publicKey,
        adminAccount: adminPDA,
      })
      .rpc();
    console.log("Get tx:", tx);

    const patientAccount = await program.account.patient.fetch(patientPDA);
    const decryptedData = decrypt(patientAccount.encryptedData);
    const patientData = JSON.parse(decryptedData);
    expect(patientData.name).to.equal("Jane Doe");
  });

  it("Fails to get patient data (unauthorized)", async () => {
    const unauthorizedWallet = Keypair.generate();
    const connection = provider.connection;

    const airdropSignature = await connection.requestAirdrop(
      unauthorizedWallet.publicKey,
      LAMPORTS_PER_SOL
    );
    await connection.confirmTransaction(airdropSignature);

    try {
      await program.methods
        .getPatient()
        .accounts({
          patient: patientPDA,
          patientSeed: patientSeed.publicKey,
          authority: unauthorizedWallet.publicKey,
          adminAccount: adminPDA,
        })
        .signers([unauthorizedWallet])
        .rpc();
      expect.fail("Should have thrown an Unauthorized error");
    } catch (error: any) {
      console.log("Error details:", error);
      const errorString = error.toString();
      expect(errorString).to.include("Unauthorized access");
    }
  });
});