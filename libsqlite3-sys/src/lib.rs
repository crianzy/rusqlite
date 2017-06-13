#![allow(non_snake_case, non_camel_case_types)]

pub use self::error::*;

use std::mem;

mod error;

pub fn SQLITE_STATIC() -> sqlite3_destructor_type {
    Some(unsafe { mem::transmute(0isize) })
}

pub fn SQLITE_TRANSIENT() -> sqlite3_destructor_type {
    Some(unsafe { mem::transmute(-1isize) })
}

/// Run-Time Limit Categories
#[repr(C)]
pub enum Limit {
    /// The maximum size of any string or BLOB or table row, in bytes.
    SQLITE_LIMIT_LENGTH = SQLITE_LIMIT_LENGTH as isize,
    /// The maximum length of an SQL statement, in bytes.
    SQLITE_LIMIT_SQL_LENGTH = SQLITE_LIMIT_SQL_LENGTH as isize,
    /// The maximum number of columns in a table definition or in the result set of a SELECT
    /// or the maximum number of columns in an index or in an ORDER BY or GROUP BY clause.
    SQLITE_LIMIT_COLUMN = SQLITE_LIMIT_COLUMN as isize,
    /// The maximum depth of the parse tree on any expression.
    SQLITE_LIMIT_EXPR_DEPTH = SQLITE_LIMIT_EXPR_DEPTH as isize,
    /// The maximum number of terms in a compound SELECT statement.
    SQLITE_LIMIT_COMPOUND_SELECT = SQLITE_LIMIT_COMPOUND_SELECT as isize,
    /// The maximum number of instructions in a virtual machine program used to implement an SQL statement.
    SQLITE_LIMIT_VDBE_OP = SQLITE_LIMIT_VDBE_OP as isize,
    /// The maximum number of arguments on a function.
    SQLITE_LIMIT_FUNCTION_ARG = SQLITE_LIMIT_FUNCTION_ARG as isize,
    /// The maximum number of attached databases.
    SQLITE_LIMIT_ATTACHED = SQLITE_LIMIT_ATTACHED as isize,
    /// The maximum length of the pattern argument to the LIKE or GLOB operators.
    SQLITE_LIMIT_LIKE_PATTERN_LENGTH = SQLITE_LIMIT_LIKE_PATTERN_LENGTH as isize,
    /// The maximum index number of any parameter in an SQL statement.
    SQLITE_LIMIT_VARIABLE_NUMBER = SQLITE_LIMIT_VARIABLE_NUMBER as isize,
    /// The maximum depth of recursion for triggers.
    SQLITE_LIMIT_TRIGGER_DEPTH = 10,
    /// The maximum number of auxiliary worker threads that a single prepared statement may start.
    SQLITE_LIMIT_WORKER_THREADS = 11,
}

include!(concat!(env!("OUT_DIR"), "/bindgen.rs"));

#[cfg(test)]
extern crate rand;

#[cfg(all(feature = "sqlcipher", test))]
mod tests {
    use std::env;
    use std::mem;
    use std::path::{Path, PathBuf};
    use std::fs;
    use std::ffi::CString;
    use std::ptr;
    use std::os::raw::c_int;
    use rand::{thread_rng, Rng};

    const FIRST_KEY: &'static str = "my first passphrase";
    const SECOND_KEY: &'static str = "my second passphrase";
    const DB_PATH: &'static str = "bindings_test_sqlcipher";
    const OPEN_FLAGS: c_int = super::SQLITE_OPEN_READWRITE|super::SQLITE_OPEN_URI|super::SQLITE_OPEN_NOMUTEX;

    fn new_db() -> PathBuf {
        let out_dir = env::var("OUT_DIR").unwrap();
        let stub: String = thread_rng().gen_ascii_chars().take(10).collect();
        let path = Path::new(out_dir.as_str());
        let mut pbuf = path.join(DB_PATH);
        pbuf.set_extension(stub);
        pbuf
    }

    fn create_test_db(path: &PathBuf, key: &str) -> *mut super::sqlite3 {
        fs::remove_file(path);
        let flags = super::SQLITE_OPEN_CREATE | OPEN_FLAGS;
        _open_test_db(path, flags, key)
    }

    fn open_test_db(path: &PathBuf, key: &str) -> *mut super::sqlite3 {
        _open_test_db(path, OPEN_FLAGS, key)
    }

