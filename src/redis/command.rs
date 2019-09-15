use crate::redis::*;
use libc::{c_int, size_t};
use crate::minhash::sketch::MinHash;
use crate::redis::store::SimpleDMARegVector;
use std::os::raw::c_longlong;
use crate::minhash::NUM_REGISTERS;
use std::mem::size_of;
use std::slice::from_raw_parts;

const ERR_WRONGTYPE: &str = "WRONGTYPE Key is not a valid HyperMinHash string value.";

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn MinHashAdd_RedisCommand(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: c_int) -> c_int {

    unsafe {
        RedisModule_AutoMemory(ctx);

        if argc < 2 {
            return RedisModule_WrongArity(ctx);
        }

        let key = RedisModule_OpenKey(
            ctx, *argv.add(1), REDISMODULE_READ | REDISMODULE_WRITE);
        let key_type = RedisModule_KeyType(key);

        let mut data: *mut u8;

        if key_type != REDISMODULE_KEYTYPE_EMPTY && key_type != REDISMODULE_KEYTYPE_STRING {
            return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr());
        }
        if key_type == REDISMODULE_KEYTYPE_EMPTY {
            if RedisModule_StringTruncate(key, size_of::<u32>() * NUM_REGISTERS) != REDISMODULE_OK {
                return REDISMODULE_ERR;
            }
        }

        let mut len: size_t = 0;
        let ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);

        let mut minhash =
            MinHash::from(SimpleDMARegVector::new(ptr, len));

        for i in 2..argc {
            let mut len: size_t = 0;
            let arg = RedisModule_StringPtrLen(*argv.add(i as usize), &mut len);

            minhash.add(from_raw_parts(arg, len));
        }

        RedisModule_ReplyWithSimpleString(ctx, "OK\0".as_ptr())
    }
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn MinHashCount_RedisCommand(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: c_int) -> c_int {

    unsafe {
        RedisModule_AutoMemory(ctx);

        if argc < 2 {
            return RedisModule_WrongArity(ctx);
        }

        let key = RedisModule_OpenKey(ctx, *argv.add(1), REDISMODULE_READ | REDISMODULE_WRITE);
        let key_type = RedisModule_KeyType(key);

        if key_type == REDISMODULE_KEYTYPE_EMPTY {
            return RedisModule_ReplyWithLongLong(ctx, 0);
        }
        if key_type != REDISMODULE_KEYTYPE_STRING {
            return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr());
        }

        let mut len: size_t = 0;
        let ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);

        let minhash =
            MinHash::from(SimpleDMARegVector::new(ptr, len));

        RedisModule_ReplyWithLongLong(ctx, minhash.cardinality() as c_longlong)
    }
}
