import { initCusper } from '@miraplex/cusper';
// @ts-ignore
import { errorFromCode } from './generated';

export const cusper = initCusper(errorFromCode);
