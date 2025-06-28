import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import { assert } from "chai";

describe("escrow", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  const program = anchor.workspace.escrow as Program<Escrow>;

  const user = anchor.web3.Keypair.generate();
  const party = anchor.web3.Keypair.generate();
  console.log(user.publicKey)
  console.log(party.publicKey)
  const index1 = new anchor.BN(0);

  const [escrow_pda, escrow_bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), user.publicKey.toBuffer(), index1.toArrayLike(Buffer, "le", 8)],
    program.programId
  );

  const [vault_pda, vault_bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("sol_vault"), user.publicKey.toBuffer(), index1.toArrayLike(Buffer, "le", 8)],
    program.programId
  );

  const index2 = new anchor.BN(1);
  const [escrow_pda2, escrow_bump2] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("escrow"), user.publicKey.toBuffer(), index2.toArrayLike(Buffer, "le", 8)],
    program.programId
  );

  const [vault_pda2, vault_bump2] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("sol_vault"), user.publicKey.toBuffer(), index2.toArrayLike(Buffer, "le", 8)],
    program.programId
  );

  before(async()=>{
    const airdropSig = await provider.connection.requestAirdrop(
      user.publicKey,
      anchor.web3.LAMPORTS_PER_SOL * 5
    );
    await provider.connection.confirmTransaction(airdropSig);
  })
