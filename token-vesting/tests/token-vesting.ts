import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenVesting } from "../target/types/token_vesting";
import { createAssociatedTokenAccount, createMint, getAccount, getAssociatedTokenAddress, mintTo } from "@solana/spl-token";
import { assert } from "chai";

describe("token-vesting", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();
  const program = anchor.workspace.tokenVesting as Program<TokenVesting>;

  const admin = anchor.web3.Keypair.generate();
  const beneficiary = anchor.web3.Keypair.generate();

  const index1 = new anchor.BN(2);

  const [vesting_account_pda, vesting_account_bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vesting"), beneficiary.publicKey.toBuffer(), index1.toArrayLike(Buffer, "le", 8)], program.programId
  );
  let mint: anchor.web3.PublicKey;
  let userAta: anchor.web3.PublicKey;
  let vaultPda: anchor.web3.PublicKey;
  let vaultAta: anchor.web3.PublicKey;
  let beneficiary_ata: anchor.web3.PublicKey;
  const connection = provider.connection;

  before(async()=>{
    const airdropSig = await provider.connection.requestAirdrop(
      admin.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 100
    );
    await provider.connection.confirmTransaction(airdropSig);
    mint = await createMint(
      connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      9
    );

    userAta = await getAssociatedTokenAddress(mint, admin.publicKey);
    beneficiary_ata = await getAssociatedTokenAddress(mint, beneficiary.publicKey);
    await createAssociatedTokenAccount(
      connection,
      provider.wallet.payer,
      mint,
      admin.publicKey
    );

    await createAssociatedTokenAccount(
      connection,
      provider.wallet.payer,
      mint,
      beneficiary.publicKey
    );

    await mintTo(
      connection,
      provider.wallet.payer,
      mint,
      userAta,
      provider.wallet.payer,
      2_000_000_000
    );

    [vaultPda] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), mint.toBuffer()],
      program.programId
    );

    // vaultAta = await getAssociatedTokenAddress(mint, vesting_account_pda, true);
    [vaultAta] = anchor.web3.PublicKey.findProgramAddressSync(
  [Buffer.from("vault"), mint.toBuffer(), index1.toArrayLike(Buffer, "le", 8)],
  program.programId
);
  })
  
  it("Initialize token vesting", async () => {
    const vestingPeriod = new anchor.BN(30 * 24 * 60 * 60); // 30 days in seconds
    const duration = new anchor.BN(6 * 30 * 24 * 60 * 60);  // 6 months in seconds
    const totalAmount = new anchor.BN(1_000_000_000);       // 1000 tokens (9 decimals)
    const userBefore = await getAccount(connection, userAta);
  // const vaultBefore = await getAccount(connection, vaultAta);

  console.log("Before:");
  console.log("- User ATA:", userBefore.amount.toString());
  // console.log("- Vault ATA:", vaultBefore.amount.toString());
console.log(mint)
console.log(beneficiary.publicKey)
console.log(admin.publicKey)
console.log(vaultAta)
console.log(userAta)
console.log(vesting_account_pda)
    await program.methods
      .initializeVesting(mint, beneficiary.publicKey, vestingPeriod, duration, totalAmount, index1)
      .accounts({
        user: admin.publicKey,
        vestingAccount: vesting_account_pda,
        vaultAta:vaultAta,
        mint,
        adminAta:userAta,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,

      })
      .signers([admin])
      .rpc();

      const userAfter = await getAccount(connection, userAta);
  const vaultAfter = await getAccount(connection, vaultAta);

  console.log("After:");
  console.log("- User ATA:", userAfter.amount.toString());
  console.log("- Vault ATA:", vaultAfter.amount.toString());

  });

  it("Fails with error vesting started", async()=>{
    try{
      await program.methods.claimVestedToken(index1)
      .accounts({
        beneficiary: beneficiary.publicKey,
        vestingAccount: vesting_account_pda,
        vaultAta,
        beneficiaryAta: beneficiary_ata,
        mint,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        
      }).signers([beneficiary])
      .rpc()
    }catch(err) {
      const anchorError = err as anchor.AnchorError;

      assert.equal(anchorError.error.errorCode.code, "VestingNotStarted");
    }
  })

  it("Fails with error vesting period not reached", async()=>{
    try{
      await new Promise((res) => setTimeout(res, 3000));
      await program.methods.claimVestedToken(index1)
      .accounts({
        beneficiary: beneficiary.publicKey,
        vestingAccount: vesting_account_pda,
        vaultAta,
        beneficiaryAta: beneficiary_ata,
        mint,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        
      }).signers([beneficiary])
      .rpc()
    }catch(err) {
      const anchorError = err as anchor.AnchorError;

      assert.equal(anchorError.error.errorCode.code, "VestingPeriodNotReached");
    }
  })
});
