# CW2981 Leveling

This contract builds on top of the [CW-2981 royalties contract](https://github.com/CosmWasm/cw-nfts/tree/main/contracts/cw2981-royalties). It uses our own [CW721-base](../cw721-base/), so all of the CW-721 logic and behavior you would expect from an NFT is implemented as usual. Additionally, it adds a novel leveling functionality for the [PlayerX NFT collection](https://www.playerx.quest/).

## Leveling

The CW2981 Leveling contract introduces a unique leveling system for NFTs. This staking system allows each token to gain experience (EXP) and "level up" over time. NFTs can accumulate experience points both passively over time and actively by gaining bonus EXP through participation in events.
