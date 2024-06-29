FROM debian:bookworm-slim

RUN apt update && apt install -y git cmake libxml2-dev fuse clang lzma-dev libssl-dev liblzma-dev cpio libbz2-dev bzip2

WORKDIR /build
ADD --chmod=500 build.sh build.sh

RUN git clone https://github.com/tpoechtrager/osxcross.git

ENTRYPOINT ["./build.sh"]
