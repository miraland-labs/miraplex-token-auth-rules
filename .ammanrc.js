// @ts-check
'use strict';
const path = require('path');
const accountProviders = require('./packages/sdk/dist/src/generated/accounts');

const localDeployDir = path.join(__dirname, 'program', 'target', 'deploy');
const { LOCALHOST, tmpLedgerDir } = require('@metaplex-foundation/amman');
const MY_PROGRAM_ID = require("./packages/sdk/idl/mpl_token_auth_rules.json").metadata.address;

function localDeployPath(programName) {
    return path.join(localDeployDir, `${programName}.so`);
}

const programs = {
    token_auth_rules: {
        label: 'mpl_token_auth_rules',
        programId: MY_PROGRAM_ID,
        deployPath: localDeployPath('mpl_token_auth_rules')
    },
};

const accounts = [
    {
        label: 'Token Metadata Program',
        accountId:'Meta88XpDHcSJZDFiHop6c9sXaufkZX5depkZyrYBWv',
        // marking executable as true will cause Amman to pull the executable data account as well automatically
        executable: true,
    },
    {
        label: 'Random other account',
        accountId:'4VLgNs1jXgdciSidxcaLKfrR9WjATkj6vmTm5yCwNwui',
        // by default executable is false
        // providing a cluster here will override the accountsCluster field
        cluster: 'https://metaplex.devnet.rpcpool.com'
    }
];

const validator = {
    killRunningValidators: true,
    programs,
    // The accounts below is commented out. Uncomment if you want to pull remote accounts. Check Amman docs for more info
    accounts,
    verifyFees: false,
    limitLedgerSize: 10000000,
    commitment: 'singleGossip',
    resetLedger: true,
    jsonRpcUrl: LOCALHOST,
    websocketUrl: '',
    ledgerDir: tmpLedgerDir(),
};

module.exports = {
    programs,
    validator,
    relay: {
        accountProviders,
    },
};