#!/usr/bin/env bash

num_labels=10
data_bits=8
data_limit=4
data_skip=4
train_data="../data/mnist/mnist-train.csv.xz"
train_limit=30000
train_skip=0
test_data="../data/mnist/mnist-test.csv.xz"
test_limit=10000
test_skip=0
address_size=14
counter_size=12
therm_size=4
therm_type="log"
activation="bounded-log"
threshold=0
sigma="4.884981308350689e-16"
l=2
bg_bit=15
upper_n=2048
t=3
base_bit=11

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
