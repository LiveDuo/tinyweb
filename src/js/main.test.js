const test = require('node:test')
const assert = require('node:assert')

const { readParams, _wasmModule } = require('./main')

// node src/js/main.test.js
test('check read params', () => {

    const testCases = [
        {buffer: [0], result: [undefined]},
        {buffer: [1], result: [null]}
    ]
    for (const testCase of testCases) {
        _wasmModule.instance = { exports: { memory: { buffer: testCase.buffer } } }
        
        const result = readParams(0, testCase.buffer.length)
        assert.deepStrictEqual(result, testCase.result)
    }
})

