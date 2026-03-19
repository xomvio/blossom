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
sha256sums=('51e8558f6bda6ee37c6a31fd4154be4597e3258e0bfdd5250019aa693ff8feb4')

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
