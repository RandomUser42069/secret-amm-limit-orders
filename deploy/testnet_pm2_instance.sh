pm2 start ../orderTriggerer/testnet/trigger_sSCRT_sETH.sh
pm2 start ../orderTriggerer/testnet/trigger_sSCRT_sOCEAN.sh
pm2 serve ../www/build 80 --spa 