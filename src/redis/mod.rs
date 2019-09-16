//! Redis FFI layer

extern crate libc;

mod command;
mod dma;
mod store;

use command::*;
use libc::{c_int, c_longlong, size_t};

const MODULE_NAME: &str = "redis-minhash";
const MODULE_VERSION: c_int = 1;

const REDISMODULE_APIVER_1: c_int = 1;
const REDISMODULE_OK: c_int = 0;
const REDISMODULE_ERR: c_int = 1;

const REDISMODULE_KEYTYPE_EMPTY: c_int = 0;
const REDISMODULE_KEYTYPE_STRING: c_int = 1;

const REDISMODULE_READ: c_int = 1 << 0;
const REDISMODULE_WRITE: c_int = 1 << 1;

// Opaque types for Redis Module structs
pub enum RedisModuleCtx {}
pub enum RedisModuleString {}
pub enum RedisModuleKey {}

type RedisModuleCmdFunc = extern "C" fn(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: c_int) -> c_int;

#[allow(non_upper_case_globals)]
#[link(name="redismodule", kind="static")]
extern "C" {
    fn Export_RedisModule_Init(
        ctx: *mut RedisModuleCtx,
        modulename: *const u8,
        module_version: c_int,
        api_version: c_int) -> c_int;

    // RedisModule_* commands are declared in redismodule.h as function pointer.

    static RedisModule_CreateCommand: extern "C" fn(
        ctx: *mut RedisModuleCtx,
        name: *const u8,
        cmdfunc: RedisModuleCmdFunc,
        strflags: *const u8,
        firstkey: c_int,
        lastkey: c_int,
        keystep: c_int) -> c_int;

    static RedisModule_ReplyWithLongLong: extern "C" fn(
        ctx: *mut RedisModuleCtx,
        ll: c_longlong) -> c_int;

    static RedisModule_StringDMA: extern "C" fn(
        key: *mut RedisModuleKey,
        len: *mut size_t,
        mode: c_int) -> *mut u8;

    static RedisModule_StringTruncate: extern "C" fn(
        key: *mut RedisModuleKey,
        newlen: size_t) -> c_int;

    static RedisModule_StringPtrLen: extern "C" fn(
        str: *const RedisModuleString,
        len: *mut size_t) -> *const u8;

    static RedisModule_OpenKey: extern "C" fn(
        ctx: *mut RedisModuleCtx,
        keyname: *mut RedisModuleString,
        mode: c_int) -> *mut RedisModuleKey;

    static RedisModule_AutoMemory: extern "C" fn(ctx: *mut RedisModuleCtx);

    static RedisModule_WrongArity: extern "C" fn(ctx: *mut RedisModuleCtx) -> c_int;

    static RedisModule_CreateString: extern "C" fn(
        ctx: *mut RedisModuleCtx,
        ptr: *const u8,
        len: size_t) -> *mut RedisModuleString;

    static RedisModule_ReplyWithError: extern "C" fn(
        ctx: *mut RedisModuleCtx,
        err: *const u8) -> c_int;

    static RedisModule_Calloc: extern "C" fn(nmemb: size_t, size: size_t) -> *mut u8;

    static RedisModule_KeyType: extern "C" fn(kp: *mut RedisModuleKey) -> c_int;

    static RedisModule_Free: extern "C" fn(ptr: *mut u8);

    static RedisModule_ReplyWithSimpleString: extern "C" fn(
        ctx: *mut RedisModuleCtx,
        msg: *const u8) -> c_int;

    static RedisModule_Log: extern "C" fn(
        ctx: *mut RedisModuleCtx,
        level: *const u8,
        fmt: *const u8);
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
extern "C" fn RedisModule_OnLoad(ctx: *mut RedisModuleCtx,
                                 argv: *mut *mut RedisModuleString,
                                 argc: c_int) -> c_int {
    unsafe {
        if Export_RedisModule_Init(
            ctx,
            format!("{}\0", MODULE_NAME).as_ptr(),
            MODULE_VERSION,
            REDISMODULE_APIVER_1) != REDISMODULE_OK {
            return REDISMODULE_ERR;
        }

        if RedisModule_CreateCommand(
            ctx,
            "mh.add\0".as_ptr(),
            MinHashAdd_RedisCommand,
            "write fast\0".as_ptr(),
            1, -1, 1) != REDISMODULE_OK {
            return REDISMODULE_ERR;
        }

        if RedisModule_CreateCommand(
            ctx,
            "mh.count\0".as_ptr(),
            MinHashCount_RedisCommand,
            "write fast\0".as_ptr(),
            1, -1, 1) != REDISMODULE_OK {
            return REDISMODULE_ERR;
        }

        REDISMODULE_OK
    }
}
