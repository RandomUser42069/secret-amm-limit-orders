import React, { useState, useEffect } from "react";
import MyActiveLimitOrders from "./MyActiveLimitOrders"
import MyHistoryLimitOrders from "./MyHistoryLimitOrders"

import {
    Card,
    Button,
    Spinner,
    Modal,
    DropdownButton,
    Dropdown,
    Table,
} from "react-bootstrap";

// eslint-disable-next-line import/no-anonymous-default-export
export default ({
    ORDERS_FACTORY_ADDRESS,
    remountMyLimitOrders,
    tokensData,
    client,
    viewKey,
}: any) => {
    const [secretOrderBooks, setSecretOrderBooks] = useState<any>(null);
    const [selectedFactoryPairIndex, setSelectedFactoryPairIndex] = useState<any>(
        null
    );

    useEffect(() => {
        async function init() {
            const response = await client.execute.queryContractSmart(
                ORDERS_FACTORY_ADDRESS,
                {
                    secret_order_books: {},
                }
            );
            setSecretOrderBooks(response.secret_order_books);
        }
        init();
    }, []);

    const displaySymbolPair = (pair: any) => {
        const token1Address = pair.asset_infos[0].token
            ? pair.asset_infos[0].token.contract_addr
            : pair.asset_infos[0].native_token.denom;
        const token1Data = tokensData.find(
            (data: any) => data.dst_address === token1Address
        );
        const token2Address = pair.asset_infos[1].token
            ? pair.asset_infos[1].token.contract_addr
            : pair.asset_infos[1].native_token.denom;
        const token2Data = tokensData.find(
            (data: any) => data.dst_address === token2Address
        );
        return (
            (token1Data ? token1Data.display_props.symbol : token1Address) +
            " / " +
            (token2Data ? token2Data.display_props.symbol : token2Address)
        );
    };

    return (
        <div>
            <br/><br/>
            {
                tokensData && secretOrderBooks && (
                    <DropdownButton
                        id="dropdown-basic-button"
                        title={
                            selectedFactoryPairIndex !== null
                                ? displaySymbolPair(
                                    secretOrderBooks.secret_order_books[selectedFactoryPairIndex]
                                )
                                : "Select Pair..."
                        }
                    >
                        {secretOrderBooks!.secret_order_books.map(
                            (pair: any, index: number) => (
                                <Dropdown.Item
                                    key={pair.contract_addr}
                                    onClick={() => {
                                        setSelectedFactoryPairIndex(index);
                                    }}
                                >
                                    {displaySymbolPair(pair)}
                                </Dropdown.Item>
                            )
                        )}
                    </DropdownButton>
                )}
            <br/>
            {
                selectedFactoryPairIndex !== null && 
                    <MyActiveLimitOrders 
                        key={"active_" + selectedFactoryPairIndex}
                        remountMyLimitOrders={remountMyLimitOrders}
                        ORDERS_FACTORY_ADDRESS={ORDERS_FACTORY_ADDRESS}
                        tokensData={tokensData}
                        client={client}
                        viewKey={viewKey}
                        pair={secretOrderBooks.secret_order_books[selectedFactoryPairIndex]}
                    />
            }
            <br />
            <br />
            <br />
            {
                    selectedFactoryPairIndex !== null && <MyHistoryLimitOrders 
                      key={"history_" + selectedFactoryPairIndex}
                      remountMyLimitOrders={remountMyLimitOrders}
                      ORDERS_FACTORY_ADDRESS={ORDERS_FACTORY_ADDRESS}
                      tokensData={tokensData}
                      client={client}
                      viewKey={viewKey}
                      pair={secretOrderBooks.secret_order_books[selectedFactoryPairIndex]}
                    />
            }
        </div>
    );
};
