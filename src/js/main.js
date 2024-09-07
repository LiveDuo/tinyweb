'use strict'

const MAX_GENERATION = 0xfffffff0
const INDEX_MASK = 0xffffffff

const _objects = []
const _generations = []
const _freeList = []

const functions = []

let wasmModule = null
let _nextIndex = 0

const utf8enc = new TextEncoder()

const utf8dec = new TextDecoder('utf-8')
const utf16dec = new TextDecoder('utf-16')

// return handle as big integer that contains index in low 32 bits and generation in high 32 bits
const allocate = function(o) {
    let index
    if (_freeList.length > 0) index = _freeList.pop()
    else index = _nextIndex++
    const currentGeneration = _generations[index]
    _objects[index] = o
    _generations[index] = currentGeneration === undefined ? 1 : Math.abs(currentGeneration) + 1
    const low = BigInt(index)
    const high = BigInt(_generations[index]) << BigInt(32)
    const merged = low | high
    return merged
}

const deallocate = function(handle) {
    const index = Number(handle & BigInt(INDEX_MASK))
    const generation = Number(handle >> BigInt(32))
    if (generation >= MAX_GENERATION) _generations[index] = -_generations[index]
    else if (generation === _generations[index]) {
        _generations[index] = -_generations[index]
        _freeList.push(index)
    } else throw new Error('attempt to deallocate invalid handle')
}

const retrieve = function(handle) {
    const index = Number(handle & BigInt(INDEX_MASK))
    const generation = Number(handle >> BigInt(32))
    if (generation === _generations[index]) return _objects[index]
    else throw new Error('attempt to retrieve invalid handle')
}

const createAllocation = (size) => {
    const allocationId = wasmModule.instance.exports.create_allocation(size)
    const allocationPtr = wasmModule.instance.exports.allocation_ptr(allocationId)
    return [allocationId, allocationPtr]
}

const getMemory = () => new Uint8Array(wasmModule.instance.exports.memory.buffer)

const writeUtf8ToMemory = (str) => {
    const bytes = utf8enc.encode(str)
    const [id, start] = createAllocation(bytes.length)
    getMemory().set(bytes, start)
    return id
}

const readParameters = function(start, length) {
        
    // convert bytes to array of values parameters are preceded by a 32 bit integer indicating its type
    // 0 = undefined
    // 1 = null
    // 2 = float-64
    // 3 = bigint
    // 4 = string (followed by 32-bit start and size of string in memory)
    // 5 = extern ref
    // 6 = array of float-64 (followed by 32-bit start and size of string in memory)
    // 7 = true
    // 8 = false

    const memorySlice = getMemory().slice(start, start + length)
    const parameters = new Uint8Array(memorySlice)
    const values = []
    let i = 0
    while (i < parameters.length) {
        const type = parameters[i]
        i++
        switch(type){
            case 0:
                values.push(undefined)
                break
            case 1:
                values.push(null)
                break
            case 2:
                values.push(new DataView(parameters.buffer).getFloat64(i, true))
                i += 8
                break
            case 3:
                values.push(new DataView(parameters.buffer).getBigInt64(i, true))
                i += 8
                break
            case 4:
                {
                    const start1 = new DataView(parameters.buffer).getInt32(i, true)
                    i += 4
                    const len = new DataView(parameters.buffer).getInt32(i, true)
                    i += 4
                    const value = utf8dec.decode(getMemory().subarray(start1, start1 + len))
                    values.push(value)
                    break
                }
            case 5:
                {
                    const handle = new DataView(parameters.buffer).getBigInt64(i, true)
                    values.push(retrieve(handle))
                    i += 8
                    break
                }
            case 6:
                {
                    const start2 = new DataView(parameters.buffer).getInt32(i, true)
                    i += 4
                    const len1 = new DataView(parameters.buffer).getInt32(i, true)
                    i += 4
                    const memory = getMemory()
                    const slice = memory.buffer.slice(start2, start2 + len1 * 4)
                    const array = new Float32Array(slice)
                    values.push(array)
                    break
                }
            case 7:
                values.push(true)
                break
            case 8:
                values.push(false)
                break
            case 9:
                {
                    const start3 = new DataView(parameters.buffer).getInt32(i, true)
                    i += 4
                    const len2 = new DataView(parameters.buffer).getInt32(i, true)
                    i += 4
                    const memory1 = getMemory()
                    const slice1 = memory1.buffer.slice(start3, start3 + len2 * 8)
                    const array1 = new Float64Array(slice1)
                    values.push(array1)
                    break
                }
            case 10:
                {
                    const start4 = new DataView(parameters.buffer).getInt32(i, true)
                    i += 4
                    const len3 = new DataView(parameters.buffer).getInt32(i, true)
                    i += 4
                    const memory2 = getMemory()
                    const slice2 = memory2.buffer.slice(start4, start4 + len3 * 4)
                    const array2 = new Uint32Array(slice2)
                    values.push(array2)
                    break
                }
            default:
                throw new Error('Unknown parameter type')
        }
    }
    return values
}

