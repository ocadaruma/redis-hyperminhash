require "redis"
require "fileutils"
require "json"

ITERATIONS = 500
TRUE_CARDINALITY = 10000

REDIS_PORT = ENV["REDIS_PORT"] || "6379"

class Experiment
  def initialize(type, num, conn)
    @type = type
    @add_cmd = type == "PF" ? "PFADD" : "MH.ADD"
    @count_cmd = type == "PF" ? "PFCOUNT" : "MH.COUNT"
    @num = num
    @conn = conn
  end

  def execute()
    key = "#{@type}:#{@num}"
    TRUE_CARDINALITY.times.each_slice(10) do |xs|
      @conn.send(@add_cmd, key, *(xs.map{|x| x * (@num + 1)}))
    end

    @conn.send(@count_cmd, key)
  end
end

def print_histogram(histo)
  left = 0
  histo.each.with_index do |c, i|
    if c.nonzero?
      left = i
      break
    end
  end

  right = 0
  (0...histo.size).each do |i|
    if histo[histo.size - i - 1].nonzero?
      right = histo.size - i - 1
      break
    end
  end

  range = right - left

  buckets = Array.new(20, 0)
  (left..right).each do |i|
    bucket = (20 * (i - left) / range.to_f).to_i

    buckets[bucket] ||= 0
    buckets[bucket] += histo[i]
  end

  buckets.each.with_index do |b, i|
    puts "#{sprintf("%05d- ", left + i * (range.to_f / 20))}: #{'*' * b}"
  end
end

conn = Redis.new(port: REDIS_PORT.to_i)

## experiment HyperMinHash

mh_histo = Array.new(TRUE_CARDINALITY * 2, 0)
ITERATIONS.times do |num|
  experiment = Experiment.new("MH", num, conn)
  card = experiment.execute()

  puts "MH DONE EXPERIMENT: #{num}" if num % 10 == 0

  mh_histo[card] += 1
end

pf_histo = Array.new(TRUE_CARDINALITY * 2, 0)
ITERATIONS.times do |num|
  experiment = Experiment.new("PF", num, conn)
  card = experiment.execute()

  puts "PF DONE EXPERIMENT: #{num}" if num % 10 == 0

  pf_histo[card] += 1
end

puts "============== HyperMinHash =============="
print_histogram(mh_histo)

puts "============== built-in HyperLogLog =============="
print_histogram(pf_histo)
