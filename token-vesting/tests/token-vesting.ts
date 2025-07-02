import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { TokenVesting } from "../target/types/token_vesting";

describe("token-vesting", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const provider = anchor.getProvider();
  const program = anchor.workspace.tokenVesting as Program<TokenVesting>;

  const admin = anchor.web3.Keypair.generate();
  const beneficiary = anchor.web3.Keypair.generate();

  const index1 = new anchor.BN(1);

  const [vesting_account_pda, vesting_account_bump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vesting"), beneficiary.publicKey.toBuffer(), index1.toArrayLike(Buffer, "le", 8)], program.programId
  );

  const mint = ""
  it("Initialize token vesting", async () => {
    await program.methods.initializeVesting()
    .accounts()

  });
});
