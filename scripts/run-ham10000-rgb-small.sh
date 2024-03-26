#!/usr/bin/env bash

num_labels=7
data_bits=8
data_limit=4
data_skip=4
train_data="../data/ham10000/ham10000-rgb-train.csv.xz"
train_limit=1002
train_skip=0
test_data="../data/ham10000/ham10000-rgb-test.csv.xz"
test_limit=2003
test_skip=0
address_size=12
counter_size=10
therm_size=5
therm_type="linear"
activation="binary"
threshold=0
sigma="4.884981308350689e-16"
l=1
bg_bit=23
upper_n=2048
t=2
base_bit=15

cargo run --release -- --num-labels=$num_labels \
    --data-bits=$data_bits --data-limit=$data_limit \
    --data-skip=$data_skip --train-data=$train_data \
    --train-limit=$train_limit --train-skip=$train_skip \
    --test-data=$test_data --test-limit=$test_limit \
    --test-skip=$test_skip --address-size=$address_size \
    --counter-size=$counter_size --therm-size=$therm_size \
    --therm-type=$therm_type --activation=$activation \
    --threshold=$threshold --sigma=$sigma --l=$l --bg-bit=$bg_bit \
    --upper-n=$upper_n --t=$t --base-bit=$base_bit --balance $@
