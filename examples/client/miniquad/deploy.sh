#!/bin/bash
xdg-open http://localhost:3177/

# replace 'client' & 'webserver' below with the appropriate directory names for your project
working_dir='examples/client/miniquad'
client='naia-client-socket-miniquad-example'
webserver_dir='dev-http-server'

get_reload_actions(){
  local OUTPUT=''
  local d=$1
  local c=$2
  local w=$3
  FMT='rm -rf %s/%s/dist &&
  mkdir %s/%s/dist &&
  cargo build --target wasm32-unknown-unknown --bin %s &&
  cp target/wasm32-unknown-unknown/debug/%s.wasm %s/%s/dist/%s.wasm &&
  cp -a %s/static/. %s/%s/dist/ &&
  cd %s/%s &&
  cargo run --bin %s'
  printf -v OUTPUT "$FMT" $d $w $d $w $c $c $d $w $c $d $d $w $d $w $w
  echo $OUTPUT
}

actions="$(get_reload_actions $working_dir $client $webserver_dir)"
watchexec -r -s SIGKILL --ignore $webserver_dir/dist --ignore target --clear "$actions"