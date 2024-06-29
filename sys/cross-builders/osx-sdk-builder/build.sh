#!/bin/sh

set -e

cd osxcross

./tools/gen_sdk_package_pbzx.sh /build/xcode.xip

mv "/build/osxcross/$SDK_FILENAME" "/out/$SDK_FILENAME" 