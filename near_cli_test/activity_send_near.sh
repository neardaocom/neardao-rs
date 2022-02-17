#!/bin/bash

near call $DCID treasury_send_near '{"proposal_id":2,"receiver_id":"petrstudynka.testnet","amount":"1000000000000000000000000"}' --accountId $CID1 --gas $TGAS_100