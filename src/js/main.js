'use strict'

let wasmModule = {}

const objects = []

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
        __invoke_and_return (c_ptr, c_len, p_ptr, p_len, r_type) {
            // invoke function
            const result = runFunction(c_ptr, c_len, p_ptr, p_len)
            if (r_type !== 0 && (result === undefined || result === null)) throw new Error('Invalid return')

            // return result
            if (r_type === 0) {
              return result
            }  else if (r_type === 1) {
              return result
            } else if (r_type === 2) {
              objects.push(result)
              return objects.length - 1
            } else if (r_type === 3) {
              return writeBufferToMemory(textEncoder.encode(result))
            } else if (r_type === 4) {
              return writeBufferToMemory(new Uint8Array(result))
            }
        },
      __deallocate(object_id) {
          const index = objects.indexOf(object_id)
          objects.splice(index, 1);
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
