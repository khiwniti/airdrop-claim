const web3 = globalThis.solanaWeb3;

const PROGRAM_ID = new web3.PublicKey('2NepmY26ip3y5gPWaY5ZoAUUWNtqqMjDobiAyFtCj5Jf');
const CLAIM_DISC = new Uint8Array([62, 198, 214, 193, 213, 159, 108, 210]);
const TOKEN_PROGRAM = new web3.PublicKey('TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA');
const ATA_PROGRAM = new web3.PublicKey('ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL');

const $ = (id) => document.getElementById(id);
const log = (m) => { $('log').textContent += m + '\n'; };

let wallet = null;      // window.solana (Phantom or injected mock)
let campaignData = null;

function rpcUrl() { return $('rpc').value.trim(); }
function connection() { return new web3.Connection(rpcUrl(), 'confirmed'); }

function parseCampaign(buf) {
  const o = 8; // anchor discriminator
  const pk = (i) => new web3.PublicKey(buf.slice(i, i + 32));
  const u64le = (i) => Number(new DataView(buf.buffer, buf.byteOffset + i, 8).getBigUint64(0, true));
  return {
    authority: pk(o), mint: pk(o + 32), vault: pk(o + 64), root: pk(o + 96),
    id: u64le(o + 128), deposited: u64le(o + 136), claimed: u64le(o + 144),
  };
}

function ata(owner, mint) {
  return web3.PublicKey.findProgramAddressSync(
    [owner.toBytes(), TOKEN_PROGRAM.toBytes(), mint.toBytes()], ATA_PROGRAM)[0];
}
function receiptPda(campaign, claimant) {
  return web3.PublicKey.findProgramAddressSync(
    [new TextEncoder().encode('claim'), campaign.toBytes(), claimant.toBytes()], PROGRAM_ID)[0];
}

function claimIxData(amount, proofHex) {
  const bytes = proofHex.match(/../g) || [];
  const buf = new Uint8Array(8 + 8 + 4 + bytes.length);
  buf.set(CLAIM_DISC, 0);
  const dv = new DataView(buf.buffer);
  dv.setBigUint64(8, BigInt(amount), true);
  dv.setUint32(16, bytes.length / 32, true);
  bytes.forEach((b, i) => { buf[20 + i] = parseInt(b, 16); });
  return buf;
}

async function connect() {
  if (!window.solana) { log('no wallet injected (phantom or test mock missing)'); return; }
  await window.solana.connect();
  log('wallet: ' + window.solana.publicKey.toBase58());
}

async function loadCampaign() {
  const c = new web3.PublicKey($('campaign').value.trim());
  const info = await connection().getAccountInfo(c);
  if (!info) { $('campaignInfo').textContent = 'campaign not found'; return; }
  campaignData = { pubkey: c, ...parseCampaign(new Uint8Array(info.data)) };
  $('campaignInfo').textContent =
    `mint=${campaignData.mint.toBase58()}\nvault=${campaignData.vault.toBase58()}\n` +
    `claimed=${campaignData.claimed}/${campaignData.deposited}`;
  const receipt = receiptPda(c, window.solana.publicKey);
  const rcpt = await connection().getAccountInfo(receipt);
  $('campaignInfo').textContent += rcpt ? '\nreceipt: ALREADY CLAIMED' : '\nreceipt: none — eligible to try';
  $('claim').disabled = false;
  log('campaign loaded');
}

async function doClaim(mine) {
  const proofHex = mine.proof.join('');
  const claimantAta = ata(window.solana.publicKey, campaignData.mint);

  const ix = new web3.TransactionInstruction({
    programId: PROGRAM_ID,
    keys: [
      { pubkey: window.solana.publicKey, isSigner: true, isWritable: true },
      { pubkey: campaignData.pubkey, isSigner: false, isWritable: true },
      { pubkey: campaignData.mint, isSigner: false, isWritable: false },
      { pubkey: campaignData.vault, isSigner: false, isWritable: true },
      { pubkey: claimantAta, isSigner: false, isWritable: true },
      { pubkey: receiptPda(campaignData.pubkey, window.solana.publicKey), isSigner: false, isWritable: true },
      { pubkey: TOKEN_PROGRAM, isSigner: false, isWritable: false },
      { pubkey: ATA_PROGRAM, isSigner: false, isWritable: false },
      { pubkey: web3.SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data: claimIxData(mine.amount, proofHex),
  });

  const tx = new web3.Transaction();
  tx.feePayer = window.solana.publicKey;
  tx.recentBlockhash = (await connection().getLatestBlockhash()).blockhash;
  tx.add(ix);
  const signed = await window.solana.signTransaction(tx);
  const sig = await connection().sendRawTransaction(signed.serialize());
  await connection().confirmTransaction(sig, 'confirmed');
  log('claimed ' + mine.amount + ' → ' + sig);
}

async function claim() {
  const addr = window.solana.publicKey.toBase58();
  const file = $('proofFile').files[0];
  if (!file) { log('choose proofs.json first'); return; }
  const bundle = JSON.parse(await file.text());
  const mine = bundle.allocations.find((a) => a.address === addr);
  if (!mine) { log('address not in allocation list'); return; }
  return doClaim(mine);
}

$('connect').onclick = connect;
$('load').onclick = loadCampaign;
$('claim').onclick = () => claim().catch((e) => log('ERROR: ' + e.message));

// test/smoke hook: window.__app = { connect, loadCampaign, claim, doClaim, setCampaign }
window.__app = {
  connect, loadCampaign, claim, doClaim, log,
  setCampaign: (pk, rpc) => { $('campaign').value = pk; if (rpc) $('rpc').value = rpc; },
};
