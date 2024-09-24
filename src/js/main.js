'use strict'

const MAX_GENERATION = 0xfffffff0
const INDEX_MASK = 0xffffffff

const _objects = []
const _generations = []
const _freeList = []
const _functions = []

let _wasmModule = {}
let _nextIndex = 0

// returns index as bigint in low 32-bits and generation in high 32-bits
const allocate = (object) => {

    // get index
    let index
    if (_freeList.length > 0) index = _freeList.pop()
    else index = _nextIndex++

    // update variables
    const currentGeneration = _generations[index]
    _objects[index] = object
    _generations[index] = currentGeneration === undefined ? 1 : Math.abs(currentGeneration) + 1

    // get merged
    const low = BigInt(index)
    const high = BigInt(_generations[index]) << BigInt(32)
    const merged = low | high
    return merged
}

const deallocate = (handle) => {
    const index = Number(handle & BigInt(INDEX_MASK))
    const generation = Number(handle >> BigInt(32))
    if (generation >= MAX_GENERATION) {
        _generations[index] = -_generations[index]
    } else if (generation === _generations[index]) {
        _generations[index] = -_generations[index]
        _freeList.push(index)
    } else {
        throw new Error('Invalid deallocate handle')
    }
}

// 0 = undefined, 1 = null, 2 = f64, 3 = bigint, 4 = string, 5 = extern ref, 6 = array of f64, 7 = true, 8 = false
const readParams = (start, length) => {
    
    const memory = new Uint8Array(_wasmModule.instance.exports.memory.buffer)
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
            const handle = dataView.getBigInt64(i + 1, true)
            const index = Number(handle & BigInt(INDEX_MASK))
            values.push(_objects[index])
            i += 1 + 8
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
        js_register_function (start, len, utfByteLen) {
            const decoder = (utfByteLen === 16) ? new TextDecoder('utf-16') : new TextDecoder('utf-8')
            const memory = new Uint8Array(_wasmModule.instance.exports.memory.buffer)
            const functionBody = decoder.decode(memory.subarray(start, start + len))
            const id = _functions.length
            _functions.push(Function(`'use strict';return(${functionBody})`)())
            return id
        },
        js_invoke_function (handle, start, len) {
            const values = readParams(start, len)
            const result = _functions[handle].call({}, ...values)
            return result
        },
        js_invoke_function_and_return_object (handle, start, len) {
            const values = readParams(start, len)
            const result = _functions[handle].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return object')
            return allocate(result)
        },
        js_invoke_function_and_return_bool (handle, start, len) {
            const values = readParams(start, len)
            const result = _functions[handle].call({}, ...values)
            return result ? 1 : 0
        },
        js_invoke_function_and_return_bigint (handle, start, len) {
            const values = readParams(start, len)
            const result = _functions[handle].call({}, ...values)
            return result
        },
        js_invoke_function_and_return_string (handle, start, len) {
            const values = readParams(start, len)
            const result = _functions[handle].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return string')

            const bytes = (new TextEncoder()).encode(str)
            const id = _wasmModule.instance.exports.create_allocation(bytes.length)
            const ptr = _wasmModule.instance.exports.allocation_ptr(id)
            const memory = new Uint8Array(_wasmModule.instance.exports.memory.buffer)
            memory.set(bytes, ptr)
            return id
        },
        js_invoke_function_and_return_array_buffer (handle, start, len) {
            const values = readParams(start, len)
            const result = _functions[handle].call({}, ...values)
            if (result === undefined || result === null) throw new Error('Invalid return arraybuffer')

            const bytes = new Uint8Array(result)
            const id = _wasmModule.instance.exports.create_allocation(bytes.length)
            const ptr = _wasmModule.instance.exports.allocation_ptr(id)
            const memory = new Uint8Array(_wasmModule.instance.exports.memory.buffer)
            memory.set(bytes, ptr)
            return id
        },
        js_externref_drop (obj) {
            deallocate(obj)
        },
    }
    return { env }
}

const loadWasm = async () => {
    const imports = getWasmImports()
    const wasmScript = document.querySelector('script[type="application/wasm"]')
    const wasmBuffer = await fetch(wasmScript.src).then(r => r.arrayBuffer())
    _wasmModule = await WebAssembly.instantiate(wasmBuffer, imports)
    _wasmModule.instance.exports.main()
}

const loadExports = () => {
    exports._wasmModule = _wasmModule
    exports.allocate = allocate
    exports.deallocate = deallocate
    exports.readParams = readParams
}

if (typeof window !== 'undefined') { // load wasm (browser)
    document.addEventListener('DOMContentLoaded', loadWasm)
} else { // load exports (nodejs)
    loadExports()
}
