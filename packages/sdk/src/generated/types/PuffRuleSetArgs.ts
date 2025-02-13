/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@miraplex/beet';
/**
 * This type is used to derive the {@link PuffRuleSetArgs} type as well as the de/serializer.
 * However don't refer to it in your code but use the {@link PuffRuleSetArgs} type instead.
 *
 * @category userTypes
 * @category enums
 * @category generated
 * @private
 */
export type PuffRuleSetArgsRecord = {
  V1: { ruleSetName: string };
};

/**
 * Union type respresenting the PuffRuleSetArgs data enum defined in Rust.
 *
 * NOTE: that it includes a `__kind` property which allows to narrow types in
 * switch/if statements.
 * Additionally `isPuffRuleSetArgs*` type guards are exposed below to narrow to a specific variant.
 *
 * @category userTypes
 * @category enums
 * @category generated
 */
export type PuffRuleSetArgs = beet.DataEnumKeyAsKind<PuffRuleSetArgsRecord>;

export const isPuffRuleSetArgsV1 = (x: PuffRuleSetArgs): x is PuffRuleSetArgs & { __kind: 'V1' } =>
  x.__kind === 'V1';

/**
 * @category userTypes
 * @category generated
 */
export const puffRuleSetArgsBeet = beet.dataEnum<PuffRuleSetArgsRecord>([
  [
    'V1',
    new beet.FixableBeetArgsStruct<PuffRuleSetArgsRecord['V1']>(
      [['ruleSetName', beet.utf8String]],
      'PuffRuleSetArgsRecord["V1"]',
    ),
  ],
]) as beet.FixableBeet<PuffRuleSetArgs, PuffRuleSetArgs>;
