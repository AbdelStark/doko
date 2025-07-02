#!/bin/bash

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark createwallet "specter_recovery" false false "" false true

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery importprivkey "L3f9ZfW6dwwfh17e7VsPA4cknnU5Nwhg6jrSFgEcnuuExNSSrZij" "specter_key_0" false

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery importprivkey "Kx5e834ErFW4uRt539R6WRUntkVUmSV7FYd7kh9aEFDrUe49GvHx" "specter_key_1" false

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery importprivkey "L5LbnhMkpDGcGZE1891SvrGSLbEbLUgrUygcatBgZcvW1wnWaRYT" "specter_key_2" false

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery importprivkey "Kz9MrRzHGit5UeD1gXgotQvUAvBFsHf8kmt9KU215RG1PMuPpuEf" "specter_key_3" false

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery importprivkey "L3AaW9zbpQuoXQLStWvCXceacSFh7zEL9sBHEJDzmu71LmEH13iz" "specter_key_4" false

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery importprivkey "KzrM1QSeVH89UwVJ3fud7sdbMBTKMYa7Gi34aNkdpfcAowPFBWRK" "specter_key_5" false

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery importprivkey "L1prGtFTpNYiSeZM6XnYJvojEniouDqc4hoDktZiXi9wSYd7kR2V" "specter_key_6" false

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery importprivkey "KxsDKg8mqYBw376hoex5EDrwon1kxXjzreTPsHnY6KMreejBnxuv" "specter_key_7" false

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery importprivkey "L1yBQ7kwwmLiUyheCEbtJVNGwFVfp7fpAqrj9HbajhJxou5PzK8j" "specter_key_8" false

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery importprivkey "Kz4HmF34w1rStuv6CERADgzjymkhEMo3XXFgPqXCZuPoEUkgucYk" "specter_key_9" false

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=specter_recovery rescanblockchain