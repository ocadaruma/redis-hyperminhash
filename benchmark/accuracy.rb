require "redis"

ITERATIONS = 100000

RedisConn = Redis.new(port: 6380)

def bench_command(*args, &block)
  start = Time.now

  ITERATIONS.times do
    resp = RedisConn.send(*args)
    block.yield(resp)
  end

  puts args[0]

  duration = Time.now - start
  puts "Took total of: #{duration} s"

  per_iteration = duration / ITERATIONS
  per_iteration_ms = per_iteration * 1000
  puts "Per iteration: #{per_iteration} s (#{per_iteration_ms} ms)"

  puts ""
end

bench_command(:"PFADD", "hello", 123) do |resp|
  if resp != 1 && resp != 0
    raise "Unexpected response. You probably have a bug."
  end
end

bench_command(:"MH.ADD", "hellomh", 123) do |resp|
  if resp != 1 && resp != 0
    raise "Unexpected response. You probably have a bug."
  end
end
