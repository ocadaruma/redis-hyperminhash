use super::*;
use crate::hyperminhash::sketch::{HyperMinHash, MinHashCombiner};
use crate::hyperminhash::NUM_REGISTERS;
use dma::CByteArray;
use repr::{HyperMinHashRepr, Registers};
use libc::{c_double, c_int, size_t, c_longlong};
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

        let mut new_key: bool = false;
        if key_type != REDISMODULE_KEYTYPE_EMPTY && key_type != REDISMODULE_KEYTYPE_STRING {
            return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr());
        }
        if key_type == REDISMODULE_KEYTYPE_EMPTY {
            if RedisModule_StringTruncate(key, HyperMinHashRepr::dense_len()) != REDISMODULE_OK {
                return REDISMODULE_ERR;
            }
            new_key = true
        }

        let mut len: size_t = 0;
        let ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);
        let mut bytes = CByteArray::wrap(ptr, len);

        if new_key {
            HyperMinHashRepr::initialize(&mut bytes);
            RedisModule_Log(ctx, "debug\0".as_ptr(), "key initialized\0".as_ptr());
        }
        match HyperMinHashRepr::parse(bytes) {
            None => RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr()),
            Some(mut repr) => {
                let mut updated_count = 0;
                let mut sketch = match repr.registers() {
                    Registers::Dense(registers) => HyperMinHash::wrap(registers),
                };
                for i in 2..argc {
                    let mut len: size_t = 0;
                    let arg = RedisModule_StringPtrLen(*argv.add(i as usize), &mut len);

                    let updated = sketch.add(from_raw_parts(arg, len));
                    if updated {
                        updated_count += 1;
                    }
                }
                if updated_count > 0 {
                    repr.invalidate_cache();
                }

                RedisModule_ReplyWithSimpleString(ctx, "OK\0".as_ptr())
            },
        }
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

            return match HyperMinHashRepr::parse(CByteArray::wrap(ptr, len)) {
                None =>
                    RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr()),
                Some(mut repr) => {
                    if repr.cache_valid() {
                        RedisModule_ReplyWithLongLong(ctx, repr.get_cache() as c_longlong)
                    } else {
                        let sketch = match repr.registers() {
                            Registers::Dense(registers) => HyperMinHash::wrap(registers),
                        };
                        let cardinality = sketch.cardinality();
                        repr.set_cache(cardinality as u64);
                        RedisModule_ReplyWithLongLong(ctx, cardinality as c_longlong)
                    }
                },
            }
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

            match HyperMinHashRepr::parse(CByteArray::wrap(ptr, len)) {
                None => return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr()),
                Some(repr) => {
                    union_sketch.merge(&match repr.registers() {
                        Registers::Dense(registers) => HyperMinHash::wrap(registers),
                    });
                },
            }
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

        let mut new_key: bool = false;
        if key_type != REDISMODULE_KEYTYPE_EMPTY && key_type != REDISMODULE_KEYTYPE_STRING {
            return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr());
        }
        if key_type == REDISMODULE_KEYTYPE_EMPTY {
            if RedisModule_StringTruncate(key, HyperMinHashRepr::dense_len()) != REDISMODULE_OK {
                return REDISMODULE_ERR;
            }
            new_key = true;
        }

        let mut len: size_t = 0;
        let ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);
        let mut bytes = CByteArray::wrap(ptr, len);

        if new_key {
            HyperMinHashRepr::initialize(&mut bytes);
        }

        let mut union_sketch = match HyperMinHashRepr::parse(bytes) {
            None =>
                return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr()),
            Some(repr) => {
                match repr.registers() {
                    Registers::Dense(registers) => HyperMinHash::wrap(registers),
                }
            },
        };

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

            match HyperMinHashRepr::parse(CByteArray::wrap(ptr, len)) {
                None => return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr()),
                Some(repr) => {
                    union_sketch.merge(&match repr.registers() {
                        Registers::Dense(registers) => HyperMinHash::wrap(registers),
                    });
                },
            }
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

            match HyperMinHashRepr::parse(CByteArray::wrap(ptr, len)) {
                None =>
                    return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr()),
                Some(repr) => {
                    combiner.combine(&match repr.registers() {
                        Registers::Dense(registers) => HyperMinHash::wrap(registers),
                    })
                },
            }
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

            match HyperMinHashRepr::parse(CByteArray::wrap(ptr, len)) {
                None =>
                    return RedisModule_ReplyWithError(ctx, format!("{}\0", ERR_WRONGTYPE).as_ptr()),
                Some(repr) => {
                    combiner.combine(&match repr.registers() {
                        Registers::Dense(registers) => HyperMinHash::wrap(registers),
                    })
                },
            }
        }

        RedisModule_ReplyWithLongLong(ctx, combiner.intersection() as c_longlong)
    }
}
