# Private order book - Limit order contract for secretAMM

[Issue Description](https://github.com/enigmampc/SecretNetwork/issues/699)

TODO:
* Interact with AMM
    * On the creation of the limit order, add a max splipage input
    * On the Triggerer query it needs to account for that splipage
* Create most basic frontend to interact with this contract
    * Create Order Book Pairs
        * secretcli q compute query secret1d3de9fsj0m6jkju94sc8yzecw7f6tfklydrwvc '{"pair": {"asset_infos": [{"token":{"contract_addr":"secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx","token_code_hash":"CD400FB73F5C99EDBC6AAB22C2593332B8C9F2EA806BF9B42E3A523F3AD06F62","viewing_key":""}},{"token":{"contract_addr":"secret1ttg5cn3mv5n9qv8r53stt6cjx8qft8ut9d66ed","token_code_hash":"2DA545EBC441BE05C9FA6338F3353F35AC02EC4B02454BC49B1A66F4B9866AED","viewing_key":""}}]}}'
    * Displays the AMM price
    * Displays my Limit Orders
    * Create Limit Order on pairs available
    * Widthdraw from Limit Orders
* Deploy to testnet
* Create the Triggerer script
    * Needs to have a wallet with SCRT for the gas prices on the triggers
* On the creation/execution of limit orders, a fee needs to be accomodated/payed to cover the triggerer gas prices
* As I'm aggregating the limit orders and doing a single transaction need to implement splipage calculations and correct distribution for the limit orders triggered
* Descentralized way to check the trigger and trigger every block
* Adapt to multiple limit orders per user on the same trading pair?

## What's done?
### Secret Order Book Factory Contract
* Handles
    * CreateViewingKey => For users to create their VK to see their Limit Orders
    * ChangeSecretOrderBookContractCodeId => For Admin to change the Secret Order Book Contract codeid (code updates, ...)
    * NewSecretOrderBookInstanciate => Create a nem Secret Order Book for a specific pair of the AMM
    * InitCallBackFromSecretOrderBookToFactory => Callback from the previous method, so it receives the address that the contract was instanciated and save it for query.
* Queries
    * IsKeyValid => Called by Secret Order Book Contracts to check if a user provided the correct vk
    * SecretOrderBookContractCodeId => Get the current Secret Order Book code id for new instanciated contracts
    * SecretOrderBooks => Get the address of the secret order book associated with a specific amm pair

### Secret Order Book Contract
* Handles
    * Receive => Create Limit Order from SNIP20 Tokens
        * ***TODO: Receive the max slipage too***
    * ReceiveNativeToken => Create Limit Order from the native token
        * ***TODO: Receive the max slipage too***
    * WithdrawLimitOrder => Widthdraw assets locked on a limit order
    * ***TODO: TriggerLimitOrders***
* Queries
    * GetLimitOrder => Receives a user and vk and returns the limit order info
    * CheckOrderBookTrigger => Checks if a limit order needs to be triggered
### WWW
* Query pairs of AMM
* Get current price from AMM
* Create new secret orderbook for specific pair 
* Create my VK
* Get my Limit orders (x)
## Testnet AMM
* factory_address => secret1d3de9fsj0m6jkju94sc8yzecw7f6tfklydrwvc
* factory_hash => f5a2aa6982d44b7754ba11a63eb5d4dc982221cf8af978a6eeade9cd9ac1bace
* pair_address => secret165mcaz2e8lsd3qsa5k5fy9kpq8adc5ep2q9smt
* pair_hash => 6928becbab8de124d478993df5ca6ea41f5aca5d916b2eeb3fd781c0838e4039
* 1 token_address => secret1s7c6xp9wltthk5r6mmavql4xld5me3g37guhsx
* 1 token hash => CD400FB73F5C99EDBC6AAB22C2593332B8C9F2EA806BF9B42E3A523F3AD06F62
* 2 token_address => secret1ttg5cn3mv5n9qv8r53stt6cjx8qft8ut9d66ed
* 2 token hash => 2DA545EBC441BE05C9FA6338F3353F35AC02EC4B02454BC49B1A66F4B9866AED
## References
* [Secret Contracts Template](https://github.com/enigmampc/secret-template)
* [secretSCRT SNIP20 Token Contract](https://github.com/enigmampc/secretSCRT)
* [Secret Swap](https://github.com/enigmampc/SecretSwap)
* [Sealed Bid Auction Factory](https://github.com/baedrik/secret-auction-factory)
* [Rust Order Book Example repo](https://github.com/dgtony/orderbook-rs/blob/master/src)