'use strict'

let wasmModule = {}

const objects = {}
const MAX_U32 = 2 ** 32

const textEncoder = new TextEncoder()
const textDecoder = new TextDecoder()

const readParamsFromMemory = (ptr, len) => {

    const memory = new Uint8Array(wasmModule.instance.exports.memory.buffer)
    const params = new Uint8Array(memory.slice(ptr, ptr + len))
    const dataView = new DataView(params.buffer)
    const values = []
    let i = 0
    while (i < params.length) {
        if (params[i] === 0) { // undefined
            values.push(undefined)
            i += 1
        } else if (params[i] === 1) { // null
            values.push(null)
            i += 1
        } else if (params[i] === 2) { // f64
            values.push(dataView.getFloat64(i + 1, true))
            i += 1 + 8
        } else if (params[i] === 3) { // big int
            values.push(dataView.getBigInt64(i + 1, true))
            i += 1 + 8
        } else if (params[i] === 4) { // string
            const ptr = dataView.getInt32(i + 1, true)
            const len = dataView.getInt32(i + 1 + 4, true)
            values.push(textDecoder.decode(memory.subarray(ptr, ptr + len)))
            i += 1 + 4 + 4
        } else if (params[i] === 5) { // true
            values.push(true)
            i += 1
        } else if (params[i] === 6) { // false
            values.push(false)
            i += 1
        } else if (params[i] === 7) { // object ref
            const objectId = dataView.getUint32(i + 1, true)
            values.push(objects[objectId])
            i += 1 + 4
        } else {
            throw new Error('Invalid parameter type')
        }
    }
    return values
}

const runFunction = (c_ptr, c_len, p_ptr, p_len) => {
  const memory = new Uint8Array(wasmModule.instance.exports.memory.buffer)
  const functionBody = textDecoder.decode(memory.subarray(c_ptr, c_ptr + c_len))
  const _function = Function(`'use strict';return(${functionBody})`)()

  const values = readParamsFromMemory(p_ptr, p_len)
  return _function.call({}, ...values)
}

const getWasmImports = () => {

    const env = {
        __invoke (c_ptr, c_len, p_ptr, p_len) {
            const result = runFunction(c_ptr, c_len, p_ptr, p_len)
            if (typeof result === "undefined") {
              return (BigInt(0) << 32n) | BigInt(0)
            }  else if (typeof result === "number") {
              const ptr = writeBufferToMemory(textEncoder.encode(result))
              return (BigInt(1) << 32n) | BigInt(ptr)
            } else if (typeof result === "function") {
              const objectId = Math.floor(Math.random() * MAX_U32)
              objects[objectId] = result
              return (BigInt(2) << 32n) | BigInt(objectId)
            } else if (typeof result === "object") {
              // because js has no primitive types for arrays
              if (result instanceof Uint8Array) {
                const ptr = writeBufferToMemory(new Uint8Array(result))
                return (BigInt(3) << 32n) | BigInt(ptr)
              } else {
                const objectId = Math.floor(Math.random() * MAX_U32)
                objects[objectId] = result
                return (BigInt(2) << 32n) | BigInt(objectId)
              }
            } else if (typeof result === "string") {
              const ptr = writeBufferToMemory(textEncoder.encode(result))
              return (BigInt(4) << 32n) | BigInt(ptr)
            } else if (typeof result === "bigint") {
              return (BigInt(5) << 32n) | BigInt(result)
            } else if (typeof result === "boolean") {
              return (BigInt(6) << 32n) | BigInt(result)
            } else {
              throw new Error("Invalid result type")
            }
        },
      __deallocate(object_id) {
          delete objects[object_id]
      }
    }
    return { env }
}

const loadWasm = async () => {
    const imports = getWasmImports()
    const wasmScript = document.querySelector('script[type="application/wasm"]')
    const wasmBuffer = await fetch(wasmScript.src).then(r => r.arrayBuffer())
    wasmModule = await WebAssembly.instantiate(wasmBuffer, imports)
    wasmModule.instance.exports.main()
}

const writeBufferToMemory = (buffer) => {
    const allocationId = wasmModule.instance.exports.create_allocation(buffer.length)
    const allocationPtr = wasmModule.instance.exports.get_allocation(allocationId)
    const memory = new Uint8Array(wasmModule.instance.exports.memory.buffer)
    memory.set(buffer, allocationPtr)
    return allocationId
}

const loadExports = () => {
    exports.wasmModule = wasmModule
    exports.writeBufferToMemory = writeBufferToMemory
    exports.readParamsFromMemory = readParamsFromMemory
}

if (typeof window !== 'undefined') { // load wasm (browser)
    document.addEventListener('DOMContentLoaded', loadWasm)
} else { // load exports (nodejs)
    loadExports()
}
