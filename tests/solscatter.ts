import * as anchor from "@project-serum/anchor";
import {Program} from "@project-serum/anchor";
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    getAssociatedTokenAddress,
    getMint,
    getOrCreateAssociatedTokenAccount,
    TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
    Callback,
    loadSwitchboardProgram,
    OracleQueueAccount,
    PermissionAccount,
    ProgramStateAccount,
    VrfAccount,
} from "@switchboard-xyz/switchboard-v2";
import {Solscatter} from "../target/types/solscatter";
import {loadKeypair} from "./utils";
import {clusterApiUrl} from "@solana/web3.js";
import {parseReserve} from "@solendprotocol/solend-sdk";
import {assert} from "chai";

type OptionWinner = anchor.web3.PublicKey | null;

const STATE_SEED = "STATE";
const DEVNET_CLUSTER = "devnet";
const SWITCHBOARD_QUEUE_PUBKEY = new anchor.web3.PublicKey(
    "F8ce7MsckeZAbAGmxjJNetxYXQa9mKr9nnrC3qKubyYy"
);

const YI_PROGRAM_ADDRESS = new anchor.web3.PublicKey("YiiTopEnX2vyoWdXuG45ovDFYZars4XZ4w6td6RVTFm");
const YI_MINT = new anchor.web3.PublicKey("6XyygxFmUeemaTvA9E9mhH9FvgpynZqARVyG3gUdCMt7");
const YI_UNDERLYING_MINT = new anchor.web3.PublicKey("5fjG31cbSszE6FodW37UJnNzgVTyqg5WHWGCmL3ayAvA");
const SOL_UST_AUTHORITY = new anchor.web3.PublicKey("8yazwmgc66uKrDBy3TZpNCgLa8qUDcuH8PZCz9jy6dzd");

