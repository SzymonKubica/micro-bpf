for chunks in {0..7}
do
 aiocoap-client -m POST 'coap://[fe80::a8e8:48ff:fee0:523c%enp34s0u2u1u2]/benchmark' --payload "$chunks" | tee -a results.txt
 # Add a newline
 echo "" >> results.txt
done
