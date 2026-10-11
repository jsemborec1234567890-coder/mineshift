--[[
  MINE//SHIFT: BEYOND — Lua Rules Sandbox
  Experimental sandboxed modifier and rule evaluation engine
--]]

print("[LUA RULES SANDBOX] Testing sandboxed modifier evaluation...")

-- 1. Sandboxed Rule Definitions
local Rules = {}

function Rules.validate_limited_flags(max_flags, mine_count)
    if max_flags <= 0 then
        return false, "Limited flags must be > 0"
    end
    if max_flags > mine_count * 2 then
        return false, "Limited flags cannot exceed 2x mine count"
    end
    return true, "OK"
end

function Rules.check_no_flags_victory(unrevealed_safes)
    return unrevealed_safes == 0
end

-- 2. Test Tactical Rationing Modifier
local ok, msg = Rules.validate_limited_flags(5, 10)
assert(ok, "Tactical Rationing (5 flags for 10 mines) should be valid in Lua")
print("  -> Tactical Rationing (5 flags / 10 mines): " .. msg .. " (PASS)")

local invalid_zero, err_z = Rules.validate_limited_flags(0, 10)
assert(not invalid_zero, "0 flags should be rejected in limited flags policy")
print("  -> Invalid 0 flags: Rejected as expected (PASS)")

local invalid_huge, err_h = Rules.validate_limited_flags(25, 10)
assert(not invalid_huge, "25 flags for 10 mines should be rejected")
print("  -> Invalid excessive flags (25 > 2*10): Rejected as expected (PASS)")

-- 3. Test No Flags Victory Evaluation
local win = Rules.check_no_flags_victory(0)
assert(win, "0 unrevealed safe cells must trigger victory under No Flags")
print("  -> No Flags victory condition (unrevealed_safes == 0): Won (PASS)")

print("[LUA RULES SANDBOX] SUCCESS: 100% verified.")
