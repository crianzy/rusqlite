SCRIPT_DIR=$(cd "$(dirname "$_")" && pwd)
echo $SCRIPT_DIR
cd $SCRIPT_DIR
SQLITE3_LIB_DIR=$SCRIPT_DIR/sqlite3


SQLITE3_INCLUDE_DIR=$SQLITE3_LIB_DIR
# Just to make sure there is only one bindgen.rs file in target dir
find $SCRIPT_DIR/target -type f -name bindgen.rs -exec rm {} \;
cargo build --features "buildtime_bindgen" --no-default-features
find $SCRIPT_DIR/target -type f -name bindgen.rs -exec cp {} $SQLITE3_LIB_DIR/bindgen_bundled_version.rs \;
# Sanity check
cd $SCRIPT_DIR/..
cargo test --features "backup blob chrono functions limits load_extension serde_json trace bundled"
echo 'You should increment the version in libsqlite3-sys/Cargo.toml'
