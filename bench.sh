#!/usr/bin/env bash

depth="$1"
samples="$2"

help()
{
	echo "usage: ${0} DEPTH SAMPLES"
}

[[ -z "${depth}" ]] && help && exit 1
[[ -z "${samples}" ]] && help && exit 1

echo -n "Benchmarking: take ${samples} samples at depth ${depth}... "

sum=0
nums=""

for ((i = 0; i < samples; i++)); do
	echo -n "[$((i+1))] "
	output=$(echo -e "uci\nposition startpos moves e2e4\ngo depth ${depth}\nquit" | target/release/kopyto uci | grep "^info depth ${depth}" | grep -o "nps [0-9]*" | sed 's#nps ##')
	sum=$((sum+output))
	nums="${nums}${output} "
done

echo "done"

numbers=(${nums})
mean=$(echo "scale=10; ${sum} / ${samples}" | bc)
variance_sum=0
for num in "${numbers[@]}"; do
    variance_sum=$(echo "${variance_sum} + (${num} - ${mean})^2" | bc)
done
variance=$(echo "scale=10; ${variance_sum} / ${samples}" | bc)
std_dev=$(echo "scale=10; sqrt($variance)" | bc)

echo "Result: ${mean%%.*} ± ${std_dev%%.*} nps"
