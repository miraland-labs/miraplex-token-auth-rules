import { PublicKey } from '@solarti/web3.js';
import { PREFIX, PROGRAM_ID } from './miraplex-token-auth-rules';

export const findRuleSetPDA = async (payer: PublicKey, name: string) => {
  return await PublicKey.findProgramAddress(
    [Buffer.from(PREFIX), payer.toBuffer(), Buffer.from(name)],
    PROGRAM_ID,
  );
};

export const findRuleSetBufferPDA = async (payer: PublicKey) => {
  return await PublicKey.findProgramAddress([Buffer.from(PREFIX), payer.toBuffer()], PROGRAM_ID);
};
