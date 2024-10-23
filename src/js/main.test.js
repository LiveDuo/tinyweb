const test = require('node:test')
const assert = require('node:assert')

const { readParamsFromMemory, wasmModule } = require('./main')

// node src/js/main.test.js
test('check read params', () => {

    const float64View = new DataView(new ArrayBuffer(1 + 8))
    float64View.setInt8(0, 2)
    float64View.setFloat64(1, 42.42, true)
    const float64Array = new Uint8Array(float64View.buffer)

    const bigInt64View = new DataView(new ArrayBuffer(1 + 8))
    bigInt64View.setInt8(0, 3)
    bigInt64View.setBigInt64(1, 42n, true)
    const bigInt64Array = new Uint8Array(bigInt64View.buffer)

    const testCases = [
        {memory: [0], expected: [undefined]},
        {memory: [1], expected: [null]},
        {memory: float64Array, expected: [42.42]},
        {memory: bigInt64Array, expected: [42n]}
    ]
    for (const testCase of testCases) {
        wasmModule.instance = { exports: { memory: { buffer: testCase.memory } } }

        const result = readParamsFromMemory(0, testCase.memory.length)
        assert.deepStrictEqual(result, testCase.expected)
    }
})
