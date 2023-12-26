# Build rust with rust base image
FROM rust:1.73.0-bullseye AS builder

RUN apt-get update && apt-get install -y unixodbc-dev unixodbc

# Build rust inside /tmp/build/ directory
WORKDIR /tmp/build

COPY . .

RUN cargo build --release

# Actual running image
#FROM 851255665500.dkr.ecr.ap-southeast-1.amazonaws.com/seceng/base-images/debian:11-slim.21 AS app
FROM debian:11-slim AS app

USER root

# Prepare for basic folder structures
RUN mkdir -p /home/nexus

# This is required by unixodbc logging
RUN mkdir -p /home/nexus/logs

# add non root user
#RUN groupmod -n nexus nonroot && usermod -d /home/nexus -l nexus nonroot
#RUN useradd -rm -d /home/nexus -s /bin/bash -g root -G sudo -u 1001 nexus

# make sure that the nexus user is able to operate on /home/nexus and /etc folders
#RUN chown -R nexus:nexus /home/nexus && chown -R nexus:nexus . && chown -R nexus:nexus /etc
#RUN chown -R nexus /home/nexus && chown -R nexus . && chown -R nexus /etc

# install required packages
RUN  apt-get update && apt-get --no-install-recommends --yes upgrade && \
	apt-get install -y git curl tini unixodbc-dev unixodbc wget jq &&  \
    apt-get install -y glibc-source && \
	rm -rf /var/lib/apt/lists/**

WORKDIR /tmp

# install unixodbc snowflake driver
RUN wget -O snowflake_odbc.tgz "https://sfc-repo.snowflakecomputing.com/odbc/linux/3.1.3/snowflake_linux_x8664_odbc-3.1.3.tgz" && \
    gunzip snowflake_odbc.tgz && \
    tar -xvf snowflake_odbc.tar && \
    mv snowflake_odbc /usr/lib/snowflake_odbc && \
    rm snowflake_odbc.tar

# uninstall wget since it is not used
RUN apt-get remove -y --auto-remove wget

# running setup
WORKDIR /usr/lib/snowflake_odbc

RUN ./unixodbc_setup.sh

WORKDIR /etc

# overwrite the odbc related config files. Note that odbc.ini file will be modified in entrypoint.sh
COPY odbc_config/etc/odbc.ini.tmp odbc.ini
COPY odbc_config/etc/odbcinst.ini odbcinst.ini

WORKDIR /usr/lib/snowflake_odbc/lib

# overwrite Snowflake driver config
COPY odbc_config/lib/simba.snowflake.ini simba.snowflake.ini

RUN mkdir config
COPY config /home/nexus/config

WORKDIR /home/nexus

# copy the executatble application over
COPY --from=builder /tmp/build/target/release/nexus /home/nexus/nexus

COPY bin bin

RUN chmod +x ./nexus && chmod +x bin/entrypoint.sh

USER 1000

#ENTRYPOINT ["tini", "-s", "--", "bin/entrypoint.sh"]
#ENTRYPOINT ["/home/nexus/nexus"]