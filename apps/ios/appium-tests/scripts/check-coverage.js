#!/usr/bin/env node
/**
 * Coverage Gate Script
 * Verifies that all PRD test IDs have corresponding test implementations
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const testsDir = path.join(__dirname, '..', 'tests');

// All required test IDs from PRD
const requiredTestIds = [
	// Authentication
	'AUTH-001',
	'AUTH-002',
	'AUTH-003',
	'AUTH-004',
	'AUTH-005',
	'AUTH-006',
	'AUTH-007',
	'AUTH-008',
	'AUTH-010',
	// Services
	'SVC-001',
	'SVC-002',
	'SVC-003',
	'SVC-005',
	// Booking Flow
	'BKG-001',
	'BKG-002',
	'BKG-003',
	'BKG-006',
	'BKG-009',
	'BKG-010',
	// Booking Management
	'BMG-001',
	'BMG-002',
	'BMG-003',
	'BMG-004',
	'BMG-005',
	'BMG-006',
	// Profile
	'PRF-001',
	'PRF-002',
	'PRF-003',
	'PRF-004',
	'PRF-005',
	'PRF-006',
	'PRF-007',
	'PRF-008',
	// Payments
	'PAY-001',
	'PAY-002',
	'PAY-004',
	'PAY-005',
	// Walker Onboarding
	'WON-001',
	'WON-002',
	'WON-003',
	'WON-004',
	// Walker Dashboard
	'WDB-001',
	'WDB-002',
	'WDB-003',
	'WDB-004',
	'WDB-005',
	// Walker Booking Requests
	'WBR-001',
	'WBR-002',
	'WBR-003',
	'WBR-004',
	'WBR-005',
	// Walker Calendar
	'WCL-001',
	'WCL-002',
	'WCL-003',
	// Walker Map
	'WMP-001',
	'WMP-002',
	'WMP-003',
	// Walker Settings
	'WST-001',
	'WST-002',
	'WST-003',
	'WST-004',
	'WST-005',
	// Multi-tenant
	'MTN-001',
	'MTN-002',
	'MTN-003',
	// Error Handling
	'ERR-001',
	'ERR-002',
	'ERR-003',
	'ERR-004',
	'ERR-005',
];

function findTestFiles(dir) {
	const files = [];
	const items = fs.readdirSync(dir, { withFileTypes: true });

	for (const item of items) {
		const fullPath = path.join(dir, item.name);
		if (item.isDirectory()) {
			files.push(...findTestFiles(fullPath));
		} else if (item.name.endsWith('.spec.ts')) {
			files.push(fullPath);
		}
	}

	return files;
}

function extractTestIds(content) {
	const ids = [];
	// Match patterns like: it('AUTH-001: or // AUTH-001:
	const regex = /(?:it\(['"`]|\/\/\s*)([A-Z]{2,4}-\d{3})/g;
	let match;
	while ((match = regex.exec(content)) !== null) {
		ids.push(match[1]);
	}
	return ids;
}

function main() {
	console.log('Checking test coverage against PRD requirements...\n');

	const testFiles = findTestFiles(testsDir);
	const foundIds = new Set();

	for (const file of testFiles) {
		const content = fs.readFileSync(file, 'utf-8');
		const ids = extractTestIds(content);
		ids.forEach((id) => foundIds.add(id));
	}

	const missingIds = requiredTestIds.filter((id) => !foundIds.has(id));
	const coverage = (
		((requiredTestIds.length - missingIds.length) / requiredTestIds.length) *
		100
	).toFixed(1);

	console.log(`Test files scanned: ${testFiles.length}`);
	console.log(`Required test IDs: ${requiredTestIds.length}`);
	console.log(`Implemented test IDs: ${foundIds.size}`);
	console.log(`Coverage: ${coverage}%\n`);

	if (missingIds.length > 0) {
		console.log('Missing test IDs:');
		missingIds.forEach((id) => console.log(`  - ${id}`));
		console.log('\n');
	}

	// Require 100% coverage (all PRD test IDs implemented)
	if (missingIds.length > 0) {
		console.error(`ERROR: Coverage gate failed. ${missingIds.length} test IDs missing.`);
		process.exit(1);
	}

	console.log('SUCCESS: All PRD test IDs are implemented.');
	process.exit(0);
}

main();
