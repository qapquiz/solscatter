import * as anchor from "@project-serum/anchor";
import {Solscatter} from "../target/types/solscatter";
import {isAccountAlreadyInitialize, loadKeypair} from "./utils";
import {createVrfAccount} from "./vrf";
import {getMint} from "@solana/spl-token";
import {clusterApiUrl} from "@solana/web3.js";
import {DEVNET_CLUSTER} from "./constant";
import {web3} from "@project-serum/anchor";

describe("solscatter specs", () => {
    anchor.setProvider(anchor.Provider.env());

    const program = anchor.workspace.Solscatter as anchor.Program<Solscatter>;
    const connection = new anchor.web3.Connection(clusterApiUrl(DEVNET_CLUSTER), "processed");
    const yiUnderlyingMint = new anchor.web3.PublicKey("5fjG31cbSszE6FodW37UJnNzgVTyqg5WHWGCmL3ayAvA");
    const yiMint = new anchor.web3.PublicKey("6XyygxFmUeemaTvA9E9mhH9FvgpynZqARVyG3gUdCMt7");
    const quarryProgram = new anchor.web3.PublicKey("QMNeHCGYnLVDn1icRAfQZpjPLBNkfGbSKRB83G5d8KB");
    const quarry = new anchor.web3.PublicKey("5pE3ch5haoHp7wYKdsbKJwrNZzeuXr5nCj6Nn2JQf4JS");
    const rewarder = new anchor.web3.PublicKey("57fEKyj9C7dhdcrFCXFGQZp68CmjfVS4sgNecy2PdnTC");
    const yiProgram = new anchor.web3.PublicKey("YiiTopEnX2vyoWdXuG45ovDFYZars4XZ4w6td6RVTFm");
    const yiToken = new anchor.web3.PublicKey("8yazwmgc66uKrDBy3TZpNCgLa8qUDcuH8PZCz9jy6dzd");
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
        const [metadata] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from("metadata")], program.programId);
        const isProgramInitialize = await isAccountAlreadyInitialize(
            program.provider.connection,
            metadata,
        );

        if (isProgramInitialize) {
            return;
        }

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
                yiProgram,
                yiToken,
                yiUnderlyingMint,
                yiMint,
                yiMintTokenAccount,
                yiUnderlyingTokenAccount,
                quarryProgram,
                quarry,
                rewarder,
                minerVault,
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

    it("should create deposit initialize", async () => {
        const [mainStatePDA] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from("main_state")], program.programId)
        const mainState = await program.account.mainState.fetch(mainStatePDA)
        const newSlot = mainState.currentSlot.add(new anchor.BN(1));

        const depositInitializeTx = await program
            .methods
            .depositInitialize(newSlot)
            .accounts({
                depositor: program.provider.wallet.publicKey
            })
            .rpc()

        console.log("depositInitialize tx:", depositInitializeTx)
    });

    it("deposit", async () => {
        const [metadataPDA] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from("metadata")], program.programId)
        const metadata = await program.account.metadata.fetch(metadataPDA)
        const underlyingMintInfo = await getMint(connection, metadata.yiUnderlyingMint);
        const depositorUnderlyingTokenAccount = await anchor.utils.token.associatedAddress({
            mint: underlyingMintInfo.address,
            owner: program.provider.wallet.publicKey,
        });
        const [userDepositReferencePubKey] = await anchor.web3.PublicKey.findProgramAddress([program.provider.wallet.publicKey.toBuffer()], program.programId)
        const userDepositReference = await program.account.userDepositReference.fetch(userDepositReferencePubKey)
        const [userDepositPDA] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from(userDepositReference.slot.toArray("le", 8))], program.programId)
        const yiUnderlyingTokenAccount = await anchor.utils.token.associatedAddress({
            mint: underlyingMintInfo.address,
            owner: metadata.yiToken,
        });

        const deposit = await program
            .methods
            .deposit({
                uiAmount: 1,
                decimals: underlyingMintInfo.decimals
            })
            .accounts({
                depositor: program.provider.wallet.publicKey,
                depositorYiUnderlyingTokenAccount: depositorUnderlyingTokenAccount,
                userDeposit: userDepositPDA,
                yiProgram: metadata.yiProgram,
                yiToken: metadata.yiToken,
                yiUnderlyingMint: metadata.yiUnderlyingMint,
                yiMint: metadata.yiMint,
                yiUnderlyingTokenAccount: yiUnderlyingTokenAccount,
                platformYiUnderlyingTokenAccount: metadata.yiUnderlyingTokenAccount,
                platformYiTokenAccount: metadata.yiMintTokenAccount,
                quarryProgram: metadata.quarryProgram,
                miner: metadata.quarryMiner,
                quarry: metadata.quarry,
                rewarder: metadata.quarryRewarder,
                minerVault: metadata.quarryMinerVault,
                clock: web3.SYSVAR_CLOCK_PUBKEY
            })
            .rpc()

        console.log("deposit tx: ", deposit)
    });

    it("withdraw", async () => {
        const [metadataPDA] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from("metadata")], program.programId)
        const metadata = await program.account.metadata.fetch(metadataPDA)
        const underlyingMintInfo = await getMint(connection, metadata.yiUnderlyingMint);
        const depositorUnderlyingTokenAccount = await anchor.utils.token.associatedAddress({
            mint: underlyingMintInfo.address,
            owner: program.provider.wallet.publicKey,
        });
        const [userDepositReferencePubKey] = await anchor.web3.PublicKey.findProgramAddress([program.provider.wallet.publicKey.toBuffer()], program.programId)
        const userDepositReference = await program.account.userDepositReference.fetch(userDepositReferencePubKey)
        const [userDepositPDA] = await anchor.web3.PublicKey.findProgramAddress([Buffer.from(userDepositReference.slot.toArray("le", 8))], program.programId)
        const yiUnderlyingTokenAccount = await anchor.utils.token.associatedAddress({
            mint: underlyingMintInfo.address,
            owner: metadata.yiToken,
        });

        const withdrawTx = await program
            .methods
            .withdraw({
                uiAmount: 1,
                decimals: underlyingMintInfo.decimals
            })
            .accounts({
                depositor: program.provider.wallet.publicKey,
                depositorYiUnderlyingTokenAccount: depositorUnderlyingTokenAccount,
                userDeposit: userDepositPDA,
                yiProgram: metadata.yiProgram,
                yiToken: metadata.yiToken,
                yiUnderlyingMint: metadata.yiUnderlyingMint,
                yiMint: metadata.yiMint,
                yiUnderlyingTokenAccount: yiUnderlyingTokenAccount,
                platformYiUnderlyingTokenAccount: metadata.yiUnderlyingTokenAccount,
                platformYiTokenAccount: metadata.yiMintTokenAccount,
                quarryProgram: metadata.quarryProgram,
                miner: metadata.quarryMiner,
                quarry: metadata.quarry,
                rewarder: metadata.quarryRewarder,
                minerVault: metadata.quarryMinerVault,
                clock: web3.SYSVAR_CLOCK_PUBKEY
            })
            .rpc()

        console.log("withdraw tx: ", withdrawTx)
    });
});