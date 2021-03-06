//! Redis commands implementation.

use super::*;
use crate::hyperminhash::sketch::{HyperMinHash, MinHashCombiner};
use crate::hyperminhash::new_array_registers;
use dma::CByteArray;
use repr::{HyperMinHashRepr, Registers};
use libc::{c_double, c_int, size_t, c_longlong};
use std::slice::from_raw_parts;


/// Add given elements to HyperMinHash sketch.
/// Key will be initialized regardless of any element is passed or not.
///
/// `redis-cli> MH.ADD key [element ...]`
#[allow(non_snake_case)]
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

        let Key(key, key_type) = open_rw(ctx, *argv.add(1));

        let mut new_key: bool = false;
        if key_type != REDISMODULE_KEYTYPE_EMPTY && key_type != REDISMODULE_KEYTYPE_STRING {
            return reply_wrong_type(ctx);
        }
        if key_type == REDISMODULE_KEYTYPE_EMPTY {
            if RedisModule_StringTruncate(key, HyperMinHashRepr::dense_len()) != REDISMODULE_OK {
                return REDISMODULE_ERR;
            }
            new_key = true
        }

        let mut bytes = string_dma(key);

        if new_key {
            HyperMinHashRepr::initialize(&mut bytes);
        }
        match HyperMinHashRepr::parse(bytes) {
            None => reply_wrong_type(ctx),
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
                    RedisModule_ReplicateVerbatim(ctx);
                }

                RedisModule_ReplyWithLongLong(ctx, if updated_count > 0 { 1 } else { 0 })
            },
        }
    }
}

