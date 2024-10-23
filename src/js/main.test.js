const test = require('node:test')
const assert = require('node:assert')

const { readParamsFromMemory, wasmModule } = require('./main')

// node src/js/main.test.js
test('check read params', () => {

    const float64View = new DataView(new ArrayBuffer(8))
    float64View.setFloat64(0, 42.42, true)
    const float64Array = new Uint8Array(float64View.buffer)

    const bigInt64View = new DataView(new ArrayBuffer(8))
    bigInt64View.setBigInt64(0, 42n, true)
    const bigInt64Array = new Uint8Array(bigInt64View.buffer)

    const int32View = new DataView(new ArrayBuffer(4))
    int32View.setInt32(0, 42, true)
    const int32Array = new Uint8Array(int32View.buffer)

    const uint32View = new DataView(new ArrayBuffer(4))
    uint32View.setInt32(0, 42, true)
    const uint32Array = new Uint8Array(uint32View.buffer)

    const testCases = [
        {memory: [0], expected: [undefined]},
        {memory: [1], expected: [null]},
        {memory: [2, ...float64Array], expected: [42.42]},
        {memory: [3, ...bigInt64Array], expected: [42n]},
        {memory: [4, ...int32Array, ...int32Array], expected: ['']},
        {memory: [5, ...uint32Array], expected: [undefined]},
        {memory: [6, ...int32Array, ...int32Array], expected: [new Float32Array(0)]},
        {memory: [7], expected: [true]},
        {memory: [8], expected: [false]},
        {memory: [9, ...int32Array, ...int32Array], expected: [new Float64Array(0)]},
        {memory: [10, ...int32Array, ...int32Array], expected: [new Uint32Array(0)]},
    ]
    for (const testCase of testCases) {
        wasmModule.instance = { exports: { memory: { buffer: testCase.memory } } }

        const result = readParamsFromMemory(0, testCase.memory.length)
        assert.deepStrictEqual(result, testCase.expected)
    }
})
