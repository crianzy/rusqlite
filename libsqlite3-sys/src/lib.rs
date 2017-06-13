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

#[cfg(all(feature = "sqlcipher", test))]
mod tests {

    use std::env;
    use std::mem;
    use std::path::Path;
    use std::fs;
    use std::ffi::{CStr, CString};
    use std::ptr;
    use std::vec;
    use std::os::raw::c_int;

    const FIRST_KEY: &'static str = "my first passphrase";
    const SECOND_KEY: &'static str = "my second passphrase";
    const DB_PATH: &'static str = "bindings_test.sqlite3_enc";
    const OPEN_FLAGS: c_int = super::SQLITE_OPEN_READWRITE|super::SQLITE_OPEN_URI|super::SQLITE_OPEN_NOMUTEX;

    fn db_name(path: &str) -> String {
        let out_dir = env::var("OUT_DIR").unwrap();
        out_dir + path
    }

    fn create_test_db(path: &str) -> *mut super::sqlite3 {
        let os_path = Path::new(path);
        fs::remove_file(os_path);
        let mut flags = super::SQLITE_OPEN_CREATE | OPEN_FLAGS;
        _open_test_db(path, flags)
    }

    fn open_test_db(path: &str) -> *mut super::sqlite3 {
        let os_path = Path::new(path);
        fs::remove_file(os_path);
        _open_test_db(path, OPEN_FLAGS)
    }

    fn _open_test_db(path: &str, flags: c_int) -> *mut super::sqlite3 {
        unsafe {
            let mut db: *mut super::sqlite3 = mem::uninitialized();
            let path = CString::new(path).unwrap();
            let r = super::sqlite3_open_v2(path.as_ptr(), &mut db, flags, ptr::null());

            assert_eq!(r, super::SQLITE_OK);
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
                _ => panic!("SQlite returned a weird error code: {}", r)
            }
        }
    }

    #[test]
    fn test_sqlcipher_create_and_encrypt() {
        unsafe {
            let db_path = db_name(DB_PATH);
            let db = create_test_db(db_path.as_str());

            let passphrase = format!("PRAGMA key = \"{}\";", FIRST_KEY);
            let cmds = vec![
                passphrase.as_str(),
                "CREATE TABLE lemming ( first INTEGER PRIMARY KEY );",
                "INSERT INTO lemming VALUES (11);",
                "INSERT INTO lemming VALUES (13);",
                "INSERT INTO lemming VALUES (17);",
                //"SELECT * FROM lemming LIMIT 1;",
            ];
            for cmd in cmds {
                exec(db, cmd);
            }

            let r = super::sqlite3_close(db);
            assert_eq!(r, super::SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlcipher_open_and_decrypt() {
        test_sqlcipher_create_and_encrypt();
        unsafe {
            let db_path = db_name(DB_PATH);
            let db = open_test_db(db_path.as_str());

            let passphrase = format!("PRAGMA key = \"{}\";", FIRST_KEY);
            let cmds = vec![
                passphrase.as_str(),
                "SELECT * FROM lemming LIMIT 1;",
            ];
            for cmd in cmds {
                exec(db, cmd);
            }

            let r = super::sqlite3_close(db);
            assert_eq!(r, super::SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlcipher_open_and_rekey() {
        test_sqlcipher_create_and_encrypt();
        unsafe {
            let db_path = db_name(DB_PATH);
            let db = open_test_db(db_path.as_str());

            let old_passphrase = format!("PRAGMA key = \"{}\";", FIRST_KEY);
            let new_passphrase = format!("PRAGMA key = \"{}\";", SECOND_KEY);
            let cmds = vec![
                old_passphrase.as_str(),
                new_passphrase.as_str(),
                "SELECT * FROM lemming LIMIT 1;",
            ];
            for cmd in cmds {
                exec(db, cmd);
            }

            let r = super::sqlite3_close(db);
            assert_eq!(r, super::SQLITE_OK);
        }
    }

    #[test]
    fn test_sqlcipher_verify_rekey() {
        test_sqlcipher_create_and_encrypt();
        test_sqlcipher_open_and_rekey();
        unsafe {
            let db_path = db_name(DB_PATH);
            let db = open_test_db(db_path.as_str());

            let passphrase = format!("PRAGMA key = \"{}\";", SECOND_KEY);
            let cmds = vec![
                passphrase.as_str(),
                "SELECT * FROM lemming LIMIT 1;",
            ];
            for cmd in cmds {
                exec(db, cmd);
            }

            let r = super::sqlite3_close(db);
            assert_eq!(r, super::SQLITE_OK);
        }
    }
}