import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { Token } from "@solana/spl-token";
import {
  Callback,
  loadSwitchboardProgram,
  OracleQueueAccount,
  PermissionAccount,
  ProgramStateAccount,
  VrfAccount,
} from "@switchboard-xyz/switchboard-v2";
import { assert } from "chai";
import { Solscatter } from "../target/types/solscatter";
import { loadKeypair } from "./utils";

type OptionWinner = anchor.web3.PublicKey | null;

const STATE_SEED = "STATE";
const DEVNET_CLUSTER = "devnet";
const SWITCHBOARD_QUEUE_PUBKEY = new anchor.web3.PublicKey(
  "F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy"
);

async function createVrfAccount(
  program: anchor.Program<Solscatter>
): Promise<void> {
  const switchboardProgram = await loadSwitchboardProgram(DEVNET_CLUSTER);
  const vrfClientProgram = program;
  // 5JQN2QPJG6vXFsKDAjuSkiuBqfy4juwP3hRvmPZw43pC
  const vrfSecret = loadKeypair("./secrets/vrf-keypair.json");

  console.log("wallet:", program.provider.wallet.publicKey.toBase58());
  console.log("vrfSecret:", vrfSecret.publicKey.toBase58());
  const [stateAccountPda, stateBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(STATE_SEED),
        vrfSecret.publicKey.toBuffer(),
        program.provider.wallet.publicKey.toBuffer(),
      ],
      vrfClientProgram.programId
    );

  console.log("######## CREATE VRF ACCOUNT ########");

  const queue = new OracleQueueAccount({
    program: switchboardProgram,
    publicKey: SWITCHBOARD_QUEUE_PUBKEY,
  });
  const { unpermissionedVrfEnabled, authority } = await queue.loadData();

  assert.isTrue(unpermissionedVrfEnabled);

  const ixCoder = new anchor.BorshInstructionCoder(vrfClientProgram.idl);
  const callback: Callback = {
    programId: vrfClientProgram.programId,
    accounts: [
      { pubkey: stateAccountPda, isSigner: false, isWritable: true },
      { pubkey: vrfSecret.publicKey, isSigner: false, isWritable: false },
    ],
    ixData: ixCoder.encode("callbackRequestRandomness", ""),
  };

  // switchTokenMint.payer: AKnL4NNf3DGWZJS6cPknBuEGnVsV4A4m5tgebLHaRSZ9
  const vrfAccount = await VrfAccount.create(switchboardProgram, {
    queue,
    callback,
    authority: stateAccountPda,
    keypair: vrfSecret,
  });

  console.log("VRF Account:", vrfAccount.publicKey.toBase58());

  const permissionAccount = await PermissionAccount.create(switchboardProgram, {
    authority: (await queue.loadData()).authority,
    granter: queue.publicKey,
    grantee: vrfAccount.publicKey,
  });

  console.log("VRF Permission:", permissionAccount.publicKey.toBase58());

  const permissionData = await permissionAccount.loadData();

  // const token = new Token(program.provider.connection, anchor.web3.Keypair.generate().publicKey, program.programId, vrfSecret);
  // token.createAssociatedTokenAccount
}

