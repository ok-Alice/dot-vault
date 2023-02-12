import { ContractPromise } from '@polkadot/api-contract';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';

import { deployContract, contractTx } from './curry-contract';

const start = async () => {
    const contract = await deployOracle();

    setInterval(() => {
        updateOracle(contract, 'test', 'lol');
    }, 10000);
};

const deployOracle = async (): Promise<ContractPromise> => {
    const wsProvider = new WsProvider(); //defaults to localhost
    const api = await ApiPromise.create({ provider: wsProvider });

    // Get Alice Account for signing transactions
    const keyring = new Keyring({ type: 'sr25519'});
    const alice = keyring.addFromUri('//Alice', { name: 'Alice' });

    const contract = await deployContract(api, alice, 'oracle');
    
    console.log("Contract deployed at: ", contract.address.toString());
    return contract;
};

const updateOracle = async (contract: ContractPromise, key: string, value: string) => {
    const keyring = new Keyring({ type: 'sr25519'});
    const alice = keyring.addFromUri('//Alice', { name: 'Alice' });
    const method = 'set';

    return contractTx(alice, contract, method, key, value);
};

start();
