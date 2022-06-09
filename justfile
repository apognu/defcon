set dotenv-load

run:
  parallel --ungroup ::: 'just backend' 'just frontend'

backend:
  JQ_LIB_DIR=/usr/bin SCRIPTS_PATH=/tmp cargo run --all-features

frontend:
  yarn --cwd assets/ build --watch --mode development

migrate:
  cargo run migrate
