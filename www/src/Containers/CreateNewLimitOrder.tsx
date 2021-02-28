import React, {useState,useEffect} from 'react';
import {Card, Button, Spinner, Modal, DropdownButton, Dropdown, Form, Nav} from 'react-bootstrap'

// eslint-disable-next-line import/no-anonymous-default-export
export default ({
    AMM_FACTORY_ADDRESS,
    ORDERS_FACTORY_ADDRESS,
    tokensData,
    client,
    viewKey,
    remountMyLimitOrders
}: CreateNewLimitOrderProps) => {
    const [showCreateLimitOrderModal, setShowCreateLimitOrderModal] = useState(false);
    const [secretOrderBooks, setSecretOrderBooks] = useState<any>(null);
    const [selectedAmmFactoryPairIndex, setSelectedAmmFactoryPairIndex] = useState<any>(null)

    const [createLimitOrderLoading, setCreateLimitOrderLoading] = useState<boolean>(false);
    const [selectedAmmPriceLoading, setSelectedAmmPairPriceLoading] = useState<boolean>(false);
    const [selectedAmmPairPrice, setSelectedAmmPairPrice] = useState<any>(null);

    const [limitOrderIsBidInput, setLimitOrderIsBidInput] = useState<boolean>(true);
    const [limitOrderAmountInput, setLimitOrderAmountInput] = useState<any>(null);
    const [limitOrderPriceInput, setLimitOrderPriceInput] = useState<any>(null);

    useEffect(() => {
        async function init() {
            const response = await client.execute.queryContractSmart(ORDERS_FACTORY_ADDRESS, { 
                secret_order_books: {}
              })
            setSecretOrderBooks(response.secret_order_books);
            /*setAmmFactoryPairs(await client.execute.queryContractSmart(AMM_FACTORY_ADDRESS, { 
                pairs: {}
              }));*/
          }
        init()
    }, [])

    useEffect(() => {
        async function getData() {
            if (selectedAmmFactoryPairIndex !== null) {
                try {
                    const responsePromiseAMM = getAmmPrice();
                    //const responsePromiseOrderBook = getOrderBook();
    
                    const [responseAMM] = await Promise.all([responsePromiseAMM]);
    
                    setSelectedAmmPairPrice(responseAMM)
                    setSelectedAmmPairPriceLoading(false)
                    /*setOrderBookPair({
                        isInstanciated: responseOrderBook.secret_order_books.secret_order_book ? true : false,
                        data: responseOrderBook.secret_order_books.secret_order_book
                    })*/
                } catch(e){alert(e)}
            }
        }
        getData()
    }, [selectedAmmFactoryPairIndex])

    const getAmmPrice = async () => {
        return client.execute.queryContractSmart(secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].amm_pair_contract_addr, { 
            simulation: {
                offer_asset: {
                    info: {
                        token: {
                            ...secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[0].token,
                            viewing_key: ""
                        }
                    },
                    amount: "" + Math.pow(10, tokensData.find((data: any) => data.dst_address === secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[0].token.contract_addr).decimals)
                }
            }
          })
    }

    /*const getOrderBook = async () => {
        return client.execute.queryContractSmart(ORDERS_FACTORY_ADDRESS, { 
            secret_order_books: {
                contract_address: secretOrderBooks.secretOrderBooks[selectedAmmFactoryPairIndex].contract_addr
            }
          })
    }*/

    const getTokenSymbol = (address: string) => {
        const tokenData = tokensData.find((data: any) => data.dst_address === address);
        return (tokenData ? tokenData.display_props.symbol : address)
    } 

    const displaySymbolPair = (pair: any) => {
        const token1Address = pair.asset_infos[0].token ? pair.asset_infos[0].token.contract_addr : pair.asset_infos[0].native_token.denom;
        const token1Data = tokensData.find((data: any) => data.dst_address === token1Address);
        const token2Address = pair.asset_infos[1].token ? pair.asset_infos[1].token.contract_addr : pair.asset_infos[1].native_token.denom;
        const token2Data = tokensData.find((data: any) => data.dst_address === token2Address);
        return (token1Data ? token1Data.display_props.symbol : token1Address) + " / " + (token2Data ? token2Data.display_props.symbol : token2Address) 
    }

    const getCurrentPrice = () => {
        if(selectedAmmPairPrice) {
            const tokenData = tokensData.find((data: any) => data.dst_address === secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[1].token.contract_addr)
            const otherTokenData = tokensData.find((data: any) => data.dst_address === secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[0].token.contract_addr)
            return selectedAmmPairPrice.return_amount / Math.pow(10, tokenData.decimals) + " " + tokenData.display_props.symbol + " per " + otherTokenData.display_props.symbol
        }
                                    
    }

    console.log(tokensData && secretOrderBooks && secretOrderBooks.secret_order_books)

    return (
        <div>
            <Button onClick={() => setShowCreateLimitOrderModal(true)}>Create New Limit Order</Button>
            <Modal show={showCreateLimitOrderModal} onHide={() => {
                setShowCreateLimitOrderModal(false)
                setSelectedAmmFactoryPairIndex(null)
            }}>
                <Modal.Header closeButton>
                <Modal.Title>Create Limit Order</Modal.Title>
                </Modal.Header>
                <Modal.Body>
                    {
                        tokensData && secretOrderBooks && 
                            <DropdownButton id="dropdown-basic-button" title={
                                selectedAmmFactoryPairIndex !== null ? 
                                    displaySymbolPair(secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex])
                                : "Select Pair..."
                            }>
                                {
                                    secretOrderBooks!.secret_order_books.map((pair: any, index: number) =>
                                        <Dropdown.Item key={pair.contract_addr} onClick={() => {
                                            setSelectedAmmPairPriceLoading(true)
                                            setSelectedAmmFactoryPairIndex(index)
                                        }}>
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
                            <br/>
                            { selectedAmmPriceLoading && <Spinner animation="border"/> }
                            { !selectedAmmPriceLoading && selectedAmmPairPrice && getCurrentPrice()}
                            <br/>
                            <br/>
                            <Button style={{marginRight: "20px"}} variant={limitOrderIsBidInput ? "success" : "light"} onClick={() => { setLimitOrderIsBidInput(true)}}> 
                                    Buy 
                            </Button>
                            <Button variant={!limitOrderIsBidInput ? "danger" : "light"} onClick={() => { setLimitOrderIsBidInput(false)}}> 
                                    Sell 
                            </Button>
                            <br/>
                            <br/>
                            <label>{`Price of Limit Order (` + getTokenSymbol(secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[1].token.contract_addr) + ")"}</label><br/>
                            <input onChange={(e) => setLimitOrderPriceInput(e.target.value)}></input><br/>
                            <label>{
                                limitOrderIsBidInput ?
                                `Deposit Amount (` + getTokenSymbol(secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[1].token.contract_addr) + ")" 
                                :
                                `Deposit Amount (` + getTokenSymbol(secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[0].token.contract_addr) + ")"
                            }
                            </label><br/>
                            <input onChange={(e) => setLimitOrderAmountInput(e.target.value)}></input><br/>
                            <label>{
                                limitOrderIsBidInput ?
                                `Expected Amount (` + getTokenSymbol(secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[0].token.contract_addr) + ")" 
                                :
                                `Expected Amount (` + getTokenSymbol(secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[1].token.contract_addr) + ")"
                            }
                            </label><br/>
                            <input disabled value={
                                limitOrderAmountInput && limitOrderPriceInput ? 
                                    (
                                        limitOrderIsBidInput ? ("" + limitOrderAmountInput/limitOrderPriceInput)
                                        : ("" + limitOrderAmountInput*limitOrderPriceInput)
                                    )
                                : ""
                            }></input><br/>
                            <br/>
                            { 
                                <Button 
                                    style={{width: "100%"}}
                                    variant={limitOrderIsBidInput ? "success" : "danger"}
                                    onClick={async() => {
                                        try {
                                            if(
                                                limitOrderAmountInput === null || limitOrderAmountInput === "" || limitOrderAmountInput === "0" ||
                                                limitOrderPriceInput === null || limitOrderPriceInput === "" || limitOrderPriceInput === "0") {
                                                throw Error("Bad Inputs");
                                            }   

                                            const limitOrderExpectedAmount: any = 
                                                limitOrderIsBidInput ? ("" + limitOrderAmountInput/limitOrderPriceInput)
                                                    : ("" + limitOrderAmountInput*limitOrderPriceInput)
                                            // loading
                                            setCreateLimitOrderLoading(true)
                                            await client.execute.execute(
                                                secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[limitOrderIsBidInput ? 1 : 0].token.contract_addr,
                                                { 
                                                    send: {
                                                        recipient: secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].contract_addr,
                                                        amount: "" + Math.floor(limitOrderAmountInput*Math.pow(10, tokensData.find((data: any) => data.dst_address === secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[limitOrderIsBidInput ? 1 : 0].token.contract_addr).decimals)),
                                                        msg: btoa(JSON.stringify({
                                                            create_limit_order: {
                                                                is_bid: limitOrderIsBidInput,
                                                                price: "" + Math.floor(limitOrderPriceInput*Math.pow(10, tokensData.find((data: any) => data.dst_address === secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[1].token.contract_addr).decimals)),
                                                                expected_amount: "" + Math.floor(limitOrderExpectedAmount*Math.pow(10, tokensData.find((data: any) => data.dst_address === secretOrderBooks.secret_order_books[selectedAmmFactoryPairIndex].asset_infos[limitOrderIsBidInput ? 0 : 1].token.contract_addr).decimals)),
                                                            }
                                                        }))
                                                    } 
                                                }
                                            )
                                            setCreateLimitOrderLoading(false)
                                            // sair deste e fazer refresh do outro
                                            setShowCreateLimitOrderModal(false)
                                            setSelectedAmmFactoryPairIndex(null)
                                            remountMyLimitOrders()
                                        } catch (e) {
                                            alert(e)
                                            setCreateLimitOrderLoading(false)
                                        }
                                    }
                                }> 
                                    {
                                        createLimitOrderLoading ? <Spinner animation="border"/> : limitOrderIsBidInput ? "Buy" : "Sell"
                                    }
                                </Button>
                            }
                        </div>
                    }
                </Modal.Body>
                <Modal.Footer>
                <Button variant="secondary" onClick={() => {
                    setSelectedAmmFactoryPairIndex(null)
                    setShowCreateLimitOrderModal(false)
                }}>
                    Close
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
    viewKey: string | null,
    remountMyLimitOrders: any
}

