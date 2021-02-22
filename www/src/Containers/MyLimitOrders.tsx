import React, {useState,useEffect} from 'react';
import {Card, Button, Spinner, Modal, DropdownButton, Dropdown, Table} from 'react-bootstrap'

// eslint-disable-next-line import/no-anonymous-default-export
export default ({
    ORDERS_FACTORY_ADDRESS,
    tokensData,
    client,
    viewKey
}: MyLimitOrdersProps) => {
    const [myLimitOrders, setMyLimitOrders] = useState<any>(null)

    useEffect(() => {
        async function init() {
            setMyLimitOrders(await client.execute.queryContractSmart(ORDERS_FACTORY_ADDRESS, { 
                user_secret_order_books: {
                    address: client.accountData.address,
                    viewing_key: viewKey
                }
              }))
          }
        init()
    }, [])

    return (
        <Table striped bordered hover>
            <thead>
                <tr>
                    <th>Creation Date</th>
                    <th>Pair</th>
                    <th>Status</th>
                    <th>Limit Order</th>
                    <th>Current Price</th>
                    <th>Withdraw</th>
                </tr>
            </thead>
            <tbody>
            {!myLimitOrders && <Spinner animation="border"/>}
            {
                myLimitOrders && myLimitOrders.user_secret_order_books.user_secret_order_book &&
                    myLimitOrders.user_secret_order_books.user_secret_order_book.map((orderBookAddress: string) => 
                        <MyLimitOrder 
                            orderBookAddress={orderBookAddress}
                            tokensData={tokensData}
                            client={client}
                            viewKey={viewKey}
                            myLimitOrders={myLimitOrders}
                            setMyLimitOrders={setMyLimitOrders}
                        />)
            }
            </tbody>
        </Table>
    )
}


