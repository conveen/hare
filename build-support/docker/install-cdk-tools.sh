#!/usr/bin/env bash

set -e

. /build-support/shell/common/log.sh


if [ -z "${USERNAME}" ]
then
    error "Must set USERNAME environment variable"
    exit 1
fi

# Deno install requires unzip
apt install unzip

# Install Deno to install and run the CDK CLI
sudo -Hiu $USERNAME bash -c 'curl -fsSL https://deno.land/install.sh > /tmp/install-deno.sh && chmod 555 /tmp/install-deno.sh && /tmp/install-deno.sh && mv $HOME/.deno/bin/deno /tmp/deno && rm -rf $HOME/.deno'
chown root:root /tmp/deno
chmod 555 /tmp/deno
mv /tmp/deno /usr/local/bin
rm /tmp/install-deno.sh
sudo -Hiu $USERNAME bash -c "echo 'export PATH=\$HOME/.deno/bin:\$PATH' >> \$HOME/.bashrc"

# Install CDK CLI "globally" (for USERNAME)
sudo -Hiu $USERNAME bash -c '/usr/local/bin/deno install -A -g npm:aws-cdk'