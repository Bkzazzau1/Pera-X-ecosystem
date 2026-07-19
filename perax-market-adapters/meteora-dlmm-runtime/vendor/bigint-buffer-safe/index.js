'use strict';
function assertBuffer(value) { if (!Buffer.isBuffer(value) && !(value instanceof Uint8Array)) throw new TypeError('Expected Buffer or Uint8Array'); return Buffer.from(value); }
function assertBigInt(value) { if (typeof value !== 'bigint') throw new TypeError('Expected bigint'); if (value < 0n) throw new RangeError('Expected non-negative bigint'); }
function assertWidth(width) { if (!Number.isSafeInteger(width) || width < 0) throw new RangeError('width must be a non-negative safe integer'); }
function toBigIntLE(value) { const buffer=assertBuffer(value); let result=0n; for (let i=buffer.length-1;i>=0;i--) result=(result<<8n)|BigInt(buffer[i]); return result; }
function toBigIntBE(value) { const buffer=assertBuffer(value); let result=0n; for (const byte of buffer) result=(result<<8n)|BigInt(byte); return result; }
function toBufferLE(value,width) { assertBigInt(value); assertWidth(width); const out=Buffer.alloc(width); let remaining=value; for(let i=0;i<width;i++){out[i]=Number(remaining & 255n); remaining >>= 8n;} if(remaining!==0n) throw new RangeError('BigInt does not fit in buffer width'); return out; }
function toBufferBE(value,width) { return Buffer.from(toBufferLE(value,width)).reverse(); }
module.exports={toBigIntLE,toBigIntBE,toBufferLE,toBufferBE};
