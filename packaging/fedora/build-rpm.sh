#!/bin/sh
# Builds dist/channex-<version>-1.<dist>.<arch>.rpm from the working tree
# with rpmbuild (Fedora: `sudo dnf install rpm-build`; Arch: rpm-tools).
# Run it on Fedora 43+ so the package matches that system's libraries.
set -eu

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n1)
TOP=$(mktemp -d)
trap 'rm -rf "$TOP"' EXIT
mkdir -p "$TOP/SOURCES" "$TOP/SPECS"

# Source tarball from the working tree (so uncommitted changes count),
# without build output.
tar -czf "$TOP/SOURCES/channex-$VERSION.tar.gz" \
  --exclude=./target --exclude=./build --exclude=./dist --exclude=./.git \
  --exclude=./packaging/arch/src --exclude=./packaging/arch/pkg \
  --transform "s|^\.|channex-$VERSION|" -C "$ROOT" .
cp "$ROOT/packaging/fedora/channex.spec" "$TOP/SPECS/"

# Outside Fedora/RHEL (e.g. Arch with rpm-tools) the RPM database is
# empty, so the BuildRequires can't be checked; the system's own GTK and
# libadwaita are used instead.
NODEPS=
[ -f /etc/redhat-release ] || NODEPS=--nodeps
rpmbuild --define "_topdir $TOP" $NODEPS -bb "$TOP/SPECS/channex.spec"
mkdir -p "$ROOT/dist"
cp "$TOP"/RPMS/*/channex-*.rpm "$ROOT/dist/"
ls "$ROOT"/dist/channex-*.rpm
