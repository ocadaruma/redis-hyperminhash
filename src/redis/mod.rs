//! redis FFI layer

mod store;

use std::os::raw::{c_int, c_longlong};

const MODULE_NAME: &str = "redis-minhash";
const MODULE_VERSION: c_int = 1;
const REDISMODULE_APIVER_1: c_int = 1;

const REDISMODULE_OK: c_int = 0;
const REDISMODULE_ERR: c_int = 1;

enum RedisModuleCtx {}
enum RedisModuleString {}

type RedisModuleCmdFunc = extern "C" fn(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: c_int) -> c_int;

#[link(name="redismodule", kind="static")]
extern "C" {
    fn Export_RedisModule_Init(ctx: *mut RedisModuleCtx,
                               modulename: *const u8,
                               module_version: c_int,
                               api_version: c_int) -> c_int;

    // RedisModule_* commands are declared in redismodule.h as function pointer.

    static RedisModule_CreateCommand: extern "C" fn(ctx: *mut RedisModuleCtx,
                                                    name: *const u8,
                                                    cmdfunc: RedisModuleCmdFunc,
                                                    strflags: *const u8,
                                                    firstkey: c_int,
                                                    lastkey: c_int,
                                                    keystep: c_int) -> c_int;

    static RedisModule_ReplyWithLongLong: extern "C" fn(ctx: *mut RedisModuleCtx,
                                                        ll: c_longlong) -> c_int;

    static RedisModule_Log: extern "C" fn(ctx: *mut RedisModuleCtx,
                                          level: *const u8,
                                          fmt: *const u8);
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
extern "C" fn MinHashAdd_RedisCommand(ctx: *mut RedisModuleCtx,
                                      argv: *mut *mut RedisModuleString,
                                      argc: c_int) -> c_int {
    unsafe {
        RedisModule_ReplyWithLongLong(ctx, 55301)
    }
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
extern "C" fn RedisModule_OnLoad(ctx: *mut RedisModuleCtx,
                                 argv: *mut *mut RedisModuleString,
                                 argc: c_int) -> c_int {
    unsafe {
        if Export_RedisModule_Init(ctx,
                                   format!("{}\0", MODULE_NAME).as_ptr(),
                                   MODULE_VERSION,
                                   REDISMODULE_APIVER_1) != REDISMODULE_OK {
            return REDISMODULE_ERR;
        }

        RedisModule_Log(ctx, "notice\0".as_ptr(), "tttteeeessst\0".as_ptr());

        RedisModule_CreateCommand(ctx,
                                  "mh.add\0".as_ptr(),
                                  MinHashAdd_RedisCommand,
                                  "write\0".as_ptr(),
                                  0,
                                  0,
                                  1)
    }
}
