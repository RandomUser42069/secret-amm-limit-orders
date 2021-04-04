sudo apt-get update
sudo apt-get install jq nodejs npm
sudo npm i -g pm2 serve
cd ..
mkdir Downloads
cd Downloads
wget https://github.com/enigmampc/SecretNetwork/releases/download/v1.0.4/secretnetwork_1.0.4_amd64.deb
sudo dpkg -i secret*.deb
secretcli config chain-id "holodeck-2"
secretcli config indent true
secretcli config node "http://bootstrap.secrettestnet.io:26657"
secretcli config output "json"
secretcli config trust-node true
cd ../secret-amm-limit-orders/deploy/testnet_pm2_instance.sh
