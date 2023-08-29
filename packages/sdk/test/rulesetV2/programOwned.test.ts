import { Keypair } from '@solarti/web3.js';
import test from 'ava';
import {
  deserializeRuleV2,
  RuleTypeV2,
  serializeRuleV2,
  programOwnedV2,
} from '../../src/miraplex-token-auth-rules';
import { serializeString32 } from '../../src/ruleSetV2/helpers';

test('serialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const rule = programOwnedV2('myAccount', program);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0b000000' + // Rule type
      '40000000' + // Rule length
      program.toBuffer().toString('hex') + // PublicKey
      serializeString32('myAccount').toString('hex'), // Field
  );
});

test('deserialize', async (t) => {
  const program = Keypair.generate().publicKey;
  const hexBuffer =
    '0b000000' + // Rule type
    '40000000' + // Rule length
    program.toBuffer().toString('hex') + // PublicKey
    serializeString32('myAccount').toString('hex'); // Field
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.ProgramOwned,
    field: 'myAccount',
    program,
  });
});
