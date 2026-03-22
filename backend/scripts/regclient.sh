#!/usr/bin/env bash

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

OCAS_URL="http://localhost:3143"
CLIENT_NAME="cwdb"
REDIRECT_URI="http://localhost:3000/api/auth/callback"

echo "registering $CLIENT_NAME

RESPONSE=$(curl -s -X POST "$OCAS_URL/clients" \
    -H "Content-Type: application/json" \
    -d "{\"name\": \"$CLIENT_NAME\", \"redirecturis\": [\"$REDIRECT_URI\"]}")

if [ $? -ne 0 ]; then
    echo -e "${RED}Error: Failed to connect to $OCAS_URL${NC}"
    exit 1
fi

ERROR=$(echo "$RESPONSE" | grep -o '"error":"[^"]*' | cut -d'"' -f4)
if [ -n "$ERROR" ]; then
    echo -e "${RED}server error: $ERROR${NC}"
    exit 1
fi

CLIENT_ID=$(echo "$RESPONSE" | grep -o '"clientid":"[^"]*' | cut -d'"' -f4)
SECRET=$(echo "$RESPONSE" | grep -o '"secret":"[^"]*' | cut -d'"' -f4)

echo -e "${GREEN}success. add to backend/.env:${NC}\n"
echo "OCAS_CLIENT_ID=$CLIENT_ID"
echo "OCAS_CLIENT_SECRET=$SECRET"
