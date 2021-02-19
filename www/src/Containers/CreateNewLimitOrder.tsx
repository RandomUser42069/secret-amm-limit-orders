import React, {useState,useEffect} from 'react';
import {Card, Button, Spinner, Modal, DropdownButton, Dropdown} from 'react-bootstrap'

// eslint-disable-next-line import/no-anonymous-default-export
export default ({
    AMM_FACTORY_ADDRESS,
    ORDERS_FACTORY_ADDRESS,
    tokensData,
    client,
    viewKey
}: CreateNewLimitOrderProps) => {
    const [showCreateLimitOrderModal, setShowCreateLimitOrderModal] = useState(false);
    const [ammFactoryPairs, setAmmFactoryPairs] = useState<any>(null);
    const [selectedAmmFactoryPairIndex, setSelectedAmmFactoryPairIndex] = useState<any>(null)
    
    const [selectedAmmPairPrice, setSelectedAmmPairPrice] = useState<any>(null);
    const [orderBookPair, setOrderBookPair] = useState<any>({
        isInstanciated: null,
        data: null
    })
    const [limitOrderAmountInput, setLimitOrderAmountInput] = useState<any>(null);
    const [limitOrderPriceInput, setLimitOrderPriceInput] = useState<any>(null);

    useEffect(() => {
        async function init() {
            setAmmFactoryPairs(await client.execute.queryContractSmart(AMM_FACTORY_ADDRESS, { 
                pairs: {}
              }));
          }
        init()
    }, [])

    useEffect(() => {
        async function getData() {
            if (selectedAmmFactoryPairIndex !== null) {
                const responsePromiseAMM = client.execute.queryContractSmart(ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].contract_addr, { 
                    simulation: {
                        offer_asset: {
                            info: {
                                ...ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].asset_infos[0]
                            },
                            amount: "" + Math.pow(10, tokensData.find((data: any) => data.dst_address === ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].asset_infos[0].token.contract_addr).decimals)
                        }
                    }
                  })
                const responsePromiseOrderBook = client.execute.queryContractSmart(ORDERS_FACTORY_ADDRESS, { 
                    secret_order_books: {
                        contract_address: ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].contract_addr
                    }
                  })

                const [responseAMM, responseOrderBook] = await Promise.all([responsePromiseAMM,responsePromiseOrderBook]);

                setSelectedAmmPairPrice(responseAMM)
                setOrderBookPair({
                    isInstanciated: responseOrderBook.secret_order_books.secret_order_book ? true : false,
                    data: responseOrderBook.secret_order_books.secret_order_book
                })
            }
        }
        getData()
    }, [selectedAmmFactoryPairIndex])

    const displaySymbolPair = (pair: any) => {
        const token1Address = pair.asset_infos[0].token ? pair.asset_infos[0].token.contract_addr : pair.asset_infos[0].native_token.denom;
        const token1Data = tokensData.find((data: any) => data.dst_address === token1Address);
        const token2Address = pair.asset_infos[1].token ? pair.asset_infos[1].token.contract_addr : pair.asset_infos[1].native_token.denom;
        const token2Data = tokensData.find((data: any) => data.dst_address === token2Address);
        
        return (token1Data ? token1Data.display_props.symbol : token1Address) + " / " + (token2Data ? token2Data.display_props.symbol : token2Address)
    }

    return (
        <div>
            <Button onClick={() => setShowCreateLimitOrderModal(true)}>Create New Limit Order</Button>
            <Modal show={showCreateLimitOrderModal} onHide={() => setShowCreateLimitOrderModal(false)}>
                <Modal.Header closeButton>
                <Modal.Title>Create Limit Order</Modal.Title>
                </Modal.Header>
                <Modal.Body>
                    {
                        ammFactoryPairs && 
                            <DropdownButton id="dropdown-basic-button" title="Dropdown button">
                                {
                                    ammFactoryPairs!.pairs.map((pair: any, index: number) =>
                                        <Dropdown.Item key={pair.contract_addr} onClick={() => setSelectedAmmFactoryPairIndex(index)}>
                                            {
                                                displaySymbolPair(pair)
                                            }
                                        </Dropdown.Item>
                                    )
                                }   
                            </DropdownButton>
                    }
                    {
                        selectedAmmFactoryPairIndex !== null &&
                        <div>
                            { displaySymbolPair(ammFactoryPairs.pairs[selectedAmmFactoryPairIndex]) }
                            <br/>
                            { 
                                selectedAmmPairPrice ?
                                    (selectedAmmPairPrice.return_amount / Math.pow(10, tokensData.find((data: any) => data.dst_address === ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].asset_infos[1].token.contract_addr).decimals))
                                    :
                                    <Spinner animation="border"/>
                                }
                            <br/>
                            <label>Price</label><br/>
                            <input onChange={(e) => setLimitOrderPriceInput(e.target.value)}></input><br/>
                            <label>Amount</label><br/>
                            <input onChange={(e) => setLimitOrderAmountInput(e.target.value)}></input><br/>
                            <br/>
                            { orderBookPair.isInstanciated === null && <Spinner animation="border"/> }
                            { orderBookPair.isInstanciated === true && 
                                <Button onClick={async() => {
                                        await client.execute.execute(
                                            ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].asset_infos[0].token.contract_addr,
                                            { 
                                                send: {
                                                    recipient: orderBookPair.data.contract_addr,
                                                    amount: "" + limitOrderAmountInput*Math.pow(10, tokensData.find((data: any) => data.dst_address === ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].asset_infos[0].token.contract_addr).decimals),
                                                    msg: btoa(JSON.stringify({
                                                        create_limit_order: {
                                                            is_bid: true,
                                                            price: "" + limitOrderPriceInput*Math.pow(10, tokensData.find((data: any) => data.dst_address === ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].asset_infos[1].token.contract_addr).decimals)
                                                        }
                                                    }))
                                                } 
                                            }
                                        )
                                    }
                                        
                                }> 
                                    Create Create Limit Order
                                </Button>
                            }
                            { orderBookPair.isInstanciated === false && 
                                <Button onClick={async() => {
                                    const hash = await client.execute.getCodeHashByContractAddr(ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].contract_addr)
                                    await client.execute.execute(ORDERS_FACTORY_ADDRESS,  { 
                                        new_secret_order_book_instanciate: {
                                            amm_pair_address: ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].contract_addr,
                                            amm_pair_hash: hash
                                        } 
                                    })
                                }}> 
                                    Instanciate Order Book Pair
                                </Button>
                            }
                        </div>
                    }
                </Modal.Body>
                <Modal.Footer>
                <Button variant="secondary" onClick={() => setShowCreateLimitOrderModal(false)}>
                    Close
                </Button>
                <Button variant="primary" onClick={() => setShowCreateLimitOrderModal(false)}>
                    Save Changes
                </Button>
                </Modal.Footer>
            </Modal>
        </div>
        )
}

type CreateNewLimitOrderProps = {
    AMM_FACTORY_ADDRESS: string,
    ORDERS_FACTORY_ADDRESS: string,
    client: any,
    tokensData: any,
    viewKey: string | null
}

