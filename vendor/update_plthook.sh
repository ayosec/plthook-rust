#!/bin/bash
#
# Download source files for the latest version of plthook.
#
# It requires the bsdtar and curl/wget commands.
#
# Usage:
#
#   $ cd vendor
#   $ ./update_plthook.sh [branch]

set -euo pipefail

if [ -z "$(command -v bsdtar)" ]
then
  echo Missing bsdtar command.
  exit 1
fi

BRANCH=${1:-master}

download() {
  if [ -n "$(command -v wget)" ]
  then
    wget -qO - "$1"
  elif [ -n "$(command -v curl)" ]
  then
    curl -sL "$1"
  else
    echo "Both wget and curl are missing." 1>&2
    exit 1
  fi
}

cd "$(dirname "$0")"
mkdir -p plthook
cd plthook

download "https://github.com/kubo/plthook/archive/refs/heads/$BRANCH.zip" | \
  bsdtar --strip-components 1 -xvf - \
    "plthook-$BRANCH"/plthook{.h,_elf.c,_osx.c,_win32.c}
