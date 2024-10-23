const test = require('node:test')
const assert = require('node:assert')

const { readParamsFromMemory, wasmModule } = require('./main')

// node src/js/main.test.js
test('check read params', () => {

    const float64View = new DataView(new ArrayBuffer(1 + 8))
    float64View.setInt8(0, 2)
    float64View.setFloat64(1, 42.42, true)
    const float64Array = new Uint8Array(float64View.buffer)

    const testCases = [
        {buffer: [0], result: [undefined]},
        {buffer: [1], result: [null]},
        {buffer: float64Array, result: [42.42]}
    ]
    for (const testCase of testCases) {
        wasmModule.instance = { exports: { memory: { buffer: testCase.buffer } } }

        const result = readParamsFromMemory(0, testCase.buffer.length)
        assert.deepStrictEqual(result, testCase.result)
    }
})
