import { ApiPromise } from '@polkadot/api';
import { KeyringPair } from '@polkadot/keyring/types';
const { CodePromise, ContractSubmittableResult } = require('@polkadot/api-contract')
import { ContractPromise } from '@polkadot/api-contract';

import fs from 'fs';

function fetchContractJson(name: string) {
    const fullname = `./target/ink/${name}.contract`;

    return JSON.parse(fs.readFileSync(fullname,'utf-8'));
}

export const deployContract = async (
    api: ApiPromise,
    account: KeyringPair,
    contractName: string,
    ...args: any[]
): Promise<ContractPromise> => {
    return new Promise(async (resolve, reject) => {
        const contractJson = fetchContractJson(contractName);

        const code = new CodePromise(api, contractJson, contractJson.wasm);

        const gasLimit = 100000n * 1000000n;
        const storageDepositLimit = null;
        
        const tx = code.tx.new({ gasLimit, storageDepositLimit}, ...args);

        const deploy = new Promise<string>(async (resolve, reject) => {
            const unsub = await tx.signAndSend(account, ({ contract, status }) => {

                const rejectPromise = (error: any) => {
                    console.error(`Error sending tx`, error);
                    console.log(`tx for the error above`, tx.toHuman());
                    unsub();
                    reject(error);
                  }
    
                if (status.isInBlock || status.isFinalized) {
                    const address = contract.address.toString();
                    unsub();                
                    resolve(address);
                } else if(status.isInvalid) {
                    rejectPromise(new Error(`Extrinsic isInvalid`))
                }
            });
        });

        const address = await deploy;

        resolve(new ContractPromise(api, contractJson, address));
    });
};


export const contractQuery = async (
    account: string,
    contract: ContractPromise,
    method: string,
    ...args: any[]
) : Promise<any> => {
    return new Promise(async (resolve, reject) => {
        const gasLimit = 100000n * 1000000n;
        const storageDepositLimit = null;

        var { result, output } = await contract.query[method](
            account,
            {
                gasLimit,
                storageDepositLimit,
            },
            ...args,
        );
    
        if(result.isOk) {
            resolve(output);
        } else {
            reject(new Error('contractQuery failed!'));
        }
    });
};

export const contractTx = async (
    account: KeyringPair,
    contract: ContractPromise,
    method: string,
    ...args: any[]
) : Promise<any> => {
    return new Promise(async (resolve, reject) => {        
        const gasLimit = 100000n * 1000000n;
        const storageDepositLimit = null;

        const txresult = new Promise<typeof ContractSubmittableResult>(async(resolve, reject) => {
            await contract.tx[method]({ storageDepositLimit, gasLimit }, ...args)
                .signAndSend(
                    account, 
                    (result: typeof ContractSubmittableResult) => {
                        const rejectPromise = (error: any) => {
                            console.error(`Error sending tx`, error);
                            reject(error);
                        }
    
                        if (result.status.isInBlock || result.status.isFinalized) {
                            resolve(result);
                        } else if(result.status.isInvalid) {
                            rejectPromise(new Error(`Extrinsic isInvalid`));
                        }
                    });
                });
            const result = await txresult;            

            if(result.dispatchError) {
                reject(new Error(`Dispatch error: ${result.dispatchError}`));
            } if(result.internalError) {
                reject(new Error(`Dispatch error: ${result.internalError}`));
            }  else {
                resolve(result);
            }
    });
};
