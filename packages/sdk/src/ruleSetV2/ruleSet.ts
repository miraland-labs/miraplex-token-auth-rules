import * as beet from '@miraplex/beet';
import * as beetMiraland from '@miraplex/beet-miraland';
import { PublicKey } from '@solarti/web3.js';
import { deserializeRulesV2, RuleV2, serializeRulesV2 } from './rule';

export type RuleSetV2 = {
  // libVersion = 2
  name: string;
  owner: PublicKey;
  operations: Record<string, RuleV2>;
};

export const serializeRuleSetV2 = (ruleSet: RuleSetV2): Buffer => {
  const tuples = Object.entries(ruleSet.operations);
  const operations = tuples.map(([operation]) => operation);
  const rules = tuples.map(([, rule]) => rule);
  const ruleSize = operations.length;

  // Header.
  const headerBuffer = Buffer.alloc(8);
  beet.u32.write(headerBuffer, 0, 2);
  beet.u32.write(headerBuffer, 4, ruleSize);

  // Owner.
  const ownerBuffer = Buffer.alloc(32);
  beetMiraland.publicKey.write(ownerBuffer, 0, ruleSet.owner);

  // Name.
  const nameBuffer = Buffer.alloc(32);
  nameBuffer.write(ruleSet.name);

  // Operations.
  const operationsBuffer = Buffer.concat(
    operations.map((operation) => {
      const buffer = Buffer.alloc(32);
      buffer.write(operation);
      return buffer;
    }),
  );

  // Rules.
  const rulesBuffer = serializeRulesV2(rules);

  return Buffer.concat([headerBuffer, ownerBuffer, nameBuffer, operationsBuffer, rulesBuffer]);
};

export const deserializeRuleSetV2 = (buffer: Buffer, offset = 0): RuleSetV2 => {
  const libVersion = beet.u32.read(buffer, offset);
  offset += 4;
  if (libVersion !== 2) {
    throw new Error('Expected a rule set version 2, got version ' + libVersion);
  }

  // Rule size.
  const ruleSize = beet.u32.read(buffer, offset);
  offset += 4;

  // Owner.
  const owner = beetMiraland.publicKey.read(buffer, offset);
  offset += 32;

  // Name.
  const name = buffer
    .subarray(offset, offset + 32)
    .toString()
    .replace(/\u0000/g, '');
  offset += 32;

  // Operations.
  const operations: string[] = [];
  for (let index = 0; index < ruleSize; index++) {
    operations.push(
      buffer
        .subarray(offset, offset + 32)
        .toString()
        .replace(/\u0000/g, ''),
    );
    offset += 32;
  }

  // Rules.
  const rules = deserializeRulesV2(buffer, ruleSize, offset);
  const tuples: [string, RuleV2][] = operations.map((operation, index) => [
    operation,
    rules[index],
  ]);

  return { name, owner, operations: Object.fromEntries(tuples) };
};
