const fs = require('fs');
const path = require('path');

const LIB_PATH = path.resolve(__dirname, '../programs/perax-core/src/lib.rs');
const ANCHOR_PATH = path.resolve(__dirname, '../Anchor.toml');
const DEPLOYMENT_RECORD_PATH = path.resolve(__dirname, '../config/deployment-record.devnet.public.json');
const PLACEHOLDER_PROGRAM_ID = '11111111111111111111111111111111';

function readText(filePath, label) {
  if (!fs.existsSync(filePath)) throw new Error(`${label} not found at ${filePath}`);
  return fs.readFileSync(filePath, 'utf8');
}

function readJson(filePath, label) {
  if (!fs.existsSync(filePath)) throw new Error(`${label} not found at ${filePath}`);
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function extractDeclareId(lib) {
  const match = lib.match(/declare_id!\("([1-9A-HJ-NP-Za-km-z]{32,44})"\)/);
  return match ? match[1] : null;
}

function extractAnchorProgramId(anchorToml) {
  const match = anchorToml.match(/perax_core\s*=\s*"([1-9A-HJ-NP-Za-km-z]{32,44})"/);
  return match ? match[1] : null;
}

function main() {
  const lib = readText(LIB_PATH, 'perax core lib.rs');
  const anchor = readText(ANCHOR_PATH, 'Anchor.toml');
  const record = readJson(DEPLOYMENT_RECORD_PATH, 'Devnet deployment record');

  const declareId = extractDeclareId(lib);
  const anchorProgramId = extractAnchorProgramId(anchor);
  const recordProgramId = record.program && record.program.programId;

  console.log('==============================================');
  console.log('Pera-X Anchor Devnet Deploy Readiness');
  console.log('==============================================');
  console.log('');

  console.log('Program ID status:');
  console.log('----------------------------------------------');
  console.log(`declare_id: ${declareId || 'MISSING'}`);
  console.log(`Anchor.toml program ID: ${anchorProgramId || 'MISSING'}`);
  console.log(`Deployment record program ID: ${recordProgramId || 'MISSING'}`);
  console.log('');

  console.log('Required initialization values:');
  console.log('----------------------------------------------');
  console.log(`PEX mint: ${record.token.mintAddress}`);
  console.log(`Trading Company locked token account: ${record.tradingCompany.lockedStrategicTokenAccount}`);
  console.log(`Trading Company revenue token account: ${record.tradingCompany.revenueTokenAccount}`);
  console.log(`Safety admin wallet: ${record.program.safetyAdminWallet}`);
  console.log(`Oracle/Bot wallet: ${record.program.oracleBotWallet}`);
  console.log('');

  const blockers = [];
  if (!declareId) blockers.push('declare_id missing from lib.rs');
  if (!anchorProgramId) blockers.push('perax_core program ID missing from Anchor.toml');
  if (declareId === PLACEHOLDER_PROGRAM_ID) blockers.push('declare_id is still placeholder 11111111111111111111111111111111');
  if (anchorProgramId === PLACEHOLDER_PROGRAM_ID) blockers.push('Anchor.toml program ID is still placeholder 11111111111111111111111111111111');
  if (recordProgramId && recordProgramId.startsWith('REPLACE_')) blockers.push('deployment record programId is still placeholder');
  if (declareId && anchorProgramId && declareId !== anchorProgramId) blockers.push('declare_id and Anchor.toml program ID do not match');

  console.log('Readiness:');
  console.log('----------------------------------------------');
  if (blockers.length === 0) {
    console.log('Status: READY for Anchor devnet deploy workflow.');
  } else {
    console.log('Status: NOT READY for Anchor deploy.');
    blockers.forEach((blocker) => console.log(`- ${blocker}`));
  }

  console.log('');
  console.log('Next required action:');
  console.log('----------------------------------------------');
  console.log('1. Generate a devnet program keypair.');
  console.log('2. Update declare_id!() and Anchor.toml with the new program public key.');
  console.log('3. Store the program keypair as a GitHub Secret for deployment.');
  console.log('4. Run the guarded devnet deploy workflow.');
}

main();
