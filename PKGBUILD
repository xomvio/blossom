# Maintainer: xomvio <xomvio at proton dot me>

pkgname=blossom
pkgver=0.0.4
pkgrel=1
pkgdesc='Secure decentralized chat via Yggdrasil network'
url="https://github.com/xomvio/$pkgname"
license=('GPL-3.0-or-later')
arch=('x86_64')
depends=('gcc-libs' 'glibc')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('a7446e1931dba8b8fe80636a4d277ad3820ecc4c6013e9354347fd469c683614')

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
