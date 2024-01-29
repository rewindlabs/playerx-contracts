# CW721 Base

This is a basic implementation of a CW721 NFT contract. It implements the [CW721 spec](https://github.com/CosmWasm/cw-nfts/blob/main/packages/cw721/README.md) and extends it for the [PlayerX NFT collection](https://www.playerx.quest/).

## Minting

In addition to the standard CW721 functionalities, our contract introduces several new minting functions designed specifically for the PlayerX minting experience.

- `mintTeam`: This function is designed for usage by the team treasury.

- `mintAllowlist`: This function allows minting by users who are on a predetermined allowlist.

- `mintPublic`: This function facilitates public minting.
