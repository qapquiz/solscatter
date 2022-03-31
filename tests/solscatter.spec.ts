import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Solscatter } from "../target/types/solscatter";
import { isAccountAlreadyInitialize, loadKeypair } from "./utils";
import { createVrfAccount } from "./vrf";

describe("solscatter specs", () => {
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Solscatter as Program<Solscatter>;
  const yiUnderlyingMint = new anchor.web3.PublicKey("5fjG31cbSszE6FodW37UJnNzgVTyqg5WHWGCmL3ayAvA");
  const yiMint = new anchor.web3.PublicKey("6XyygxFmUeemaTvA9E9mhH9FvgpynZqARVyG3gUdCMt7");
  const quarryProgram = new anchor.web3.PublicKey("QMNeHCGYnLVDn1icRAfQZpjPLBNkfGbSKRB83G5d8KB");
  const quarry = new anchor.web3.PublicKey("5pE3ch5haoHp7wYKdsbKJwrNZzeuXr5nCj6Nn2JQf4JS");
  const rewarder = new anchor.web3.PublicKey("57fEKyj9C7dhdcrFCXFGQZp68CmjfVS4sgNecy2PdnTC");
  const vrfKeypair = loadKeypair("./secrets/vrf-keypair.json");

  it("should create VrfAccount", async () => {
    const isVrfInitialize = await isAccountAlreadyInitialize(
      program.provider.connection,
      vrfKeypair.publicKey
    );
    console.log("is vrf initialize yet? :", isVrfInitialize);

    if (isVrfInitialize) {
      return;
    }

    await createVrfAccount(program, vrfKeypair);
  });

  it("should initialize program and yi and quarry", async () => {
    const [platformAuthorityPda] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("PLATFORM_AUTHORITY"),
      ],
      program.programId,
    );
    const [miner] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("Miner"),
        quarry.toBuffer(),
        platformAuthorityPda.toBuffer(),
      ],
      quarryProgram,
    );
    const yiMintTokenAccount = await anchor.utils.token.associatedAddress({
      mint: yiMint,
      owner: platformAuthorityPda,
    })
    const yiUnderlyingTokenAccount = await anchor.utils.token.associatedAddress({
      mint: yiUnderlyingMint,
      owner: platformAuthorityPda,
    });
    const minerVault = await anchor.utils.token.associatedAddress({
      mint: yiMint,
      owner: miner
    });

    const initializeTx = await program
      .methods
      .initialize()
      .accounts({
        yiUnderlyingMint,
        yiMint,
        yiMintTokenAccount,
        yiUnderlyingTokenAccount,
        quarryProgram,
        quarry,
        vrfAccountInfo: vrfKeypair.publicKey,
        signer: program.provider.wallet.publicKey,
      })
      .preInstructions([
        await program.methods.initializeVrf().accounts({
          vrfAccountInfo: vrfKeypair.publicKey,
          signer: program.provider.wallet.publicKey,
        }).instruction(),
        await program.methods.initializeQuarry().accounts({
          yiMint: yiMint,
          quarryProgram,
          quarry,
          rewarder,
          minerVault,
          signer: program.provider.wallet.publicKey,
        }).instruction(),
      ])
      .rpc();

    console.log("initialize tx:", initializeTx)
  });
});