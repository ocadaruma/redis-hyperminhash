require "redis"

ITERATIONS = 100000

REDIS_PORT = ENV["REDIS_PORT"] || "6379"

class Benchmarker
  def initialize(conn)
    @conn = conn
  end

  def bench_command(cmd, &block)
    start = Time.now

    ITERATIONS.times do
      block.yield(@conn)
    end

    puts cmd

    duration = Time.now - start
    puts "Took total of: #{duration} s"

    per_iteration = duration / ITERATIONS
    per_iteration_ms = per_iteration * 1000
    puts "Per iteration: #{per_iteration} s (#{per_iteration_ms} ms)"

    puts ""
  end

  def setup(&block)
    block.yield(@conn)
  end
end

bench = Benchmarker.new(Redis.new(port: REDIS_PORT.to_i))

## benchmark MH.ADD and PFADD
i = 0
bench.bench_command("MH.ADD") do |conn|
  resp = conn.send("MH.ADD", "mh:key", i.to_s)
  if resp != 0 && resp != 1
    raise "Unexpected response."
  end
  i += 1
end

i = 0
bench.bench_command("PFADD") do |conn|
  resp = conn.send("PFADD", "pf:key", i.to_s)
  if resp != 0 && resp != 1
    raise "Unexpected response."
  end
  i += 1
end

## benchmark MH.COUNT AND PFCOUNT

### to avoid cardinality cache, setup different keys
ITERATIONS.times do |i|
  bench.setup do |conn|
    conn.send("MH.ADD", "mh:key#{i}", 1, 2, 3)
    conn.send("PFADD", "pf:key#{i}", 1, 2, 3)
  end
end

i = 0
bench.bench_command("MH.COUNT") do |conn|
  resp = conn.send("MH.COUNT", "mh:key#{i}")
  if resp != 3
    raise "Unexpected response."
  end
  i += 1
end

i = 0
bench.bench_command("PFCOUNT") do |conn|
  resp = conn.send("PFCOUNT", "pf:key#{i}")
  if resp != 3
    raise "Unexpected response."
  end
  i += 1
end

## benchmark MH.MERGE AND PFMERGE
i = 0
bench.bench_command("MH.MERGE") do |conn|
  resp = conn.send("MH.MERGE", "mh:dest", "mh:key#{i}")
  if resp != "OK"
    raise "Unexpected response."
  end
  i += 1
end

i = 0
bench.bench_command("PFMERGE") do |conn|
  resp = conn.send("PFMERGE", "pf:dest", "pf:key#{i}")
  if resp != "OK"
    raise "Unexpected response."
  end
  i += 1
end

## benchmark MH.SIMILARITY and MH.INTERSECTION
i = 0
bench.bench_command("MH.SIMILARITY") do |conn|
  resp = conn.send("MH.SIMILARITY", "mh:key0", "mh:key#{i}")
  i += 1
end

i = 0
bench.bench_command("MH.INTERSECTION") do |conn|
  resp = conn.send("MH.INTERSECTION", "mh:key0", "mh:key#{i}")
  if resp != 3
    raise "Unexpected response."
  end
  i += 1
end
