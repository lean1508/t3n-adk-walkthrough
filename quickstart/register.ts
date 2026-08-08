// Walkthrough pasos 3-5: registrar, invocar y testear el contrato TEE.
//
// Reusa el flujo de autenticacion del Quickstart. Ojo con el orden: primero
// hay que tener el T3nClient autenticado y el tenantDid leido de la sesion
// (la doc insiste en no derivarlo ni hardcodearlo), y recien despues se
// construye el TenantClient.
import { readFile } from "fs/promises";
import {
  T3nClient,
  TenantClient,
  setEnvironment,
  getNodeUrl,
  loadWasmComponent,
  eth_get_address,
  metamask_sign,
  createEthAuthInput,
} from "@terminal3/t3n-sdk";

setEnvironment("testnet");

const T3N_API_KEY = process.env.T3N_API_KEY!;
if (!T3N_API_KEY) {
  console.error("falta T3N_API_KEY");
  process.exit(1);
}

const WASM_PATH = "../contract/target/wasm32-wasip2/release/z_agent_approvals.wasm";
const CONTRACT_TAIL = "agent-approvals";
const CONTRACT_VERSION = "0.1.0";

// --- autenticacion (igual que el Quickstart) ---
const wasmComponent = await loadWasmComponent();
const address = eth_get_address(T3N_API_KEY);
const t3n = new T3nClient({
  wasmComponent,
  handlers: { EthSign: metamask_sign(address, undefined, T3N_API_KEY) },
  trustAnchor: { unsafe_trust_server: true }, // ver BUGS.md B-1
});
await t3n.handshake();
const did = await t3n.authenticate(createEthAuthInput(address));
const tenantDid = did.value;
console.log("autenticado como:", tenantDid);

// --- TenantClient ---
const tenant = new TenantClient({
  t3n,
  baseUrl: getNodeUrl(),
  tenantDid,
});
// La doc de "Set Up Dev Env" usa `await tenant.me()` como chequeo de vida,
// pero ese metodo NO EXISTE en el SDK 4.30.0 (ver BUGS.md D-7). Se usa una
// lectura real de la API en su lugar.
const contratosPrevios = await tenant.contracts.list();
console.log("TenantClient listo. nodo:", getNodeUrl());
console.log("contratos ya registrados:", JSON.stringify(contratosPrevios));

// --- paso 3: registrar ---
const wasmBytes = await readFile(WASM_PATH);
console.log(`wasm: ${wasmBytes.length} bytes desde ${WASM_PATH}`);

const t0 = Date.now();
const result = await tenant.contracts.register({
  tail: CONTRACT_TAIL,
  version: CONTRACT_VERSION,
  wasm: wasmBytes,
});
const contractId = (result as any).contract_id;
const tenantId = tenantDid.slice("did:t3n:".length);
const scriptName = `z:${tenantId}:${CONTRACT_TAIL}`;
console.log(`REGISTRADO ${scriptName} -> contract id ${contractId}  (${Date.now() - t0} ms)`);
console.log("respuesta completa:", JSON.stringify(result, null, 2));
