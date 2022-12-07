/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

type ErrorWithCode = Error & { code: number };
type MaybeErrorWithCode = ErrorWithCode | null | undefined;

const createErrorFromCodeLookup: Map<number, () => ErrorWithCode> = new Map();
const createErrorFromNameLookup: Map<string, () => ErrorWithCode> = new Map();

/**
 * NumericalOverflow: 'Numerical Overflow'
 *
 * @category Errors
 * @category generated
 */
export class NumericalOverflowError extends Error {
  readonly code: number = 0x0;
  readonly name: string = 'NumericalOverflow';
  constructor() {
    super('Numerical Overflow');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NumericalOverflowError);
    }
  }
}

createErrorFromCodeLookup.set(0x0, () => new NumericalOverflowError());
createErrorFromNameLookup.set('NumericalOverflow', () => new NumericalOverflowError());

/**
 * DataTypeMismatch: 'Data type mismatch'
 *
 * @category Errors
 * @category generated
 */
export class DataTypeMismatchError extends Error {
  readonly code: number = 0x1;
  readonly name: string = 'DataTypeMismatch';
  constructor() {
    super('Data type mismatch');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DataTypeMismatchError);
    }
  }
}

createErrorFromCodeLookup.set(0x1, () => new DataTypeMismatchError());
createErrorFromNameLookup.set('DataTypeMismatch', () => new DataTypeMismatchError());

/**
 * IncorrectOwner: 'Incorrect account owner'
 *
 * @category Errors
 * @category generated
 */
export class IncorrectOwnerError extends Error {
  readonly code: number = 0x2;
  readonly name: string = 'IncorrectOwner';
  constructor() {
    super('Incorrect account owner');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, IncorrectOwnerError);
    }
  }
}

createErrorFromCodeLookup.set(0x2, () => new IncorrectOwnerError());
createErrorFromNameLookup.set('IncorrectOwner', () => new IncorrectOwnerError());

/**
 * PayloadVecIndexError: 'Could not index into PayloadVec'
 *
 * @category Errors
 * @category generated
 */
export class PayloadVecIndexErrorError extends Error {
  readonly code: number = 0x3;
  readonly name: string = 'PayloadVecIndexError';
  constructor() {
    super('Could not index into PayloadVec');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PayloadVecIndexErrorError);
    }
  }
}

createErrorFromCodeLookup.set(0x3, () => new PayloadVecIndexErrorError());
createErrorFromNameLookup.set('PayloadVecIndexError', () => new PayloadVecIndexErrorError());

/**
 * DerivedKeyInvalid: 'Derived key invalid'
 *
 * @category Errors
 * @category generated
 */
export class DerivedKeyInvalidError extends Error {
  readonly code: number = 0x4;
  readonly name: string = 'DerivedKeyInvalid';
  constructor() {
    super('Derived key invalid');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DerivedKeyInvalidError);
    }
  }
}

createErrorFromCodeLookup.set(0x4, () => new DerivedKeyInvalidError());
createErrorFromNameLookup.set('DerivedKeyInvalid', () => new DerivedKeyInvalidError());

/**
 * AdditionalSignerCheckFailed: 'Additional Signer check failed'
 *
 * @category Errors
 * @category generated
 */
export class AdditionalSignerCheckFailedError extends Error {
  readonly code: number = 0x5;
  readonly name: string = 'AdditionalSignerCheckFailed';
  constructor() {
    super('Additional Signer check failed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, AdditionalSignerCheckFailedError);
    }
  }
}

createErrorFromCodeLookup.set(0x5, () => new AdditionalSignerCheckFailedError());
createErrorFromNameLookup.set(
  'AdditionalSignerCheckFailed',
  () => new AdditionalSignerCheckFailedError(),
);

/**
 * PubkeyMatchCheckFailed: 'Pubkey Match check failed'
 *
 * @category Errors
 * @category generated
 */
export class PubkeyMatchCheckFailedError extends Error {
  readonly code: number = 0x6;
  readonly name: string = 'PubkeyMatchCheckFailed';
  constructor() {
    super('Pubkey Match check failed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PubkeyMatchCheckFailedError);
    }
  }
}

createErrorFromCodeLookup.set(0x6, () => new PubkeyMatchCheckFailedError());
createErrorFromNameLookup.set('PubkeyMatchCheckFailed', () => new PubkeyMatchCheckFailedError());

