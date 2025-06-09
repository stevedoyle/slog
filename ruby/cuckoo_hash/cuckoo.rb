
def hash1(k)
  k % 11
end

def hash2(k)
  (k/11) % 11
end

puts hash1(20)
puts hash2(20)
