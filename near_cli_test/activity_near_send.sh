#!/bin/bash

near call $DCID treasury_near_send '{"proposal_id":3,"receiver_id":"petrstudynka.testnet","amount":"1000000000000000000000000"}' --accountId $CID1 --gas $TGAS_100