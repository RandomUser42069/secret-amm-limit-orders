import React, {useState,useEffect} from 'react';
import {Card, Button, Spinner} from 'react-bootstrap'
import createPairOrderBook from "../requests/createPairOrderBook"
import ViewKeyButton from './ViewKeyButton';

// eslint-disable-next-line import/no-anonymous-default-export
export default ({
    AMM_FACTORY_ADDRESS,
    ORDERS_FACTORY_ADDRESS,
    client,
    viewKey
}: PairsAvailableProps) => {
    const [pairs, setPairs] = useState<PairState[]>([])
    const [getPairsCompleted, setGetPairsCompleted] = useState<boolean>(false)
    const [getLimitOrdersCompleted, setGetLimitOrdersCompleted] = useState<boolean>(false)

    useEffect(() => {
        async function init() {
            if (client.ready && !getPairsCompleted) {
                let pairsState:PairState[] = [];

                const pairs_response:PairsResponse  = await client.execute.queryContractSmart(AMM_FACTORY_ADDRESS, { 
                    pairs: {}
                  });

                for (let pair of pairs_response.pairs) {
                    let newPair = {...pair};

                    let promises:Promise<any>[] = []

                    // Tokens Info
                    for  (let asset of pair.asset_infos) {
                        promises.push(client.execute.queryContractSmart(asset.token.contract_addr, { 
                            token_info: {}
                          }))
                    }

                    // AMM Pair info
                    promises.push(client.execute.queryContractSmart(pair.contract_addr, { 
                        simulation: {
                            offer_asset: {
                                info: pair.asset_infos[0],
                                amount: "1"
                            }
                        }
                      }))

                    promises.push(client.execute.getCodeHashByContractAddr(pair.contract_addr))

                    promises.push(client.execute.queryContractSmart(ORDERS_FACTORY_ADDRESS, { 
                        secret_order_books: {
                            contract_address: pair.contract_addr 
                        }
                      }))
                    
                    const [
                        token1_info_response, 
                        token2_info_response, 
                        amm_pair_response,
                        amm_pair_hash_response,
                        order_book_factory_response
                    ] = await Promise.all(promises)

                    newPair.asset_infos[0].token_info = token1_info_response.token_info
                    newPair.asset_infos[0].is_native_token = newPair.asset_infos[0].native_token ? true : false
                    newPair.asset_infos[1].token_info = token2_info_response.token_info
                    newPair.asset_infos[1].is_native_token = newPair.asset_infos[1].native_token ? true : false
                    newPair.current_price = (amm_pair_response.return_amount / Math.pow(10, 12)).toString()
                    newPair.contract_hash = amm_pair_hash_response
                    newPair.order_book_data = order_book_factory_response.secret_order_books.secret_order_book ? {
                        contract_addr: order_book_factory_response.secret_order_books.secret_order_book.contract_addr,
                        loading_limit_orders: true,
                        limit_order: null,
                    } : null
                    pairsState.push(newPair)
                }
                setPairs(pairsState);
                setGetPairsCompleted(true)
            }
        }
        init();
      }, [client])

    useEffect(() => {
        async function init() {
            if (pairs.length > 0 && !getLimitOrdersCompleted) {
                let updatedPairs = [...pairs];
                for (let [i, updatedPair] of updatedPairs.entries()) {
                    // Get my limit orders for this pair if I gave a VK
                    if (updatedPair.order_book_data && updatedPair.order_book_data.loading_limit_orders && viewKey) {
                        const response = await client.execute.queryContractSmart(updatedPair.order_book_data.contract_addr, { 
                            get_limit_order: {
                                user_address: client.accountData.address,
                                user_viewkey: viewKey
                            }
                          }) 
                        if (!response) {
                            updatedPairs[i].order_book_data!.loading_limit_orders = false
                        }
                    }
                }
                setPairs(updatedPairs)
                setGetLimitOrdersCompleted(true)
            }
        }
       init()
    }, [pairs])

    return (
        <div>
            {!getPairsCompleted && <Spinner animation="border"/>}
            {
                pairs && pairs.map((pair,i) => 
                    <Card style={{ width: '18rem' }} key={pair.contract_addr}>
                        <Card.Body>
                        <Card.Title>
                            <div>
                                {pair.asset_infos[0].token_info.symbol + " / " + pair.asset_infos[1].token_info.symbol}
                                <br/>
                                <Button variant="primary" onClick={() => {
                                    //newPair = {...pair}
                                }}>Switch</Button>
                            </div>
                            
                        </Card.Title>
                        <Card.Subtitle className="mb-2 text-muted">{"AMM Price: " + pair.current_price}</Card.Subtitle>
                        {
                            !pair.order_book_data ? 
                                <Button variant="primary" onClick={async () => {
                                    await createPairOrderBook(client.execute, pair, ORDERS_FACTORY_ADDRESS)
                                }}>Create Pair Order Book</Button> 
                                : pair.order_book_data.loading_limit_orders && viewKey ?
                                    <Button variant="primary"><Spinner animation="border" /></Button>
                                : !pair.order_book_data.loading_limit_orders && viewKey && !pair.order_book_data.limit_order ?
                                    <div>
                                        <Button variant="success" style={{margin:"5px"}} onClick={() => {}}>Buy</Button>
                                        <Button variant="danger" style={{margin:"5px"}} onClick={() => {}}>Sell</Button>
                                    </div> :
                                        !pair.order_book_data.loading_limit_orders && viewKey && pair.order_book_data.limit_order &&
                                            <Button variant="primary" disabled>View</Button>
                        }
                        </Card.Body>
                    </Card>)
            }
        </div>
    )
}

type PairsAvailableProps = {
    AMM_FACTORY_ADDRESS: string,
    ORDERS_FACTORY_ADDRESS: string,
    client: any,
    viewKey: null | string
  }

interface PairsResponse {
    pairs: PairState[]
}

interface PairState {
    asset_infos: AssetInfo[],
    contract_addr: string,
    contract_hash: string,
    liquidity_token: string,
    token_code_hash: string,
    current_price: string,
    order_book_data: {
        contract_addr: string,
        loading_limit_orders: boolean,
        limit_order: {} | null
    } | null
  }

interface AssetInfo {
    is_native_token: boolean,
    native_token: NativeToken,
    token: Token,
    token_info: TokenInfo,
}

interface NativeToken {
    denom: string
}

interface Token {
    contract_addr: string,
    token_code_hash: string,
    viewing_key: string
}

interface TokenInfo {
    decimals: number,
    name: string,
    symbol: string,
    total_supply: string
}

