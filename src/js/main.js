'use strict'

let wasmModule = {}

const state = { objects: [], objectFreeList: [], objectIndex: 0, functions: [] }

const allocate = (object) => {
    const index = (state.objectFreeList.length > 0) ? state.objectFreeList.pop() : state.objectIndex++
    state.objects[index] = object
    return BigInt(index)
}

const deallocate = (handle) => {
    if (handle) {
        state.objectFreeList.push(Number(handle))
    } else {
        throw new Error('Invalid deallocate handle')
    }
}

// 0 = undefined, 1 = null, 2 = f64, 3 = bigint, 4 = string, 5 = extern ref, 6 = array of f64, 7 = true, 8 = false
const readParams = (start, length) => {
    
    const memory = new Uint8Array(wasmModule.instance.exports.memory.buffer)
    const parameters = new Uint8Array(memory.slice(start, start + length))
    const dataView = new DataView(parameters.buffer)
    const values = []
    let i = 0
    while (i < parameters.length) {
        if (parameters[i] === 0) {
            values.push(undefined)
            i += 1
        } else if (parameters[i] === 1) {
            values.push(null)
            i += 1
        } else if (parameters[i] === 2) {
            values.push(dataView.getFloat64(i + 1, true))
            i += 1 + 8
        } else if (parameters[i] === 3) {
            values.push(dataView.getBigInt64(i + 1, true))
            i += 1 + 8
        } else if (parameters[i] === 4) {
            const start = dataView.getInt32(i + 1, true)
            const len = dataView.getInt32(i + 1 + 4, true)
            values.push((new TextDecoder('utf-8')).decode(memory.subarray(start, start + len)))
            i += 1 + 4 + 4
        } else if (parameters[i] === 5) {
            const handle = dataView.getUint32(i + 1, true)
            const index = Number(handle)
            values.push(state.objects[index])
            i += 1 + 4
        } else if (parameters[i] === 6) {
            const start = dataView.getInt32(i + 1, true)
            const len = dataView.getInt32(i + 1 + 4, true)
            values.push(new Float32Array(memory.buffer.slice(start, start + len * 4)))
            i += 1 + 4 + 4
        } else if (parameters[i] === 7) {
            values.push(true)
            i += 1
        } else if (parameters[i] === 8) {
            values.push(false)
            i += 1
        } else if (parameters[i] === 9) {
            const start = dataView.getInt32(i + 1, true)
            const len = dataView.getInt32(i + 1 + 4, true)
            values.push(new Float64Array(memory.buffer.slice(start, start + len * 8)))
            i += 1 + 4 + 4
        } else if (parameters[i] === 10) {
            const start = dataView.getInt32(i + 1, true)
            const len = dataView.getInt32(i + 1 + 4, true)
            values.push(new Uint32Array(memory.buffer.slice(start, start + len * 4)))
            i += 1 + 4 + 4
        } else {
            throw new Error('Invalid parameter type')
        }
    }
    return values
}

const getWasmImports = () => {
    
    const env = {
        __register_function (start, len) {
            const decoder = new TextDecoder('utf-8')
            const memory = new Uint8Array(wasmModule.instance.exports.memory.buffer)
            const functionBody = decoder.decode(memory.subarray(start, start + len))
            state.functions.push(Function(`'use strict';return(${functionBody})`)())
            const id = state.functions.length - 1
            return id
        },
        __invoke_function (handle, start, len) {
            const values = readParams(start, len)
            const result = state.functions[handle].call({}, ...values)
            return result
        },
        __invoke_function_and_return_object (handle, start, len) {
            const values = readParams(start, len)
            const result = state.functions[handle].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return object')
            return allocate(result)
        },
        __invoke_function_and_return_bool (handle, start, len) {
            const values = readParams(start, len)
            const result = state.functions[handle].call({}, ...values)
            return result ? 1 : 0
        },
        __invoke_function_and_return_bigint (handle, start, len) {
            const values = readParams(start, len)
            const result = state.functions[handle].call({}, ...values)
            return result
        },
        __invoke_function_and_return_string (handle, start, len) {
            const values = readParams(start, len)
            const result = state.functions[handle].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return string')

            const bytes = (new TextEncoder()).encode(result)
            const id = wasmModule.instance.exports.create_allocation(bytes.length)
            const ptr = wasmModule.instance.exports.allocation_ptr(id)
            const memory = new Uint8Array(wasmModule.instance.exports.memory.buffer)
            memory.set(bytes, ptr)
            return id
        },
        __invoke_function_and_return_array_buffer (handle, start, len) {
            const values = readParams(start, len)
            const result = state.functions[handle].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return array buffer')

            const bytes = new Uint8Array(result)
            const id = wasmModule.instance.exports.create_allocation(bytes.length)
            const ptr = wasmModule.instance.exports.allocation_ptr(id)
            const memory = new Uint8Array(wasmModule.instance.exports.memory.buffer)
            memory.set(bytes, ptr)
            return id
        },
        __drop_externref (obj) {
            deallocate(obj)
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

const loadExports = () => {
    exports.wasmModule = wasmModule
    exports.allocate = allocate
    exports.deallocate = deallocate
    exports.readParams = readParams
}

if (typeof window !== 'undefined') { // load wasm (browser)
    document.addEventListener('DOMContentLoaded', loadWasm)
} else { // load exports (nodejs)
    loadExports()
}
