'use strict'

let wasmModule = {}

const objects = []
const functions = []

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
        } else if (params[i] === 5) { // extern ref
            const objectId = dataView.getUint32(i + 1, true)
            const index = BigInt(objectId)
            values.push(objects[index])
            i += 1 + 4
        } else if (params[i] === 6) { // float32 array
            const ptr = dataView.getInt32(i + 1, true)
            const len = dataView.getInt32(i + 1 + 4, true)
            values.push(new Float32Array(memory.buffer.slice(ptr, ptr + len * 4)))
            i += 1 + 4 + 4
        } else if (params[i] === 7) { // true
            values.push(true)
            i += 1
        } else if (params[i] === 8) { // false
            values.push(false)
            i += 1
        } else if (params[i] === 9) { // float64 array
            const ptr = dataView.getInt32(i + 1, true)
            const len = dataView.getInt32(i + 1 + 4, true)
            values.push(new Float64Array(memory.buffer.slice(ptr, ptr + len * 8)))
            i += 1 + 4 + 4
        } else if (params[i] === 10) { // uint32 array
            const ptr = dataView.getInt32(i + 1, true)
            const len = dataView.getInt32(i + 1 + 4, true)
            values.push(new Uint32Array(memory.buffer.slice(ptr, ptr + len * 4)))
            i += 1 + 4 + 4
        } else {
            throw new Error('Invalid parameter type')
        }
    }
    return values
}

const getWasmImports = () => {

    const env = {
        __register_function (ptr, len) {
            const memory = new Uint8Array(wasmModule.instance.exports.memory.buffer)
            const functionBody = textDecoder.decode(memory.subarray(ptr, ptr + len))
            functions.push(Function(`'use strict';return(${functionBody})`)())
            const functionId = functions.length - 1
            return functionId
        },
        __invoke_function_and_return (functionId, ptr, len) {
            const values = readParamsFromMemory(ptr, len)
            const result = functions[functionId].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return')

            return result
        },
        __invoke_function_and_return_object (functionId, ptr, len) {
            const values = readParamsFromMemory(ptr, len)
            const result = functions[functionId].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return object')

            objects.push(result)
            return BigInt(objects.length - 1)
        },
        __invoke_function_and_return_bool (functionId, ptr, len) {
            const values = readParamsFromMemory(ptr, len)
            const result = functions[functionId].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return bool')

            return result ? 1 : 0
        },
        __invoke_function_and_return_bigint (functionId, ptr, len) {
            const values = readParamsFromMemory(ptr, len)
            const result = functions[functionId].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return big int')

            return result
        },
        __invoke_function_and_return_string (functionId, ptr, len) {
            const values = readParamsFromMemory(ptr, len)
            const result = functions[functionId].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return string')

            const allocationId = writeBufferToMemory(textEncoder.encode(result))
            return allocationId
        },
        __invoke_function_and_return_array_buffer (functionId, ptr, len) {
            const values = readParamsFromMemory(ptr, len)
            const result = functions[functionId].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return array buffer')

            const buffer = new Uint8Array(result)
            const allocationId = writeBufferToMemory(buffer)
            return allocationId
        },
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
    const allocationPtr = wasmModule.instance.exports.allocation_ptr(allocationId)
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
