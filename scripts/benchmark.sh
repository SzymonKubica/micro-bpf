board_ip=$1
sum=0
iterations=100
for ((i=0; i<$iterations; i++));
do
  time=$(./execute-bpf.sh wlp3s0 $board_ip 0)
  echo "Iteration $i: $time [us]"
  sum=$(($sum + $time))
done

echo "Average: $(($sum / $iterations)) [us]"
