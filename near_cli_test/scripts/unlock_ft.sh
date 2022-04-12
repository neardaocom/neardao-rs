#!/bin/bash

near call $DCID unlock_tokens '{"group": "Council"}' --accountId=$CID4
#near call $DCID unlock_tokens '{"group": "Community"}' --accountId=$CID4
#near call $DCID unlock_tokens '{"group": "Foundation"}' --accountId=$CID4

near view $DCID statistics_ft ''