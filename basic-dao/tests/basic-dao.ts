import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { BasicDao } from "../target/types/basic_dao";
import {
  createAssociatedTokenAccount,
  createMint,
  getAccount,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";
import { assert } from "chai";
describe("basic-dao", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.basicDao as Program<BasicDao>;

  const provider = anchor.getProvider();

  const authority = anchor.web3.Keypair.generate();

  const daoMember1 = anchor.web3.Keypair.generate();
  const daoMember2 = anchor.web3.Keypair.generate();
  const daoMember3 = anchor.web3.Keypair.generate();
  const daoMember4 = anchor.web3.Keypair.generate();
  const daoMember5 = anchor.web3.Keypair.generate();

  let daoParams: {
    mint: anchor.web3.PublicKey;
    authorityAta: anchor.web3.PublicKey;
    daoPda: anchor.web3.PublicKey;
    vaultAta: anchor.web3.PublicKey;
  } = {} as any;
  const connection = provider.connection;

  async function setupDao() {
    const mint = await createMint(
      connection,
      provider.wallet.payer,
      provider.wallet.publicKey,
      null,
      9
    );

    const authorityAta = await createAssociatedTokenAccount(
      connection,
      provider.wallet.payer,
      mint,
      authority.publicKey
    );

    await mintTo(
      connection,
      provider.wallet.payer,
      mint,
      authorityAta,
      provider.wallet.payer,
      5_000_000_000
    );

    const [daoPda, daoBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("dao"), mint.toBuffer()],
      program.programId
    );

    const [vaultAta, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("vault"), mint.toBuffer()],
      program.programId
    );

    return {
      mint,
      authorityAta,
      daoPda,
      vaultAta,
    };
  }

  async function prepareAccountWithToken(
    mint: anchor.web3.PublicKey,
    wallet: anchor.web3.PublicKey,
    amount: number
  ) {
    const walletAta = await createAssociatedTokenAccount(
      connection,
      provider.wallet.payer,
      mint,
      wallet
    );

    await mintTo(
      connection,
      provider.wallet.payer,
      mint,
      walletAta,
      provider.wallet.payer,
      amount
    );
    return walletAta;
  }
  async function airdropSol(wallet: anchor.web3.PublicKey, amount: u64) {
    const airdropSig = await provider.connection.requestAirdrop(
      wallet,
      anchor.web3.LAMPORTS_PER_SOL * amount
    );
    await provider.connection.confirmTransaction(airdropSig);
  }
  before(async () => {
    const airdropSig = await provider.connection.requestAirdrop(
      authority.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 100
    );
    await provider.connection.confirmTransaction(airdropSig);
    daoParams = await setupDao();
  });
  it("Initialize Dao", async () => {
    const proposalDuration = new anchor.BN(5); // 30 days in seconds
    const quoram = new anchor.BN(100_000);
    const minVotingThreshold = new anchor.BN(180_000);
    const mintProposalCreationThreshold = new anchor.BN(2_000_000_000);
    const tokenAllocation = new anchor.BN(2_000_000_000);
    const { mint, authorityAta, daoPda, vaultAta } = daoParams;
    const tx = await program.methods
      .initializeDao(
        quoram,
        proposalDuration,
        minVotingThreshold,
        mintProposalCreationThreshold,
        tokenAllocation
      )
      .accounts({
        authority: authority.publicKey,
        dao: daoPda,
        vaultAta: vaultAta,
        authorityAta: authorityAta,
        tokenMint: mint,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .signers([authority])
      .rpc();
    const vaultAfter = await getAccount(connection, vaultAta);
    const account = await program.account.daoState.fetch(daoPda);
    assert.equal(vaultAfter.amount.toString(), tokenAllocation.toString());
    assert.equal(
      account.minVotingThreshold.toString(),
      minVotingThreshold.toString()
    );
    assert.equal(account.quoram.toString(), quoram.toString());
    assert.equal(account.tokenMint.toString(), mint.toString());
  });

  it("Create proposal", async () => {
    const description = "Send tokens to the best marketing guy";
    const proposalIndex = new anchor.BN(1);
    const actionAmount = new anchor.BN(1_000_000_000);
    const { mint, authorityAta, daoPda, vaultAta } = daoParams;
    const [proposalPda, proposalBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("proposal"),
          daoPda.toBuffer(),
          daoMember2.publicKey.toBuffer(),
          proposalIndex.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

    const daoMember2Ata = await prepareAccountWithToken(
      mint,
      daoMember2.publicKey,
      50_000_000_000
    );
    await airdropSol(daoMember2.publicKey, 100);
    const daoMember1Ata = await createAssociatedTokenAccount(
      connection,
      provider.wallet.payer,
      mint,
      daoMember1.publicKey
    );
    await program.methods
      .createProposal(
        proposalIndex,
        description,

        actionAmount,
        daoMember1Ata
      )
      .accounts({
        proposer: daoMember2.publicKey,
        dao: daoPda,
        proposal: proposalPda,
        proposerTokenAccount: daoMember2Ata,
      })
      .signers([daoMember2])
      .rpc();

    const account = await program.account.proposal.fetch(proposalPda);
    console.log(account.description);
    assert.equal(account.description, description);
    assert.equal(account.actionAmount.toString(), actionAmount.toString());
    assert.equal(account.dao.toString(), daoPda.toString());
    assert.equal(account.actionTarget.toString(), daoMember1Ata.toString());
    assert.equal(account.proposer.toString(), daoMember2.publicKey.toString());
  });

  it("Cast vote", async () => {
    const proposalIndex = new anchor.BN(1);
    const { mint, authorityAta, daoPda, vaultAta } = daoParams;
    const [proposalPda, proposalBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("proposal"),
          daoPda.toBuffer(),
          daoMember2.publicKey.toBuffer(),
          proposalIndex.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );

    const [voteReceiptPda1, voteReceiptBump1] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote"),
          proposalPda.toBuffer(),
          daoMember2.publicKey.toBuffer(),
        ],
        program.programId
      );
    const daoMember2Ata = await getAssociatedTokenAddress(
      mint,
      daoMember2.publicKey
    );

    console.log(daoMember2.publicKey);
    console.log(daoPda);
    console.log(proposalPda);
    console.log(voteReceiptPda1);
    console.log(daoMember2Ata);
    await program.methods
      .castVote(true)
      .accounts({
        voter: daoMember2.publicKey,
        dao: daoPda,
        proposal: proposalPda,
        voteReceipt: voteReceiptPda1,
        voterTokenAccount: daoMember2Ata,
      })
      .signers([daoMember2])
      .rpc();
    console.log("Voter 1 voted");
    const [voteReceiptPda2, voteReceiptBump2] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("vote"),
          proposalPda.toBuffer(),
          daoMember3.publicKey.toBuffer(),
        ],
        program.programId
      );
    const daoMember3Ata = await prepareAccountWithToken(
      mint,
      daoMember3.publicKey,
      100_000_000_000
    );
    await airdropSol(daoMember3.publicKey, 100);
    await program.methods
      .castVote(false)
      .accounts({
        voter: daoMember3.publicKey,
        dao: daoPda,
        proposal: proposalPda,
        voteReceipt: voteReceiptPda2,
        voterTokenAccount: daoMember3Ata,
      })
      .signers([daoMember3])
      .rpc();

    const account = await program.account.proposal.fetch(proposalPda);
    assert.equal(account.yesVotes.toNumber(), 223606);
    assert.equal(account.noVotes.toNumber(), 316227);
  });

  it("execute proposal", async () => {
    await new Promise((res) => setTimeout(res, 5000));
    const proposalIndex = new anchor.BN(1);
    const { mint, authorityAta, daoPda, vaultAta } = daoParams;
    const [proposalPda, proposalBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [
          Buffer.from("proposal"),
          daoPda.toBuffer(),
          daoMember2.publicKey.toBuffer(),
          proposalIndex.toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );
    const daoMember1Ata = await getAssociatedTokenAddress(
      mint,
      daoMember1.publicKey
    );
    await program.methods
      .executeProposal()
      .accounts({
        dao: daoPda,
        proposal: proposalPda,
        vaultAta: vaultAta,
        recipientTokenAccount: daoMember1Ata,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      })
      .rpc();

    const account = await program.account.proposal.fetch(proposalPda);
    const daoMember1AtaAfter = await getAccount(connection, daoMember1Ata);
    assert.equal(
      daoMember1AtaAfter.amount.toString(),
      account.actionAmount.toString()
    );
  });
});
