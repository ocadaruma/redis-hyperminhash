require "redis"

ITERATIONS = 100000

REDIS_PORT = ENV["REDIS_PORT"].to_i || 6379

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
end

bench = Benchmarker.new(Redis.new(port: REDIS_PORT))

bench.bench_command("PFADD") do |conn|
  if resp != "OK"
    raise "Unexpected response. You probably have a bug."
  end
end

bench_command(:"MH.MERGE", "hellomh2", "hellomh") do |conn|
  if resp != "OK"
    raise "Unexpected response. You probably have a bug."
  end
end
