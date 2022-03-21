#! /usr/bin/env node

import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import { deployContractAccountExists, deployContractNewAccount } from './../deploy/deploy_contract.js';
import { initializeContract, emergencyPanic, getContractState, updateContract, retrieveDevFunds, retrieveNftFunds } from "./../deploy/call_contract_function.js"

//import 

yargs(hideBin(process.argv))
    .command(
        'deploy <accountToDeploy> <binaryLocation>', 'deploy contract to the blockchain',
        (yargs) => {
            yargs.positional(
                'accountToDeploy', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account to which contract will be deployed'
                });
            yargs.positional(
                'binaryLocation', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'absolute path to contract binary'
                });
        },
        (argv) => {
            deployContractAccountExists(argv.accountToDeploy, argv.binaryLocation);
        }
    )
    .command(
        'initialize <ownerAccount> <contractAccount> <nftFee> <devFee> <houseFee> <winMultiplier> <maxBet> <minBet> <minBalanceFraction>', 'initialize state for deployed contract',
        (yargs) => {
            yargs.positional(
                'ownerAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that will send the transaction'
                });
            yargs.positional(
                'contractAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that hosts the contract'
                });
            yargs.positional(
                'nftFee', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'fee to be paid to nft holders from every bet base /100000'
                });
            yargs.positional(
                'devFee', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'fee to be paid to devs from every bet base /100000'
                });
            yargs.positional(
                'houseFee', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'fee to be paid to house from every bet base /100000'
                });
            yargs.positional(
                'winMultiplier', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'multiplication to be applied on net bet base /100000'
                });
            yargs.positional(
                'maxBet', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'maximum bet in yoctonear'
                });
            yargs.positional(
                'minBet', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'minimum bet in yoctonear'
                });
            yargs.positional(
                'minBalanceFraction', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'minimum balance to be kept in relation to min bat (minBet/this = minimum balance)'
                });

        },
        (argv) => {
            initializeContract(argv.ownerAccount, argv.contractAccount, argv);
        }
    )
    .command(
        'retrieve-dev <ownerAccount> <contractAccount>', 'send cumulated tokens to owner account',
        (yargs) => {
            yargs.positional(
                'ownerAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that will send the transaction'
                });
            yargs.positional(
                'contractAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that hosts the contract'
                });
        },
        (argv) => {
            retrieveDevFunds(argv.ownerAccount, argv.contractAccount);
        }
    )
    .command(
        'retrieve-nft <ownerAccount> <contractAccount> <nftContractAccount>', 'send cumulated tokens to nft holders',
        (yargs) => {
            yargs.positional(
                'ownerAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that will send the transaction'
                });
            yargs.positional(
                'contractAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that hosts the contract'
                });
            yargs.positional(
                'nftContractAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that hosts the nft contract'
                });
        },
        (argv) => {
            retrieveNftFunds(argv.ownerAccount, argv.contractAccount, argv.nftContractAccount);
        }
    )
    .command(
        'emergency-panic <ownerAccount> <contractAccount> <withdrawalAmount>', 'activate emergency panic mode',
        (yargs) => {
            yargs.positional(
                'ownerAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that will send the transaction'
                });
            yargs.positional(
                'contractAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that hosts the contract'
                });
            yargs.positional(
                'withdrawalAmount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'amount that you want to withdrawal from the contract account'
                });
        },
        (argv) => {
            emergencyPanic(argv.ownerAccount, argv.contractAccount, argv.withdrawalAmount);
        }
    )
    .command(
        'get-state <ownerAccount> <contractAccount>', 'read current state of contract',
        (yargs) => {
            yargs.positional(
                'ownerAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that will send the transaction'
                });
            yargs.positional(
                'contractAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that hosts the contract'
                });
        },
        (argv) => {
            getContractState(argv.ownerAccount, argv.contractAccount);
        }
    )
    .command(
        'update-state <ownerAccount> <contractAccount> <nftFee> <devFee> <houseFee> <winMultiplier> <maxBet> <minBet> <minBalanceFraction>', 'initialize state for deployed contract',
        (yargs) => {
            yargs.positional(
                'ownerAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that will send the transaction'
                });
            yargs.positional(
                'contractAccount', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'account that hosts the contract'
                });
            yargs.positional(
                'nftFee', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'fee to be paid to nft holders from every bet base /100000'
                });
            yargs.positional(
                'devFee', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'fee to be paid to devs from every bet base /100000'
                });
            yargs.positional(
                'houseFee', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'fee to be paid to house from every bet base /100000'
                });
            yargs.positional(
                'winMultiplier', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'multiplication to be applied on net bet base /100000'
                });
            yargs.positional(
                'maxBet', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'maximum bet in yoctonear'
                });
            yargs.positional(
                'minBet', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'minimum bet in yoctonear'
                });
            yargs.positional(
                'minBalanceFraction', {
                    type: 'string',
                    default: 'Cambi',
                    describe: 'minimum balance to be kept in relation to min bat (minBet/this = minimum balance)'
                });

        },
        (argv) => {
            updateContract(argv.ownerAccount, argv.contractAccount, argv);
        }
    )
    .demandCommand(1)
    .parse();