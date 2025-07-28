#!/bin/env bash

FDB_VERSION=${1}

# install clients
wget "https://github.com/apple/foundationdb/releases/download/${FDB_VERSION}/foundationdb-clients-${FDB_VERSION}-1.el7.x86_64.rpm"
dnf install -y  "./foundationdb-clients-${FDB_VERSION}-1.el7.x86_64.rpm"
rm "foundationdb-clients-${FDB_VERSION}-1.el7.x86_64.rpm"

# install server
wget "https://github.com/apple/foundationdb/releases/download/${FDB_VERSION}/foundationdb-server-${FDB_VERSION}-1.el7.x86_64.rpm"
dnf install -y "./foundationdb-server-${FDB_VERSION}-1.el7.x86_64.rpm"
rm "foundationdb-server-${FDB_VERSION}-1.el7.x86_64.rpm"

dnf install jq -y

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# install build libs

wget https://www.rpmfind.net/linux/almalinux/9.6/CRB/x86_64/os/Packages/jq-devel-1.6-17.el9_6.2.x86_64.rpm
dnf install ./jq-devel-1.6-17.el9_6.2.x86_64.rpm
rm jq-devel-1.6-17.el9_6.2.x86_64.rpm


wget https://rpmfind.net/linux/centos-stream/9-stream/BaseOS/x86_64/os/Packages/oniguruma-6.9.6-1.el9.5.x86_64.rpm
dnf install ./oniguruma-6.9.6-1.el9.5.x86_64.rpm
rm oniguruma-6.9.6-1.el9.5.x86_64.rpm

wget https://rpmfind.net/linux/centos-stream/9-stream/CRB/x86_64/os/Packages/oniguruma-devel-6.9.6-1.el9.5.x86_64.rpm
dnf install ./oniguruma-devel-6.9.6-1.el9.5.x86_64.rpm
rm oniguruma-devel-6.9.6-1.el9.5.x86_64.rpm