describe("solscatter", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Solscatter as Program<Solscatter>;

  const userKeypairs = [
    anchor.web3.Keypair.generate(),
    anchor.web3.Keypair.generate(),
    anchor.web3.Keypair.generate(),
    anchor.web3.Keypair.generate(),
    anchor.web3.Keypair.generate(),
  ];

  const users = userKeypairs.slice(0, 1);
  // it.only("Create Vrf", async () => {
  //   await createVrfAccount(program);
  // });

  it.only("check", async () => {
    const state = (await program.account.vrfClientState.all())[0];
    console.log(state);
  });

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
    const vrfSecret = loadKeypair("./secrets/vrf-keypair.json");
    let [mainStatePda] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("main_state")],
      program.programId
    );

    const [stateAccountPda, stateBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(STATE_SEED),
          vrfSecret.publicKey.toBuffer(),
          program.provider.wallet.publicKey.toBuffer(),
        ],
        program.programId
      );

    const tx = await program.rpc.initialize({
      accounts: {
        mainState: mainStatePda,
        vrfClientState: stateAccountPda,
        vrfAccountInfo: vrfSecret.publicKey,
        signer: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
    });
    console.log("Your transaction signature", tx);
  });

  // it("deposit initialize each user", async () => {
  //   const mainState = (await program.account.mainState.all())[0];
  //   let currentSlot = mainState.account.currentSlot;

  //   for (let user of users) {
  //     currentSlot = currentSlot.add(new anchor.BN(1));
  //     const [userDeposit] = await anchor.web3.PublicKey.findProgramAddress(
  //       [Buffer.from(currentSlot.toArray("le", 8))],
  //       program.programId
  //     );

  //     await program.rpc.depositInitialize({
  //       accounts: {
  //         userDeposit,
  //         mainState: mainState.publicKey.toBase58(),
  //         depositor: user.publicKey,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //       },
  //       signers: [user],
  //     });
  //   }
  // });

  // it("deposit each user", async () => {
  //   const mainState = (await program.account.mainState.all())[0];
  //   let currentSlot = new anchor.BN(1);

  //   for (let user of users) {
  //     const [userDeposit] = await anchor.web3.PublicKey.findProgramAddress(
  //       [Buffer.from(currentSlot.toArray("le", 8))],
  //       program.programId
  //     );

  //     const randomAmountBetween100To500 = Math.floor(
  //       Math.random() * (500 - 100) + 100
  //     );
  //     console.log(
  //       "user: %s ->  deposit amount: %s",
  //       user.publicKey.toBase58(),
  //       randomAmountBetween100To500
  //     );

  //     await program.rpc.deposit(new anchor.BN(randomAmountBetween100To500), {
  //       accounts: {
  //         userDeposit,
  //         mainState: mainState.publicKey,
  //         owner: user.publicKey,
  //         clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
  //       },
  //       signers: [user],
  //     });

  //     currentSlot = currentSlot.add(new anchor.BN(1));
  //   }
  // });

  it("request randomness", async () => {
    const vrfSecret = loadKeypair("./secrets/vrf-keypair.json");
    const switchboardProgram = await loadSwitchboardProgram(DEVNET_CLUSTER);

    const [stateAccountPda, stateBump] =
    await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from(STATE_SEED),
        vrfSecret.publicKey.toBuffer(),
        program.provider.wallet.publicKey.toBuffer(),
      ],
      program.programId
    );

    const vrfAccount = new VrfAccount({
      program: switchboardProgram,
      publicKey: vrfSecret.publicKey,
    });

    const vrfClientState = (await program.account.vrfClientState.all())[0];
    const vrf = await vrfAccount.loadData(); 
    const queueAccount = new OracleQueueAccount({
      program: switchboardProgram,
      publicKey: vrf.oracleQueue,
    });
    const queue = await queueAccount.loadData();
    const queueAuthority = queue.authority;
    const dataBuffer = queue.dataBuffer;
    const escrow = vrf.escrow;
    const [programStateAccount, programStateBump] = ProgramStateAccount.fromSeed(switchboardProgram);
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      switchboardProgram,
      queueAuthority,
      queueAccount.publicKey,
      vrfClientState.account.vrf,
    );
    try {
      await permissionAccount.loadData();
    } catch {
      throw new Error(
        "A requested permission pda account has not been initialized."
      );
    }

    const switchTokenMint = await programStateAccount.getTokenMint();
    const payerTokenAccount = await switchTokenMint.getOrCreateAssociatedAccountInfo(program.provider.wallet.publicKey);
    console.log(payerTokenAccount.address.toBase58());

    console.log(program.provider.wallet["payer"])
    const requestTxn = await program.rpc.requestRandomness({
      clientStateBump: stateBump,
      permissionBump: permissionBump,
      switchboardStateBump: programStateBump
    },
    {
      accounts: {
        state: vrfClientState.publicKey,
        authority: program.provider.wallet.publicKey,
        switchboardProgram: switchboardProgram.programId,
        vrf: vrfClientState.account.vrf,
        oracleQueue: queueAccount.publicKey,
        queueAuthority,
        dataBuffer,
        permission: permissionAccount.publicKey,
        escrow,
        payerWallet: payerTokenAccount.address,
        payerAuthority: program.provider.wallet.publicKey,
        recentBlockhashes: anchor.web3.SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
        programState: programStateAccount.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
      },
      signers: [program.provider.wallet["payer"], program.provider.wallet["payer"]],
    });
    console.log(`https://solscan.io/tx/${requestTxn}?cluster=devnet`);
  });

  // it("start drawing phase", async () => {
  //   const mainState = (await program.account.mainState.all())[0];
  //   const [drawingResultPda] = await anchor.web3.PublicKey.findProgramAddress(
  //     [
  //       Buffer.from("drawing_result"),
  //       Buffer.from(mainState.account.currentRound.toArray("le", 8)),
  //     ],
  //     program.programId
  //   );

  //   const numberOfRewards = 5;
  //   const randomNumbers: anchor.BN[] = [];
  //   for (let i = 0; i < numberOfRewards; i++) {
  //     randomNumbers[i] = new anchor.BN(
  //       Math.floor(Math.random() * mainState.account.totalDeposit.toNumber())
  //     );
  //   }
  //   console.log("numberOfRewards:", numberOfRewards);
  //   console.log(
  //     "randomNumbers:",
  //     randomNumbers.map((randomNumber) => randomNumber.toString())
  //   );

  //   await program.rpc.startDrawingPhase(numberOfRewards, randomNumbers, {
  //     accounts: {
  //       drawingResult: drawingResultPda,
  //       mainState: mainState.publicKey,
  //       signer: program.provider.wallet.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //     },
  //   });
  // });

  // it("drawing each user", async () => {
  //   const mainState = (await program.account.mainState.all())[0];
  //   const [drawingResultPda] = await anchor.web3.PublicKey.findProgramAddress(
  //     [
  //       Buffer.from("drawing_result"),
  //       Buffer.from(mainState.account.currentRound.toArray("le", 8)),
  //     ],
  //     program.programId
  //   );

  //   let drawingResult = await program.account.drawingResult.fetch(
  //     drawingResultPda
  //   );
  //   let processSlot = drawingResult.lastProcessedSlot;

  //   let totalFees = 0;

  //   for (let user of users) {
  //     processSlot = processSlot.add(new anchor.BN(1));
  //     const [userDepositPda] = await anchor.web3.PublicKey.findProgramAddress(
  //       [Buffer.from(processSlot.toArray("le", 8))],
  //       program.programId
  //     );

  //     const tx = await program.rpc.drawing({
  //       accounts: {
  //         mainState: mainState.publicKey,
  //         drawingResult: drawingResultPda,
  //         userDeposit: userDepositPda,
  //         clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
  //       },
  //     });

  //     await program.provider.connection.confirmTransaction(tx, "confirmed");
  //     const fee = (
  //       await program.provider.connection.getTransaction(tx, {
  //         commitment: "confirmed",
  //       })
  //     ).meta.fee;
  //     totalFees = totalFees + fee;

  //     console.log(
  //       "processSlot: %s \n\t drawing: %s \n\t tx: %s",
  //       processSlot.toString(),
  //       user.publicKey.toBase58(),
  //       tx
  //     );

  //     drawingResult = await program.account.drawingResult.fetch(
  //       drawingResultPda
  //     );

  //     const drawWinners = drawingResult.winners as OptionWinner[];
  //     const haveAllWinners =
  //       drawWinners
  //         .map((winner) => winner !== null)
  //         .filter((isHaveWinner) => isHaveWinner).length ===
  //       drawingResult.numberOfRewards;
  //     if (haveAllWinners) {
  //       break;
  //     }
  //   }

  //   drawingResult = await program.account.drawingResult.fetch(drawingResultPda);
  //   (drawingResult.winners as OptionWinner[])
  //     .map((winner) => winner.toBase58())
  //     .forEach((winner) => console.log("winner:", winner));

  //   console.log("\ntotalFees: %s SOL", totalFees * 0.000000001);
  //   console.log(
  //     "finished_timestamp: %s\n",
  //     drawingResult.finishedTimestamp.toString()
  //   );
  // });
});
