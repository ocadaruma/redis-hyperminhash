use super::*;
use crate::hyperminhash::sketch::{HyperMinHash, MinHashCombiner};
use crate::hyperminhash::NUM_REGISTERS;
use super::store::SimpleDMARegVector;
use libc::{c_double, c_int, size_t, c_longlong};
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

        let mut sketch =
            HyperMinHash::wrap(SimpleDMARegVector::wrap(ptr, len));

        for i in 2..argc {
            let mut len: size_t = 0;
            let arg = RedisModule_StringPtrLen(*argv.add(i as usize), &mut len);

            sketch.add(from_raw_parts(arg, len));
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
    }

    // single key case
    if argc == 2 {
        unsafe {
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

            let sketch = HyperMinHash::wrap(SimpleDMARegVector::wrap(ptr, len));
            return RedisModule_ReplyWithLongLong(ctx, sketch.cardinality() as c_longlong);
        }
    }

    // multiple key case
    let mut union_sketch = HyperMinHash::wrap([0; NUM_REGISTERS]);
    unsafe {
        for i in 1..argc {
            let key = RedisModule_OpenKey(
                ctx, *argv.add(i as usize), REDISMODULE_READ | REDISMODULE_WRITE);
            let key_type = RedisModule_KeyType(key);

            if key_type == REDISMODULE_KEYTYPE_EMPTY {
                continue;
            }
            if key_type != REDISMODULE_KEYTYPE_STRING {
                return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr());
            }

            let mut len: size_t = 0;
            let ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);

            let sketch =
                HyperMinHash::wrap(SimpleDMARegVector::wrap(ptr, len));

            union_sketch.merge(&sketch);
        }

        RedisModule_ReplyWithLongLong(ctx, union_sketch.cardinality() as c_longlong)
    }
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn MinHashMerge_RedisCommand(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: c_int) -> c_int {

    unsafe {
        RedisModule_AutoMemory(ctx);

        if argc < 2 {
            return RedisModule_WrongArity(ctx);
        }

        // handle target key
        let key = RedisModule_OpenKey(ctx, *argv.add(1), REDISMODULE_READ | REDISMODULE_WRITE);
        let key_type = RedisModule_KeyType(key);

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

        let mut union_sketch =
            HyperMinHash::wrap(SimpleDMARegVector::wrap(ptr, len));

        for i in 2..argc {
            let key = RedisModule_OpenKey(
                ctx, *argv.add(i as usize), REDISMODULE_READ | REDISMODULE_WRITE);
            let key_type = RedisModule_KeyType(key);

            if key_type == REDISMODULE_KEYTYPE_EMPTY {
                continue;
            }
            if key_type != REDISMODULE_KEYTYPE_STRING {
                return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr());
            }

            let mut len: size_t = 0;
            let ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);

            let sketch =
                HyperMinHash::wrap(SimpleDMARegVector::wrap(ptr, len));

            union_sketch.merge(&sketch);
        }

        RedisModule_ReplyWithSimpleString(ctx, "OK\0".as_ptr())
    }
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn MinHashSimilarity_RedisCommand(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: c_int) -> c_int {

    unsafe {
        RedisModule_AutoMemory(ctx);

        if argc < 2 {
            return RedisModule_WrongArity(ctx);
        }

        let mut combiner = MinHashCombiner::new();
        for i in 1..argc {
            let key = RedisModule_OpenKey(
                ctx, *argv.add(i as usize), REDISMODULE_READ | REDISMODULE_WRITE);
            let key_type = RedisModule_KeyType(key);

            if key_type == REDISMODULE_KEYTYPE_EMPTY {
                continue;
            }
            if key_type != REDISMODULE_KEYTYPE_STRING {
                return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr());
            }

            let mut len: size_t = 0;
            let ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);

            let sketch =
                HyperMinHash::wrap(SimpleDMARegVector::wrap(ptr, len));

            combiner.combine(&sketch);
        }

        RedisModule_ReplyWithDouble(ctx, combiner.similarity() as c_double)
    }
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
#[no_mangle]
pub extern "C" fn MinHashIntersection_RedisCommand(
    ctx: *mut RedisModuleCtx,
    argv: *mut *mut RedisModuleString,
    argc: c_int) -> c_int {

    unsafe {
        RedisModule_AutoMemory(ctx);

        if argc < 2 {
            return RedisModule_WrongArity(ctx);
        }

        let mut combiner = MinHashCombiner::new();
        for i in 1..argc {
            let key = RedisModule_OpenKey(
                ctx, *argv.add(i as usize), REDISMODULE_READ | REDISMODULE_WRITE);
            let key_type = RedisModule_KeyType(key);

            if key_type == REDISMODULE_KEYTYPE_EMPTY {
                continue;
            }
            if key_type != REDISMODULE_KEYTYPE_STRING {
                return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr());
            }

            let mut len: size_t = 0;
            let ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);

            let sketch =
                HyperMinHash::wrap(SimpleDMARegVector::wrap(ptr, len));

            combiner.combine(&sketch);
        }

        RedisModule_ReplyWithLongLong(ctx, combiner.intersection() as c_longlong)
    }
}
