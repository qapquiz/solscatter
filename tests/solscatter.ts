import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { assert } from "chai";
import { Solscatter } from "../target/types/solscatter";

describe("loki", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Solscatter as Program<Solscatter>;

  const users = [
    anchor.web3.Keypair.generate(),
    anchor.web3.Keypair.generate(),
    anchor.web3.Keypair.generate(),
    anchor.web3.Keypair.generate(),
    anchor.web3.Keypair.generate(),
  ];

  it("Airdrop to users", async () => {
    const promises = users.map(user => {
      return program.provider.connection.requestAirdrop(user.publicKey, 2 * anchor.web3.LAMPORTS_PER_SOL);
    });

    const txs = await Promise.all(promises);

    await Promise.all(
      txs.map(tx => program.provider.connection.confirmTransaction(tx))
    );
  });

  it("Is initialized!", async () => {
    let [treasuryPda] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("treasury"),
      ],
      program.programId,
    );

    let [mainStatePda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("main_state")],
      program.programId,
    );

    const tx = await program.rpc.initialize({
      accounts: {
        mainState: mainStatePda,
        switchboard: anchor.web3.Keypair.generate().publicKey,
        signer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });
    console.log("Your transaction signature", tx);
  });

  it("With remaining accounts", async () => {
    let [treasuryPda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("treasury")],
      program.programId,
    );
    await program.rpc.receiveFromRemainingAccounts({
      remainingAccounts: [
        { pubkey: treasuryPda, isSigner: false, isWritable: false },
      ],
    });
  });

  it("each transactions max accounts are around 30 accounts", async () => {
    const accounts = [];
    for (let i = 0; i < 30; i++) {
      accounts.push(anchor.web3.Keypair.generate());
    }

    try {
      await program.rpc.receiveFromRemainingAccounts({
        remainingAccounts: accounts.map(account => ({ pubkey: account.publicKey, isSigner: false, isWritable: false })),
      });
    } catch {
      assert.isTrue(true);
    }
  });

  it("deposit initialize each user", async () => {
    const mainState = (await program.account.mainState.all())[0];
    let currentSlot = mainState.account.currentSlot;

    for (let user of users) {
      currentSlot = currentSlot.add(new anchor.BN(1));
      const [userDeposit] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(currentSlot.toArray("le", 8)),
        ],
        program.programId,
      );

      await program.rpc.depositInitialize({
        accounts: {
          userDeposit,
          mainState: mainState.publicKey.toBase58(),
          depositor: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
        signers: [
          user
        ],
      });
    }
  });

  it("deposit each user", async () => {
    const mainState = (await program.account.mainState.all())[0];
    let currentSlot = new anchor.BN(1);

    for (let user of users) {
      const [userDeposit] = await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(currentSlot.toArray("le", 8)),
        ],
        program.programId,
      );

      const randomAmountBetween100To500 = Math.floor(Math.random() * (500 - 100) + 100);
      console.log("deposit amount:", randomAmountBetween100To500);

      await program.rpc.deposit(new anchor.BN(randomAmountBetween100To500), {
        accounts: {
          userDeposit,
          mainState: mainState.publicKey,
          owner: user.publicKey,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
        signers: [user],
      });

      currentSlot = currentSlot.add(new anchor.BN(1));
    }
  });

  it("start drawing phase", async () => {
    const mainState = (await program.account.mainState.all())[0];
    const [drawingResultPda] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("drawing_result"),
        Buffer.from(mainState.account.currentRound.toArray("le", 8)),
      ],
      program.programId,
    );

    const randomNumber = Math.floor(Math.random() * (mainState.account.totalDeposit.toNumber()));
    console.log("randomNumber:", randomNumber);

    await program.rpc.startDrawingPhaase(new anchor.BN(randomNumber), {
      accounts: {
        drawingResult: drawingResultPda,
        mainState: mainState.publicKey,
        signer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      }
    });
  });
});

