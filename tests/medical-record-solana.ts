import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { MedicalRecordSolana } from "../target/types/medical_record_solana";
import { PublicKey, Keypair } from "@solana/web3.js";
import { expect } from "chai";
import BN from "bn.js";

describe("medical-record-solana", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace
    .medicalRecordSolana as Program<MedicalRecordSolana>;

  const wallet = provider.wallet as anchor.Wallet;

  const [adminPDA, adminBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("admin")],
    program.programId
  );

  const patientSeed = Keypair.generate();
  const [patientPDA, patientBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("patient"), wallet.publicKey.toBuffer(), patientSeed.publicKey.toBuffer()],
    program.programId
  );

  it("Initializes the admin account (first call)", async () => {
    console.log("Admin PDA:", adminPDA.toString());
    console.log("Using wallet public key:", wallet.publicKey.toString());

    try {
      const tx = await program.methods
        .initialize()
        .accounts({
          authority: wallet.publicKey,
          adminAccount: adminPDA, // Use camelCase to match IDL
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Transaction signature (first call):", tx);
      console.log("Initialization successful!");

      const adminAccount = await program.account.admin.fetch(adminPDA);
      console.log("Admin account authority:", adminAccount.authority.toString());
      expect(adminAccount.authority.toString()).to.equal(wallet.publicKey.toString());
    } catch (error) {
      console.error("Error during first initialization:", error);
      throw error;
    }
  });

  it("Runs initialize again with init_if_needed (second call)", async () => {
    try {
      const tx = await program.methods
        .initialize()
        .accounts({
          authority: wallet.publicKey,
          adminAccount: adminPDA, // Use camelCase to match IDL
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Transaction signature (second call):", tx);
      console.log("Second call successful with init_if_needed!");

      const adminAccount = await program.account.admin.fetch(adminPDA);
      console.log("Admin account authority after second call:", adminAccount.authority.toString());
      expect(adminAccount.authority.toString()).to.equal(wallet.publicKey.toString());
    } catch (error) {
      console.error("Error during second initialization:", error);
      throw error;
    }
  });

  it("Creates a patient record", async () => {
    const patientData = {
      name: "John Doe",
      bloodType: "O+",
      previousReport: "Healthy, no issues reported.",
      phNo: new BN("1234567890"),
      file: "report.pdf",
    };

    try {
      const tx = await program.methods
        .createPatient(
          patientData.name,
          patientData.bloodType,
          patientData.previousReport,
          patientData.phNo,
          patientData.file
        )
        .accounts({
          patient: patientPDA,
          patientSeed: patientSeed.publicKey, // Use camelCase to match IDL
          authority: wallet.publicKey,
          adminAccount: adminPDA, // Use camelCase to match IDL
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Patient creation transaction signature:", tx);
      console.log("Patient record created successfully!");

      const patientAccount = await program.account.patient.fetch(patientPDA);
      console.log("Patient account data:", patientAccount);
      expect(patientAccount.patientAddress.toString()).to.equal(patientPDA.toString());
      expect(patientAccount.isInitialized).to.equal(true);
      expect(patientAccount.name).to.equal(patientData.name);
      expect(patientAccount.bloodType).to.equal(patientData.bloodType);
      expect(patientAccount.previousReport).to.equal(patientData.previousReport);
      expect(patientAccount.phNo.toString()).to.equal(patientData.phNo.toString());
      expect(patientAccount.file).to.equal(patientData.file);
    } catch (error) {
      console.error("Error during patient creation:", error);
      throw error;
    }
  });

  it("Updates a patient record", async () => {
    const updatedPatientData = {
      name: "Jane Doe",
      bloodType: "A-",
      previousReport: "Updated: Minor cold reported.",
      phNo: new BN("9876543210"),
      file: "updated_report.pdf",
    };

    try {
      const tx = await program.methods
        .updatePatient(
          updatedPatientData.name,
          updatedPatientData.bloodType,
          updatedPatientData.previousReport,
          updatedPatientData.phNo,
          updatedPatientData.file
        )
        .accounts({
          patient: patientPDA,
          patientSeed: patientSeed.publicKey, // Use camelCase to match IDL
          authority: wallet.publicKey,
          adminAccount: adminPDA, // Use camelCase to match IDL
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .rpc();

      console.log("Patient update transaction signature:", tx);
      console.log("Patient record updated successfully!");

      const patientAccount = await program.account.patient.fetch(patientPDA);
      console.log("Updated patient account data:", patientAccount);
      expect(patientAccount.name).to.equal(updatedPatientData.name);
      expect(patientAccount.bloodType).to.equal(updatedPatientData.bloodType);
      expect(patientAccount.previousReport).to.equal(updatedPatientData.previousReport);
      expect(patientAccount.phNo.toString()).to.equal(updatedPatientData.phNo.toString());
      expect(patientAccount.file).to.equal(updatedPatientData.file);
      expect(patientAccount.isInitialized).to.equal(true);
    } catch (error) {
      console.error("Error during patient update:", error);
      throw error;
    }
  });
});