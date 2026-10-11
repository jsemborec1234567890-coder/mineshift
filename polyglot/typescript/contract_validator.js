/**
 * MINE//SHIFT: BEYOND — Polyglot TypeScript/JavaScript Contract Validator
 * Validates JSON fixtures against puzzle_contract_v1 schema and mathematical invariants.
 */

const fs = require('fs');
const path = require('path');

function validateFixture(filePath) {
    const raw = fs.readFileSync(filePath, 'utf-8');
    const data = JSON.parse(raw);
    const fileName = path.basename(filePath);

    console.log(`[JS/TS VALIDATOR] Validating fixture: ${fileName}`);

    // 1. Core Schema Fields
    if (data.contract_version !== "1.0") {
        throw new Error(`Invalid contract_version: ${data.contract_version}`);
    }
    if (!data.fixture_id || typeof data.fixture_id !== 'string') {
        throw new Error(`Missing or invalid fixture_id`);
    }
    if (data.topology !== 'square' && data.topology !== 'hex') {
        throw new Error(`Invalid topology: ${data.topology}`);
    }

    // 2. Ruleset Validation
    const rs = data.ruleset;
    if (!rs || typeof rs.mine_count !== 'number' || rs.mine_count < 1) {
        throw new Error(`Ruleset requires positive mine_count`);
    }
    if (!['standard', 'no_flags', 'limited_flags'].includes(rs.flag_policy)) {
        throw new Error(`Invalid flag_policy: ${rs.flag_policy}`);
    }

    // 3. Topology Coordinate Bounds Invariant
    if (data.topology === 'square') {
        if (!data.width || !data.height || data.width < 1 || data.height < 1) {
            throw new Error(`Square topology requires positive width & height`);
        }
        for (const [mx, my] of data.secret_mines || []) {
            if (mx < 0 || mx >= data.width || my < 0 || my >= data.height) {
                throw new Error(`Secret mine (${mx}, ${my}) out of bounds for ${data.width}x${data.height}`);
            }
        }
    } else if (data.topology === 'hex') {
        if (typeof data.radius !== 'number' || data.radius < 1) {
            throw new Error(`Hex topology requires radius >= 1`);
        }
        const r = data.radius;
        for (const [mx, my] of data.secret_mines || []) {
            const q = mx - r;
            const ax_r = my - r;
            if (Math.abs(q) > r || Math.abs(ax_r) > r || Math.abs(q + ax_r) > r) {
                throw new Error(`Hex secret mine (${mx}, ${my}) outside axial radius ${r}`);
            }
        }
    }

    // 4. Deductions structure
    if (!data.expected_deductions || !Array.isArray(data.expected_deductions.forced_mines)) {
        throw new Error(`expected_deductions.forced_mines must be an array`);
    }
    if (!Array.isArray(data.expected_deductions.forced_safes)) {
        throw new Error(`expected_deductions.forced_safes must be an array`);
    }

    console.log(`  -> SUCCESS: ${fileName} adheres to contract v1!`);
    return true;
}

function main() {
    const fixturesDir = path.resolve(__dirname, '..', 'fixtures');
    const files = fs.readdirSync(fixturesDir).filter(f => f.endsWith('.json'));

    let count = 0;
    for (const f of files) {
        validateFixture(path.join(fixturesDir, f));
        count++;
    }

    console.log(`\n[JS/TS VALIDATOR] All ${count} contract fixtures successfully validated!\n`);
}

main();