    fn _open_test_db(path: &PathBuf, flags: c_int, key: &str) -> *mut super::sqlite3 {
        unsafe {
            let mut db: *mut super::sqlite3 = mem::uninitialized();
            let path_ = CString::new(path.to_str().unwrap()).unwrap();
            let r = super::sqlite3_open_v2(path_.as_ptr(), &mut db, flags, ptr::null());

            assert!(r == super::SQLITE_OK, "Can't open {0}: {1}", path.to_str().unwrap(), r);

            let database_name = CString::new("main").unwrap();
            let passphrase = CString::new(key).unwrap();
            let c_len = (key.len() + 1) as c_int;
            let r = super::sqlite3_key_v2(db, database_name.as_ptr(), passphrase.as_ptr() as *mut ::std::os::raw::c_void, c_len);

            return db;
        }
    }

    fn exec(db: *mut super::sqlite3, cmd: &str) -> Option<c_int> {
        let mut stmt: *mut super::sqlite3_stmt = unsafe { mem::uninitialized() };
        let query = CString::new(cmd);
        unsafe {
            let len_with_nul = (cmd.len() + 1) as c_int;
            let r = super::sqlite3_prepare_v2(db, query.unwrap().as_ptr(), len_with_nul, &mut stmt, ptr::null_mut());

            let r = super::sqlite3_step(stmt);
            super::sqlite3_finalize(stmt);

            match r {
                super::SQLITE_OK => None,
                super::SQLITE_DONE => None,
                super::SQLITE_ROW => None,
                super::SQLITE_ERROR => panic!("SQLite error encountered"),
                super::SQLITE_BUSY => panic!("SQLite claiming busy: finalize statement, maybe?"),
                _ => panic!("SQlite returned a weird error code: {0}: \"{1}\"", r, cmd)

            }
        }
    }

    #[test]
    fn test_sqlcipher_create_and_encrypt() {
        create_and_encrypt(FIRST_KEY);
    }

    fn create_and_encrypt(key: &str) -> PathBuf {
        unsafe {
            let db_path = new_db();
            let db = create_test_db(&db_path, key);

            let cmds = vec![
                "CREATE TABLE lemming ( first INTEGER PRIMARY KEY );",
                "INSERT INTO lemming VALUES (11);",
                "INSERT INTO lemming VALUES (13);",
                "INSERT INTO lemming VALUES (17);",
            ];
            for cmd in cmds {
                exec(db, cmd);
            }

            let r = super::sqlite3_close(db);
            assert_eq!(r, super::SQLITE_OK);
            return db_path;
        }
    }

    #[test]
    fn test_sqlcipher_open_and_decrypt() {
        let path = create_and_encrypt(FIRST_KEY);
        let db = open_and_decrypt(&path, FIRST_KEY);
        unsafe {
            let r = super::sqlite3_close(db);
            assert_eq!(r, super::SQLITE_OK)
        }
    }

    fn open_and_decrypt(path: &PathBuf, key: &str) -> *mut super::sqlite3 {
        unsafe {
            let db = open_test_db(path, key);

            let cmds = vec![
                "SELECT * FROM lemming LIMIT 1;",
            ];
            for cmd in cmds {
                exec(db, cmd);
            }
            return db;
        }
    }

    #[test]
    fn test_sqlcipher_open_and_rekey() {
        let path = create_and_encrypt(FIRST_KEY);
        let db = open_and_decrypt(&path, FIRST_KEY);
        unsafe {
            // Decrypt and then re-encrypt via the C API
            let database_name = CString::new("main").unwrap();
            let passphrase = CString::new(SECOND_KEY).unwrap();
            let c_len = (SECOND_KEY.len() + 1) as c_int;
            let r = super::sqlite3_rekey_v2(db, database_name.as_ptr(), passphrase.as_ptr() as *mut ::std::os::raw::c_void, c_len);
            exec(db, "SELECT * FROM lemming LIMIT 2;");

            let r = super::sqlite3_close(db);
            assert_eq!(r, super::SQLITE_OK);
        }
        // Verify the new key is intact
        let db = open_test_db(&path, SECOND_KEY);
        unsafe {
            exec(db, "SELECT * FROM lemming LIMIT 2;");

            let r = super::sqlite3_close(db);
            assert_eq!(r, super::SQLITE_OK);
        }
    }
}