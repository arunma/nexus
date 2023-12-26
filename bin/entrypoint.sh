#!/bin/bash

set -u

DEV_SF_ACCT=gxs-dev
STG_SF_ACCT=gxs-stg
PROD_SF_ACCT=gxs-prod

# Fetch Snowflake credential
K8S_TOKEN_PATH=/var/run/secrets/kubernetes.io/serviceaccount/token

# Check if K8S_TOKEN_PATH exists to determine if it is running in EKS environment
if [ -f "$K8S_TOKEN_PATH" ]; then
  k8s_token=$(cat $K8S_TOKEN_PATH)
  vault_role=$VAULT_ROLE                                     # pre-defined
  vault_server_address=$VAULT_SERVER_ADDRESS                 # pre-defined
  k8s_auth_path=$K8S_AUTH_PATH                               # pre-defined
  vault_secret_path=$VAULT_SECRET_PATH                       # pre-defined
  vault_slack_conn_secret_path=$VAULT_SLACK_CONN_SECRET_PATH # pre-defined
  vault_engine=$VAULT_ENGINE                                 # pre-defined
  vault_secret_jq_pattern='.data.data'

  if [ "$vault_engine" = "DB" ]; then
    vault_secret_jq_pattern='.data'
  fi

  login_path="$vault_server_address/v1/auth/$k8s_auth_path/login"

  echo "Login to get client_token..."
  login_resp=$(curl \
    --fail \
    --silent \
    --request POST \
    --data "{ \"jwt\": \"$k8s_token\", \"role\": \"$vault_role\" }" \
    $login_path)

  if [ $? -ne 0 ] ; then
    echo "Failed to login Vault server at $login_path"
    curl -sf -XPOST http://127.0.0.1:15020/quitquitquit
    exit 1
  fi

  client_token=$(echo $login_resp | jq -r '.auth.client_token')

  secret_path=$vault_server_address/v1/$vault_secret_path
  creds_resp=$(curl --fail --silent --header "X-Vault-Token: $client_token" $secret_path)
  if [ $? -ne 0 ] ; then
    echo "Failed to fetch Vault token at $login_path"
    curl -sf -XPOST http://127.0.0.1:15020/quitquitquit
    exit 1
  fi

  username=$(echo $creds_resp | jq -r "$vault_secret_jq_pattern.username")
  password=$(echo $creds_resp | jq -r "$vault_secret_jq_pattern.password")

  deploy_env=$(echo $DEPLOY_ENV | tr '[:lower:]' '[:upper:]') # pre-defined
  sf_acct=${deploy_env}_SF_ACCT
  sf_url=${!sf_acct}.snowflakecomputing.com

  sed -i "s/^server=.*/server=$sf_url/g" /etc/odbc.ini
  sed -i "s/^uid=.*/uid=$username/g" /etc/odbc.ini
  sed -i "s/^pwd=.*/pwd=$password/g" /etc/odbc.ini
  sed -i "s/^role=.*/role=$SF_ROLE/g" /etc/odbc.ini    # pre-defined
  sed -i "s/^warehouse=.*/role=$SF_WH/g" /etc/odbc.ini # pre-defined

  # for slack integration
  if [ ! -z "$vault_slack_conn_secret_path" ]; then
    secret_path=$vault_server_address/v1/$vault_slack_conn_secret_path
    creds_resp=$(curl --fail --silent --header "X-Vault-Token: $client_token" $secret_path)
    if [ $? -ne 0 ] ; then
      echo "Failed to get slack connection string"
      curl -sf -XPOST http://127.0.0.1:15020/quitquitquit
      exit 1
    fi

    base_uri=$(echo $creds_resp | jq -r ".data.data.base_uri")
    token=$(echo $creds_resp | jq -r ".data.data.token")
    export SLACK_WEBHOOK_BASE_URI=$base_uri
    export SLACK_WEBHOOK_TOKEN=$token
  fi
fi


# Execute the actual job command, such as `dbt run -m <model_name>`
eval "$@"
exitcode=$?

curl -sf -XPOST http://127.0.0.1:15020/quitquitquit

exit $exitcode
