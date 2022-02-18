#!/bin/bash

near call $DCID treasury_send_ft '{"proposal_id":2,"ft_account_id":"'$DCID'","receiver_id":"petrstudynka.testnet","amount":"420","memo":"test"}' --accountId $CID1 --gas $TGAS_100