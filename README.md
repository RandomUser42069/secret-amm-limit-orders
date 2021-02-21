# Private order book - Limit order contract for secretAMM

[Issue Description](https://github.com/enigmampc/SecretNetwork/issues/699)

TODO:
* Ask side
    * triggerer ask side on the order book contract
* websockets on www

** MVP ??


* On the creation/execution of limit orders, a fee needs to be accomodated/payed to cover the triggerer gas prices
* Descentralized way to check the trigger and trigger every block
* Adapt to multiple limit orders per user on the same trading pair?
* Clean some parts of the code that are not in use
* Delete Native tokens?
## What's done?
### Secret Order Book Factory Contract
* Handles
    * CreateViewingKey => For users to create their VK to see their Limit Orders
    * ChangeSecretOrderBookContractCodeId => For Admin to change the Secret Order Book Contract codeid (code updates, ...)
    * NewSecretOrderBookInstanciate => Create a nem Secret Order Book for a specific pair of the AMM
    * InitCallBackFromSecretOrderBookToFactory => Callback from the previous method, so it receives the address that the contract was instanciated and save it for query.
    * AddOrderBookToUser => Called from Secret Order Book Contracts to associate a user to a order book contract (for query purposes)
    * RemoveOrderBookFromUser => Called from Secret Order Book Contracts to remove an association of a user to a order book contract (for query purposes)
* Queries
    * IsKeyValid => Called by Secret Order Book Contracts to check if a user provided the correct vk
    * SecretOrderBookContractCodeId => Get the current Secret Order Book code id for new instanciated contracts
    * SecretOrderBooks => Get the address of the secret order book associated with a specific amm pair
    * UserSecretOrderBooks => Get secret order book contracts where a user have some orders
### Secret Order Book Contract
* Handles
    * Receive => Create Limit Order from SNIP20 Tokens
    * ReceiveNativeToken => Create Limit Order from the native token
    * WithdrawLimitOrder => Widthdraw assets locked on a limit order
    * ***TODO: TriggerLimitOrders***
* Queries
    * OrderBookPairInfo => Returns info about the tokens and the associated amm pair contract address
    * GetLimitOrder => Receives a user and vk and returns the limit order info
    * CheckOrderBookTrigger => Checks if a limit order needs to be triggered
## References
* [Secret Contracts Template](https://github.com/enigmampc/secret-template)
* [secretSCRT SNIP20 Token Contract](https://github.com/enigmampc/secretSCRT)
* [Secret Swap](https://github.com/enigmampc/SecretSwap)
* [Sealed Bid Auction Factory](https://github.com/baedrik/secret-auction-factory)
* [Rust Order Book Example repo](https://github.com/dgtony/orderbook-rs/blob/master/src)