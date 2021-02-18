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

    const [selectedAmmPairPrice, setSelectedAmmPairPrice] = useState<any>(null)

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
                const response = await client.execute.queryContractSmart(ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].contract_addr, { 
                    simulation: {
                        offer_asset: {
                            info: {
                                ...ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].asset_infos[0]
                            },
                            amount: "" + Math.pow(10, tokensData.find((data: any) => data.dst_address === ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].asset_infos[0].token.contract_addr).decimals)
                        }
                    }
                  })
                  setSelectedAmmPairPrice(response)
            }
        }
        getData()
    }, [selectedAmmFactoryPairIndex])

    //where is the sSCRT on the tokens from the backend ????
    console.log(selectedAmmPairPrice)

    const displaySymbolPair = (pair: any) => {
        const token1Address = pair.asset_infos[0].token.contract_addr;
        const token1Data = tokensData.find((data: any) => data.dst_address === token1Address);
        const token2Address = pair.asset_infos[1].token.contract_addr;
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
                            { selectedAmmPairPrice && (selectedAmmPairPrice.return_amount + selectedAmmPairPrice.spread_amount + selectedAmmPairPrice.commission_amount) / Math.pow(10, tokensData.find((data: any) => data.dst_address === ammFactoryPairs.pairs[selectedAmmFactoryPairIndex].asset_infos[1].token.contract_addr).decimals)}
                            <br/>
                            { "Wanted Price Input"}
                            <br/>
                            { "Button Create Pair or Create Limit Order"}
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
    viewKey: null | string
}

