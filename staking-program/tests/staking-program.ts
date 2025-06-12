import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StakingProgram } from "../target/types/staking_program";
import { assert } from "chai";

describe("staking-program",  () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();  
  const program = anchor.workspace.stakingProgram as Program<StakingProgram>;
  const user  =  anchor.web3.Keypair.generate();
  
  console.log("user: ", user.publicKey)
  let pdaAccount;
  let bump;

  const [pda, _bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("stake_client"), user.publicKey.toBuffer()],
    program.programId
  );


  const [vaultPda, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("sol_vault"), user.publicKey.toBuffer()],
    program.programId
  );

  bump = _bump;

  before(async ()=>{
    const airdropSig = await provider.connection.requestAirdrop(
      user.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 5
    );
  await provider.connection.confirmTransaction(airdropSig);
  })

  it("create pda account", async () => {
    await program.methods.createPdaAccount()
    .accounts({
      payer: user.publicKey,
      pdaAccount: pda,
      vault: vaultPda
  })
  .signers([user])
    .rpc()


    const account = await program.account.stakeAccount.fetch(pda);
    console.log(account.owner)
    assert.ok(account.owner.equals(user.publicKey));
    assert.equal(account.stakedAmount.toNumber(), 0);
  });

  it("stake", async ()=>{
    const stakeAmount = new anchor.BN(1_000_000_000);

     await program.methods.stake(stakeAmount)
     .accounts({
      user: user.publicKey,
      pdaAccount: pda,
      vault: vaultPda
     })
     .signers([user])
     .rpc();

     const account = await program.account.stakeAccount.fetch(pda);
     console.log(account.stakedAmount.toString())
     assert.equal(account.stakedAmount.toString(), stakeAmount.toString());
  })

  it("wait and unstake", async ()=>{
    await new Promise((resolve)=>setTimeout(resolve, 5000));

    const unStakeAmount = new anchor.BN(1_000_000_000/2)

    await program.methods.unstake(unStakeAmount)
    .accounts({
      user: user.publicKey,
      pdaAccount: pda,
      vault: vaultPda
    })
    .signers([user])
    .rpc();

    const account =  await program.account.stakeAccount.fetch(pda);
    assert.equal(account.stakedAmount.toNumber(), 500000000)
    console.log(account.totalPoints)
    assert.ok(account.totalPoints.toNumber()>0)
  });

  it("Claim points", async ()=>{
    await program.methods.claimPoints()
    .accounts({
      user: user.publicKey,
      pdaAccount: pda
    })
    .signers([user])
    .rpc();

    const account = await program.account.stakeAccount.fetch(pda);
    console.log(account.totalPoints);
    assert.equal(account.totalPoints.toNumber(), 0);
  });

  it("Get poitns", async ()=>{
    await program.methods.getPoints()
    .accounts({
      user: user.publicKey,
      pdaAccount: pda
    })
    .signers([user])
    .rpc();
  })
});
