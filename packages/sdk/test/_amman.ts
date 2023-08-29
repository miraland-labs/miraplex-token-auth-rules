import { Amman } from '@miraplex/amman-client';
import { PROGRAM_ADDRESS } from '../src/generated';

export const amman = Amman.instance({
  knownLabels: { [PROGRAM_ADDRESS]: 'Token Auth Rules' },
});
