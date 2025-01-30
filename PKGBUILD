# Maintainer: xomvio <xomvio at proton dot me>

pkgname=blossom
pkgver=0.0.2
pkgrel=1
pkgdesc='Secure decentralized chat via Yggdrasil network'
url="https://github.com/xomvio/$pkgname"
license=('GPL-3.0-or-later')
arch=('x86_64')
depends=('gcc-libs' 'glibc')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('05b47e0999961bda3c25bad224a8adab390f6fc93580b2aa8bb2b6b24f04e0de')

prepare() {
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    cd "$pkgname-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}

package() {
    cd "$pkgname-$pkgver"
    install -Dm755 -t "$pkgdir/usr/bin/" "target/release/$pkgname"
}