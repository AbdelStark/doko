#!/bin/bash

bitcoin-cli -signet -rpcconnect=34.10.114.163 -rpcuser=catnet -rpcpassword=stark -rpcwallet=doko_signing importdescriptors '[
  {
    "desc": "wpkh(L3f9ZfW6dwwfh17e7VsPA4cknnU5Nwhg6jrSFgEcnuuExNSSrZij)",
    "timestamp": "now",
    "label": "specter_key_0",
    "active": true
  },
  {
    "desc": "wpkh(Kx5e834ErFW4uRt539R6WRUntkVUmSV7FYd7kh9aEFDrUe49GvHx)",
    "timestamp": "now",
    "label": "specter_key_1",
    "active": true
  },
  {
    "desc": "wpkh(L5LbnhMkpDGcGZE1891SvrGSLbEbLUgrUygcatBgZcvW1wnWaRYT)",
    "timestamp": "now",
    "label": "specter_key_2",
    "active": true
  },
  {
    "desc": "wpkh(Kz9MrRzHGit5UeD1gXgotQvUAvBFsHf8kmt9KU215RG1PMuPpuEf)",
    "timestamp": "now",
    "label": "specter_key_3",
    "active": true
  },
  {
    "desc": "wpkh(L3AaW9zbpQuoXQLStWvCXceacSFh7zEL9sBHEJDzmu71LmEH13iz)",
    "timestamp": "now",
    "label": "specter_key_4",
    "active": true
  }
]'
