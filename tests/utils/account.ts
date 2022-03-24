import * as anchor from "@project-serum/anchor";

export const isAccountAlreadyInitialize = async (
  connection: anchor.web3.Connection,
  publicKey: anchor.web3.PublicKey
): Promise<boolean> => {
  const account = await connection.getAccountInfo(publicKey);
  return account !== null;
};
