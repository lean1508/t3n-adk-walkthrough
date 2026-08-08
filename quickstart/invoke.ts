// Walkthrough pasos 4 y 5: invocar el contrato y testear su comportamiento.
//
// El caso que se prueba es el que da sentido al contrato: una aprobacion
// otorgada para un alcance NO debe servir para otro. Se registra "prepare" y
// se verifica que "publish" siga sin autorizar.
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

const TAIL = "agent-approvals";
const VERSION = "0.1.0";
const MAP_TAIL = "approvals";
const ACCION = "https://github.com/tscircuit/tscircuit/issues/999";

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
const tenant = new TenantClient({ t3n, baseUrl: getNodeUrl(), tenantDid });
console.log("tenant:", tenantDid);

// El contrato escribe en el map `approvals` del namespace del tenant, y el
// map lo crea el SDK, no el contrato. El ACL se expresa por contract_id: el
// 508 es el que devolvio register.ts. Sin readers/writers explicitos el map
// queda deny-all y el contrato no puede ni leer lo que el mismo escribio.
const CONTRACT_ID = Number(process.env.CONTRACT_ID ?? 508);
for (const visibility of ["private", "tenant", "public"]) {
  try {
    await tenant.maps.create({
      tail: MAP_TAIL,
      visibility,
      writers: { only: [CONTRACT_ID] },
      readers: { only: [CONTRACT_ID] },
    } as any);
    console.log(`map '${MAP_TAIL}' creado (visibility='${visibility}', acl=[${CONTRACT_ID}])`);
    break;
  } catch (e: any) {
    const msg = String(e?.message ?? e);
    if (/exist/i.test(msg)) { console.log(`map '${MAP_TAIL}' ya existia`); break; }
    console.log(`  visibility='${visibility}' rechazado: ${msg.slice(0, 120)}`);
  }
}

async function llamar(functionName: string, input: unknown) {
  const t0 = Date.now();
  try {
    const out = await tenant.contracts.execute(TAIL, { version: VERSION, functionName, input });
    console.log(`  ${functionName} (${Date.now() - t0} ms) ->`, JSON.stringify(out));
    return out;
  } catch (e: any) {
    console.log(`  ${functionName} (${Date.now() - t0} ms) ERROR:`, String(e?.message ?? e).slice(0, 260));
    return null;
  }
}

console.log("\n=== 1. sin aprobacion, el agente NO debe poder publicar ===");
await llamar("check-approval", { "action-id": ACCION, scope: "publish" });

console.log("\n=== 2. el humano aprueba SOLO 'prepare' ===");
await llamar("record-approval", {
  "action-id": ACCION,
  approver: "leandro",
  scope: "prepare",
  note: "revisado en el reporte del scout",
});

console.log("\n=== 3. 'prepare' aprobado, pero 'publish' sigue sin estarlo ===");
console.log(" (esta es la propiedad que justifica el contrato)");
await llamar("check-approval", { "action-id": ACCION, scope: "prepare" });
await llamar("check-approval", { "action-id": ACCION, scope: "publish" });

console.log("\n=== 4. el humano aprueba tambien 'publish' ===");
await llamar("record-approval", {
  "action-id": ACCION,
  approver: "leandro",
  scope: "publish",
  note: "diff revisado, 3 archivos",
});
await llamar("check-approval", { "action-id": ACCION, scope: "publish" });

console.log("\n=== 5. el rastro de auditoria ===");
await llamar("list-approvals", { limit: "50" });

console.log("\n=== 6. entradas invalidas: debe fallar con un motivo claro ===");
await llamar("record-approval", { "action-id": "", approver: "x", scope: "publish" });
await llamar("check-approval", { "action-id": ACCION });
