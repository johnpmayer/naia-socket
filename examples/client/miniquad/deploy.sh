#!/bin/bash
xdg-open http://localhost:3179/

# replace 'client' & 'webserver' below with the appropriate directory names for your project
client='naia-interop-demo'
webserver_dir='dev-http-server'

get_reload_actions(){
  local OUTPUT=''
  local c=$1
  local w=$2
  FMT='rm -rfv %s/dist/** &&
  cp -a static/. %s/dist/ &&
  cargo build --target wasm32-unknown-unknown --bin %s &&
  cp target/wasm32-unknown-unknown/debug/%s.wasm %s/dist/%s.wasm &&
  cd %s &&
  cargo run --bin %s'
  printf -v OUTPUT "$FMT" $w $w $c $c $w $c $w $w
  echo $OUTPUT
}

actions="$(get_reload_actions $client $webserver_dir)"
watchexec -r -s SIGKILL --ignore $webserver_dir/dist --ignore target --clear "$actions"

#FMT='rm -rfv %s/dist/** &&
#  cp -a static/. %s/dist/ &&
#  cargo build --target wasm32-unknown-unknown --bin %s &&
#  cp target/wasm32-unknown-unknown/debug/%s.wasm %s/dist/%s.wasm &&
#  $ (cd %s && cargo run --bin %s)'