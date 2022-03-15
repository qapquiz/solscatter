import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { assert } from "chai";
import { Solscatter } from "../target/types/solscatter";

type OptionWinner = anchor.web3.PublicKey | null;

describe("solscatter", () => {
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
    const promises = users.map((user) => {
      return program.provider.connection.requestAirdrop(
        user.publicKey,
        2 * anchor.web3.LAMPORTS_PER_SOL
      );
    });

    const txs = await Promise.all(promises);

    await Promise.all(
      txs.map((tx) => program.provider.connection.confirmTransaction(tx))
    );
  });

  it("Is initialized!", async () => {
    let [mainStatePda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("main_state")],
      program.programId
    );

    // @todo #1 must change client states and vrf to correct one
    const tx = await program.rpc.initialize({
      accounts: {
        mainState: mainStatePda,
        vrfClientState: anchor.web3.Keypair.generate().publicKey,
        vrfAccountInfo: anchor.web3.Keypair.generate().publicKey,
        signer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });
    console.log("Your transaction signature", tx);
  });

  it("deposit initialize each user", async () => {
    const mainState = (await program.account.mainState.all())[0];
    let currentSlot = mainState.account.currentSlot;

    for (let user of users) {
      currentSlot = currentSlot.add(new anchor.BN(1));
      const [userDeposit] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(currentSlot.toArray("le", 8))],
        program.programId
      );

      await program.rpc.depositInitialize({
        accounts: {
          userDeposit,
          mainState: mainState.publicKey.toBase58(),
          depositor: user.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
        signers: [user],
      });
    }
  });

  it("deposit each user", async () => {
    const mainState = (await program.account.mainState.all())[0];
    let currentSlot = new anchor.BN(1);

    for (let user of users) {
      const [userDeposit] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(currentSlot.toArray("le", 8))],
        program.programId
      );

      const randomAmountBetween100To500 = Math.floor(
        Math.random() * (500 - 100) + 100
      );
      console.log(
        "user: %s ->  deposit amount: %s",
        user.publicKey.toBase58(),
        randomAmountBetween100To500
      );

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
      program.programId
    );

    const numberOfRewards = 5;
    const randomNumbers: anchor.BN[] = [];
    for (let i = 0; i < numberOfRewards; i++) {
      randomNumbers[i] = new anchor.BN(
        Math.floor(Math.random() * mainState.account.totalDeposit.toNumber())
      );
    }
    console.log("numberOfRewards:", numberOfRewards);
    console.log("randomNumbers:", randomNumbers.map(randomNumber => randomNumber.toString()));

    await program.rpc.startDrawingPhase(numberOfRewards, randomNumbers, {
      accounts: {
        drawingResult: drawingResultPda,
        mainState: mainState.publicKey,
        signer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });
  });

  it("drawing each user", async () => {
    const mainState = (await program.account.mainState.all())[0];
    const [drawingResultPda] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("drawing_result"),
        Buffer.from(mainState.account.currentRound.toArray("le", 8)),
      ],
      program.programId
    );

    let drawingResult = await program.account.drawingResult.fetch(
      drawingResultPda
    );
    let processSlot = drawingResult.lastProcessedSlot;

    let totalFees = 0;

    for (let user of users) {
      processSlot = processSlot.add(new anchor.BN(1));
      const [userDepositPda] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(processSlot.toArray("le", 8))],
        program.programId
      );

      

      const tx = await program.rpc.drawing({
        accounts: {
          mainState: mainState.publicKey,
          drawingResult: drawingResultPda,
          userDeposit: userDepositPda,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
      });

      await program.provider.connection.confirmTransaction(tx, "confirmed");
      const fee = (await program.provider.connection.getTransaction(tx, { commitment: "confirmed" })).meta.fee;
      totalFees = totalFees + fee;

      console.log(
        "processSlot: %s \n\t drawing: %s \n\t tx: %s",
        processSlot.toString(),
        user.publicKey.toBase58(),
        tx,
      );

      drawingResult = await program.account.drawingResult.fetch(
        drawingResultPda
      );

      const drawWinners = drawingResult.winners as OptionWinner[];
      const haveAllWinners = drawWinners.map(winner => winner !== null).filter(isHaveWinner => isHaveWinner).length === drawingResult.numberOfRewards;
      if (haveAllWinners) {
        break;
      }
    }

    drawingResult = await program.account.drawingResult.fetch(drawingResultPda);
    (drawingResult.winners as OptionWinner[]).map(winner => winner.toBase58()).forEach(winner => console.log("winner:", winner));

    console.log("\ntotalFees: %s SOL", totalFees * 0.000000001);
    console.log(
      "finished_timestamp: %s\n",
      drawingResult.finishedTimestamp.toString()
    );
  });
});
