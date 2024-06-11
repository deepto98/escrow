
# Escrow
## What is an escrow 
* In a traditional, non cryto scenario, it is a contractual arrangement in which a third party receives and disburses money or property for the primary transacting parties, with the disbursement dependent on conditions agreed to by the transacting parties. A trusted third party is hired to hold all documents and funds for both buyer and seller, and the documents and funds are held "in escrow." The escrow provider safeguards the funds and protects all parties by ensuring the terms of the purchase contract and agreement are carried out.

* In crypto, however, we do not need a Trusted Third Party. Instead we can use a Vault and add   rules to handle swaps. The key concepts of a crypto native escrow are:

    1. Party A - The Maker 
        * Sets the initial terms
        * Deposits an asset to trade
        * Requests asset in return

    2. Person B - The Taker 
        * Accepts the terms set by Party A
        * Deposits the asset requested

    3. Once the terms are met the smart contract
completes the trade.

Some common examples include - Token Swaps, Dex, In game trading
## Implementing an  Escrow Smart Contract in Anchor
We eill have 3 Instructions:

1. make -> user will set a escrow and set the terms

2. take -> another user will accept the escrow terms and take the deal

3. refund -> the maker, will be able to cancel the escrow if it has not yet
been taken

 