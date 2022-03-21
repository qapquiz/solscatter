import * as anchor from "@project-serum/anchor";

export async function createVrfAccount(argv: any): Promise<void> {
  const { payer, cluster, rpcUrl, queueKey, keypair, maxResult } = argv;

  console.log("CREATE VRF CALLED");
}
