# Private order book - Limit order contract for secretAMM

[Issue Description](https://github.com/enigmampc/SecretNetwork/issues/699)

TODO:
* TODO: Implement the fee logic after the swap (trigger of a limit order) (take the fee from the swapped token)
	* Transfer fees directly to triggerer account
	
* TODO: Admin can change the fees for an asset
* TODO: Redoo some logic of the check triggerer, dont want to have a loop there with max 10 entries
* TODO: OrderTriggererScript to verifiy multiple pairs at the same time (Nodejs backend?)
* TODO: SECURITY CHECK

* On the creation/execution of limit orders, a fee needs to be accomodated/payed to cover the triggerer gas prices
* This fee is transfered to the triggerer account when the swap message is received (Fee in Buy Token or Sell Token?)
* Clean some parts of the code that are not in use (clear native token and block native tokens)


* Future:
	* Statistics? TVL with band oracle integration

* Admin commands for updating?

## Limitations
* Centralized way of triggering on each block
* No fee taking to accomodate the triggerer gas fees

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
    * TriggerLimitOrders
* Queries
    * OrderBookPairInfo => Returns info about the tokens and the associated amm pair contract address
    * GetLimitOrder => Receives a user and vk and returns the limit order info
    * CheckOrderBookTrigger => Checks if a limit order needs to be triggered

## WWW Deploy
* Go to www, npm run build
* https://thatrand0muser.github.io/
## References
* [Secret Contracts Template](https://github.com/enigmampc/secret-template)
* [secretSCRT SNIP20 Token Contract](https://github.com/enigmampc/secretSCRT)
* [Secret Swap](https://github.com/enigmampc/SecretSwap)
* [Sealed Bid Auction Factory](https://github.com/baedrik/secret-auction-factory)
* [Rust Order Book Example repo](https://github.com/dgtony/orderbook-rs/blob/master/src)

## Check Triggers Algo example

### Pair: BTC/USDT

#### Buy

Limit order price => 47000 usdt per 1 btc
Amount USDT => 1000 usdt
Amount BTC => 0.021 btc (Calculated 1000/47000)

Buy Algo:
	Simulate => Offer 1 BTC => 48000 per 1 btc
	Check: Limit Order Price (47000) >= Simulated (48000) 
	- false

	Simulate => Offer 1 BTC => 47000 USDT
	Check: Limit Order Price (47000) >= Simulated (47000) 
	Simulate => Offer 0.021 BTC => 1010 usdt  (wSlippage)
	Check: Amount USDT (1000) >= Simulated (1010)
	- false

	Simulate => Offer 1 BTC => 46000 USDT
	Check: Limit Order Price (47000) >= Simulated (46000) 
	Simulate => Offer 0.021 BTC => 950 usdt  (wSlippage)
	Check: Amount USDT (1000) >= Simulated (950)
	- true

#### Sell

Limit order price => 50000 usdt per 1 btc
Amount BTC => 0.02 btc
Amount USDT => 1000 usdt (Calculated 0.02 * 50000)

Sell Algo:
	Simulate => Offer 1 BTC => 48000 per 1 btc
	Check: Limit Order Price (50000) <= Simulated (48000)
	- false

	Simulate => Offer 1 BTC => 50000 USDT
	Check: Limit Order Price (50000) <= Simulated (50000) 
	Simulate => Offer 0.02 BTC => 990 usdt  (wSlippage)
	Check: Amount USDT (1000) <= Simulated (990)
	- false

	Simulate => Offer 1 BTC => 51000 USDT
	Check: Limit Order Price (50000) <= Simulated (51000) 
	Simulate => Offer 0.02 BTC => 1020 usdt  (wSlippage)
	Check: Amount USDT (1000) <= Simulated (1020)
	- true

### Pair: SCRT/ETH

#### Buy

Limit order price => 0.0023e18 ETH per 1 SCRT
Amount ETH => 10e18 ETH
Amount SCRT => 4347.826086e6 SCRT (Calculated 10e18/0.0023e18)

Buy Algo:
	Simulate => Offer 1e6 SCRT => 0.002410e18 per 1 SCRT
	Check: Limit Order Price (0.0023e18) >= Simulated (0.002410e18) 
	- false

	Simulate => Offer 1e6 SCRT => 0.0023e18 ETH
	Check: Limit Order Price (0.0023e18) >= Simulated (0.0023e18) 
	Simulate => Offer 4347.826086e6 SCRT => 11e18 ETH  (wSlippage)
	Check: Amount ETH (10e18) >= Simulated (11e18)
	- false

	Simulate => Offer 1e6 SCRT => 0.0022e18 ETH
	Check: Limit Order Price (0.0023e18) >= Simulated (0.0022e18) 
	Simulate => Offer 4347.826086e6 SCRT => 9.5e18 ETH  (wSlippage)
	Check: Amount USDT (10e18) >= Simulated (9.5e18)
	- true

#### Sell

Limit order price => 0.0025e18 ETH per 1 SCRT
Amount ETH => 10e18 ETH
Amount SCRT => 4000e6 SCRT (Calculated 10e18/0.0025e18)

Sell Algo:
    Simulate => Offer 1e6 SCRT => 0.0024e18 per 1 SCRT
	Check: Limit Order Price (0.0025e18) <= Simulated (0.0024e18) 
	- false

    Simulate => Offer 1e6 SCRT => 0.0025e18 ETH
	Check: Limit Order Price (0.0025e18) >= Simulated (0.0025e18) 
	Simulate => Offer 4000e6 SCRT => 9.5e18 ETH  (wSlippage)
	Check: Amount ETH (10e18) <= Simulated (9.5e18)
	- false

	Simulate => Offer 1e6 SCRT => 0.0026e18 ETH
	Check: Limit Order Price (0.0025e18) <= Simulated (0.0026e18) 
	Simulate => Offer 4000e6 SCRT => 11e18 ETH  (wSlippage)
	Check: Amount ETH (10e18) <= Simulated (11e18)
	- true