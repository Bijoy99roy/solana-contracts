import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenVesting } from "../target/types/token_vesting";
import {
  createAssociatedTokenAccount,
  createMint,
  getAccount,
  getAssociatedTokenAddress,
  mintTo,
} from "@solana/spl-token";
import { assert } from "chai";

describe("token-vesting", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();
  const program = anchor.workspace.tokenVesting as Program<TokenVesting>;

  const admin = anchor.web3.Keypair.generate();
  const beneficiary = anchor.web3.Keypair.generate();

  const vestingsParams = {};

  async function setupVesting(index: number, amount: number) {
    const indexBN = new anchor.BN(index);
    const mint = await createMint(
      connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      9
    );

    const userAta = await getAssociatedTokenAddress(mint, admin.publicKey);
    const beneficiaryAta = await getAssociatedTokenAddress(
      mint,
      beneficiary.publicKey
    );
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

    const [vaultPda] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), mint.toBuffer()],
      program.programId
    );

    const [vaultAta] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("vault"),
        mint.toBuffer(),
        indexBN.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    const [vestingAccountPda, vestingAccountBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("vesting"),
          beneficiary.publicKey.toBuffer(),
          indexBN.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );
    vestingsParams[index] = {
      index: indexBN,
      mint,
      userAta,
      beneficiaryAta,
      vestingAccountPda,
      vaultAta,
      vaultPda,
    };

    return vestingsParams[index];
  }

  const connection = provider.connection;

  before(async () => {
    const airdropSig = await provider.connection.requestAirdrop(
      admin.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 100
    );
    await provider.connection.confirmTransaction(airdropSig);
    await setupVesting(2, 1_000_000_000);
    await setupVesting(3, 1_000_000_000);
  });

  it("Fail on initalization for total amount should be greater than zero", async () => {
    const { mint, userAta, vaultAta, vestingAccountPda, index } =
      vestingsParams[2];
    const vestingPeriod = new anchor.BN(30 * 24 * 60 * 60); // 30 days in seconds
    const duration = new anchor.BN(6 * 30 * 24 * 60 * 60); // 6 months in seconds
    const totalAmount = new anchor.BN(0); // 0 tokens (9 decimals)

    try {
      await program.methods
        .initializeVesting(
          mint,
          beneficiary.publicKey,
          vestingPeriod,
          duration,
          totalAmount,
          index
        )
        .accounts({
          user: admin.publicKey,
          vestingAccount: vestingAccountPda,
          vaultAta: vaultAta,
          mint,
          adminAta: userAta,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      const anchorError = err as anchor.AnchorError;

      assert.equal(anchorError.error.errorCode.code, "MustBeGreaterThenZero");
      assert.equal(
        anchorError.error.errorMessage,
        "Total amount must be greater than zero"
      );
    }
  });

  it("Fail on initalization for vesting period is greater than duration", async () => {
    const { mint, userAta, vaultAta, vestingAccountPda, index } =
      vestingsParams[2];
    const duration = new anchor.BN(30 * 24 * 60 * 60); // 30 days in seconds
    const vestingPeriod = new anchor.BN(6 * 30 * 24 * 60 * 60); // 6 months in seconds
    const totalAmount = new anchor.BN(1_000_000_000); // 0 tokens (9 decimals)

    try {
      await program.methods
        .initializeVesting(
          mint,
          beneficiary.publicKey,
          vestingPeriod,
          duration,
          totalAmount,
          index
        )
        .accounts({
          user: admin.publicKey,
          vestingAccount: vestingAccountPda,
          vaultAta: vaultAta,
          mint,
          adminAta: userAta,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      const anchorError = err as anchor.AnchorError;

      assert.equal(
        anchorError.error.errorCode.code,
        "VestingPeriodExceedsDuration"
      );
      assert.equal(
        anchorError.error.errorMessage,
        "Vesting period is exceeing duration"
      );
    }
  });

  it("Fail on initalization for vesting period is greater than duration", async () => {
    const { mint, userAta, vaultAta, vestingAccountPda, index } =
      vestingsParams[2];
    const vestingPeriod = new anchor.BN(30 * 24 * 60 * 60); // 30 days in seconds
    const duration = new anchor.BN(0); // 0 seconds
    const totalAmount = new anchor.BN(1_000_000_000); // 0 tokens (9 decimals)

    try {
      await program.methods
        .initializeVesting(
          mint,
          beneficiary.publicKey,
          vestingPeriod,
          duration,
          totalAmount,
          index
        )
        .accounts({
          user: admin.publicKey,
          vestingAccount: vestingAccountPda,
          vaultAta: vaultAta,
          mint,
          adminAta: userAta,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      const anchorError = err as anchor.AnchorError;

      assert.equal(anchorError.error.errorCode.code, "InvalidTimestamp");
      assert.equal(
        anchorError.error.errorMessage,
        "Provided timestamp is invalid!!"
      );
    }
  });

  it("Fail on initalization for duration is not divisible by vesting period", async () => {
    const { mint, userAta, vaultAta, vestingAccountPda, index } =
      vestingsParams[2];
    const vestingPeriod = new anchor.BN(30 * 24 * 60 * 60); // 30 days in seconds
    const duration = new anchor.BN(31 * 24 * 60 * 60); // 31 days in seconds
    const totalAmount = new anchor.BN(1_000_000_000); // 0 tokens (9 decimals)

    try {
      await program.methods
        .initializeVesting(
          mint,
          beneficiary.publicKey,
          vestingPeriod,
          duration,
          totalAmount,
          index
        )
        .accounts({
          user: admin.publicKey,
          vestingAccount: vestingAccountPda,
          vaultAta: vaultAta,
          mint,
          adminAta: userAta,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();
    } catch (err) {
      const anchorError = err as anchor.AnchorError;

      assert.equal(anchorError.error.errorCode.code, "DurationNotDivisible");
      assert.equal(
        anchorError.error.errorMessage,
        "Duration is not divisible by vesting period"
      );
    }
  });

  it("Initialize token vesting", async () => {
    const { mint, userAta, vaultAta, vestingAccountPda, index } =
      vestingsParams[2];
    const vestingPeriod = new anchor.BN(30 * 24 * 60 * 60); // 30 days in seconds
    const duration = new anchor.BN(6 * 30 * 24 * 60 * 60); // 6 months in seconds
    const totalAmount = new anchor.BN(1_000_000_000); // 1 tokens (9 decimals)
    const userBefore = await getAccount(connection, userAta);

    await program.methods
      .initializeVesting(
        mint,
        beneficiary.publicKey,
        vestingPeriod,
        duration,
        totalAmount,
        index
      )
      .accounts({
        user: admin.publicKey,
        vestingAccount: vestingAccountPda,
        vaultAta: vaultAta,
        mint,
        adminAta: userAta,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    const userAfter = await getAccount(connection, userAta);
    const vaultAfter = await getAccount(connection, vaultAta);

    assert.equal(userAfter.amount.toString(), totalAmount.toString());
    assert.equal(vaultAfter.amount.toString(), totalAmount.toString());
  });

  it("Fails with error vesting started", async () => {
    const {
      mint,
      userAta,
      vaultAta,
      vestingAccountPda,
      index,
      beneficiaryAta,
    } = vestingsParams[2];
    try {
      await program.methods
        .claimVestedToken(index)
        .accounts({
          beneficiary: beneficiary.publicKey,
          vestingAccount: vestingAccountPda,
          vaultAta,
          beneficiaryAta: beneficiaryAta,
          mint,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([beneficiary])
        .rpc();
    } catch (err) {
      const anchorError = err as anchor.AnchorError;

      assert.equal(anchorError.error.errorCode.code, "VestingNotStarted");
    }
  });

  it("Fails with error vesting period not reached", async () => {
    const {
      mint,
      userAta,
      vaultAta,
      vestingAccountPda,
      index,
      beneficiaryAta,
    } = vestingsParams[2];
    try {
      await new Promise((res) => setTimeout(res, 3000));
      await program.methods
        .claimVestedToken(index)
        .accounts({
          beneficiary: beneficiary.publicKey,
          vestingAccount: vestingAccountPda,
          vaultAta,
          beneficiaryAta: beneficiaryAta,
          mint,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([beneficiary])
        .rpc();
    } catch (err) {
      const anchorError = err as anchor.AnchorError;

      assert.equal(anchorError.error.errorCode.code, "VestingPeriodNotReached");
    }
  });

  it("Initialize token vesting (2)", async () => {
    const { mint, userAta, vaultAta, vestingAccountPda, index } =
      vestingsParams[3];
    const vestingPeriod = new anchor.BN(2); // 2 seconds
    const duration = new anchor.BN(10); // 10 seconds
    const totalAmount = new anchor.BN(1_000_000_000); // 1 tokens (9 decimals)
    const period = duration.div(vestingPeriod);
    console.log(totalAmount.div(period).toString());
    const userBefore = await getAccount(connection, userAta);

    await program.methods
      .initializeVesting(
        mint,
        beneficiary.publicKey,
        vestingPeriod,
        duration,
        totalAmount,
        index
      )
      .accounts({
        user: admin.publicKey,
        vestingAccount: vestingAccountPda,
        vaultAta: vaultAta,
        mint,
        adminAta: userAta,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([admin])
      .rpc();

    const userAfter = await getAccount(connection, userAta);
    const vaultAfter = await getAccount(connection, vaultAta);

    assert.equal(userAfter.amount.toString(), totalAmount.toString());
    assert.equal(vaultAfter.amount.toString(), totalAmount.toString());
  });

  it("Fails when non-beneficiary tries to claim", async () => {
    const {
      mint,
      userAta,
      vaultAta,
      vestingAccountPda,
      index,
      beneficiaryAta,
    } = vestingsParams[3];
    const attacker = anchor.web3.Keypair.generate();
    try {
      await new Promise((res) => setTimeout(res, 3000));
      await program.methods
        .claimVestedToken(index)
        .accounts({
          beneficiary: beneficiary.publicKey,
          vestingAccount: vestingAccountPda,
          vaultAta,
          beneficiaryAta: beneficiaryAta,
          mint,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        })
        .signers([attacker])
        .rpc();
    } catch (err) {
      assert.include(err.message, "unknown signer");
    }
  });

  it("Transfter vested period token to beneficiary", async () => {
    const {
      mint,
      vaultPda,
      userAta,
      vaultAta,
      vestingAccountPda,
      index,
      beneficiaryAta,
    } = vestingsParams[3];

    await new Promise((res) => setTimeout(res, 8000));
    await program.methods
      .claimVestedToken(index)
      .accounts({
        beneficiary: beneficiary.publicKey,
        vestingAccount: vestingAccountPda,
        vaultAta,
        beneficiaryAta: beneficiaryAta,
        mint,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([beneficiary])
      .rpc();
    const beneficiaryAccount = await getAccount(connection, beneficiaryAta);

    console.log(beneficiaryAccount.amount.toString());
  });
});
