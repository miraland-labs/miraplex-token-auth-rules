import * as beetMiraland from '@miraplex/beet-miraland';
import { PublicKey } from '@solarti/web3.js';

export const serializeString32 = (str: string): Buffer => {
  const buffer = Buffer.alloc(32);
  buffer.write(str);
  return buffer;
};

export const deserializeString32 = (buffer: Buffer, offset = 0): string => {
  return buffer
    .subarray(offset, offset + 32)
    .toString()
    .replace(/\u0000/g, '');
};

export const serializePublicKey = (publicKey: PublicKey): Buffer => {
  const buffer = Buffer.alloc(32);
  beetMiraland.publicKey.write(buffer, 0, publicKey);
  return buffer;
};

export const deserializePublicKey = (buffer: Buffer, offset = 0): PublicKey => {
  return beetMiraland.publicKey.read(buffer, offset);
};
