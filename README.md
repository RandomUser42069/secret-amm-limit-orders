# Private order book - Limit order contract for secretAMM

[Issue Description](https://github.com/enigmampc/SecretNetwork/issues/699)

TODO:
* Study AMM contract and how to interact with it
    * Alter my code to support the native token SCRT!
    * When instanciating a new limit order pair, need to query the AMM factory for that pair info so I store that on the state
        * Pair ou Pairs
    * On my query check to trigger, query AMM pair contract to get the current price
        * '{"simulation":{"offer_asset": {"info":{"native_token":{"denom":"uscrt"}},"amount":"10"}}}'
        * '{"reverse_simulation":{"ask_asset": {"info":{"native_token":{"denom":"uscrt"}},"amount":"10"}}}'
        * '{"simulation":{"offer_asset": {"info":{"token":{"contract_addr":"secret1j50u6hvume8pkq2c7lcmktrzd7lrymxhujrnax", "token_code_hash": "78BDF9CDD7538FC96DFB18372635A20162243E49CA9BCD4BD2AFF6300D2BC5E2", "viewing_key":""}},"amount":"10"}}}'


* Create most basic frontend to interact with this contract
* Deploy to testnet
* Create the Triggerer script
    * Needs to have a wallet with SCRT for the gas prices on the triggers
* On the creation/execution of limit orders, a fee needs to be accomodated/payed to cover the triggerer gas prices
* As I'm aggregating the limit orders and doing a single transaction need to implement splipage calculations and correct distribution for the limit orders triggered
* Descentralized way to check the trigger and trigger every block
* Adapt to multiple limit orders per user on the same trading pair?

## Testnet AMM
* factory_address => secret190q0suu7yxjzx4uf92kqjzuuqzm0xkk2az0gf7
* pair_address => secret104s4dm08q2hr8ruhy9qavcgyssepskfn85x534
* token_address => secret1j50u6hvume8pkq2c7lcmktrzd7lrymxhujrnax
## References
* [Secret Contracts Template](https://github.com/enigmampc/secret-template)
* [secretSCRT SNIP20 Token Contract](https://github.com/enigmampc/secretSCRT)
* [Secret Swap](https://github.com/enigmampc/SecretSwap)
* [Sealed Bid Auction Factory](https://github.com/baedrik/secret-auction-factory)
* [Rust Order Book Example repo](https://github.com/dgtony/orderbook-rs/blob/master/src)