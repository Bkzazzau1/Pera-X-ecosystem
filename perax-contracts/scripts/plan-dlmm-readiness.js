const fs = require('fs');
const path = require('path');

const RECORD_PATH = path.resolve(__dirname, '../config/deployment-record.devnet.public.json');

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function main() {
  const record = readJson(RECORD_PATH);
  const liquidity = record.initialLiquidity || {};
  const status = record.status || {};
  const verification = record.verification || {};

  const blockers = [];

  if (!status.allocationStateVerified) blockers.push('allocation state not verified');
  if (!verification.allAllocationBalancesCorrect) blockers.push('allocation balances not confirmed');
  if (!liquidity.tokenAccount) blockers.push('liquidity PEX token account missing');
  if (liquidity.finalBalance !== liquidity.pexAmount) blockers.push('liquidity PEX balance does not match planned amount');
  if (!liquidity.devnetQuoteMint) blockers.push('devnet quote mint not recorded');
  if (!liquidity.quoteTokenAccount) blockers.push('quote token account not recorded');
  if (!liquidity.dlmmPoolAddress) blockers.push('DLMM pool address not recorded');
  if (!liquidity.positionAddress) blockers.push('DLMM position address not recorded');

  console.log('==============================================');
  console.log('Pera-X DLMM Readiness Plan');
  console.log('==============================================');
  console.log('');
  console.log(`DEX: ${liquidity.dex || 'MISSING'}`);
  console.log(`Pair: ${liquidity.pair || 'MISSING'}`);
  console.log(`PEX planned amount: ${liquidity.pexAmount || 'MISSING'}`);
  console.log(`Quote planned amount: ${liquidity.quoteAmountUsd || 'MISSING'}`);
  console.log(`Liquidity owner wallet: ${liquidity.ownerWallet || 'MISSING'}`);
  console.log(`PEX token account: ${liquidity.tokenAccount || 'MISSING'}`);
  console.log(`PEX final balance: ${liquidity.finalBalance || 'MISSING'}`);
  console.log(`Quote mint: ${liquidity.devnetQuoteMint || 'MISSING'}`);
  console.log(`Quote token account: ${liquidity.quoteTokenAccount || 'MISSING'}`);
  console.log(`DLMM pool address: ${liquidity.dlmmPoolAddress || 'MISSING'}`);
  console.log(`DLMM position address: ${liquidity.positionAddress || 'MISSING'}`);
  console.log('');

  console.log('Readiness:');
  console.log('----------------------------------------------');
  if (blockers.length === 0) {
    console.log('Status: READY TO RECORD DLMM LIQUIDITY RESULT.');
  } else {
    console.log('Status: NOT READY TO MARK DLMM LIQUIDITY CREATED.');
    blockers.forEach((blocker) => console.log(`- ${blocker}`));
  }
}

main();
