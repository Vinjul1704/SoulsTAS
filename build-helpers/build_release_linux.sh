#!/bin/bash

# Store some paths
ORIG_DIR=$(pwd)
SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd) # https://stackoverflow.com/a/246128


# Build the main EXEs
cd $SCRIPT_DIR/../

XWIN_ARCH="x86_64,x86" cargo xwin build --release --target x86_64-pc-windows-msvc
ERR_CODE=$?
if [ $ERR_CODE != 0 ]; then
	cd $ORIG_DIR
	exit $ERR_CODE
fi

XWIN_ARCH="x86_64,x86" cargo xwin build --release --target i686-pc-windows-msvc
ERR_CODE=$?
if [ $ERR_CODE != 0 ]; then
	cd $ORIG_DIR
	exit $ERR_CODE
fi


# Build the soulstas-patches DLLs
cd $SCRIPT_DIR/../lib/soulstas-patches/

XWIN_ARCH="x86_64,x86" cargo xwin build --release --target x86_64-pc-windows-msvc
ERR_CODE=$?
if [ $ERR_CODE != 0 ]; then
	cd $ORIG_DIR
	exit $ERR_CODE
fi

XWIN_ARCH="x86_64,x86" cargo xwin build --release --target i686-pc-windows-msvc
ERR_CODE=$?
if [ $ERR_CODE != 0 ]; then
	cd $ORIG_DIR
	exit $ERR_CODE
fi


# Build the soulmods DLL
cd $SCRIPT_DIR/../lib/SoulSplitter/src/soulmods/

XWIN_ARCH="x86_64,x86" cargo xwin build --release --target x86_64-pc-windows-msvc
ERR_CODE=$?
if [ $ERR_CODE != 0 ]; then
	cd $ORIG_DIR
	exit $ERR_CODE
fi


# Create fresh build dir
rm -rf $SCRIPT_DIR/build-release
mkdir $SCRIPT_DIR/build-release

# Copy built files
cp $SCRIPT_DIR/../target/x86_64-pc-windows-msvc/release/soulstas.exe $SCRIPT_DIR/build-release/soulstas_x64.exe
cp $SCRIPT_DIR/../target/i686-pc-windows-msvc/release/soulstas.exe $SCRIPT_DIR/build-release/soulstas_x86.exe

cp $SCRIPT_DIR/../lib/soulstas-patches/target/x86_64-pc-windows-msvc/release/soulstas_patches.dll $SCRIPT_DIR/build-release/soulstas_patches_x64.dll
cp $SCRIPT_DIR/../lib/soulstas-patches/target/i686-pc-windows-msvc/release/soulstas_patches.dll $SCRIPT_DIR/build-release/soulstas_patches_x86.dll

cp $SCRIPT_DIR/../lib/SoulSplitter/target/x86_64-pc-windows-msvc/release/soulmods.dll $SCRIPT_DIR/build-release/soulmods_x64.dll


# Go back into original dir
cd $ORIG_DIR