/// Estimate cardinality using HyperLogLog.
/// If multiple keys are specified, estimate their union cardinality.
///
/// `redis-cli> MH.COUNT key [key ...]`
#[allow(non_snake_case)]
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
            let Key(key, key_type) = open_rw(ctx, *argv.add(1));

            if key_type == REDISMODULE_KEYTYPE_EMPTY {
                return RedisModule_ReplyWithLongLong(ctx, 0);
            }
            if key_type != REDISMODULE_KEYTYPE_STRING {
                return reply_wrong_type(ctx);
            }

            return match HyperMinHashRepr::parse(string_dma(key)) {
                None =>
                    reply_wrong_type(ctx),
                Some(mut repr) => {
                    if repr.cache_valid() {
                        RedisModule_ReplyWithLongLong(ctx, repr.get_cache() as c_longlong)
                    } else {
                        let sketch = match repr.registers() {
                            Registers::Dense(registers) => HyperMinHash::wrap(registers),
                        };
                        let cardinality = sketch.cardinality();
                        repr.set_cache(cardinality as u64);
                        RedisModule_ReplicateVerbatim(ctx);
                        RedisModule_ReplyWithLongLong(ctx, cardinality as c_longlong)
                    }
                },
            }
        }
    }

    // multiple key case
    let mut union_sketch = HyperMinHash::wrap(new_array_registers());
    unsafe {
        for i in 1..argc {
            let Key(key, key_type) = open_ro(ctx, *argv.add(i as usize));

            if key_type == REDISMODULE_KEYTYPE_EMPTY {
                continue;
            }
            if key_type != REDISMODULE_KEYTYPE_STRING {
                return reply_wrong_type(ctx);
            }

            match HyperMinHashRepr::parse(string_dma(key)) {
                None => return reply_wrong_type(ctx),
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

/// Merge multiple sketches into destination key.
/// Destination key will be initialized regardless of any sourcekey is passed or not.
///
/// `redis-cli> MH.MERGE destkey sourcekey [sourcekey ...]`
#[allow(non_snake_case)]
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
        let Key(key, key_type) = open_rw(ctx, *argv.add(1));

        let mut new_key: bool = false;
        if key_type != REDISMODULE_KEYTYPE_EMPTY && key_type != REDISMODULE_KEYTYPE_STRING {
            return reply_wrong_type(ctx);
        }
        if key_type == REDISMODULE_KEYTYPE_EMPTY {
            if RedisModule_StringTruncate(key, HyperMinHashRepr::dense_len()) != REDISMODULE_OK {
                return REDISMODULE_ERR;
            }
            new_key = true;
        }

        let mut bytes = string_dma(key);

        if new_key {
            HyperMinHashRepr::initialize(&mut bytes);
        }

        let mut repr = match HyperMinHashRepr::parse(bytes) {
            None =>
                return reply_wrong_type(ctx),
            Some(repr) => repr,
        };
        let mut union_sketch = match repr.registers() {
            Registers::Dense(registers) => HyperMinHash::wrap(registers),
        };

        for i in 2..argc {
            let Key(key, key_type) = open_ro(ctx, *argv.add(i as usize));

            if key_type == REDISMODULE_KEYTYPE_EMPTY {
                continue;
            }
            if key_type != REDISMODULE_KEYTYPE_STRING {
                return reply_wrong_type(ctx);
            }

            match HyperMinHashRepr::parse(string_dma(key)) {
                None =>
                    return reply_wrong_type(ctx),
                Some(repr) => {
                    union_sketch.merge(&match repr.registers() {
                        Registers::Dense(registers) => HyperMinHash::wrap(registers),
                    });
                },
            }
        }
        repr.invalidate_cache();
        RedisModule_ReplicateVerbatim(ctx);

        reply_ok(ctx)
    }
}

/// Estimate similarity between multiple sketches using MinHash.
///
/// `redis-cli> MH.SIMILARITY key [key ...]`
#[allow(non_snake_case)]
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
            let Key(key, key_type) = open_ro(ctx, *argv.add(i as usize));

            if key_type == REDISMODULE_KEYTYPE_EMPTY {
                continue;
            }
            if key_type != REDISMODULE_KEYTYPE_STRING {
                return reply_wrong_type(ctx);
            }

            match HyperMinHashRepr::parse(string_dma(key)) {
                None =>
                    return reply_wrong_type(ctx),
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

/// Estimate intersection cardinality of multiple sketches using MinHash.
///
/// `redis-cli> MH.INTERSECTION key [key ...]`
#[allow(non_snake_case)]
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
            let Key(key, key_type) = open_ro(ctx, *argv.add(i as usize));

            if key_type == REDISMODULE_KEYTYPE_EMPTY {
                continue;
            }
            if key_type != REDISMODULE_KEYTYPE_STRING {
                return reply_wrong_type(ctx);
            }

            match HyperMinHashRepr::parse(string_dma(key)) {
                None =>
                    return reply_wrong_type(ctx),
                Some(repr) => {
                    combiner.combine(&match repr.registers() {
                        Registers::Dense(registers) => HyperMinHash::wrap(registers),
                    })
                },
            }
        }

        RedisModule_ReplyWithLongLong(ctx, combiner.intersection().round() as c_longlong)
    }
}

struct Key(*mut RedisModuleKey, c_int);

fn open_ro(ctx: *mut RedisModuleCtx, string: *mut RedisModuleString) -> Key {
    unsafe {
        let ptr = RedisModule_OpenKey(ctx, string, REDISMODULE_READ);
        let key_type = RedisModule_KeyType(ptr);

        Key(ptr, key_type)
    }
}

fn open_rw(ctx: *mut RedisModuleCtx, string: *mut RedisModuleString) -> Key {
    unsafe {
        let ptr = RedisModule_OpenKey(ctx, string, REDISMODULE_READ | REDISMODULE_WRITE);
        let key_type = RedisModule_KeyType(ptr);

        Key(ptr, key_type)
    }
}

fn string_dma(key: *mut RedisModuleKey) -> CByteArray {
    let mut len: size_t = 0;
    unsafe {
        let ptr = RedisModule_StringDMA(key, &mut len, REDISMODULE_WRITE);
        CByteArray::wrap(ptr, len)
    }
}

fn reply_wrong_type(ctx: *mut RedisModuleCtx) -> c_int {
    unsafe {
        RedisModule_ReplyWithError(
            ctx, "WRONGTYPE Key is not a valid HyperMinHash string value.\0".as_ptr())
    }
}

fn reply_ok(ctx: *mut RedisModuleCtx) -> c_int {
    unsafe {
        RedisModule_ReplyWithSimpleString(ctx, "OK\0".as_ptr())
    }
}