it("Fail on Initialize escrow for invalid amount", async () => {
  try{
    const escrow_amount = new anchor.BN(0);
    
    await program.methods.initializeEscrow(escrow_amount, index1)
    .accounts
    ({
      payer: user.publicKey,
      party: party.publicKey,
      escrow: escrow_pda,
      vault: vault_pda
    })
    .signers([user])
    .rpc();
  }catch(err) {
    const anchorError = err as anchor.AnchorError;
      assert.equal(anchorError.error.errorCode.code, "InvalidAmount");
      assert.equal(anchorError.error.errorCode.number, 6005);
      assert.equal(anchorError.error.errorMessage, "Invalid amount");
  }
    

  });

  it("Initialize escrow", async () => {
    const escrow_amount = new anchor.BN(1_000_000);
    
    await program.methods.initializeEscrow(escrow_amount, index1)
    .accounts
    ({
      payer: user.publicKey,
      party: party.publicKey,
      escrow: escrow_pda,
      vault: vault_pda
    })
    .signers([user])
    .rpc();

    const account =  await program.account.escrowAccount.fetch(escrow_pda);
    
    assert.ok(account.party.equals(party.publicKey));
    assert.ok(account.initiator.equals(user.publicKey));
    assert.equal(account.isCancelled, false);
    assert.equal(account.isFullfulled, false);
    assert.equal(account.partyMarkedDelivered, false);
    assert.equal(account.amount.toString(), escrow_amount.toString());


  });
  it("Fail on delivery fulfilled", async()=>{
    try{
      await program.methods.deliveryFulfilled()
      .accounts({
        initiator: user.publicKey,
        party: party.publicKey,
        escrow: escrow_pda,
        vault: vault_pda
      })
      .signers([user, party])
      .rpc();
    } catch(err) {
      const anchorError = err as anchor.AnchorError;
      assert.equal(anchorError.error.errorCode.code, "NotDelivered");
      assert.equal(anchorError.error.errorCode.number, 6004);
      assert.equal(anchorError.error.errorMessage, "Party has not yet marked delivery");
    }
  });

  it("Fail on mark as delivered for unauthorized access", async()=>{
    try{
      await program.methods.markAsDelivered()
    .accounts({
      party: user.publicKey,
      escrow: escrow_pda,
    }).signers([user])
    .rpc();
    } catch(err){
      const anchorError = err as anchor.AnchorError;
      assert.equal(anchorError.error.errorCode.code, "Unauthorized");
      assert.equal(anchorError.error.errorCode.number, 6000);
      assert.equal(anchorError.error.errorMessage, "Unauthorized operation");
    }
    
  })

  it("Fail to cancel unautorized access", async()=>{
  try{
    const fakeUser = anchor.web3.Keypair.generate()
    await program.methods.cancelEscrow()
    .accounts({
      initiator: fakeUser.publicKey,
      escrow: escrow_pda,
      vault: vault_pda
    }).signers([fakeUser])
    .rpc()
  }catch(err) {
    const anchorError = err as anchor.AnchorError;
    assert.equal(anchorError.error.errorCode.code, "Unauthorized");
    assert.equal(anchorError.error.errorCode.number, 6000);
    assert.equal(anchorError.error.errorMessage, "Unauthorized operation")
  }
  })

  it("mark as delivered", async()=>{
    await program.methods.markAsDelivered()
    .accounts({
      party: party.publicKey,
      escrow: escrow_pda
    })
    .signers([party])
    .rpc();

    const account = await program.account.escrowAccount.fetch(escrow_pda);
    assert.equal(account.partyMarkedDelivered, true);

  });

  it("Fail on delivery fulfuilled for unauthorized access", async()=>{
    try{

      const fakeUser = anchor.web3.Keypair.generate()
      
      await program.methods.deliveryFulfilled()
    .accounts({
      initiator: fakeUser.publicKey,
      party: party.publicKey,
      escrow: escrow_pda,
      vault: vault_pda
    })
    .signers([fakeUser, party])
    .rpc();
    } catch(err){
      const anchorError = err as anchor.AnchorError;
      console.log(err)
      assert.equal(anchorError.error.errorCode.code, "Unauthorized");
      assert.equal(anchorError.error.errorCode.number, 6000);
      assert.equal(anchorError.error.errorMessage, "Unauthorized operation")
    }
  })

  it("delivery fulfuilled", async()=>{
    const escrow_amount = new anchor.BN(1_000_000);
    await program.methods.markAsDelivered()
    .accounts({
      party: party.publicKey,
      escrow: escrow_pda
    })
    .signers([party])
    .rpc();
    await program.methods.deliveryFulfilled()
    .accounts({
      initiator: user.publicKey,
      party: party.publicKey,
      escrow: escrow_pda,
      vault: vault_pda
    })
    .signers([user, party])
    .rpc();
    const vaultBalance = await provider.connection.getBalance(party.publicKey);
    const account = await program.account.escrowAccount.fetch(escrow_pda);
    assert.equal(account.partyMarkedDelivered, true);
    assert.equal(account.isFullfulled, true);
    assert.equal(vaultBalance.toString(), escrow_amount.toString())

  });

  it("Fail on delivery already fulfilled", async()=>{
    try{
      await program.methods.deliveryFulfilled()
    .accounts({
      initiator: user.publicKey,
      party: party.publicKey,
      escrow: escrow_pda,
      vault: vault_pda
    })
    .signers([user, party])
    .rpc();
    } catch(err) {
      const anchorError =  err as anchor.AnchorError;

      assert.equal(anchorError.error.errorCode.code, "AlreadyFulfilled");
      assert.equal(anchorError.error.errorCode.number, 6002);
      assert.equal(anchorError.error.errorMessage, "Request already fulfulled");
    }
  })

  it("Fail to cancel after delivery already fulfilled", async()=>{
    try{
    await program.methods.cancelEscrow()
    .accounts({
      initiator: user.publicKey,
      escrow: escrow_pda,
      vault: vault_pda
    }).signers([user])
    .rpc()
  }catch(err) {
    const anchorError = err as anchor.AnchorError;
    assert.equal(anchorError.error.errorCode.code, "AlreadyFulfilled");
    assert.equal(anchorError.error.errorCode.number, 6002);
    assert.equal(anchorError.error.errorMessage, "Request already fulfulled")
  }
  })

  it("cancel escrow", async()=>{
    const escrow_amount = new anchor.BN(1_000_000);

    await program.methods.initializeEscrow(escrow_amount, index2)
    .accounts
    ({
      payer: user.publicKey,
      party: party.publicKey,
      escrow: escrow_pda2,
      vault: vault_pda2
    })
    .signers([user])
    .rpc();

    await program.methods.cancelEscrow()
    .accounts({
      initiator: user.publicKey,
      escrow: escrow_pda2,
      vault: vault_pda2
    }).signers([user])
    .rpc()

    const account = await program.account.escrowAccount.fetch(escrow_pda2);
    assert.equal(account.isCancelled, true);
  });

it("Fail on already cancelled", async()=>{
  try{
    await program.methods.cancelEscrow()
    .accounts({
      initiator: user.publicKey,
      escrow: escrow_pda2,
      vault: vault_pda2
    }).signers([user])
    .rpc()
  }catch(err) {
    const anchorError = err as anchor.AnchorError;
    assert.equal(anchorError.error.errorCode.code, "AlreadyCancelled");
    assert.equal(anchorError.error.errorCode.number, 6003);
    assert.equal(anchorError.error.errorMessage, "Escrow already cancelled")
  }
})

});

