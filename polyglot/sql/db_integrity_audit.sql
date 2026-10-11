-- MINE//SHIFT: BEYOND — SQLite Database Integrity & Invariant Audit
-- Verifies schema consistency, foreign key constraints, and progression invariants

-- 1. Integrity Check
PRAGMA integrity_check;
PRAGMA foreign_key_check;

-- 2. Verify Profile Isolation Invariant
-- No challenge completion should reference a non-existent profile
SELECT COUNT(*) AS orphaned_challenge_records
FROM challenge_completions c
LEFT JOIN profiles p ON c.profile_id = p.id
WHERE p.id IS NULL;

-- 3. Verify Challenge Attempt Invariant
-- Attempt count must always be >= 1 for any recorded attempt
SELECT COUNT(*) AS invalid_attempt_records
FROM challenge_completions
WHERE attempts < 1;

-- 4. Verify Positive XP Invariant
SELECT COUNT(*) AS negative_xp_profiles
FROM profiles
WHERE total_xp < 0;
