SCRIPT_DIR=$(cd "$(dirname "$_")" && pwd)
echo $SCRIPT_DIR
cd $SCRIPT_DIR
sqlcipher_LIB_DIR=$SCRIPT_DIR/sqlcipher


# Regenerate bindgen file
rm -f $sqlcipher_LIB_DIR/bindgen_bundled_version.rs
sqlcipher_INCLUDE_DIR=$sqlcipher_LIB_DIR
cargo update
# Just to make sure there is only one bindgen.rs file in target dir
find $SCRIPT_DIR/target -type f -name bindgen.rs -exec rm {} \;
cargo clean
cargo build --features "buildtime_bindgen" --no-default-features -vv
find $SCRIPT_DIR/target -type f -name bindgen.rs -exec cp {} $sqlcipher_LIB_DIR/bindgen_bundled_version.rs \;
# Sanity check
cd $SCRIPT_DIR/..
#cargo update
#cargo test --features "backup blob chrono functions limits load_extension serde_json trace bundled"
echo 'You should increment the version in libsqlcipher-sys/Cargo.toml'