/**
 * DerivedKeyMatchCheckFailed: 'Derived Key Match check failed'
 *
 * @category Errors
 * @category generated
 */
export class DerivedKeyMatchCheckFailedError extends Error {
  readonly code: number = 0x7;
  readonly name: string = 'DerivedKeyMatchCheckFailed';
  constructor() {
    super('Derived Key Match check failed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DerivedKeyMatchCheckFailedError);
    }
  }
}

createErrorFromCodeLookup.set(0x7, () => new DerivedKeyMatchCheckFailedError());
createErrorFromNameLookup.set(
  'DerivedKeyMatchCheckFailed',
  () => new DerivedKeyMatchCheckFailedError(),
);

/**
 * ProgramOwnedCheckFailed: 'Program Owned check failed'
 *
 * @category Errors
 * @category generated
 */
export class ProgramOwnedCheckFailedError extends Error {
  readonly code: number = 0x8;
  readonly name: string = 'ProgramOwnedCheckFailed';
  constructor() {
    super('Program Owned check failed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, ProgramOwnedCheckFailedError);
    }
  }
}

createErrorFromCodeLookup.set(0x8, () => new ProgramOwnedCheckFailedError());
createErrorFromNameLookup.set('ProgramOwnedCheckFailed', () => new ProgramOwnedCheckFailedError());

/**
 * AmountCheckFailed: 'Amount checked failed'
 *
 * @category Errors
 * @category generated
 */
export class AmountCheckFailedError extends Error {
  readonly code: number = 0x9;
  readonly name: string = 'AmountCheckFailed';
  constructor() {
    super('Amount checked failed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, AmountCheckFailedError);
    }
  }
}

createErrorFromCodeLookup.set(0x9, () => new AmountCheckFailedError());
createErrorFromNameLookup.set('AmountCheckFailed', () => new AmountCheckFailedError());

/**
 * FrequencyCheckFailed: 'Frequency check failed'
 *
 * @category Errors
 * @category generated
 */
export class FrequencyCheckFailedError extends Error {
  readonly code: number = 0xa;
  readonly name: string = 'FrequencyCheckFailed';
  constructor() {
    super('Frequency check failed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, FrequencyCheckFailedError);
    }
  }
}

createErrorFromCodeLookup.set(0xa, () => new FrequencyCheckFailedError());
createErrorFromNameLookup.set('FrequencyCheckFailed', () => new FrequencyCheckFailedError());

/**
 * PubkeyTreeMatchCheckFailed: 'Pubkey Tree Match check failed'
 *
 * @category Errors
 * @category generated
 */
export class PubkeyTreeMatchCheckFailedError extends Error {
  readonly code: number = 0xb;
  readonly name: string = 'PubkeyTreeMatchCheckFailed';
  constructor() {
    super('Pubkey Tree Match check failed');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PubkeyTreeMatchCheckFailedError);
    }
  }
}

createErrorFromCodeLookup.set(0xb, () => new PubkeyTreeMatchCheckFailedError());
createErrorFromNameLookup.set(
  'PubkeyTreeMatchCheckFailed',
  () => new PubkeyTreeMatchCheckFailedError(),
);

/**
 * PayerIsNotSigner: 'Payer is not a signer'
 *
 * @category Errors
 * @category generated
 */
export class PayerIsNotSignerError extends Error {
  readonly code: number = 0xc;
  readonly name: string = 'PayerIsNotSigner';
  constructor() {
    super('Payer is not a signer');
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PayerIsNotSignerError);
    }
  }
}

createErrorFromCodeLookup.set(0xc, () => new PayerIsNotSignerError());
createErrorFromNameLookup.set('PayerIsNotSigner', () => new PayerIsNotSignerError());

/**
 * Attempts to resolve a custom program error from the provided error code.
 * @category Errors
 * @category generated
 */
export function errorFromCode(code: number): MaybeErrorWithCode {
  const createError = createErrorFromCodeLookup.get(code);
  return createError != null ? createError() : null;
}

/**
 * Attempts to resolve a custom program error from the provided error name, i.e. 'Unauthorized'.
 * @category Errors
 * @category generated
 */
export function errorFromName(name: string): MaybeErrorWithCode {
  const createError = createErrorFromNameLookup.get(name);
  return createError != null ? createError() : null;
}
