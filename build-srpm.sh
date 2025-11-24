#!/bin/bash
#

set -e

source ./version.sh
TARBALL=logria-$VERSION.tar.gz

mkdir -p output
rm -rf rpmbuild

cargo vendor
tar -czf logria-$VERSION-vendor.tar.gz vendor
sed -e "s/@VERSION@/$VERSION/; s/@RELEASE@/$RELEASE/" ./packaging/logria.spec.in > logria.spec
find ./* \
	! -name 'packaging' \
	! -name 'rpmbuild' \
	! -name 'exported-artifacts' \
	! -name '.git' \
	-type f | tar --files-from /proc/self/fd/0 -czf "$TARBALL" --transform "s,^,logria-$VERSION/,"

mkdir -p rpmbuild/{SPECS,SOURCES,BUILD,BUILDROOT,SRPMS}
mv *.tar.gz rpmbuild/SOURCES
mv logria.spec rpmbuild/SPECS

rpmbuild -bs --define "_topdir `pwd`/rpmbuild" `pwd`/rpmbuild/SPECS/logria.spec

mv rpmbuild/SRPMS/* output
