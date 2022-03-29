import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { createAssociatedTokenAccount, transfer } from "@solana/spl-token";
import {
  parseReserve,
  refreshObligationInstruction,
  refreshReserveInstruction,
  Reserve,
} from "@solendprotocol/solend-sdk";
import {
  loadSwitchboardProgram,
  OracleQueueAccount,
  PermissionAccount,
  ProgramStateAccount,
  VrfAccount,
} from "@switchboard-xyz/switchboard-v2";
import { assert } from "chai";
import { Solscatter } from "../target/types/solscatter";
import {
  MAIN_STATE_SEED,
  METADATA_SEED,
  PROGRAM_AUTHORITY_SEED,
  STATE_SEED,
} from "./constant";
import { isAccountAlreadyInitialize, loadKeypair } from "./utils";
import { createVrfAccount } from "./vrf";

type PdaAccount = {
  publicKey: anchor.web3.PublicKey;
  bump: number;
};

type ProgramAccount = {
  vrfKeypair: anchor.web3.Keypair;
  stateAccountPda: PdaAccount;
  mainStatePda: PdaAccount;
  metadataPda: PdaAccount;
  programAuthorityPda: PdaAccount;
  usdcReserve: Reserve;
  usdcReservePubkey: anchor.web3.PublicKey;
  programUsdcTokenAccountPubkey: anchor.web3.PublicKey;
  programCollateralTokenAccountPubkey: anchor.web3.PublicKey;
  obligationKeypair: anchor.web3.Keypair;
  lendingProgramPubkey: anchor.web3.PublicKey;
};

