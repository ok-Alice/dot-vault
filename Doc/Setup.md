# Setup

## Astar node

Download the latest release of the [Astar node](https://github.com/AstarNetwork/Astar/releases) to run it locally:

    $ astar-collator --dev --tmp

### XC20 asset

Create 2 assets (loan, collateral):

From under [Network -> Assets](https://polkadot.js.org/apps/#/assets) click create asset and create them as follows

![img/createAsset.png]
![img/createAsset2.png]

You have to mint some token for each asset.

Afterwards the tokens can be used as ERC20 tokes (via XC20):

AssetID 4242 -> 0xFFFFFFFF00000000000000000000000000001092
AssetID 4243 -> 0xFFFFFFFF00000000000000000000000000001093

### ERC721

Deploy a ERC721 contract from Openzeppelin that can mint