async function createVrfAccount(
  program: anchor.Program<Solscatter>
): Promise<void> {
  const switchboardProgram = await loadSwitchboardProgram(DEVNET_CLUSTER);
  const vrfClientProgram = program;
  // B14WdxwY3LsipUJPLJfCXFBaJeM4y6GHJXB39B7oSUma
  const vrfSecret = loadKeypair("./tests/secrets/vrf-keypair.json");

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
    const connection = new anchor.web3.Connection(clusterApiUrl(DEVNET_CLUSTER), "processed");

    const users = [
        loadKeypair("./tests/secrets/user1.json"),
        loadKeypair("./tests/secrets/user2.json"),
        loadKeypair("./tests/secrets/user3.json"),
    ];

    const usdc_mint = new anchor.web3.PublicKey("zVzi5VAf4qMEwzv7NXECVx5v2pQ7xnqVVjCXZwS9XzA");
    const reservePubKey = new anchor.web3.PublicKey("FNNkz4RCQezSSS71rW2tvqZH1LCkTzaiG7Nd1LeA5x5y");
    const lendingProgram = new anchor.web3.PublicKey("ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx");

    // it.only("Create Vrf", async () => {
    //   await createVrfAccount(program);
    // });

    // it("Airdrop to users", async () => {
    //     const promises = users.map((user) => {
    //         return program.provider.connection.requestAirdrop(
    //             user.publicKey,
    //             2 * anchor.web3.LAMPORTS_PER_SOL
    //         );
    //     });
    //
    //     const txs = await Promise.all(promises);
    //
    //     await Promise.all(
    //         txs.map((tx) => program.provider.connection.confirmTransaction(tx))
    //     );
    // });

    it("Is initialized!", async () => {
        return
        const vrfSecret = loadKeypair("./tests/secrets/vrf-keypair.json");
        const [stateAccountPda, stateBump] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from(STATE_SEED), vrfSecret.publicKey.toBuffer(), program.provider.wallet.publicKey.toBuffer(),], program.programId);
        const [mainState] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from("main_state")], program.programId);
        const [metadata] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from("metadata")], program.programId);
        const [programAuthority] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from("program_authority")], program.programId)
        const [usdcTokenAccount] = await anchor.web3.PublicKey.findProgramAddress([programAuthority.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), usdc_mint.toBuffer()], ASSOCIATED_TOKEN_PROGRAM_ID)
        const obligation = anchor.web3.Keypair.generate()
        const reserveAccountInfo = await program.provider.connection.getAccountInfo(reservePubKey)
        const reserve = parseReserve(reservePubKey, reserveAccountInfo).info
        const collateral = await getOrCreateAssociatedTokenAccount(
            program.provider.connection,
            program.provider.wallet.payer,
            reserve.collateral.mintPubkey,
            programAuthority,
            true
        );

        console.log("programAuthority : ", programAuthority.toString())

        const tx = await program.rpc.initialize({
            accounts: {
                mainState: mainState,
                metadata: metadata,
                programAuthority: programAuthority,
                usdcMint: usdc_mint,
                usdcTokenAccount: usdcTokenAccount,
                reserve: reservePubKey,
                collateral: collateral.address,
                obligation: obligation.publicKey,
                lendingMarket: reserve.lendingMarket,
                lendingProgram: lendingProgram,
                vrfClientState: stateAccountPda,
                vrfAccountInfo: vrfSecret.publicKey,
                signer: program.provider.wallet.publicKey,
                clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                rent: anchor.web3.SYSVAR_RENT_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                systemProgram: anchor.web3.SystemProgram.programId,
            },
            signers: [obligation]
        });

        console.log("Your transaction signature", tx);
    });

    it("deposit initialize each user", async () => {
        return
        const mainState = (await program.account.mainState.all())[0];
        let currentSlot = mainState.account.currentSlot;

        for (let user of users) {
            currentSlot = currentSlot.add(new anchor.BN(1));

            const [userDepositReference] = await anchor.web3.PublicKey.findProgramAddress([user.publicKey.toBuffer()], program.programId)
            const [userDeposit] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from(currentSlot.toArray("le", 8))], program.programId);

            await program.rpc.depositInitialize({
                accounts: {
                    userDeposit: userDeposit,
                    userDepositReference: userDepositReference,
                    mainState: mainState.publicKey,
                    depositor: user.publicKey,
                    systemProgram: anchor.web3.SystemProgram.programId,
                },
                signers: [user],
            });
        }
    });

    it("deposit each user", async () => {
        return
        const mainState = (await program.account.mainState.all())[0];
        const metadata = (await program.account.metadata.all())[0];
        const usdcMint = await getMint(connection, metadata.account.usdcMint)
        const reserveAccountInfo = await program.provider.connection.getAccountInfo(metadata.account.reserve)
        const reserve = parseReserve(metadata.account.reserve, reserveAccountInfo).info
        const [lendingMarketAuthority] = await anchor.web3.PublicKey.findProgramAddress([metadata.account.lendingMarketAuthoritySeed.toBuffer()], lendingProgram)

        console.log("before deposit : ", mainState.account.totalDeposit.toNumber())

        for (let user of users) {
            const [userDepositReferencePubKey] = await anchor.web3.PublicKey.findProgramAddress([user.publicKey.toBuffer()], program.programId)
            const userDepositReference = await program.account.userDepositReference.fetch(userDepositReferencePubKey);
            const [userDeposit] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from(userDepositReference.slot.toArray("le", 8))], program.programId);
            const userUSDCTokenAccount = await getAssociatedTokenAddress(metadata.account.usdcMint, user.publicKey);
            const randomAmountBetween1To5 = Math.ceil(Math.random() * 5);
            const depositParams = {
                uiAmount: randomAmountBetween1To5,
                decimals: usdcMint.decimals
            }

            console.log("user: %s ->  deposit amount: %s", user.publicKey.toBase58(), randomAmountBetween1To5);

            const tx = await program.rpc.deposit(
                depositParams,
                {
                    accounts: {
                        userDeposit: userDeposit,
                        mainState: mainState.publicKey,
                        metadata: metadata.publicKey,
                        programAuthority: metadata.account.programAuthority,
                        usdcMint: usdcMint.address,
                        programUsdcTokenAccount: metadata.account.usdcTokenAccount,
                        userUsdcTokenAccount: userUSDCTokenAccount,
                        collateral: metadata.account.collateral,
                        reserve: metadata.account.reserve,
                        reserveLiquiditySupply: reserve.liquidity.supplyPubkey,
                        reserveCollateralMint: reserve.collateral.mintPubkey,
                        lendingMarket: reserve.lendingMarket,
                        lendingMarketAuthority: lendingMarketAuthority,
                        destinationDepositCollateral: reserve.collateral.supplyPubkey,
                        obligation: metadata.account.obligation,
                        reserveLiquidityPythOracle: reserve.liquidity.pythOracle,
                        lendingProgram: metadata.account.lendingProgram,
                        reserveLiquiditySwitchboardOracle: reserve.liquidity.switchboardOracle,
                        owner: user.publicKey,
                        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                        tokenProgram: TOKEN_PROGRAM_ID,
                    },
                    signers: [user],
                });

            const updatedMainState = (await program.account.mainState.all())[0]
            console.log("deposit signature: %s -> after deposit : %s", tx, updatedMainState.account.totalDeposit.toNumber())
        }
    });

    it("withdraw each user", async () => {
        const mainState = (await program.account.mainState.all())[0];
        const metadata = (await program.account.metadata.all())[0];
        const usdcMint = await getMint(connection, metadata.account.usdcMint)
        const reserveAccountInfo = await program.provider.connection.getAccountInfo(metadata.account.reserve)
        const reserve = parseReserve(metadata.account.reserve, reserveAccountInfo).info
        const [lendingMarketAuthority] = await anchor.web3.PublicKey.findProgramAddress([metadata.account.lendingMarketAuthoritySeed.toBuffer()], lendingProgram)

        console.log("before withdraw : ", mainState.account.totalDeposit.toNumber())

        for (let user of users) {
            const [userDepositReferencePubKey] = await anchor.web3.PublicKey.findProgramAddress([user.publicKey.toBuffer()], program.programId)
            const userDepositReference = await program.account.userDepositReference.fetch(userDepositReferencePubKey);
            const [userDeposit] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from(userDepositReference.slot.toArray("le", 8))], program.programId);
            const userUSDCTokenAccount = await getAssociatedTokenAddress(metadata.account.usdcMint, user.publicKey);
            const withdrawParams = {
                uiAmount: 1,
                decimals: usdcMint.decimals
            }

            const tx = await program.rpc.withdraw(
                withdrawParams,
                {
                    accounts: {
                        userDeposit: userDeposit,
                        mainState: mainState.publicKey,
                        metadata: metadata.publicKey,
                        programAuthority: metadata.account.programAuthority,
                        usdcMint: usdcMint.address,
                        programUsdcTokenAccount: metadata.account.usdcTokenAccount,
                        userUsdcTokenAccount: userUSDCTokenAccount,
                        collateral: metadata.account.collateral,
                        reserve: metadata.account.reserve,
                        obligation: metadata.account.obligation,
                        lendingMarket: reserve.lendingMarket,
                        lendingMarketAuthority: lendingMarketAuthority,
                        reserveCollateralMint: reserve.collateral.mintPubkey,
                        reserveCollateralSupply: reserve.collateral.supplyPubkey,
                        reserveLiquiditySupply: reserve.liquidity.supplyPubkey,
                        reserveLiquidityPythOracle: reserve.liquidity.pythOracle,
                        reserveLiquiditySwitchboardOracle: reserve.liquidity.switchboardOracle,
                        lendingProgram: metadata.account.lendingProgram,
                        owner: user.publicKey,
                        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
                        tokenProgram: TOKEN_PROGRAM_ID,
                    },
                    signers: [user]
                });

            const updatedMainState = (await program.account.mainState.all())[0]
            console.log("withdraw signature: %s -> after withdraw : %s", tx, updatedMainState.account.totalDeposit.toNumber())
        }
    });

    // it("request randomness", async () => {
    //     const vrfSecret = loadKeypair("./secrets/vrf-keypair.json");
    //     const switchboardProgram = await loadSwitchboardProgram(DEVNET_CLUSTER);
    //
    //     const [stateAccountPda, stateBump] =
    //         await anchor.web3.PublicKey.findProgramAddress(
    //             [
    //                 Buffer.from(STATE_SEED),
    //                 vrfSecret.publicKey.toBuffer(),
    //                 program.provider.wallet.publicKey.toBuffer(),
    //             ],
    //             program.programId
    //         );
    //
    //     const vrfAccount = new VrfAccount({
    //         program: switchboardProgram,
    //         publicKey: vrfSecret.publicKey,
    //     });
    //
    //     const vrfClientState = (await program.account.vrfClientState.all())[0];
    //     const vrf = await vrfAccount.loadData();
    //     const queueAccount = new OracleQueueAccount({
    //         program: switchboardProgram,
    //         publicKey: vrf.oracleQueue,
    //     });
    //     const queue = await queueAccount.loadData();
    //     const queueAuthority = queue.authority;
    //     const dataBuffer = queue.dataBuffer;
    //     const escrow = vrf.escrow;
    //     const [programStateAccount, programStateBump] = ProgramStateAccount.fromSeed(switchboardProgram);
    //     const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
    //         switchboardProgram,
    //         queueAuthority,
    //         queueAccount.publicKey,
    //         vrfClientState.account.vrf,
    //     );
    //     try {
    //         await permissionAccount.loadData();
    //     } catch {
    //         throw new Error(
    //             "A requested permission pda account has not been initialized."
    //         );
    //     }
    //
    //     const switchTokenMint = await programStateAccount.getTokenMint();
    //     const payerTokenAccount = await switchTokenMint.getOrCreateAssociatedAccountInfo(program.provider.wallet.publicKey);
    //     console.log(payerTokenAccount.address.toBase58());
    //
    //     console.log(program.provider.wallet["payer"])
    //     const requestTxn = await program.rpc.requestRandomness({
    //             clientStateBump: stateBump,
    //             permissionBump: permissionBump,
    //             switchboardStateBump: programStateBump
    //         },
    //         {
    //             accounts: {
    //                 state: vrfClientState.publicKey,
    //                 authority: program.provider.wallet.publicKey,
    //                 switchboardProgram: switchboardProgram.programId,
    //                 vrf: vrfClientState.account.vrf,
    //                 oracleQueue: queueAccount.publicKey,
    //                 queueAuthority,
    //                 dataBuffer,
    //                 permission: permissionAccount.publicKey,
    //                 escrow,
    //                 payerWallet: payerTokenAccount.address,
    //                 payerAuthority: program.provider.wallet.publicKey,
    //                 recentBlockhashes: anchor.web3.SYSVAR_RECENT_BLOCKHASHES_PUBKEY,
    //                 programState: programStateAccount.publicKey,
    //                 tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
    //             },
    //             signers: [program.provider.wallet["payer"], program.provider.wallet["payer"]],
    //         });
    //     console.log(`https://solscan.io/tx/${requestTxn}?cluster=devnet`);
    // });

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
