

function bit(n, m) return n >> (m - 1) & 1 end

function bits(n) return bit(n, 8) .. bit(n, 7) .. bit(n, 6) .. bit(n, 5) .. bit(n, 4) .. bit(n, 3) .. bit(n, 2) .. bit(n, 1) end

function count_bits(n) return bit(n, 8) + bit(n, 7) + bit(n, 6) + bit(n, 5) + bit(n, 4) + bit(n, 3) + bit(n, 2) + bit(n, 1) end

function print_generic_pattern(f)
  local line = ''
  for n = 1,256 do
    line = line .. ('0b' .. bits(n) .. ' => ' .. (f(n) and '1' or '0') .. ',')
    if n % 8 == 0 then
      print(line)
      line = ''
    end
  end
end

function print_match_pattern_for_dead_cells()
  print_generic_pattern(function(n) return count_bits(n) == 3 end)
end

function print_match_pattern_for_live_cells()
  print_generic_pattern(function(n) n = count_bits(n); return n == 2 or n == 3 end)
end

function print_indexes_of_inner_cells()
  for i = 0,63 do
    if i >= 8 and i < 64-8 and i % 8 > 0 and i % 8 < 7 then
      print(i .. ', ')
    end
  end
end

function print_indexes_of_outer_cells()
  for i = 0,63 do
    if i < 8 or i >= 64-8 or i % 8 == 0 or i % 8 == 7 then
      print(i .. ', ')
    end
  end
end

local key = arg[1]
if key == 'dead' then
  print_match_pattern_for_dead_cells()
elseif key == 'live' then
  print_match_pattern_for_live_cells()
elseif key == 'inner' then
  print_indexes_of_inner_cells()
elseif key == 'outer' then
  print_indexes_of_outer_cells()
else
  print('Usage: lua patterns.lua <dead|live|inner|outer>')
end

