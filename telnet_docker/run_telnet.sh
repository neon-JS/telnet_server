cd "$(dirname "$0")"

if [ -z "$(docker images -q telnet_docker:latest 2> /dev/null)" ]; then
  docker build --tag telnet_docker .
fi

docker run --rm -it telnet_docker:latest "$@"

cd - &>/dev/null