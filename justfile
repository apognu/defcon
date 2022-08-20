set dotenv-load

run:
  parallel --ungroup ::: 'just backend' 'just frontend'

backend:
  JQ_LIB_DIR=/usr/bin SCRIPTS_PATH=/tmp cargo run --all-features

frontend:
  yarn --cwd assets/ build --watch --mode development

migrate:
  cargo run migrate

test: test-frontend test-backend

test-backend:
  sudo capsh --caps='cap_net_raw+eip cap_setpcap,cap_setuid,cap_setgid+ep' --keep=1 --user="$(whoami)" --addamb=cap_net_raw -- -c "JQ_LIB_DIR=/usr/lib DSN=$TEST_DSN cargo test --all-features"

test-frontend:
  yarn --cwd assets/ lint
