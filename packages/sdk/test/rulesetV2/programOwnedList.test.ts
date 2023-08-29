import { Keypair, PublicKey } from '@solarti/web3.js';
import test from 'ava';
import {
  deserializeRuleV2,
  RuleTypeV2,
  serializeRuleV2,
  programOwnedListV2,
} from '../../src/miraplex-token-auth-rules';
import { serializeString32 } from '../../src/ruleSetV2/helpers';

test('serialize', async (t) => {
  const programA = Keypair.generate().publicKey;
  const programB = Keypair.generate().publicKey;
  const programs: PublicKey[] = [programA, programB];
  const rule = programOwnedListV2('myAccount', programs);
  const serializedRule = serializeRuleV2(rule).toString('hex');
  t.is(
    serializedRule,
    '0c000000' + // Rule type
      '60000000' + // Rule length
      serializeString32('myAccount').toString('hex') + // Field
      programA.toBuffer().toString('hex') + // Program A
      programB.toBuffer().toString('hex'), // Program B
  );
});

test('deserialize', async (t) => {
  const programA = Keypair.generate().publicKey;
  const programB = Keypair.generate().publicKey;
  const programs: PublicKey[] = [programA, programB];
  const hexBuffer =
    '0c000000' + // Rule type
    '60000000' + // Rule length
    serializeString32('myAccount').toString('hex') + // Field
    programA.toBuffer().toString('hex') + // Program A
    programB.toBuffer().toString('hex'); // Program B
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.ProgramOwnedList,
    field: 'myAccount',
    programs,
  });
});