const MyLimitOrder = ({
    orderBookAddress,
    tokensData,
    client,
    viewKey,
    myLimitOrders,
    setMyLimitOrders
}: any) => {
    const [limitOrderData, setLimitOrderData] = useState<any>(null)
    const [orderBookTokensData, setOrderBookTokensData] = useState<any>(null)
    const [ammPriceData, setAmmPriceData] = useState<any>(null)

    useEffect(() => {
        async function init() {
            const limitOrderPromise = client.execute.queryContractSmart(orderBookAddress, { 
                get_limit_order: {
                    user_address: client.accountData.address,
                    user_viewkey: viewKey
                }
              })

            const orderBookTokenDataPromise = client.execute.queryContractSmart(orderBookAddress, { 
                order_book_pair_info: {}
              })

            const [limitOrder, orderBookTokenData] = await Promise.all([limitOrderPromise, orderBookTokenDataPromise]);

            setLimitOrderData(limitOrder)
            setOrderBookTokensData(orderBookTokenData)

            setAmmPriceData(await getAmmPrice(limitOrder.is_bid ? 0 : 1, orderBookTokenData))

            setInterval(async () => {
                setLimitOrderData(await client.execute.queryContractSmart(orderBookAddress, { 
                    get_limit_order: {
                        user_address: client.accountData.address,
                        user_viewkey: viewKey
                    }
                  }));
                setAmmPriceData(await getAmmPrice(limitOrder.is_bid ? 0 : 1, orderBookTokenData))
            },6000)
          }
        init()
    }, [])


    const getAmmPrice = async (assetIndex: number, orderBookTokenData: any) => {
        return client.execute.queryContractSmart(orderBookTokenData.amm_pair_address, { 
            simulation: {
                offer_asset: {
                    info: {
                        token: {
                            ...orderBookTokenData.assets_info[assetIndex].token,
                            viewing_key: ""
                        }
                    },
                    amount: "" + Math.pow(10, tokensData.find((data: any) => data.dst_address === orderBookTokenData.assets_info[assetIndex].token.contract_addr).decimals)
                }
            }
          })
    }

    const displayBalance = (index: number) => {
        const tokenData = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[index].token.contract_addr);
        return Math.round(limitOrderData.balances[index]/Math.pow(10,orderBookTokensData.assets_info[index].decimal_places) * 100000) / 100000 + " " + tokenData.display_props.symbol
    }

    const displayPrice = (type: string) => {
        const token1Data = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[0].token.contract_addr);
        const token2Data = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[1].token.contract_addr);
        if (limitOrderData.is_bid) {
            if (type === "order") {
                return Math.round(limitOrderData.price/Math.pow(10,orderBookTokensData.assets_info[1].decimal_places) * 100000) / 100000 + " " + token2Data.display_props.symbol + " per " + token1Data.display_props.symbol 
            } else if (type === "amm") {
                return Math.round(ammPriceData.return_amount/Math.pow(10,orderBookTokensData.assets_info[1].decimal_places) * 100000) / 100000 + " " + token2Data.display_props.symbol + " per " + token1Data.display_props.symbol 
            }
        } else {
            if (type === "order") {
                return Math.round(limitOrderData.price/Math.pow(10,orderBookTokensData.assets_info[0].decimal_places) * 100000) / 100000 + " " + token1Data.display_props.symbol + " per " + token2Data.display_props.symbol 
            } else if (type === "amm") {
                return Math.round(ammPriceData.return_amount/Math.pow(10,orderBookTokensData.assets_info[0].decimal_places) * 100000) / 100000 + " " + token1Data.display_props.symbol + " per " + token2Data.display_props.symbol 
            }
        }
    }

    const pairDisplay = () => {
        const token1Data = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[0].token.contract_addr);
        const token2Data = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[1].token.contract_addr);

        return token1Data.display_props.symbol + " / " + token2Data.display_props.symbol 
    }

    const displayDescription = () => {
        let buying = "Buying "
        let selling = ", selling "
        if (limitOrderData.is_bid) {
            const tokenDataBuy = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[1].token.contract_addr);
            const tokenDataSell = tokensData.find((data: any) => data.dst_address === orderBookTokensData.assets_info[0].token.contract_addr);
            buying = buying + Math.round(limitOrderData.price/Math.pow(10,orderBookTokensData.assets_info[1].decimal_places) * 100000) / 100000 * Math.round(limitOrderData.order_token_init_quant/Math.pow(10,orderBookTokensData.assets_info[0].decimal_places) * 100000) / 100000 + " " + tokenDataBuy.display_props.symbol
            selling= selling + Math.round(limitOrderData.order_token_init_quant/Math.pow(10,orderBookTokensData.assets_info[0].decimal_places) * 100000) / 100000 + " " + tokenDataSell.display_props.symbol
        }

        return buying + selling
    }

    const rowStyle = limitOrderData ? {
        backgroundColor: limitOrderData.status === "Filled" ? "#Cfffbc" : "#Fff4ad"
    } : undefined
    
    return (
        <tr key={orderBookAddress} style={rowStyle}>
            {limitOrderData && <td>{new Date(limitOrderData.timestamp*1000).toLocaleString()}</td>}
            {limitOrderData && orderBookTokensData && <td>{pairDisplay()}</td>}
            {limitOrderData && <td>{limitOrderData.status}</td>}
            {limitOrderData && orderBookTokensData && 
                <div>
                    {displayPrice("order")} <br/><br/>
                    {displayDescription()}
                </div>}
            {limitOrderData && ammPriceData && <td>{displayPrice("amm")}</td>}
            {limitOrderData && <td>{<Button onClick={ async () => {
                try{
                    await client.execute.execute(orderBookAddress, { 
                        withdraw_limit_order: {}
                    })
                    let update = {...myLimitOrders}
                    let arr = update.user_secret_order_books.user_secret_order_book.filter((address: string) => address !== orderBookAddress)
                    update.user_secret_order_books.user_secret_order_book = arr
                    setMyLimitOrders(update)
                } catch (e) {
                    alert("error on widthdraw: " + e)
                }
            }}>
                Widthdraw <br/>
                {limitOrderData && orderBookTokensData && displayBalance(0) + "  and  " + displayBalance(1)}
            </Button>}</td>}
        </tr>
    )
}

type MyLimitOrdersProps = {
    ORDERS_FACTORY_ADDRESS: string,
    client: any,
    tokensData: any,
    viewKey: string | null
}
