#!/bin/bash
UI_OUTPUT=ui_output
if [ $1 = "0" ]; then
    PLAYER_KEY=$(grep -A 1 "Received POST response" < $UI_OUTPUT | head -2 | tail -1 | sed -e 's/.*0, \([0-9]*\).*/\1/')
else 
    PLAYER_KEY=$(grep -A 1 "Received POST response" < $UI_OUTPUT | head -2 | tail -1 | sed -e 's/.*1, \([0-9]*\).*/\1/')
fi

cargo run -- --proxy https://icfpc2020-api.testkontur.ru $PLAYER_KEY
