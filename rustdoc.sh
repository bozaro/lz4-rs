set -ex
cargo clean
cargo doc
COMMIT=`git log -n1 --format=%H`
pushd target/doc
git init .
git add -A
git commit -m "Rustdoc for revision $COMMIT"
git push -u git@github.com:bozaro/lz4-rs.git HEAD:gh-pages -f
popd