const getWasmImports = () => {
    
    const env = {
        js_register_function (start, len, utfByteLen) {
            let functionBody
            if (utfByteLen === 16) functionBody = utf16dec.decode(getMemory().subarray(start, start + len))
            else functionBody = utf8dec.decode(getMemory().subarray(start, start + len))
            const id = functions.length
            functions.push(Function(`'use strict';return(${functionBody})`)())
            return id
        },
        js_invoke_function (funcHandle, parametersStart, parametersLength) {
            const values = readParameters(parametersStart, parametersLength)
            const result = functions[funcHandle].call({}, ...values)
            return result
        },
        js_invoke_function_and_return_object (funcHandle, parametersStart, parametersLength) {
            const values = readParameters(parametersStart, parametersLength)
            const result = functions[funcHandle].call({}, ...values)
            if (result === undefined || result === null) throw new Error('undefined or null while trying to return an object')
            return allocate(result)
        },
        js_invoke_function_and_return_bool (funcHandle, parametersStart, parametersLength) {
            const values = readParameters(parametersStart, parametersLength)
            const result = functions[funcHandle].call({}, ...values)
            return result ? 1 : 0
        },
        js_invoke_function_and_return_bigint (funcHandle, parametersStart, parametersLength) {
            const values = readParameters(parametersStart, parametersLength)
            const result = functions[funcHandle].call({}, ...values)
            return result
        },
        js_invoke_function_and_return_string (funcHandle, parametersStart, parametersLength) {
            const values = readParameters(parametersStart, parametersLength)
            const result = functions[funcHandle].call({}, ...values)
            if (result === undefined || result === null) throw new Error('undefined or null while trying to retrieve string.')

            const bytes = utf8enc.encode(result)
            const [id, start] = createAllocation(bytes.length)
            getMemory().set(bytes, start)
            return id
        },
        js_invoke_function_and_return_array_buffer (funcHandle, parametersStart, parametersLength) {
            const values = readParameters(parametersStart, parametersLength)
            const result = functions[funcHandle].call({}, ...values)
            if (result === undefined || result === null) throw new Error('undefined or null while trying to retrieve arraybuffer.')

            const bytes = new Uint8Array(result)
            const [id, start] = createAllocation(bytes.length)
            getMemory().set(bytes, start)
            return id
        },
        js_externref_drop (obj) {
            deallocate(obj)
        },
    }
    return { env }
}

document.addEventListener('DOMContentLoaded', async () => {
    const imports = getWasmImports()
    const wasmScript = document.querySelector('script[type="application/wasm"]')
    const wasmBuffer = await fetch(wasmScript.src).then(r => r.arrayBuffer())
    wasmModule = await WebAssembly.instantiate(wasmBuffer, imports)
    wasmModule.instance.exports.main()
})
