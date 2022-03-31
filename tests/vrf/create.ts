import * as anchor from "@project-serum/anchor";
import { Callback, loadSwitchboardProgram, OracleQueueAccount, PermissionAccount, VrfAccount } from "@switchboard-xyz/switchboard-v2";
import { assert } from "chai";
import { Solscatter } from "../../target/types/solscatter";
import { DEVNET_CLUSTER, STATE_SEED, SWITCHBOARD_QUEUE_PUBKEY } from "../constant";

export async function createVrfAccount(
    program: anchor.Program<Solscatter>,
    vrfKeypair: anchor.web3.Keypair,
  ): Promise<VrfAccount> {
    const switchboardProgram = await loadSwitchboardProgram(DEVNET_CLUSTER);
    const vrfClientProgram = program;
    // B14WdxwY3LsipUJPLJfCXFBaJeM4y6GHJXB39B7oSUma
    const vrfSecret = vrfKeypair;
  
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
  
    try {
        await permissionAccount.loadData();
    } catch (e) {
        console.error("cannot load permissionAccount:", e.message);
    }

    return vrfAccount;
  }