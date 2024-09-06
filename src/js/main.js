'use strict'

const MAX_GENERATION = 0xfffffff0

const this_objects = []
const this_generations = []
const this_freeList = []
let this_nextIndex = 0

const store = {
    // return handle as big integer that contains index in low 32 bits and generation in high 32 bits
    allocate: function(o) {
        let index
        if (this_freeList.length > 0) index = this_freeList.pop()
        else index = this_nextIndex++
        const currentGeneration = this_generations[index]
        this_objects[index] = o
        this_generations[index] = currentGeneration === undefined ? 1 : Math.abs(currentGeneration) + 1
        const low = BigInt(index)
        const high = BigInt(this_generations[index]) << BigInt(32)
        const merged = low | high
        return merged
    },
    deallocate: function(handle) {
        const index = Number(handle & BigInt(0xffffffff))
        const generation = Number(handle >> BigInt(32))
        if (generation >= MAX_GENERATION) this_generations[index] = -this_generations[index]
        else if (generation === this_generations[index]) {
            this_generations[index] = -this_generations[index]
            this_freeList.push(index)
        } else throw new Error('attempt to deallocate invalid handle')
    },
    retrieve: function(handle) {
        const index = Number(handle & BigInt(0xffffffff))
        const generation = Number(handle >> BigInt(32))
        if (generation === this_generations[index]) return this_objects[index]
        else throw new Error('attempt to retrieve invalid handle')
    }
}

const utf8dec = new TextDecoder('utf-8')
const utf8enc = new TextEncoder()
const utf16dec = new TextDecoder('utf-16')

const context = {
    functions: [function() { debugger; return 0 }],
    readUtf8FromMemory: function(start, len) {
        return utf8dec.decode(this.getMemory().subarray(start, start + len))
    },
    createAllocation: function(size) {
        if (!this.module) throw new Error('module not set')
        const allocationId = this.module.instance.exports.create_allocation(size)
        const allocationPtr = this.module.instance.exports.allocation_ptr(allocationId)
        return [allocationId, allocationPtr]
    },
    writeUtf8ToMemory: function(str) {
        const bytes = utf8enc.encode(str)
        const len = bytes.length
        const [id, start] = this.createAllocation(len)
        this.getMemory().set(bytes, start)
        return id
    },
    writeArrayBufferToMemory: function(ab) {
        const bytes = new Uint8Array(ab)
        const len = bytes.length
        const [id, start] = this.createAllocation(len)
        this.getMemory().set(bytes, start)
        return id
    },
    readUtf16FromMemory: function(start, len) {
        const text = utf16dec.decode(this.getMemory().subarray(start, start + len))
        return text
    },
    readUint8ArrayFromMemory (start, length) {
        if (!this.module) throw new Error('module not set')
        const b = this.getMemory().slice(start, start + length)
        return new Uint8Array(b)
    },
    storeObject: function(obj) {
        return store.allocate(obj)
    },
    getObject: function(handle) {
        return store.retrieve(handle)
    },
    releaseObject: function(handle) {
        // don't release our fixed references
        if (handle <= 4n) return
        store.deallocate(handle)
    },
    getMemory: function() {
        if (!this.module) throw new Error('module not set')
        return new Uint8Array(this.module.instance.exports.memory.buffer)
    },
    readParameters: function(start, length) {
        
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

        const parameters = this.readUint8ArrayFromMemory(start, length)
        const values = []
        let i = 0
        while(i < parameters.length){
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
                        values.push(context.readUtf8FromMemory(start1, len))
                        break
                    }
                case 5:
                    {
                        const handle = new DataView(parameters.buffer).getBigInt64(i, true)
                        values.push(context.getObject(handle))
                        i += 8
                        break
                    }
                case 6:
                    {
                        const start2 = new DataView(parameters.buffer).getInt32(i, true)
                        i += 4
                        const len1 = new DataView(parameters.buffer).getInt32(i, true)
                        i += 4
                        const memory = context.getMemory()
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
                        const memory1 = context.getMemory()
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
                        const memory2 = context.getMemory()
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
}

const getWasmEnv = () => {
    
    return {
        js_register_function (start, len, utfByteLen) {
            let functionBody
            if (utfByteLen === 16) functionBody = context.readUtf16FromMemory(start, len)
            else functionBody = context.readUtf8FromMemory(start, len)
            const id = context.functions.length
            context.functions.push(Function(`'use strict';return(${functionBody})`)())
            return id
        },
        js_invoke_function (funcHandle, parametersStart, parametersLength) {
            const values = context.readParameters(parametersStart, parametersLength)
            return context.functions[funcHandle].call(context, ...values)
        },
        js_invoke_function_and_return_object (funcHandle, parametersStart, parametersLength) {
            const values = context.readParameters(parametersStart, parametersLength)
            const result = context.functions[funcHandle].call(context, ...values)
            if (result === undefined || result === null) throw new Error('undefined or null while trying to return an object')
            return context.storeObject(result)
        },
        js_invoke_function_and_return_bool (funcHandle, parametersStart, parametersLength) {
            const values = context.readParameters(parametersStart, parametersLength)
            const result = context.functions[funcHandle].call(context, ...values)
            return result ? 1 : 0
        },
        js_invoke_function_and_return_bigint (funcHandle, parametersStart, parametersLength) {
            const values = context.readParameters(parametersStart, parametersLength)
            const result = context.functions[funcHandle].call(context, ...values)
            return result
        },
        js_invoke_function_and_return_string (funcHandle, parametersStart, parametersLength) {
            const values = context.readParameters(parametersStart, parametersLength)
            const result = context.functions[funcHandle].call(context, ...values)
            if (result === undefined || result === null) throw new Error('undefined or null while trying to retrieve string.')
            return context.writeUtf8ToMemory(result)
        },
        js_invoke_function_and_return_array_buffer (funcHandle, parametersStart, parametersLength) {
            const values = context.readParameters(parametersStart, parametersLength)
            const result = context.functions[funcHandle].call(context, ...values)
            if (result === undefined || result === null) throw new Error('undefined or null while trying to retrieve arraybuffer.')
            return context.writeArrayBufferToMemory(result)
        },
        js_externref_drop (obj) {
            context.releaseObject(obj)
        },
    }
}

document.addEventListener('DOMContentLoaded', async function() {
    const wasmScripts = document.querySelectorAll('script[type="application/wasm"]')
    for (const wasmScript of wasmScripts) {

        store.allocate(undefined);
        store.allocate(null);
        store.allocate(self);
        store.allocate(typeof document != 'undefined' ? document : null);
        store.allocate(typeof document != 'undefined' ? document.body : null);

        const env = getWasmEnv()
        const response = await fetch(wasmScript.src)
        const bytes = await response.arrayBuffer()
        const module = await WebAssembly.instantiate(bytes, { env: env })
        context.module = module

        context.module.instance.exports.main()
        
    }
})
