NDK_STANDALONE = /Users/miku/Library/Android/sdk/ndk/$(NDK_VERSION)
ANDROID_DEST = /Users/miku/project/airx4a/airx/android/app/src/main/jniLibs
NDK_VERSION = 25.1.8937393
HOST_ARCH = darwin-x86_64

NAME = libairx
API = 33

ARCHS_ANDROID = aarch64-linux-android i686-linux-android x86_64-linux-android
LIB = $(NAME).a
SO = $(NAME).so
CC_PATH = $(NDK_STANDALONE)/toolchains/llvm/prebuilt/$(HOST_ARCH)/bin

all: android

android: $(ARCHS_ANDROID)

clean:
	rm -rf target

install-android: android
	mkdir -p $(ANDROID_DEST)
	mkdir -p $(ANDROID_DEST)/x86
	mkdir -p $(ANDROID_DEST)/x86_64
	mkdir -p $(ANDROID_DEST)/arm64-v8a

	cp ./target/aarch64-linux-android/release/deps/$(SO) $(ANDROID_DEST)/arm64-v8a/$(SO)
	cp ./target/i686-linux-android/release/deps/$(SO) $(ANDROID_DEST)/x86/$(SO)
	cp ./target/x86_64-linux-android/release/deps/$(SO) $(ANDROID_DEST)/x86_64/$(SO)

aarch64-linux-android:
	LIBRARY_PATH=$(NDK_STANDALONE)/toolchains/llvm/prebuilt/$(HOST_ARCH)/sysroot/usr/lib/$@/$(API) \
	CC=$(CC_PATH)/$@-clang \
	CXX=$(CC_PATH)/$@-clang++ \
	cargo build --target $@ --release --lib

i686-linux-android:
	LIBRARY_PATH=$(NDK_STANDALONE)/toolchains/llvm/prebuilt/$(HOST_ARCH)/sysroot/usr/lib/$@/$(API) \
	CC=$(CC_PATH)/$@-clang \
	CXX=$(CC_PATH)/$@-clang++ \
	cargo build --target $@ --release --lib

x86_64-linux-android:
	LIBRARY_PATH=$(NDK_STANDALONE)/toolchains/llvm/prebuilt/$(HOST_ARCH)/sysroot/usr/lib/$@/$(API) \
	CC=$(CC_PATH)/$@-clang \
	CXX=$(CC_PATH)/$@-clang++ \
	cargo build --target $@ --release --lib