describe.only("solscatter spec", () => {
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.Solscatter as Program<Solscatter>;
  let programAccount: ProgramAccount;
  let userKeypairs: anchor.web3.Keypair[] = [];
  let obligationKeypair: anchor.web3.Keypair;

  before(async () => {
    const vrfKeypair = loadKeypair("./secrets/vrf-keypair.json");
    obligationKeypair = loadKeypair("./secrets/obligation-keypair.json");
    userKeypairs.push(loadKeypair("./secrets/first-user.json"));
    userKeypairs.push(loadKeypair("./secrets/second-user.json"));
    const [stateAccountPda, stateBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          Buffer.from(STATE_SEED),
          vrfKeypair.publicKey.toBuffer(),
          program.provider.wallet.publicKey.toBuffer(),
        ],
        program.programId
      );
    const [mainStatePda, mainStateBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(MAIN_STATE_SEED)],
        program.programId
      );

    const [metadataPda, metadataBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(METADATA_SEED)],
        program.programId
      );

    const [programAuthorityPda, progarmAuthorityBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(PROGRAM_AUTHORITY_SEED)],
        program.programId
      );

    const usdcReservePubkey = new anchor.web3.PublicKey(
      "FNNkz4RCQezSSS71rW2tvqZH1LCkTzaiG7Nd1LeA5x5y"
    );
    const reserveAccountInfo = await program.provider.connection.getAccountInfo(
      usdcReservePubkey
    );
    const usdcReserve = parseReserve(
      usdcReservePubkey,
      reserveAccountInfo
    ).info;

    const programUsdcTokenAccountPubkey =
      await anchor.utils.token.associatedAddress({
        mint: usdcReserve.liquidity.mintPubkey,
        owner: programAuthorityPda,
      });

    const programCollateralTokenAccountPubkey =
      await anchor.utils.token.associatedAddress({
        mint: usdcReserve.collateral.mintPubkey,
        owner: programAuthorityPda,
      });

    programAccount = {
      vrfKeypair: vrfKeypair,
      stateAccountPda: {
        publicKey: stateAccountPda,
        bump: stateBump,
      },
      mainStatePda: {
        publicKey: mainStatePda,
        bump: mainStateBump,
      },
      metadataPda: {
        publicKey: metadataPda,
        bump: metadataBump,
      },
      programAuthorityPda: {
        publicKey: programAuthorityPda,
        bump: progarmAuthorityBump,
      },
      usdcReserve: usdcReserve,
      usdcReservePubkey: usdcReservePubkey,
      programUsdcTokenAccountPubkey: programUsdcTokenAccountPubkey,
      programCollateralTokenAccountPubkey: programCollateralTokenAccountPubkey,
      obligationKeypair: obligationKeypair,
      lendingProgramPubkey: new anchor.web3.PublicKey(
        "ALend7Ketfx5bxh6ghsCDXAoDrhvEmsXT3cynB6aPLgx"
      ),
    };
  });

  it.skip("should create VrfAccount", async () => {
    const isVrfInitialize = await isAccountAlreadyInitialize(
      program.provider.connection,
      programAccount.vrfKeypair.publicKey
    );
    console.log("is vrf initialize yet? :", isVrfInitialize);

    if (isVrfInitialize) {
      return;
    }

    await createVrfAccount(program, programAccount.vrfKeypair);
  });

  it.skip("should initialize", async () => {
    const isProgramInitialize = await isAccountAlreadyInitialize(
      program.provider.connection,
      programAccount.mainStatePda.publicKey
    );

    console.log("is program initialize yet? :", isProgramInitialize);

    if (isProgramInitialize) {
      return;
    }

    const tx = await program.rpc.initialize({
      accounts: {
        mainState: programAccount.mainStatePda.publicKey,
        metadata: programAccount.metadataPda.publicKey,
        programAuthority: programAccount.programAuthorityPda.publicKey,
        usdcMint: programAccount.usdcReserve.liquidity.mintPubkey,
        usdcTokenAccount: programAccount.programUsdcTokenAccountPubkey,
        reserve: programAccount.usdcReservePubkey,
        reserveCollateralMint: programAccount.usdcReserve.collateral.mintPubkey,
        collateralTokenAccount:
          programAccount.programCollateralTokenAccountPubkey,
        obligation: programAccount.obligationKeypair.publicKey,
        lendingMarket: programAccount.usdcReserve.lendingMarket,
        lendingProgram: programAccount.lendingProgramPubkey,
        vrfClientState: programAccount.stateAccountPda.publicKey,
        vrfAccountInfo: programAccount.vrfKeypair.publicKey,
        signer: program.provider.wallet.publicKey,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
      },
      signers: [programAccount.obligationKeypair],
    });

    console.log("initialize tx:", tx);
  });

  it.skip("should user request faucet from cli", async () => {
    for (const userKeypair of userKeypairs) {
      const userTokenAccount = await anchor.utils.token.associatedAddress({
        mint: programAccount.usdcReserve.liquidity.mintPubkey,
        owner: userKeypair.publicKey,
      });

      if (
        !(await isAccountAlreadyInitialize(
          program.provider.connection,
          userTokenAccount
        ))
      ) {
        // create usdc associated token account
        await createAssociatedTokenAccount(
          program.provider.connection,
          userKeypair,
          programAccount.usdcReserve.liquidity.mintPubkey,
          userKeypair.publicKey
        );
      }

      const cliTokenAccount = await anchor.utils.token.associatedAddress({
        mint: programAccount.usdcReserve.liquidity.mintPubkey,
        owner: program.provider.wallet.publicKey,
      });

      if (
        !(await isAccountAlreadyInitialize(
          program.provider.connection,
          cliTokenAccount
        ))
      ) {
        assert.fail("cli wallet should have usdc in associated token account");
      }
      try {
        const transferTx = await transfer(
          program.provider.connection,
          program.provider.wallet["payer"],
          cliTokenAccount,
          userTokenAccount,
          program.provider.wallet.publicKey,
          3 * Math.pow(10, programAccount.usdcReserve.liquidity.mintDecimals)
        );

        console.log("transfer tx:", transferTx);
      } catch (e) {
        console.error(e);
      }
    }
  });

  it.skip("should deposit_initialize each user", async () => {
    for (const userKeypair of userKeypairs) {
      const mainState = await program.account.mainState.fetch(
        programAccount.mainStatePda.publicKey
      );
      const currentSlot = mainState.currentSlot;
      const processSlot = currentSlot.add(new anchor.BN(1));

      const [userDepositReferencePda] =
        await anchor.web3.PublicKey.findProgramAddress(
          [userKeypair.publicKey.toBuffer()],
          program.programId
        );

      const isThisUserAlreadyDepositInitialize =
        await isAccountAlreadyInitialize(
          program.provider.connection,
          userDepositReferencePda
        );

      if (isThisUserAlreadyDepositInitialize) {
        console.log(
          "%s already initialized -> skip",
          userKeypair.publicKey.toBase58()
        );
        continue;
      }

      const [userDepositPda] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(processSlot.toArray("le", 8))],
        program.programId
      );

      console.log(
        "%s not initialized yet -> initialize",
        userKeypair.publicKey
      );

      await program.rpc.depositInitialize({
        accounts: {
          userDeposit: userDepositPda,
          userDepositReference: userDepositReferencePda,
          mainState: programAccount.mainStatePda.publicKey,
          depositor: userKeypair.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
        signers: [userKeypair],
      });

      assert.isTrue(
        await isAccountAlreadyInitialize(
          program.provider.connection,
          userDepositReferencePda
        )
      );
      assert.isTrue(
        await isAccountAlreadyInitialize(
          program.provider.connection,
          userDepositPda
        )
      );
    }
  });

  it.skip("should deposit 1-3 USDC each user", async () => {
    const randomMin = 1;
    const randomMax = 3;
    const metadata = await program.account.metadata.fetch(
      programAccount.metadataPda.publicKey
    );

    for (const userKeypair of userKeypairs) {
      const [userDepositReferencePda] =
        await anchor.web3.PublicKey.findProgramAddress(
          [userKeypair.publicKey.toBuffer()],
          program.programId
        );

      const userDepositReference =
        await program.account.userDepositReference.fetch(
          userDepositReferencePda
        );

      const [userDepositPda] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from(userDepositReference.slot.toArray("le", 8))],
        program.programId
      );

      const [lendingMarketAuthority] =
        await anchor.web3.PublicKey.findProgramAddress(
          [metadata.lendingMarketAuthoritySeed.toBuffer()],
          programAccount.lendingProgramPubkey
        );

      const randomDepositAmount =
        Math.floor(Math.random() * randomMax) + randomMin;
      const userTokenAccount = await anchor.utils.token.associatedAddress({
        mint: programAccount.usdcReserve.liquidity.mintPubkey,
        owner: userKeypair.publicKey,
      });

      console.log(
        "user: %s -> deposit amount: %s",
        userKeypair.publicKey.toBase58(),
        randomDepositAmount
      );

      const depositParams = {
        uiAmount: randomDepositAmount,
        decimals: programAccount.usdcReserve.liquidity.mintDecimals,
      };
      const depositTx = await program.rpc.deposit(depositParams, {
        accounts: {
          userDeposit: userDepositPda,
          mainState: programAccount.mainStatePda.publicKey,
          metadata: programAccount.metadataPda.publicKey,
          programAuthority: programAccount.programAuthorityPda.publicKey,
          usdcMint: programAccount.usdcReserve.liquidity.mintPubkey,
          programUsdcTokenAccount: programAccount.programUsdcTokenAccountPubkey,
          userUsdcTokenAccount: userTokenAccount,
          programCollateralTokenAccount: metadata.collateralTokenAccount,
          reserve: metadata.reserve,
          reserveLiquiditySupply:
            programAccount.usdcReserve.liquidity.supplyPubkey,
          reserveCollateralMint:
            programAccount.usdcReserve.collateral.mintPubkey,
          lendingMarket: programAccount.usdcReserve.lendingMarket,
          lendingMarketAuthority: lendingMarketAuthority,
          destinationDepositCollateral:
            programAccount.usdcReserve.collateral.supplyPubkey,
          obligation: metadata.obligation,
          reserveLiquidityPythOracle:
            programAccount.usdcReserve.liquidity["pythOracle"],
          lendingProgram: programAccount.lendingProgramPubkey,
          reserveLiquiditySwitchboardOracle:
            programAccount.usdcReserve.liquidity["switchboardOracle"],
          owner: userKeypair.publicKey,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        },
        signers: [userKeypair],
      });

      console.log("deposit tx:", depositTx);
    }
  });

  it.skip("request randomness", async () => {
    const vrfSecret = loadKeypair("./secrets/vrf-keypair.json");
    const switchboardProgram = await loadSwitchboardProgram("devnet");

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
    const [programStateAccount, programStateBump] =
      ProgramStateAccount.fromSeed(switchboardProgram);
    const [permissionAccount, permissionBump] = PermissionAccount.fromSeed(
      switchboardProgram,
      queueAuthority,
      queueAccount.publicKey,
      vrfClientState.account.vrf
    );
    try {
      await permissionAccount.loadData();
    } catch {
      throw new Error(
        "A requested permission pda account has not been initialized."
      );
    }

    const switchTokenMint = await programStateAccount.getTokenMint();
    const payerTokenAccount =
      await switchTokenMint.getOrCreateAssociatedAccountInfo(
        program.provider.wallet.publicKey
      );
    const requestTxn = await program.rpc.requestRandomness(
      {
        clientStateBump: stateBump,
        permissionBump: permissionBump,
        switchboardStateBump: programStateBump,
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
        signers: [
          program.provider.wallet["payer"],
          program.provider.wallet["payer"],
        ],
      }
    );
    console.log(`https://solscan.io/tx/${requestTxn}?cluster=devnet`);
  });

  it.skip("should start_drawing_phase", async () => {
    const mainState = await program.account.mainState.fetch(
      programAccount.mainStatePda.publicKey
    );

    const metadata = await program.account.metadata.fetch(
      programAccount.metadataPda.publicKey
    );

    const [drawingResultPda] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("drawing_result"),
        Buffer.from(mainState.currentRound.toArray("le", 8)),
      ],
      program.programId
    );

    const [drawingPda] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("drawing_pda"),
        Buffer.from(mainState.currentRound.toArray("le", 8)),
      ],
      program.programId
    );

    const drawingRewardTokenAccount =
      await anchor.utils.token.associatedAddress({
        mint: metadata.usdcMint,
        owner: drawingPda,
      });

    const [lendingMarketAuthority] =
      await anchor.web3.PublicKey.findProgramAddress(
        [metadata.lendingMarketAuthoritySeed.toBuffer()],
        programAccount.lendingProgramPubkey
      );

    const numberOfRewards = 1;
    const randomNumbers = [
      new anchor.BN(
        Math.floor(Math.random() * mainState.totalDeposit.toNumber())
      ),
    ];

    const startDrawingPhaseTx = await program.rpc.startDrawingPhase(
      numberOfRewards,
      randomNumbers,
      {
        preInstructions: [
          refreshReserveInstruction(
            metadata.reserve,
            metadata.lendingProgram,
            programAccount.usdcReserve.liquidity["pythOracle"],
            programAccount.usdcReserve.liquidity["switchboardOracle"]
          ),
          refreshObligationInstruction(
            obligationKeypair.publicKey,
            [metadata.reserve],
            [],
            metadata.lendingProgram
          ),
        ],
        accounts: {
          state: programAccount.stateAccountPda.publicKey,
          drawingResult: drawingResultPda,
          mainState: programAccount.mainStatePda.publicKey,
          collateralMint: metadata.usdcMint,
          drawingPda: drawingPda,
          drawingRewardTokenAccount: drawingRewardTokenAccount,
          signer: program.provider.wallet.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,

          sourceCollateral: programAccount.usdcReserve.collateral.supplyPubkey,
          destinationCollateral: metadata.collateralTokenAccount,
          withdrawReserve: metadata.reserve,

          obligation: metadata.obligation,
          lendingMarket: programAccount.usdcReserve.lendingMarket,
          lendingMarketAuthority: lendingMarketAuthority,
          destinationLiquidity: drawingRewardTokenAccount,
          reserveCollateralMint:
            programAccount.usdcReserve.collateral.mintPubkey,
          reserveLiquiditySupply:
            programAccount.usdcReserve.liquidity.supplyPubkey,
          obligationOwner: metadata.programAuthority,
          transferAuthority: metadata.programAuthority,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
          tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
          solendProgramAddress: metadata.lendingProgram,
        },
      }
    );

    console.log("startDrawingPhaseTx:", startDrawingPhaseTx);
  });

  it("check randomness", async () => {
    const vrfClientState = (await program.account.vrfClientState.all())[0];
    console.log(vrfClientState.account);
    console.log(vrfClientState.account.lastTimestamp.toNumber());
  });

  it.skip("check winner", async () => {
    const mainState = await program.account.mainState.fetch(programAccount.mainStatePda.publicKey);
    const currentRound = mainState.currentRound.sub(new anchor.BN(1));
    const [drawingResultPda] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("drawing_result"),
        Buffer.from(currentRound.toArray("le", 8)),
      ],
      program.programId
    );
    const drawingResult = await program.account.drawingResult.fetch(
      drawingResultPda
    );

    console.log(drawingResult);
    console.log(drawingResult.winners[0].pubkey.toBase58());
  });

  it("drawing each user", async () => {
    const mainState = await program.account.mainState.fetch(programAccount.mainStatePda.publicKey);
    const currentSlot = mainState.currentSlot.toNumber();
    const currentRound = mainState.currentRound;
    const [drawingResultPda] = await anchor.web3.PublicKey.findProgramAddress(
      [
        Buffer.from("drawing_result"),
        Buffer.from(currentRound.toArray("le", 8)),
      ],
      program.programId
    );
    const drawingResult = await program.account.drawingResult.fetch(
      drawingResultPda
    );

    const lastProcessedSlot = drawingResult.lastProcessedSlot.toNumber();

    for (let processSlot = lastProcessedSlot + 1; processSlot <= currentSlot; processSlot++) {
      const [userDepositPda] = await anchor.web3.PublicKey.findProgramAddress(
        [Buffer.from((new anchor.BN(processSlot)).toArray("le", 8))],
        program.programId,
      );

      const drawingTx = await program.rpc.drawing({
        accounts: {
          mainState: programAccount.mainStatePda.publicKey,
          drawingResult: drawingResultPda,
          userDeposit: userDepositPda,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        }
      });

      console.log("processSlot:", processSlot);
      console.log("drawingTx:", drawingTx);
    }
  });
});
