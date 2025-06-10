import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StakingProgram } from "../target/types/staking_program";

describe("staking-program", async () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
const provider = anchor.getProvider();  
  const program = anchor.workspace.stakingProgram as Program<StakingProgram>;
  const user  =  anchor.web3.Keypair.generate();
  const airdropSig = await provider.connection.requestAirdrop(
      user.publicKey,
      anchor.web3.LAMPORTS_PER_SOL
    );
  await provider.connection.confirmTransaction(airdropSig);
  it("Is initialized!", async () => {
    // Add your test here.


    console.log("Your transaction signature");
  });
});
