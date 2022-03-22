# Generic NFT coin-flip contract for NEAR protocol

## Contract  
  
Contract lives in the "coin_flip_contract" folder. compile it using the instructions for rust near sdk available in: https://github.com/near/near-sdk-rs

## CLI usage  
For the convenience of the non technical user, a command line interface (CLI) has been built to perform the main administrative tasks in running your coin-flip app.  

### Install the CLI  
To install the CLI you must have node.js installed in your system and you must have added it to the PATH environment variable. You can check that by typing "node" in your command prompt, if the command is not recognized, you have to add it to PATH.  

Installation: https://nodejs.org/en/download/  
  
Add to PATH: https://www.tutorialspoint.com/nodejs/nodejs_environment_setup.htm  
  
After that you can install the command line interface by opening the "deploy_scripts" folder in your command line and typing:  
```
npm install -g .
```   

  
### Wallet setup  
Before starting the deployment you must have 2 addresses for use:  
1. Your owner address  
2. A contract address  

The owner address will receive the fees from the contract designated under "dev_fee".  
  
Both contracts should have a NEAR balance for payment of gas fees and staking of storage.  
  
Log in with both accounts using the NEAR CLI.

### Deploy the contract  
To deploy the contract, you'll run the following script in your command line
```
nft-coinflip deploy <accountToDeploy> <binaryLocation>
```  
Substitute accountToDeploy for the contract address you created  
Substitute binaryLocation for the absolute path to the contract's compiled binaries  

### Initialize the contract  
Before the contract can be used, you'll need to setup its configuration:
```
nft-coinflip initialize <ownerAccount> <contractAccount> <nftFee> <devFee> <houseFee> <winMultiplier> <maxBet> <minBet> <minBalanceFraction>
```
ownerAccount is the owner account you created to manage the game  
contractAccount is the account to which the contract was deployed  
nftFee is how many % of each bet you want to distribute to the holders of your NFT collection. this number must be an integer. The percentage of each transaction that goes to the holders will be nftFee/100000  
devFee is how many % of each bet you want to distribute to the owner account. this number must be an integer. The percentage of each transaction that goes to the owner will be devFee/100000  
houseFee is how many % of each bet the house (meaning the contract account) will keep as a fee. this number must be an integer. The percentage of each transaction that goes to the house will be houseFee/100000  
winMultiplier is the multiplier for the winning bet (after discounting all fees). this number must be an integer. The multiplier winMultiplier/100000  
maxBet is the maximum amount in yoctonear that a user can bet in each coinflip  
minBet is the minimum amount in yoctonear that a user can bet in each coinflip  
minBalanceFraction represents the minimum amount of balance that a user is allowed to deposit into the contract. The minum balance is minBet/minBalanceFraction  
  
### retrive dev fees  
To retrieve the dev fees collected from players to the owners account, use this call:
```
nft-coinflip retrieve-dev <ownerAccount> <contractAccount>
```
ownerAccount is the owner account you created to manage the game  
contractAccount is the account to which the contract was deployed  
  
### retrive fees for NFT holders
To retrieve the fees collected to nft holders and distribute it to them, use this call:
```
nft-coinflip retrieve-nft <ownerAccount> <contractAccount> <nftContractAccount>
```
ownerAccount is the owner account you created to manage the game  
contractAccount is the account to which the contract was deployed 
nftContractAccount is the account to which the NFT contract is deployed  
  
### read current contract state  
To read the contract's state, containing its initialization values and nft_holders and dev balances, use this call:
```
nft-coinflip get-state <ownerAccount> <contractAccount>
```
ownerAccount is the owner account you created to manage the game  
contractAccount is the account to which the contract was deployed 
  
### update initialization state  
If you need to change any of the initialization values, use this call:
```
nft-coinflip update-state <ownerAccount> <contractAccount> <nftFee> <devFee> <houseFee> <winMultiplier> <maxBet> <minBet> <minBalanceFraction>
```
ownerAccount is the owner account you created to manage the game  
contractAccount is the account to which the contract was deployed  
nftFee is how many % of each bet you want to distribute to the holders of your NFT collection. this number must be an integer. The percentage of each transaction that goes to the holders will be nftFee/100000  
devFee is how many % of each bet you want to distribute to the owner account. this number must be an integer. The percentage of each transaction that goes to the owner will be devFee/100000  
houseFee is how many % of each bet the house (meaning the contract account) will keep as a fee. this number must be an integer. The percentage of each transaction that goes to the house will be houseFee/100000  
winMultiplier is the multiplier for the winning bet (after discounting all fees). this number must be an integer. The multiplier winMultiplier/100000  
maxBet is the maximum amount in yoctonear that a user can bet in each coinflip  
minBet is the minimum amount in yoctonear that a user can bet in each coinflip  
minBalanceFraction represents the minimum amount of balance that a user is allowed to deposit into the contract. The minum balance is minBet/minBalanceFraction  
  
### Activate panic mode
In case you believe the contract is under attack or want to pause it for any reason, use this call:
```
nft-coinflip emergency-panic <ownerAccount> <contractAccount> <withdrawalAmount>
```
ownerAccount is the owner account you created to manage the game  
contractAccount is the account to which the contract was deployed  
withdrawalAmount is the amount in yoctonear that you want to withdrawal from the contract account to the owner account  
  
Note that this call will stop anyone who is not the owner from interacting with the contract, all interactions will be rejected and throw an error.  
In case you want to reactivate the contract after using the panic mode, just use this call one more time with withdrawalAmount as 0