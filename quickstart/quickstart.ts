// Quickstart del ADK de Terminal 3 — primera llamada autenticada.
// Copiado de la documentacion oficial:
// https://docs.terminal3.io/developers/adk/get-started/quickstart
//
// La key se lee del entorno, nunca se escribe en el archivo (asi lo pide la
// doc, y ademas es una clave de FIRMA: el SDK deriva de ella una direccion
// Ethereum con eth_get_address y firma con metamask_sign).
import {
  T3nClient,
  setEnvironment,
  loadWasmComponent,
  eth_get_address,
  metamask_sign,
  createEthAuthInput,
} from "@terminal3/t3n-sdk";

setEnvironment("testnet"); // el SDK apunta a produccion por defecto

const T3N_API_KEY = process.env.T3N_API_KEY!;
if (!T3N_API_KEY) {
  console.error("falta T3N_API_KEY en el entorno");
  process.exit(1);
}

const t0 = Date.now();
const wasmComponent = await loadWasmComponent(); // toda la criptografia corre acá adentro
console.log(`loadWasmComponent(): ${Date.now() - t0} ms`);

const address = eth_get_address(T3N_API_KEY);
console.log("direccion derivada de la key:", address);

const t3n = new T3nClient({
  wasmComponent,
  handlers: {
    EthSign: metamask_sign(address, undefined, T3N_API_KEY),
  },
  // NO ESTA EN EL QUICKSTART OFICIAL, pero el SDK lo exige: sin este campo
  // handshake() muere con "Cannot read properties of undefined (reading
  // 'unsafe_trust_server')". Ver BUGS.md, hallazgo B-1.
  //
  // OJO con lo que significa. Los tipos del SDK avisan que sin verificar la
  // attestation "un atacante de red con su propia VM TDX puede entregarle al
  // SDK una attestation falsa pero valida para una clave que el controla, y
  // leer todas las sesiones". Se usa la opcion insegura SOLO porque esto es
  // testnet y es lo unico que la documentacion permite hoy; en produccion va
  // un TrustAnchor con expected_peer_ids y rtmr3_allowlist.
  trustAnchor: { unsafe_trust_server: true },
});

const t1 = Date.now();
await t3n.handshake();
console.log(`handshake(): ${Date.now() - t1} ms`);

const t2 = Date.now();
const did = await t3n.authenticate(createEthAuthInput(address));
const tenantDid = did.value;
console.log(`authenticate(): ${Date.now() - t2} ms`);

console.log("Connected as:", tenantDid);
