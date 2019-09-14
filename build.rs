fn main() {
    cc::Build::new()
        .file("src/redismodule.c")
        .include("include/")
        .compile("libredismodule.a");
}
