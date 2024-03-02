cd "$(dirname "$0")"

docker build --tag telnet_docker .
docker run --rm -it telnet_docker:latest "$@"

cd - &>/dev/null