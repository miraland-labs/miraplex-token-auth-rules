import { Keypair } from '@solarti/web3.js';
import test from 'ava';
import {
  additionalSignerV2,
  deserializeRuleV2,
  notV2,
  RuleTypeV2,
  serializeRuleV2,
} from '../../src/miraplex-token-auth-rules';

test('serialize', async (t) => {
  const publicKey = Keypair.generate().publicKey;
  const rule = notV2(additionalSignerV2(publicKey));
  const serializedRule = serializeRuleV2(rule).toString('hex');

  const expectedChildRule = '0100000020000000' + publicKey.toBuffer().toString('hex');
  t.is(
    serializedRule,
    '08000000' + // Rule type (8)
      '28000000' + // Rule length (8 + 32)
      expectedChildRule,
  );
});

test('deserialize', async (t) => {
  const publicKey = Keypair.generate().publicKey;
  const childRule = '0100000020000000' + publicKey.toBuffer().toString('hex');
  const hexBuffer =
    '08000000' + // Rule type (8)
    '28000000' + // Rule length (8 + 32)
    childRule;
  const buffer = Buffer.from(hexBuffer, 'hex');
  const rule = deserializeRuleV2(buffer);
  t.deepEqual(rule, {
    type: RuleTypeV2.Not,
    rule: additionalSignerV2(publicKey),
  });
});
